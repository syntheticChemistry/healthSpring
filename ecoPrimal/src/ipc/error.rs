// SPDX-License-Identifier: AGPL-3.0-or-later
//! Structured IPC error types for cross-primal communication.
//!
//! Follows the airSpring/groundSpring pattern for consistent error handling
//! across the ecoPrimals ecosystem.

use thiserror::Error;

/// Errors during IPC communication with other primals.
#[derive(Debug, Error)]
pub enum IpcError {
    /// No socket path found for the requested primal.
    #[error("socket not found for primal: {0}")]
    SocketNotFound(String),

    /// Low-level I/O failure while connecting.
    #[error("connection failed: {0}")]
    Connect(#[from] std::io::Error),

    /// Failure while writing to the transport.
    #[error("write failed: {0}")]
    Write(String),

    /// Failure while reading from the transport.
    #[error("read failed: {0}")]
    Read(String),

    /// Operation exceeded the configured timeout (milliseconds).
    #[error("timeout after {0}ms")]
    Timeout(u64),

    /// JSON encode/decode error on the wire.
    #[error("invalid JSON: {0}")]
    Codec(#[from] serde_json::Error),

    /// JSON-RPC error object returned by the peer.
    #[error("RPC error {code}: {message}")]
    RpcReject {
        /// JSON-RPC error code.
        code: i64,
        /// Error message from the peer.
        message: String,
    },

    /// Peer returned an empty payload where a body was required.
    #[error("empty response from primal")]
    EmptyResponse,
}

impl IpcError {
    /// Whether this error is retriable (transient failure).
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::Connect(_) | Self::Write(_) | Self::Read(_) | Self::Timeout(_)
        )
    }

    /// Whether this error likely indicates a timeout.
    #[must_use]
    pub const fn is_timeout_likely(&self) -> bool {
        matches!(self, Self::Timeout(_) | Self::Read(_))
    }

    /// Whether the remote method was not found.
    #[must_use]
    pub const fn is_method_not_found(&self) -> bool {
        matches!(self, Self::RpcReject { code, .. } if *code == -32601)
    }

    /// Whether the error is a connection-level failure.
    #[must_use]
    pub const fn is_connection_error(&self) -> bool {
        matches!(self, Self::Connect(_) | Self::SocketNotFound(_))
    }
}

/// Result classification for RPC responses (groundSpring/wetSpring pattern).
///
/// Distinguishes protocol-level (transport, codec) errors that may be retried
/// from application-level errors (invalid params, not found) that should not.
#[derive(Debug)]
pub enum DispatchOutcome<T> {
    /// Operation completed successfully.
    Success(T),
    /// Protocol-level error (transport, codec) — retriable.
    Protocol(IpcError),
    /// Application-level error (invalid params, not found) — not retriable.
    Application(IpcError),
}

impl<T> DispatchOutcome<T> {
    /// Whether this outcome should be retried.
    #[must_use]
    pub const fn should_retry(&self) -> bool {
        matches!(self, Self::Protocol(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_not_found_is_connection_error() {
        let err = IpcError::SocketNotFound("toadstool".into());
        assert!(err.is_connection_error());
    }

    #[test]
    fn connect_is_retriable() {
        let err = IpcError::Connect(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "refused",
        ));
        assert!(err.is_retriable());
    }

    #[test]
    fn timeout_is_retriable_and_timeout_likely() {
        let err = IpcError::Timeout(5000);
        assert!(err.is_retriable());
        assert!(err.is_timeout_likely());
    }

    #[test]
    fn rpc_reject_method_not_found() {
        let err = IpcError::RpcReject {
            code: -32601,
            message: "method not found".into(),
        };
        assert!(err.is_method_not_found());
        assert!(!err.is_retriable());
    }

    #[test]
    fn dispatch_outcome_protocol_should_retry() {
        let outcome = DispatchOutcome::<()>::Protocol(IpcError::Timeout(1000));
        assert!(outcome.should_retry());
    }

    #[test]
    fn dispatch_outcome_application_should_not_retry() {
        let outcome = DispatchOutcome::<()>::Application(IpcError::RpcReject {
            code: -32601,
            message: "method not found".into(),
        });
        assert!(!outcome.should_retry());
    }

    #[test]
    fn dispatch_outcome_success_should_not_retry() {
        let outcome = DispatchOutcome::Success(42);
        assert!(!outcome.should_retry());
    }
}
