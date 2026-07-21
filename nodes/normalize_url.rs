use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ UrlInput, UrlResult };

#[path = "urlutil.rs"]
mod urlutil;

/// Canonicalize a URL string via the WHATWG URL parsing and serialization
/// algorithm — lowercases the scheme and domain host, IDNA/punycode-encodes
/// an internationalized domain, percent-encodes each component per its
/// WHATWG encode set, resolves "." and ".." path segments, and omits a port
/// that matches the scheme's default. Returns just the normalized URL
/// string (use ParseUrl for the full component breakdown). Malformed input
/// returns a structured error.
pub fn normalize_url(
    ax: &dyn AxiomContext,
    input: UrlInput,
) -> Result<UrlResult, Box<dyn std::error::Error>> {
    let _ = ax;
    match urlutil::parse_bounded(&input.url) {
        Ok(u) => Ok(UrlResult { url: u.to_string(), error: String::new() }),
        Err(e) => Ok(UrlResult { url: String::new(), error: e.to_string() }),
    }
}
