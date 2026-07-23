# url-tools

Composable **URL parsing & manipulation** nodes for the
[Axiom](https://axiomide.com) marketplace, published as
`christiangeorgelucas/url-tools`. Parse a URL into components, normalize/
canonicalize it, resolve a relative reference against a base, get/add/set/
remove query parameters, percent-encode/decode, IDNA/punycode-encode and
-decode internationalized domains, validate, and extract the registrable
domain (eTLD+1) / public suffix.

Written in **Rust**, wrapping the WHATWG-URL-Standard-compliant
[`url`](https://github.com/servo/rust-url) crate (the reference parser
browsers converge on) plus its sibling `idna` and `percent-encoding` crates,
and the [`psl`](https://github.com/addr-rs/psl) crate for the Mozilla Public
Suffix List (compiled offline at `psl`'s own build time).

| Concern | Library | License |
|---|---|---|
| URL parsing, serialization, resolution, query manipulation | [`url`](https://github.com/servo/rust-url) | MIT OR Apache-2.0 |
| IDNA / punycode | [`idna`](https://github.com/servo/rust-url/tree/main/idna) | MIT OR Apache-2.0 |
| Percent-encoding | [`percent-encoding`](https://github.com/servo/rust-url/tree/main/percent_encoding) | MIT OR Apache-2.0 |
| Public Suffix List (registrable domain) | [`psl`](https://github.com/addr-rs/psl) | MIT OR Apache-2.0 |

Every node is **stateless**, **offline**, and **deterministic** — pure
string-in/string-out. **No node ever fetches, dereferences, or connects to a
URL**; this package only parses and manipulates the string. Size/resource
guarding (input bounds, DoS/bomb protection) is the Axiom platform's
concern, not this package's — every node here is a pure function.

## Nodes

| Node | Input → Output | Purpose |
|---|---|---|
| `ParseUrl` | `UrlInput` → `UrlComponents` | Full WHATWG breakdown: scheme, userinfo, host+kind, ports, path+segments, query+pairs, fragment, origin |
| `NormalizeUrl` | `UrlInput` → `UrlResult` | Canonicalize a URL string (case, default ports, dot-segments, encoding) |
| `JoinUrl` | `JoinInput` → `UrlResult` | Resolve a relative reference against a base URL (WHATWG relative resolution) |
| `GetQueryParams` | `UrlInput` → `QueryParams` | Extract all query key/value pairs, order- and duplicate-preserving |
| `AddQueryParam` | `QueryParamInput` → `UrlResult` | Append a query parameter (always adds, allows duplicates) |
| `SetQueryParam` | `QueryParamInput` → `UrlResult` | Replace every occurrence of a key with one new pair |
| `RemoveQueryParam` | `RemoveQueryParamInput` → `UrlResult` | Remove every pair matching a key |
| `PercentEncode` | `PercentEncodeInput` → `TextResult` | Percent-encode for a URL context (userinfo/fragment/path/query/component) |
| `PercentDecode` | `TextInput` → `TextResult` | Percent-decode, validated as UTF-8 |
| `IdnaEncode` | `IdnaInput` → `TextResult` | Unicode domain → punycode (ToASCII) |
| `IdnaDecode` | `IdnaInput` → `TextResult` | Punycode domain → Unicode (ToUnicode) |
| `ValidateUrl` | `UrlInput` → `ValidationResult` | Validate without needing the full component breakdown |
| `RegistrableDomain` | `UrlInput` → `RegistrableDomainResult` | eTLD+1 and public suffix, via the Mozilla Public Suffix List |

## PercentEncode component modes

Each named mode is produced by the **real `url` crate component setter** for
that context — never a hand-rolled encode table:

- **`userinfo`** / **`fragment`**: no substructure, the whole value is
  encoded as-is (`Url::set_username` / `Url::set_fragment`).
- **`path`** / **`query`**: the value is treated as a **whole** path/query,
  so `/` (path) and `&`/`=` (query) are left un-escaped, and a `path` result
  always begins with `/`. Use `component` for a single segment/value that may
  itself contain those characters.
- **`component`** (default): a generic, maximally-conservative set — every
  byte except ASCII letters, digits, and `-_.~` — via
  `percent_encoding::NON_ALPHANUMERIC`.

## Error contract

Malformed input never crashes a node — it returns a stable, machine-readable
`error` token with empty result fields: `EMPTY_URL`, `PARSE_ERROR`,
`BASE_PARSE_ERROR`, `REFERENCE_PARSE_ERROR`, `EMPTY_KEY`,
`UNKNOWN_COMPONENT`, `ENCODE_ERROR`, `INVALID_UTF8`, `EMPTY_DOMAIN`,
`IDNA_ERROR`, `EMPTY_INPUT`, `NO_HOST`. `ValidateUrl` is the one exception by
design: an invalid URL is a **normal** result (`valid=false`), not an error
path fault.

**proto3 JSON note:** default scalar values (`false`, `""`, `0`) are omitted
from the JSON emitted over the HTTP bridge, so a consumer must treat a
missing `error` as success and a missing field as its zero value.

## Correctness

The test suite (`axiom test`, 81 tests) enforces every accuracy claim with
**independent oracles** — published, external reference values that do not
go through this code:

- **`ParseUrl` / `NormalizeUrl`** — the WHATWG URL Standard's own worked
  component example, and RFC 3986 §5.2.4 dot-segment resolution.
- **`JoinUrl`** — the canonical RFC 3986 §5.4.1/5.4.2 "Normal" and
  "Abnormal" relative-resolution test vectors (base `http://a/b/c/d;p?q`),
  verbatim from the published RFC.
- **`GetQueryParams` / `AddQueryParam`** — the `application/x-www-form-
  urlencoded` spec's own space-vs-plus decoding rule.
- **`PercentEncode`** — RFC 3986 §2.3's unreserved-character set, and the
  WHATWG userinfo/fragment percent-encode sets (why `:`/`@` are escaped in
  userinfo, why `#` is *not* escaped in a fragment).
- **`PercentDecode`** — the ASCII table and the fixed UTF-8 byte sequence for
  a known Unicode codepoint (U+2713).
- **`IdnaEncode` / `IdnaDecode`** — the textbook "münchen.de" ↔
  "xn--mnchen-3ya.de" Punycode worked example (RFC 3492).
- **`RegistrableDomain`** — published Mozilla Public Suffix List entries
  (`co.uk` ICANN-section, `github.io` private-section), independently
  verifiable at [publicsuffix.org](https://publicsuffix.org).

## Composability

`JoinUrl` → `ParseUrl` → `RegistrableDomain` chains cleanly: `JoinUrl.url`
feeds `ParseUrl.url`, and `ParseUrl.host` feeds `RegistrableDomain.url`
(which accepts a bare host as well as a full URL). A runnable proof flow
ships at `flows/join-parse-domain.flow.yaml`:

```bash
axiom flow compile flows/join-parse-domain.flow.yaml
axiom flow run <artifact-id> -d '{"base":"https://www.example.co.uk/articles/","relative":"../about"}'
# → {"registrableDomain":"example.co.uk","suffix":"co.uk","isIcann":true,"isKnown":true}
```

## Security notes (honest edges)

- **No network access.** No node fetches, resolves DNS for, or otherwise
  dereferences a URL — every node is pure parsing/manipulation of the input
  string.
- **`RegistrableDomain`** accepts either a full URL (the host is extracted)
  or a bare hostname. It does **not** strip a `host:port` suffix passed
  without a scheme (e.g. `"example.com:8080"` alone parses as a URL with
  scheme `"example.com"`, not a host+port) — pass a full URL or a bare host.
- **`PercentEncode`'s `path`/`query` modes** intentionally leave `/`
  (path) and `&`/`=` (query) un-escaped, matching what setting a real URL's
  whole path/query does. Use `component` mode when embedding a single
  segment/value that may itself contain those characters.

## Development

```bash
axiom validate     # static checks
axiom test         # unit tests (independent-oracle checks + error paths)
axiom dev          # local HTTP bridge (prints the port it binds)
```

## License

MIT — © 2026 Christian George Lucas. Built for the Axiom marketplace. All
wrapped crates are permissively licensed; see `THIRD_PARTY_NOTICES.md`.
