---
'nakama-daemon': minor
'@nakama/api': minor
'nakama-desktop': minor
---

Let a daemon outlive the app. `DaemonBuilder::with_ipc_method` registers custom IPC methods, `IpcClient::call` calls them, and `DetachedDaemon` starts a daemon as its own process unless one already answers. The desktop app picks `DaemonMode::Embedded` or `DaemonMode::Detached` in code, and the Daemon page shows the daemon's lifetime and process id.
