use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ PercentEncodeInput, TextResult };
use percent_encoding::{ utf8_percent_encode, NON_ALPHANUMERIC };
use url::Url;

#[path = "urlutil.rs"]
mod urlutil;

/// Percent-encode a text value for safe inclusion in a URL, using the real
/// WHATWG encode set for the requested context, each produced by the
/// actual `url` crate component setter (not a hand-rolled table): so the
/// exact bytes encoded match what constructing a real URL with that value
/// would produce.
///
/// - "userinfo" / "fragment": no substructure, the whole value is encoded
///   as-is (via `Url::set_username` / `Url::set_fragment`).
/// - "path" / "query": the value is treated as a WHOLE path/query (via
///   `Url::set_path` / `Url::set_query`), so '/' (path) and '&'/'='
///   (query) are left un-escaped — use "component" for a single
///   segment/value that may itself contain those characters.
/// - "component" (default): a generic, maximally-conservative set — every
///   byte except ASCII letters, digits, and "-_.~" — via
///   `percent_encoding::NON_ALPHANUMERIC`.
///
/// Malformed input returns a structured error.
pub fn percent_encode(
    ax: &dyn AxiomContext,
    input: PercentEncodeInput,
) -> Result<TextResult, Box<dyn std::error::Error>> {
    let _ = ax;
    let err = |e: &str| TextResult { value: String::new(), error: e.to_string() };

    if input.value.len() > urlutil::MAX_TEXT_LEN {
        return Ok(err("INPUT_TOO_LARGE"));
    }

    let component = if input.component.is_empty() {
        "component".to_string()
    } else {
        input.component.to_lowercase()
    };

    // A fixed scratch URL used purely as a carrier for the real component
    // setters below — never fetched or otherwise dereferenced.
    let scratch = || Url::parse("http://x/").expect("static scratch URL is valid");

    let encoded = match component.as_str() {
        "component" => utf8_percent_encode(&input.value, NON_ALPHANUMERIC).to_string(),
        "userinfo" => {
            let mut u = scratch();
            if u.set_username(&input.value).is_err() {
                return Ok(err("ENCODE_ERROR"));
            }
            u.username().to_string()
        }
        "fragment" => {
            let mut u = scratch();
            u.set_fragment(Some(&input.value));
            u.fragment().unwrap_or("").to_string()
        }
        "path" => {
            let mut u = scratch();
            u.set_path(&input.value);
            u.path().to_string()
        }
        "query" => {
            let mut u = scratch();
            u.set_query(Some(&input.value));
            u.query().unwrap_or("").to_string()
        }
        _ => return Ok(err("UNKNOWN_COMPONENT")),
    };

    Ok(TextResult { value: encoded, error: String::new() })
}
