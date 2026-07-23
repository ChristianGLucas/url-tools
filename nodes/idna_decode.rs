use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ IdnaInput, TextResult };

/// Convert a punycode/ASCII-compatible domain name back to its Unicode
/// form (IDNA ToUnicode, e.g. "xn--mnchen-3ya.de" to "münchen.de"), using
/// the same `idna` crate the `url` crate uses internally for host
/// processing. A domain with no punycode labels is returned unchanged.
/// Malformed punycode returns a structured error.
pub fn idna_decode(
    ax: &dyn AxiomContext,
    input: IdnaInput,
) -> Result<TextResult, Box<dyn std::error::Error>> {
    let _ = ax;
    if input.domain.is_empty() {
        return Ok(TextResult { value: String::new(), error: "EMPTY_DOMAIN".to_string() });
    }
    let (unicode, result) = idna::domain_to_unicode(&input.domain);
    match result {
        Ok(()) => Ok(TextResult { value: unicode, error: String::new() }),
        Err(_) => Ok(TextResult { value: String::new(), error: "IDNA_ERROR".to_string() }),
    }
}
