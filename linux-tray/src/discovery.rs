use std::path::PathBuf;

#[derive(Debug)]
pub enum SupervisorError {
    ExecutableNotFound(PathBuf),
    InvalidReadiness,
    Spawn(std::io::Error),
    Stop,
}

pub struct CliDiscovery;

impl CliDiscovery {
    pub fn resolve(
        override_path: Option<PathBuf>,
        sibling: Option<PathBuf>,
        xdg: Option<PathBuf>,
        path: Option<PathBuf>,
    ) -> Result<PathBuf, SupervisorError> {
        let candidate = [override_path, sibling, xdg, path]
            .into_iter()
            .flatten()
            .find(|candidate| candidate.is_file());
        candidate.ok_or_else(|| SupervisorError::ExecutableNotFound(PathBuf::from("CodexBarCLI")))
    }

    pub fn from_override(value: &str) -> Result<PathBuf, SupervisorError> {
        let path = PathBuf::from(value);
        if path.is_file() {
            Ok(path)
        } else {
            Err(SupervisorError::ExecutableNotFound(path))
        }
    }
}
