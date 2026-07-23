use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ UrlInput, UrlComponents, QueryParam };

#[path = "urlutil.rs"]
mod urlutil;

/// Parse a URL string into its full WHATWG breakdown (scheme, userinfo,
/// host + kind, ports, path + segments, query + parsed pairs, fragment,
/// serialized origin, cannot-be-a-base) plus the re-serialized, normalized
/// URL itself. Wraps the servo/rust-url crate — the WHATWG-URL-Standard
/// reference parser. Malformed input returns a structured `error` token
/// (EMPTY_URL, PARSE_ERROR) with every other field at its zero value,
/// never a crash.
pub fn parse_url(
    ax: &dyn AxiomContext,
    input: UrlInput,
) -> Result<UrlComponents, Box<dyn std::error::Error>> {
    let _ = ax;

    let url = match urlutil::parse_bounded(&input.url) {
        Ok(u) => u,
        Err(e) => {
            return Ok(UrlComponents { error: e.to_string(), ..Default::default() });
        }
    };

    let query_pairs = urlutil::query_pairs_vec(&url)
        .into_iter()
        .map(|(key, value)| QueryParam { key, value })
        .collect();

    let path_segments: Vec<String> = url
        .path_segments()
        .map(|segs| segs.map(|s| s.to_string()).collect())
        .unwrap_or_default();

    Ok(UrlComponents {
        url: url.to_string(),
        scheme: url.scheme().to_string(),
        username: url.username().to_string(),
        password: url.password().unwrap_or("").to_string(),
        has_password: url.password().is_some(),
        host: url.host_str().unwrap_or("").to_string(),
        host_type: urlutil::host_type(&url).to_string(),
        has_port: url.port().is_some(),
        port: url.port().unwrap_or(0) as u32,
        effective_port: url.port_or_known_default().unwrap_or(0) as u32,
        path: url.path().to_string(),
        path_segments,
        query: url.query().unwrap_or("").to_string(),
        has_query: url.query().is_some(),
        query_pairs,
        fragment: url.fragment().unwrap_or("").to_string(),
        has_fragment: url.fragment().is_some(),
        origin: urlutil::origin_string(&url),
        cannot_be_a_base: url.cannot_be_a_base(),
        error: String::new(),
    })
}
