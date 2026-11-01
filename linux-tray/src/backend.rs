use crate::SupervisorError;
use serde::Deserialize;
use std::{
    io::Write,
    os::unix::{io::FromRawFd, process::CommandExt},
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};

pub struct Readiness {
    pub port: u16,
}

#[derive(Deserialize)]
struct ReadinessPayload {
    #[serde(rename = "instanceID")]
    instance_id: String,
    port: u16,
}

impl Readiness {
    pub fn parse(value: &str, instance_id: &str) -> Result<Self, SupervisorError> {
        let payload: ReadinessPayload =
            serde_json::from_str(value).map_err(|_| SupervisorError::InvalidReadiness)?;
        if payload.instance_id == instance_id && payload.port > 0 {
            Ok(Self { port: payload.port })
        } else {
            Err(SupervisorError::InvalidReadiness)
        }
    }
}

pub struct Supervisor {
    child: Option<Child>,
    restarts: u8,
    max_restarts: u8,
    diagnostics: String,
    output_limit: usize,
    reaped: bool,
}

impl Supervisor {
    pub fn new(max_restarts: u8, output_limit: usize) -> Self {
        Self {
            child: None,
            restarts: 0,
            max_restarts,
            diagnostics: String::new(),
            output_limit,
            reaped: false,
        }
    }
    pub fn spawn(binary: &str, arguments: &[&str], token: &str) -> Result<Self, SupervisorError> {
        let mut fds = [0; 2];

        if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
            return Err(SupervisorError::Spawn(std::io::Error::last_os_error()));
        }

        let read_fd = fds[0];
        let mut command = Command::new(binary);
        command
            .args(arguments)
            .env("CODEXBAR_DASHBOARD_TOKEN_FD", "3");
        
        unsafe {
            command.pre_exec(move || {
                libc::setpgid(0, 0);
                if libc::dup2(read_fd, 3) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        
        let child = command.spawn().map_err(SupervisorError::Spawn)?;
        
        unsafe {
            libc::close(read_fd);
        }
        
        let mut writer = unsafe { std::fs::File::from_raw_fd(fds[1]) };
        writer
            .write_all(token.as_bytes())
            .map_err(SupervisorError::Spawn)?;
        Ok(Self {
            child: Some(child),
            ..Self::new(3, 4096)
        })
    }
    
    pub fn record_exit(&mut self) -> Result<(), SupervisorError> {
        self.restarts += 1;
        
        if self.restarts > self.max_restarts {
            Err(SupervisorError::Stop)
        } else {
            Ok(())
        }
    }
    
    pub fn capture(&mut self, value: &str) {
        let value = if value.contains("token=") {
            "[redacted]"
        } else {
            value
        };
        self.diagnostics = value.chars().take(self.output_limit).collect();
    }
    
    pub fn diagnostics(&self) -> &str {
        &self.diagnostics
    }
    
    pub fn is_reaped(&self) -> bool {
        self.reaped
    }
    
    pub fn stop(&mut self, timeout: Duration) -> Result<(), SupervisorError> {
        let child = self.child.as_mut().ok_or(SupervisorError::Stop)?;
        
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGTERM);
        }
        
        let deadline = Instant::now() + timeout;
        
        while Instant::now() < deadline {
            if child.try_wait().map_err(SupervisorError::Spawn)?.is_some() {
                self.reaped = true;
                return Ok(());
            }
            thread::sleep(Duration::from_millis(5));
        }
        
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGKILL);
        }
        
        child.wait().map_err(SupervisorError::Spawn)?;
        self.reaped = true;
        Ok(())
    }
}
