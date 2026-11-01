import Commander
import Foundation
import Testing
@testable import CodexBarCLI
@testable import CodexBarCore

struct CLIServeDashboardLinuxTests {
    @Test
    func `serve accepts an ephemeral port and emits token-free readiness`() {
        #expect(CodexBarCLI.decodeServePort(from: .init(positional: [], options: ["port": ["0"]], flags: [])) == 0)
        let readiness = CLIServeReadiness(instanceID: "fixture-instance", port: 4321).encoded
        #expect(readiness.contains("fixture-instance"))
        #expect(readiness.contains("4321"))
        #expect(!readiness.contains("fixture-secret"))
    }

    @Test
    func `serve reads a bounded inherited token and rejects legacy transports`() throws {
        let pipe = Pipe()
        try pipe.fileHandleForWriting.write(contentsOf: Data("fixture-secret\n".utf8))
        try pipe.fileHandleForWriting.close()
        #expect(try CLIServeInheritedToken.read(from: pipe.fileHandleForReading, maximumBytes: 64) == "fixture-secret")
        #expect(throws: CLIServeInheritedTokenError.self) {
            _ = try CLIServeInheritedToken.read(Data(repeating: 1, count: 65), maximumBytes: 64)
        }
        #expect(CodexBarCLI.legacyDashboardTokenTransportRejected(
            values: .init(positional: [], options: ["dashboardBearer": ["fixture-secret"]], flags: []),
            environment: ["CODEXBAR_DASHBOARD_TOKEN": "fixture-secret"]))
    }

    @Test
    func `inherited token returns at newline without waiting for EOF`() throws {
        let pipe = Pipe()
        try pipe.fileHandleForWriting.write(contentsOf: Data("newline-token\nignored".utf8))
        #expect(try CLIServeInheritedToken.read(from: pipe.fileHandleForReading, maximumBytes: 64) == "newline-token")
        try pipe.fileHandleForWriting.close()
    }

    @Test
    func `inherited token caps input before buffering beyond its limit`() throws {
        let pipe = Pipe()
        try pipe.fileHandleForWriting.write(contentsOf: Data(repeating: 0x61, count: 65))
        try pipe.fileHandleForWriting.close()
        #expect(throws: CLIServeInheritedTokenError.tooLarge) {
            _ = try CLIServeInheritedToken.read(from: pipe.fileHandleForReading, maximumBytes: 64)
        }
    }

    @Test
    func `dashboard forced refresh reaches provider cost collection`() {
        #expect(CodexBarCLI.dashboardCostRefreshFlag(forceRefresh: true))
        #expect(!CodexBarCLI.dashboardCostRefreshFlag(forceRefresh: false))
    }

    @Test
    func `desktop managed mode requires fd token and instance ID without legacy credentials`() throws {
        let values = Commander.ParsedValues(
            positional: [],
            options: ["dashboardTokenFD": ["7"], "dashboardInstanceID": ["fixture-instance"]],
            flags: [])
        #expect(try CodexBarCLI.desktopManagedDashboardConfiguration(
            values: values,
            environment: [:]) == .init(tokenFD: 7, instanceID: "fixture-instance"))
        #expect(throws: CLIServeDesktopManagedError.self) {
            _ = try CodexBarCLI.desktopManagedDashboardConfiguration(
                values: values,
                environment: ["CODEXBAR_DASHBOARD_TOKEN": "fixture-secret"])
        }
    }

    @Test
    func `dashboard snapshot accepts a scoped forced refresh`() throws {
        let request = try CLIServeDashboardRequest(queryItems: [
            "provider": " codex ",
            "refresh": "1",
        ])

        #expect(request.provider == "codex")
        #expect(request.forceRefresh)
    }

    @Test(arguments: ["token", "access_token", "dashboard-token"])
    func `dashboard snapshot rejects credentials in its query`(credentialName: String) {
        #expect(throws: CLIServeDashboardRequestError.self) {
            _ = try CLIServeDashboardRequest(queryItems: [credentialName: "fixture-secret"])
        }
    }

    @Test
    func `dashboard snapshot rejects malformed refresh values`() {
        #expect(throws: CLIServeDashboardRequestError.self) {
            _ = try CLIServeDashboardRequest(queryItems: ["refresh": "true"])
        }
    }

    @Test
    func `dashboard actions originate only from trusted descriptor HTTPS URLs`() {
        #expect(DashboardSnapshotBuilder.dashboardActions(for: .codex)?.dashboardURL
            == "https://chatgpt.com/codex/settings/usage")
        #expect(DashboardSnapshotBuilder.dashboardActions(for: .synthetic) == nil)
    }
}
