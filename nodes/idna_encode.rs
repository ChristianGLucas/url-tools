use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ IdnaInput, TextResult };

/// Convert an internationalized (Unicode) domain name to its ASCII-
/// compatible punycode form (IDNA ToASCII, e.g. "münchen.de" to
/// "xn--mnchen-3ya.de"), using the same `idna` crate the `url` crate uses
/// internally for host processing. An already-ASCII domain is validated
/// and returned normalized (lowercased). This uses the WHATWG-default,
/// lenient ToASCII processing (no STD3/length enforcement) — the one clear
/// error case is a label already prefixed "xn--" whose punycode does not
/// decode (malformed existing punycode), which returns a structured error.
pub fn idna_encode(
    ax: &dyn AxiomContext,
    input: IdnaInput,
) -> Result<TextResult, Box<dyn std::error::Error>> {
    let _ = ax;
    if input.domain.is_empty() {
        return Ok(TextResult { value: String::new(), error: "EMPTY_DOMAIN".to_string() });
    }
    match idna::domain_to_ascii(&input.domain) {
        Ok(ascii) => Ok(TextResult { value: ascii, error: String::new() }),
        Err(_) => Ok(TextResult { value: String::new(), error: "IDNA_ERROR".to_string() }),
    }
}
