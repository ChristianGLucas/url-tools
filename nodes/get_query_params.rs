use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ UrlInput, QueryParams, QueryParam };

#[path = "urlutil.rs"]
mod urlutil;

/// Extract every query-string key/value pair from a URL, in original order,
/// with application/x-www-form-urlencoded decoding applied (percent-
/// decoding plus '+' as space). A repeated key (e.g. "?a=1&a=2") yields two
/// separate entries, not a merged list. Malformed input returns a
/// structured error.
pub fn get_query_params(
    ax: &dyn AxiomContext,
    input: UrlInput,
) -> Result<QueryParams, Box<dyn std::error::Error>> {
    let _ = ax;
    match urlutil::parse_bounded(&input.url) {
        Ok(u) => {
            let pairs = urlutil::query_pairs_vec(&u)
                .into_iter()
                .map(|(key, value)| QueryParam { key, value })
                .collect();
            Ok(QueryParams { pairs, error: String::new() })
        }
        Err(e) => Ok(QueryParams { pairs: vec![], error: e.to_string() }),
    }
}
