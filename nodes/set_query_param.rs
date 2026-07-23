use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ QueryParamInput, UrlResult };

#[path = "urlutil.rs"]
mod urlutil;

/// Set a query parameter on a URL, first removing every existing occurrence
/// of the key and then appending exactly one "key=value" pair (use
/// AddQueryParam to append without removing duplicates). Returns the
/// resulting URL string. Malformed input returns a structured error.
pub fn set_query_param(
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

    let remaining: Vec<(String, String)> = urlutil::query_pairs_vec(&url)
        .into_iter()
        .filter(|(k, _)| k != &input.key)
        .collect();

    {
        // The url crate's query-pairs serializer only writes back to the
        // URL when dropped, so this must stay scoped before `url.to_string()`.
        let mut qp = url.query_pairs_mut();
        qp.clear();
        for (k, v) in &remaining {
            qp.append_pair(k, v);
        }
        qp.append_pair(&input.key, &input.value);
    }

    Ok(UrlResult { url: url.to_string(), error: String::new() })
}
