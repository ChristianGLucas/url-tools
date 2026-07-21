use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ TextInput, TextResult };
use percent_encoding::percent_decode_str;

#[path = "urlutil.rs"]
mod urlutil;

/// Percent-decode a text value (reverse of PercentEncode) and validate the
/// result as UTF-8. Bytes with no percent-escape pass through unchanged.
/// Input that decodes to invalid UTF-8 returns a structured error
/// (INVALID_UTF8) rather than silently lossy-substituting replacement
/// characters.
pub fn percent_decode(
    ax: &dyn AxiomContext,
    input: TextInput,
) -> Result<TextResult, Box<dyn std::error::Error>> {
    let _ = ax;
    if input.value.len() > urlutil::MAX_TEXT_LEN {
        return Ok(TextResult { value: String::new(), error: "INPUT_TOO_LARGE".to_string() });
    }
    match percent_decode_str(&input.value).decode_utf8() {
        Ok(s) => Ok(TextResult { value: s.into_owned(), error: String::new() }),
        Err(_) => Ok(TextResult { value: String::new(), error: "INVALID_UTF8".to_string() }),
    }
}
