// Shared helpers for christiangeorgelucas/url-tools nodes. Not a node itself
// — pulled in via `#[path = "urlutil.rs"] mod urlutil;` in each node file
// that needs it (mirrors the hashutil.rs pattern used elsewhere).

use url::Url;

/// Generous superset of practical URL length limits (browsers commonly cap
/// around 2000-8000 bytes for interoperability). Enforced BEFORE any parsing
/// is attempted, so a pathological input never reaches the parser.
pub const MAX_URL_LEN: usize = 8192;

/// Bound for arbitrary text inputs to the percent-encode/decode and IDNA
/// nodes (these are not full URLs, but still bounded to avoid unbounded
/// allocation on adversarial input).
pub const MAX_TEXT_LEN: usize = 65536;

/// Parse a URL string, enforcing the length bound first. Returns a stable
/// error token on any failure; never panics.
pub fn parse_bounded(input: &str) -> Result<Url, &'static str> {
    if input.is_empty() {
        return Err("EMPTY_URL");
    }
    if input.len() > MAX_URL_LEN {
        return Err("INPUT_TOO_LARGE");
    }
    Url::parse(input).map_err(|_| "PARSE_ERROR")
}

/// The WHATWG host kind as a stable string token.
pub fn host_type(url: &Url) -> &'static str {
    match url.host() {
        Some(url::Host::Domain(_)) => "domain",
        Some(url::Host::Ipv4(_)) => "ipv4",
        Some(url::Host::Ipv6(_)) => "ipv6",
        None => "",
    }
}

/// The URL's serialized tuple origin, or "null" for an opaque origin.
pub fn origin_string(url: &Url) -> String {
    url.origin().ascii_serialization()
}

/// The URL's query string parsed into owned (key, value) pairs, in order,
/// with duplicates preserved.
pub fn query_pairs_vec(url: &Url) -> Vec<(String, String)> {
    url.query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect()
}
