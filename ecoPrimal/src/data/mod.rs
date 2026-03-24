// SPDX-License-Identifier: AGPL-3.0-or-later
//! Data provider: three-tier fetch (`biomeOS` → `NestGate` → direct HTTP).
//!
//! Follows the wetSpring pattern for transparent data access with provenance.
//! Behind the `nestgate` feature gate for network operations; local JSON
//! loading is always available.
//!
//! ## Three-tier architecture
//!
//! ```text
//! Tier 1: biomeOS capability.call  → orchestrator routes to data provider
//! Tier 2: NestGate cache           → content-addressed store/retrieve
//! Tier 3: Direct NCBI HTTP        → fallback, no caching
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use healthspring_barracuda::data::{NcbiProvider, DataError};
//!
//! fn example() -> Result<(), DataError> {
//!     let provider = NcbiProvider::discover();
//!     let result = provider.fetch("gene", "12345")?;
//!     Ok(())
//! }
//! ```

mod discovery;
mod fetch;
#[cfg(feature = "nestgate")]
pub mod ncbi_http;
pub mod provenance;
mod rpc;
mod storage;

pub use discovery::{discover_biomeos_socket, discover_data_provider_socket, is_enabled};
pub use fetch::{NcbiProvider, fetch_tiered};
pub use provenance::{
    DataProvenanceChain, ProvenanceResult, begin_data_session, complete_data_session,
    record_fetch_step, trio_available,
};
pub use rpc::{RpcError, rpc_call};
pub use storage::{content_key, exists_local, local_cache_path, store_local};

/// Errors from the data provider.
#[derive(Debug)]
pub enum DataError {
    /// No `NestGate` or `biomeOS` socket found.
    SocketNotFound,
    /// RPC call failed.
    Rpc(String),
    /// NCBI HTTP returned a non-200 status.
    NcbiHttp {
        /// HTTP status code
        status: u16,
        /// Request URL
        url: String,
    },
    /// Response parse failure.
    Parse(String),
    /// Local storage I/O error.
    Io(std::io::Error),
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SocketNotFound => write!(f, "data provider socket not found"),
            Self::Rpc(msg) => write!(f, "data provider RPC error: {msg}"),
            Self::NcbiHttp { status, url } => {
                write!(f, "NCBI HTTP error: {status} for {url}")
            }
            Self::Parse(msg) => write!(f, "Parse error: {msg}"),
            Self::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl std::error::Error for DataError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DataError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_key_format() {
        assert_eq!(content_key("gene", "12345"), "ncbi:gene:12345");
        assert_eq!(content_key("pubmed", "33456789"), "ncbi:pubmed:33456789");
    }

    #[test]
    fn data_error_display() {
        let e = DataError::SocketNotFound;
        assert!(e.to_string().contains("socket not found"));

        let e = DataError::NcbiHttp {
            status: 429,
            url: "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi".into(),
        };
        assert!(e.to_string().contains("429"));
    }

    #[test]
    fn discover_returns_option() {
        let biomeos = discover_biomeos_socket();
        let data = discover_data_provider_socket();
        drop((biomeos, data));
    }

    #[test]
    fn local_cache_path_deterministic() {
        let p1 = local_cache_path("gene", "12345");
        let p2 = local_cache_path("gene", "12345");
        assert_eq!(p1, p2, "cache path must be deterministic");
    }
}
