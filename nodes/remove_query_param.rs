use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ RemoveQueryParamInput, UrlResult };

#[path = "urlutil.rs"]
mod urlutil;

/// Remove every query-string pair matching a given key from a URL, leaving
/// other parameters (including other keys and any duplicate-key entries
/// that don't match) untouched, in their original order. Returns the
/// resulting URL string. Malformed input returns a structured error.
pub fn remove_query_param(
    ax: &dyn AxiomContext,
    input: RemoveQueryParamInput,
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

    let remaining: Vec<(String, String)> = urlutil::query_pairs_vec(&url)
        .into_iter()
        .filter(|(k, _)| k != &input.key)
        .collect();

    if remaining.is_empty() {
        url.set_query(None);
    } else {
        let mut qp = url.query_pairs_mut();
        qp.clear();
        for (k, v) in &remaining {
            qp.append_pair(k, v);
        }
    }

    Ok(UrlResult { url: url.to_string(), error: String::new() })
}
