use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ UrlInput, RegistrableDomainResult };
use std::net::{ IpAddr, Ipv6Addr };
use url::Url;

fn empty_result(error: &str) -> RegistrableDomainResult {
    RegistrableDomainResult {
        registrable_domain: String::new(),
        suffix: String::new(),
        is_icann: false,
        is_known: false,
        error: error.to_string(),
    }
}

/// True if `s` is an IP address literal — either bracketed IPv6
/// ("[2001:db8::1]", as `Url::host_str` serializes it) or bare IPv4/IPv6.
/// An IP literal is not a domain name: it must never reach the PSL lookup
/// below, which otherwise treats its dot- or colon-separated parts as
/// domain labels and reports a bogus suffix/registrable-domain (the PSL's
/// own "no match -> last label is a wildcard suffix" fallback rule matches
/// almost any dotted string, IP addresses included).
fn is_ip_literal(s: &str) -> bool {
    if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        return inner.parse::<Ipv6Addr>().is_ok();
    }
    s.parse::<IpAddr>().is_ok()
}

/// Extract the registrable domain (eTLD+1, e.g. "example.co.uk" from
/// "www.example.co.uk") and public suffix (e.g. "co.uk") from a URL or
/// bare hostname, using the Mozilla Public Suffix List compiled offline
/// into the `psl` crate at build time (no runtime list fetch). Reports
/// whether the matched suffix is an ICANN (delegated TLD) or private-
/// section entry, and whether the host has any recognized public suffix at
/// all. Accepts a full URL (the host is extracted) or a bare domain
/// string. An IP-literal host (IPv4 or IPv6) is not a domain name, so it
/// always yields an empty, unknown result rather than being run through
/// the public-suffix lookup. Malformed input returns a structured error.
pub fn registrable_domain(
    ax: &dyn AxiomContext,
    input: UrlInput,
) -> Result<RegistrableDomainResult, Box<dyn std::error::Error>> {
    let _ = ax;

    let raw = input.url.trim();
    if raw.is_empty() {
        return Ok(empty_result("EMPTY_INPUT"));
    }

    // Accept either a full URL (extract the host) or a bare hostname.
    // An IP-literal host short-circuits to an empty, non-error result: it
    // is a valid host but not a domain name, so no suffix/registrable-
    // domain concept applies.
    let host: String = match Url::parse(raw) {
        Ok(u) => match u.host() {
            Some(url::Host::Ipv4(_)) | Some(url::Host::Ipv6(_)) => return Ok(empty_result("")),
            Some(url::Host::Domain(h)) => h.to_string(),
            None => return Ok(empty_result("NO_HOST")),
        },
        Err(_) => {
            if is_ip_literal(raw) {
                return Ok(empty_result(""));
            }
            raw.to_string()
        }
    };
    if host.is_empty() {
        return Ok(empty_result("NO_HOST"));
    }
    // DNS/PSL matching is case-insensitive; the list itself is stored
    // lowercase, so normalize before lookup.
    let host_lower = host.to_lowercase();
    let bytes = host_lower.as_bytes();

    if let Some(d) = psl::domain(bytes) {
        let suffix = d.suffix();
        return Ok(RegistrableDomainResult {
            registrable_domain: String::from_utf8_lossy(d.as_bytes()).into_owned(),
            suffix: String::from_utf8_lossy(suffix.as_bytes()).into_owned(),
            is_icann: matches!(suffix.typ(), Some(psl::Type::Icann)),
            is_known: suffix.is_known(),
            error: String::new(),
        });
    }

    // No registrable domain (e.g. the host IS a bare public suffix, or an
    // unrecognized/unlisted host) — still report the suffix match if any.
    if let Some(s) = psl::suffix(bytes) {
        return Ok(RegistrableDomainResult {
            registrable_domain: String::new(),
            suffix: String::from_utf8_lossy(s.as_bytes()).into_owned(),
            is_icann: matches!(s.typ(), Some(psl::Type::Icann)),
            is_known: s.is_known(),
            error: String::new(),
        });
    }

    Ok(RegistrableDomainResult {
        registrable_domain: String::new(),
        suffix: String::new(),
        is_icann: false,
        is_known: false,
        error: String::new(),
    })
}
