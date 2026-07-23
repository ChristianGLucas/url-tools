use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ JoinInput, UrlResult };
use url::Url;

/// Resolve a (possibly relative) reference against an absolute base URL,
/// per the WHATWG URL relative-resolution algorithm — exactly how a browser
/// resolves an `<a href>` against the current page or a Location header
/// against the request URL. Handles relative paths ("../x", "./x"),
/// absolute paths ("/x"), query-only and fragment-only references
/// ("?x=1", "#frag"), protocol-relative references ("//other.example/x"),
/// and an already-absolute reference (returned normalized, ignoring the
/// base). Malformed base or unresolvable reference returns a structured
/// error.
pub fn join_url(
    ax: &dyn AxiomContext,
    input: JoinInput,
) -> Result<UrlResult, Box<dyn std::error::Error>> {
    let _ = ax;
    let err = |e: &str| UrlResult { url: String::new(), error: e.to_string() };

    if input.base.is_empty() {
        return Ok(err("EMPTY_BASE"));
    }
    let base = match Url::parse(&input.base) {
        Ok(u) => u,
        Err(_) => return Ok(err("BASE_PARSE_ERROR")),
    };
    match base.join(&input.relative) {
        Ok(joined) => Ok(UrlResult { url: joined.to_string(), error: String::new() }),
        Err(_) => Ok(err("REFERENCE_PARSE_ERROR")),
    }
}
