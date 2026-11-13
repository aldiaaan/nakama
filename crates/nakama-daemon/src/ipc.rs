use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, ReadHalf,
    WriteHalf,
};
use tokio::task::JoinSet;
use tokio::time;
use tracing::{debug, warn};

use crate::daemon::{DaemonHandle, DaemonStats};
use crate::error::DaemonError;
use crate::shutdown::{ShutdownHandle, ShutdownReason};
use crate::subsystem::SubsystemState;

const MAX_REQUEST_BYTES: usize = 64 * 1024;
const ACCEPT_RETRY: Duration = Duration::from_millis(100);

/// Where the control channel listens: a named pipe on Windows, a Unix socket elsewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcEndpoint(PathBuf);

impl IpcEndpoint {
    /// `\\.\pipe\<name>` on Windows. `<name>.sock` in `$XDG_RUNTIME_DIR`, or the temp folder, elsewhere.
    pub fn named(name: &str) -> Self {
        if cfg!(windows) {
            return Self(PathBuf::from(format!(r"\\.\pipe\{name}")));
        }
        let folder = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        Self(folder.join(format!("{name}.sock")))
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for IpcEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.display().fmt(f)
    }
}

/// One JSON object per line, answered by one `IpcResponse` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum IpcRequest {
    Status,
    Metrics,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum IpcResponse {
    Status(StatusReport),
    Metrics { prometheus: String },
    Shutdown { initiated: bool },
    Error { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusReport {
    pub name: String,
    pub uptime_ms: Option<u64>,
    pub shutdown_reason: Option<ShutdownReason>,
    pub subsystems: Vec<SubsystemReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsystemReport {
    pub name: String,
    pub state: SubsystemState,
    pub restarts: u32,
    pub last_error: Option<String>,
}

impl From<&DaemonStats> for StatusReport {
    fn from(stats: &DaemonStats) -> Self {
        Self {
            name: stats.name.clone(),
            uptime_ms: stats
                .uptime
                .map(|uptime| u64::try_from(uptime.as_millis()).unwrap_or(u64::MAX)),
            shutdown_reason: stats.shutdown_reason,
            subsystems: stats
                .subsystems
                .iter()
                .map(|status| SubsystemReport {
                    name: status.name.clone(),
                    state: status.state,
                    restarts: status.restarts,
                    last_error: status.last_error.as_ref().map(ToString::to_string),
                })
                .collect(),
        }
    }
}

pub struct IpcClient {
    reader: BufReader<ReadHalf<platform::Client>>,
    writer: WriteHalf<platform::Client>,
}

impl IpcClient {
    pub async fn connect(endpoint: &IpcEndpoint) -> Result<Self, DaemonError> {
        let stream =
            platform::connect(endpoint.path())
                .await
                .map_err(|source| DaemonError::IpcConnect {
                    endpoint: endpoint.path().to_owned(),
                    source,
                })?;
        let (reader, writer) = tokio::io::split(stream);
        Ok(Self {
            reader: BufReader::new(reader),
            writer,
        })
    }

    pub async fn request(&mut self, request: IpcRequest) -> Result<IpcResponse, DaemonError> {
        write_message(&mut self.writer, &request)
            .await
            .map_err(DaemonError::Ipc)?;
        let mut line = String::new();
        if self
            .reader
            .read_line(&mut line)
            .await
            .map_err(DaemonError::Ipc)?
            == 0
        {
            return Err(DaemonError::IpcClosed);
        }
        serde_json::from_str(&line).map_err(DaemonError::IpcResponse)
    }
}

pub(crate) struct IpcServer {
    listener: platform::Listener,
}

impl IpcServer {
    pub(crate) fn bind(endpoint: &IpcEndpoint) -> Result<Self, DaemonError> {
        let listener =
            platform::Listener::bind(endpoint.path()).map_err(|source| DaemonError::IpcBind {
                endpoint: endpoint.path().to_owned(),
                source,
            })?;
        Ok(Self { listener })
    }

    /// Accepts connections until shutdown, then waits for open connections to finish.
    pub(crate) async fn serve(mut self, daemon: DaemonHandle, shutdown: ShutdownHandle) {
        let mut connections = JoinSet::new();
        loop {
            tokio::select! {
                () = shutdown.cancelled() => break,
                accepted = self.listener.accept() => match accepted {
                    Ok(stream) => {
                        connections.spawn(serve_connection(stream, daemon.clone(), shutdown.clone()));
                    }
                    Err(source) => {
                        warn!(error = %source, "IPC accept failed");
                        time::sleep(ACCEPT_RETRY).await;
                    }
                },
                Some(_) = connections.join_next(), if !connections.is_empty() => {}
            }
        }
        while connections.join_next().await.is_some() {}
    }
}

async fn serve_connection<S>(stream: S, daemon: DaemonHandle, shutdown: ShutdownHandle)
where
    S: AsyncRead + AsyncWrite,
{
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        line.clear();
        let mut limited = (&mut reader).take(MAX_REQUEST_BYTES as u64);
        let read = tokio::select! {
            () = shutdown.cancelled() => return,
            read = limited.read_line(&mut line) => read,
        };
        let response = match read {
            Ok(0) => return,
            Ok(read) if read >= MAX_REQUEST_BYTES && !line.ends_with('\n') => {
                let message = format!("request is longer than {MAX_REQUEST_BYTES} bytes");
                let _ = write_message(&mut writer, &IpcResponse::Error { message }).await;
                return;
            }
            Ok(_) if line.trim().is_empty() => continue,
            Ok(_) => match serde_json::from_str(&line) {
                Ok(request) => respond(request, &daemon),
                Err(source) => IpcResponse::Error {
                    message: format!("invalid request: {source}"),
                },
            },
            Err(source) => {
                debug!(error = %source, "IPC read failed");
                return;
            }
        };
        if let Err(source) = write_message(&mut writer, &response).await {
            debug!(error = %source, "IPC write failed");
            return;
        }
    }
}

fn respond(request: IpcRequest, daemon: &DaemonHandle) -> IpcResponse {
    match request {
        IpcRequest::Status => IpcResponse::Status(StatusReport::from(&daemon.stats())),
        IpcRequest::Metrics => IpcResponse::Metrics {
            prometheus: daemon.metrics().render_prometheus(),
        },
        IpcRequest::Shutdown => IpcResponse::Shutdown {
            initiated: daemon.shutdown(),
        },
    }
}

async fn write_message<W, T>(writer: &mut W, message: &T) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let mut bytes = serde_json::to_vec(message)?;
    bytes.push(b'\n');
    writer.write_all(&bytes).await?;
    writer.flush().await
}

#[cfg(unix)]
mod platform {
    use std::fs::{self, Permissions};
    use std::io;
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};
    use std::path::{Path, PathBuf};

    use tokio::net::{UnixListener, UnixStream};

    pub(super) type Client = UnixStream;

    /// Removes the socket file when dropped.
    pub(super) struct Listener {
        listener: UnixListener,
        path: PathBuf,
    }

    impl Listener {
        pub(super) fn bind(path: &Path) -> io::Result<Self> {
            let listener = match UnixListener::bind(path) {
                Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
                    remove_stale_socket(path)?;
                    UnixListener::bind(path)?
                }
                bound => bound?,
            };
            fs::set_permissions(path, Permissions::from_mode(0o600))?;
            Ok(Self {
                listener,
                path: path.to_owned(),
            })
        }

        pub(super) async fn accept(&mut self) -> io::Result<UnixStream> {
            self.listener.accept().await.map(|(stream, _)| stream)
        }
    }

    impl Drop for Listener {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    /// A socket left by a crashed daemon refuses connections; a live daemon accepts them.
    fn remove_stale_socket(path: &Path) -> io::Result<()> {
        if std::os::unix::net::UnixStream::connect(path).is_ok() {
            return Err(io::ErrorKind::AddrInUse.into());
        }
        if !fs::symlink_metadata(path)?.file_type().is_socket() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "path exists and is not a Unix socket",
            ));
        }
        fs::remove_file(path)
    }

    pub(super) async fn connect(path: &Path) -> io::Result<UnixStream> {
        UnixStream::connect(path).await
    }
}

#[cfg(windows)]
mod platform {
    use std::io;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use tokio::net::windows::named_pipe::{
        ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
    };
    use tokio::time;

    const ERROR_PIPE_BUSY: i32 = 231;
    const BUSY_RETRIES: u32 = 10;
    const BUSY_DELAY: Duration = Duration::from_millis(20);

    pub(super) type Client = NamedPipeClient;

    /// Keeps one idle pipe instance open so a client can always connect.
    pub(super) struct Listener {
        name: PathBuf,
        next: NamedPipeServer,
    }

    impl Listener {
        pub(super) fn bind(name: &Path) -> io::Result<Self> {
            let next = ServerOptions::new()
                .first_pipe_instance(true)
                .create(name)?;
            Ok(Self {
                name: name.to_owned(),
                next,
            })
        }

        pub(super) async fn accept(&mut self) -> io::Result<NamedPipeServer> {
            let connected = self.next.connect().await;
            let next = ServerOptions::new().create(&self.name)?;
            let current = std::mem::replace(&mut self.next, next);
            connected.map(|()| current)
        }
    }

    pub(super) async fn connect(name: &Path) -> io::Result<NamedPipeClient> {
        let mut attempt = 0;
        loop {
            match ClientOptions::new().open(name) {
                Err(error)
                    if error.raw_os_error() == Some(ERROR_PIPE_BUSY) && attempt < BUSY_RETRIES =>
                {
                    attempt += 1;
                    time::sleep(BUSY_DELAY).await;
                }
                opened => return opened,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IpcRequest, IpcResponse};

    #[test]
    fn requests_use_a_type_tag() {
        assert_eq!(
            serde_json::to_string(&IpcRequest::Shutdown).expect("json"),
            r#"{"type":"shutdown"}"#
        );
        assert_eq!(
            serde_json::from_str::<IpcRequest>(r#"{"type":"status"}"#).expect("json"),
            IpcRequest::Status
        );
    }

    #[test]
    fn responses_round_trip() {
        let response = IpcResponse::Shutdown { initiated: true };
        let json = serde_json::to_string(&response).expect("json");
        assert_eq!(json, r#"{"type":"shutdown","initiated":true}"#);
        assert_eq!(
            serde_json::from_str::<IpcResponse>(&json).expect("json"),
            response
        );
    }
}
