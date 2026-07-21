use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ UrlInput, ValidationResult };

#[path = "urlutil.rs"]
mod urlutil;

/// Validate a URL string against the WHATWG URL parsing algorithm without
/// requiring the caller to handle a full component breakdown. Returns
/// valid=true plus the normalized URL on success, or valid=false plus a
/// structured error token on failure. Never throws — an invalid URL is a
/// normal (valid=false) result, not an error path fault.
pub fn validate_url(
    ax: &dyn AxiomContext,
    input: UrlInput,
) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    let _ = ax;
    match urlutil::parse_bounded(&input.url) {
        Ok(u) => Ok(ValidationResult { valid: true, url: u.to_string(), error: String::new() }),
        Err(e) => Ok(ValidationResult { valid: false, url: String::new(), error: e.to_string() }),
    }
}
