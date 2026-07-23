use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ QueryParamInput, UrlResult };

#[path = "urlutil.rs"]
mod urlutil;

/// Append a query parameter to a URL, always adding a new "key=value" pair
/// even if the key already exists (use SetQueryParam to replace instead).
/// The key and value are percent-encoded per the WHATWG query encode set.
/// Returns the resulting URL string. Malformed input returns a structured
/// error.
pub fn add_query_param(
    ax: &dyn AxiomContext,
    input: QueryParamInput,
) -> Result<UrlResult, Box<dyn std::error::Error>> {
    let _ = ax;
    let err = |e: &str| UrlResult { url: String::new(), error: e.to_string() };

    if input.key.is_empty() {
        return Ok(err("EMPTY_KEY"));
    }
    let mut url = match urlutil::parse_bounded(&input.url) {
        Ok(u) => u,
        Err(e) => return Ok(err(e)),
    };
    url.query_pairs_mut().append_pair(&input.key, &input.value);
    Ok(UrlResult { url: url.to_string(), error: String::new() })
}
