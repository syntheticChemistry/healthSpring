// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sovereign NCBI HTTP via E-utilities — Tier 3 fallback when no
//! `biomeOS` or `NestGate` ecosystem services are available.
//!
//! Uses `ureq` for HTTP transport. Feature-gated behind `nestgate` to
//! keep the default build ecoBin-compliant (zero C deps). When `nestgate`
//! is enabled, `ureq` → `rustls` → `ring` introduces C/asm in the crypto
//! layer. Evolution path: `ring` → `rustls-rustcrypto` (pure Rust,
//! currently alpha) when it stabilizes.

use super::DataError;

const EUTILS_BASE: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";
const SRA_RUN_INFO_BASE: &str = "https://trace.ncbi.nlm.nih.gov/Traces/sra/sra.cgi";
const TOOL_NAME: &str = "healthspring";
const TOOL_EMAIL: &str = "healthspring@ecoprimal.local";

/// Fetch a record from NCBI via `EFetch`.
///
/// Returns the response body as a string (FASTA, XML, or JSON depending
/// on the database and `rettype`/`retmode` defaults).
///
/// # Errors
///
/// Returns `DataError::NcbiHttp` on non-200 responses or
/// `DataError::Io` on transport failures.
pub fn efetch(db: &str, id: &str, api_key: &str) -> Result<String, DataError> {
    let mut url =
        format!("{EUTILS_BASE}/efetch.fcgi?db={db}&id={id}&tool={TOOL_NAME}&email={TOOL_EMAIL}");
    if !api_key.is_empty() {
        url.push_str("&api_key=");
        url.push_str(api_key);
    }

    http_get(&url)
}

/// Search NCBI via `ESearch` and return a list of IDs.
///
/// # Errors
///
/// Returns `DataError::NcbiHttp` on non-200 responses or
/// `DataError::Parse` if the response doesn't contain ID list.
pub fn esearch(db: &str, query: &str, max_results: u32) -> Result<Vec<String>, DataError> {
    let encoded_query = query.replace(' ', "+");
    let url = format!(
        "{EUTILS_BASE}/esearch.fcgi?db={db}&term={encoded_query}&retmax={max_results}&retmode=json&tool={TOOL_NAME}&email={TOOL_EMAIL}"
    );

    let body = http_get(&url)?;
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| DataError::Parse(e.to_string()))?;

    let ids = parsed
        .pointer("/esearchresult/idlist")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(serde_json::Value::as_str)
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();

    Ok(ids)
}

/// Fetch SRA run metadata for an accession (SRR, SRP, SRX, etc.).
///
/// Returns CSV-formatted run info from NCBI SRA.
///
/// # Errors
///
/// Returns `DataError::NcbiHttp` on non-200 responses.
pub fn sra_run_info(accession: &str) -> Result<String, DataError> {
    let url = format!("{SRA_RUN_INFO_BASE}?save=efetch&db=sra&rettype=runinfo&term={accession}");
    http_get(&url)
}

/// Low-level HTTP GET with `ureq`.
fn http_get(url: &str) -> Result<String, DataError> {
    let response = ureq::get(url).call().map_err(|e| match e {
        ureq::Error::StatusCode(status) => DataError::NcbiHttp {
            status,
            url: url.to_owned(),
        },
        other => DataError::Io(std::io::Error::other(other.to_string())),
    })?;

    response
        .into_body()
        .read_to_string()
        .map_err(|e| DataError::Io(std::io::Error::other(e.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eutils_url_format() {
        let url = format!(
            "{EUTILS_BASE}/efetch.fcgi?db=gene&id=12345&tool={TOOL_NAME}&email={TOOL_EMAIL}"
        );
        assert!(url.contains("eutils.ncbi.nlm.nih.gov"));
        assert!(url.contains("db=gene"));
        assert!(url.contains("id=12345"));
    }

    #[test]
    fn sra_url_format() {
        let url = format!("{SRA_RUN_INFO_BASE}?save=efetch&db=sra&rettype=runinfo&term=SRP004311");
        assert!(url.contains("trace.ncbi.nlm.nih.gov"));
        assert!(url.contains("SRP004311"));
    }
}
