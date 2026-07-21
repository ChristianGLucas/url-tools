use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ UrlInput, RegistrableDomainResult };
use url::Url;

#[path = "urlutil.rs"]
mod urlutil;

fn empty_result(error: &str) -> RegistrableDomainResult {
    RegistrableDomainResult {
        registrable_domain: String::new(),
        suffix: String::new(),
        is_icann: false,
        is_known: false,
        error: error.to_string(),
    }
}

/// Extract the registrable domain (eTLD+1, e.g. "example.co.uk" from
/// "www.example.co.uk") and public suffix (e.g. "co.uk") from a URL or
/// bare hostname, using the Mozilla Public Suffix List compiled offline
/// into the `psl` crate at build time (no runtime list fetch). Reports
/// whether the matched suffix is an ICANN (delegated TLD) or private-
/// section entry, and whether the host has any recognized public suffix at
/// all. Accepts a full URL (the host is extracted) or a bare domain
/// string. Malformed input returns a structured error.
pub fn registrable_domain(
    ax: &dyn AxiomContext,
    input: UrlInput,
) -> Result<RegistrableDomainResult, Box<dyn std::error::Error>> {
    let _ = ax;

    let raw = input.url.trim();
    if raw.is_empty() {
        return Ok(empty_result("EMPTY_INPUT"));
    }
    if raw.len() > urlutil::MAX_URL_LEN {
        return Ok(empty_result("INPUT_TOO_LARGE"));
    }

    // Accept either a full URL (extract the host) or a bare hostname.
    let host: String = match Url::parse(raw) {
        Ok(u) => match u.host_str() {
            Some(h) => h.to_string(),
            None => return Ok(empty_result("NO_HOST")),
        },
        Err(_) => raw.to_string(),
    };
    if host.is_empty() {
        return Ok(empty_result("NO_HOST"));
    }
    // DNS/PSL matching is case-insensitive; the list itself is stored
    // lowercase, so normalize before lookup.
    let host_lower = host.to_lowercase();
    let bytes = host_lower.as_bytes();

    if let Some(d) = psl::domain(bytes) {
        let suffix = d.suffix();
        return Ok(RegistrableDomainResult {
            registrable_domain: String::from_utf8_lossy(d.as_bytes()).into_owned(),
            suffix: String::from_utf8_lossy(suffix.as_bytes()).into_owned(),
            is_icann: matches!(suffix.typ(), Some(psl::Type::Icann)),
            is_known: suffix.is_known(),
            error: String::new(),
        });
    }

    // No registrable domain (e.g. the host IS a bare public suffix, or an
    // unrecognized/unlisted host) — still report the suffix match if any.
    if let Some(s) = psl::suffix(bytes) {
        return Ok(RegistrableDomainResult {
            registrable_domain: String::new(),
            suffix: String::from_utf8_lossy(s.as_bytes()).into_owned(),
            is_icann: matches!(s.typ(), Some(psl::Type::Icann)),
            is_known: s.is_known(),
            error: String::new(),
        });
    }

    Ok(RegistrableDomainResult {
        registrable_domain: String::new(),
        suffix: String::new(),
        is_icann: false,
        is_known: false,
        error: String::new(),
    })
}
