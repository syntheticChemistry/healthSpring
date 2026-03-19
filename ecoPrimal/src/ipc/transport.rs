// SPDX-License-Identifier: AGPL-3.0-or-later
//! Platform-agnostic transport layer for JSON-RPC IPC.
//!
//! Supports runtime transport selection following ecoBin v3.0:
//! - Unix domain sockets (Linux, macOS, Android)
//! - TCP localhost fallback (Windows, cross-platform testing)
//!
//! Transport type is inferred from the endpoint format:
//! - File paths (contains `/` or ends in `.sock`) → Unix socket
//! - `host:port` format → TCP
//! - Explicit prefix: `unix://path` or `tcp://host:port`
//!
//! Named pipes (Windows) will be added when targeting Windows hosts.
//! The abstraction is designed so consumers never know the transport.

use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;

use super::error::IpcError;

/// Opaque bidirectional IPC stream.
///
/// Wraps platform-specific transport (Unix socket or TCP) behind
/// `Read + Write` so consumers are transport-agnostic.
pub enum IpcStream {
    #[cfg(unix)]
    Unix(std::os::unix::net::UnixStream),
    Tcp(TcpStream),
}

impl Read for IpcStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            #[cfg(unix)]
            Self::Unix(s) => s.read(buf),
            Self::Tcp(s) => s.read(buf),
        }
    }
}

impl Write for IpcStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            #[cfg(unix)]
            Self::Unix(s) => s.write(buf),
            Self::Tcp(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::Unix(s) => s.flush(),
            Self::Tcp(s) => s.flush(),
        }
    }
}

impl IpcStream {
    /// Set read and write timeouts on the underlying transport.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if the OS rejects the timeout configuration.
    pub fn set_timeouts(&self, timeout: Duration) -> io::Result<()> {
        let t = Some(timeout);
        match self {
            #[cfg(unix)]
            Self::Unix(s) => {
                s.set_read_timeout(t)?;
                s.set_write_timeout(t)?;
            }
            Self::Tcp(s) => {
                s.set_read_timeout(t)?;
                s.set_write_timeout(t)?;
            }
        }
        Ok(())
    }

    /// Shut down the write half of the connection.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if the OS rejects the shutdown.
    pub fn shutdown_write(&self) -> io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::Unix(s) => s.shutdown(std::net::Shutdown::Write),
            Self::Tcp(s) => s.shutdown(std::net::Shutdown::Write),
        }
    }
}

/// Parsed endpoint descriptor.
#[derive(Debug, Clone)]
pub enum Endpoint {
    #[cfg(unix)]
    Unix(std::path::PathBuf),
    Tcp(String),
}

/// Parse an endpoint string into a typed [`Endpoint`].
///
/// Inference rules:
/// - `unix://path` → Unix socket at `path`
/// - `tcp://host:port` → TCP connection
/// - Path-like (contains `/` or `.sock`) → Unix socket
/// - `host:port` → TCP
#[must_use]
pub fn parse_endpoint(raw: &str) -> Endpoint {
    if let Some(path) = raw.strip_prefix("unix://") {
        #[cfg(unix)]
        return Endpoint::Unix(path.into());
        #[cfg(not(unix))]
        panic!("unix:// endpoints require a Unix platform");
    }
    if let Some(addr) = raw.strip_prefix("tcp://") {
        return Endpoint::Tcp(addr.to_owned());
    }
    if raw.contains('/')
        || std::path::Path::new(raw)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
    {
        #[cfg(unix)]
        return Endpoint::Unix(raw.into());
        #[cfg(not(unix))]
        return Endpoint::Tcp(raw.to_owned());
    }
    Endpoint::Tcp(raw.to_owned())
}

/// Connect to an endpoint, returning a transport-agnostic [`IpcStream`].
///
/// # Errors
///
/// Returns [`IpcError::Connect`] if the connection fails.
pub fn connect(endpoint: &Endpoint) -> Result<IpcStream, IpcError> {
    match endpoint {
        #[cfg(unix)]
        Endpoint::Unix(path) => {
            let stream =
                std::os::unix::net::UnixStream::connect(path).map_err(IpcError::Connect)?;
            Ok(IpcStream::Unix(stream))
        }
        Endpoint::Tcp(addr) => {
            let sock_addr = addr
                .to_socket_addrs()
                .map_err(IpcError::Connect)?
                .next()
                .ok_or_else(|| {
                    IpcError::Connect(io::Error::new(
                        io::ErrorKind::AddrNotAvailable,
                        format!("cannot resolve {addr}"),
                    ))
                })?;
            let stream = TcpStream::connect(sock_addr).map_err(IpcError::Connect)?;
            Ok(IpcStream::Tcp(stream))
        }
    }
}

/// Connect to a filesystem path (convenience for current callers).
///
/// On Unix, connects via Unix domain socket. On non-Unix, falls back to
/// TCP on the port derived from the path hash (for testing).
///
/// # Errors
///
/// Returns [`IpcError::Connect`] on failure.
pub fn connect_path(path: &Path) -> Result<IpcStream, IpcError> {
    #[cfg(unix)]
    {
        let stream = std::os::unix::net::UnixStream::connect(path).map_err(IpcError::Connect)?;
        Ok(IpcStream::Unix(stream))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(IpcError::Connect(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unix sockets not available — use TCP endpoint or set HEALTHSPRING_*_SOCKET=tcp://host:port",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_unix_prefix() {
        let ep = parse_endpoint("unix:///tmp/biomeos/healthspring.sock");
        assert!(matches!(ep, Endpoint::Unix(p) if p.to_str().unwrap().contains("healthspring")));
    }

    #[test]
    fn parse_tcp_prefix() {
        let ep = parse_endpoint("tcp://127.0.0.1:9090");
        assert!(matches!(ep, Endpoint::Tcp(ref a) if a == "127.0.0.1:9090"));
    }

    #[test]
    fn parse_path_infers_unix() {
        let ep = parse_endpoint("/tmp/biomeos/healthspring-default.sock");
        assert!(matches!(ep, Endpoint::Unix(_)));
    }

    #[test]
    fn parse_host_port_infers_tcp() {
        let ep = parse_endpoint("localhost:8080");
        assert!(matches!(ep, Endpoint::Tcp(_)));
    }

    #[test]
    fn connect_nonexistent_unix() {
        let ep = Endpoint::Unix("/tmp/nonexistent_healthspring_test.sock".into());
        assert!(connect(&ep).is_err());
    }

    #[test]
    fn connect_nonexistent_tcp() {
        let ep = Endpoint::Tcp("127.0.0.1:1".to_owned());
        assert!(connect(&ep).is_err());
    }

    #[test]
    fn connect_path_nonexistent() {
        let result = connect_path(Path::new(
            "/tmp/nonexistent_healthspring_transport_test.sock",
        ));
        assert!(result.is_err());
    }
}
