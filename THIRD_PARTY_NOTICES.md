# Third-party notices

`url-tools` is MIT-licensed (© 2026 Christian George Lucas). It builds on the
open-source Rust crates below — the WHATWG-URL-Standard reference parser
(servo/rust-url) and its own dependency tree, plus the `psl` crate for the
Mozilla Public Suffix List. The full transitive dependency tree was resolved
via the crates.io registry metadata and independently verified to be entirely
permissive (no copyleft anywhere in the tree).

## Primary libraries

| Crate | Role | License |
|---|---|---|
| [`url`](https://github.com/servo/rust-url) | WHATWG URL Standard parsing, serialization, resolution (`join`), query manipulation | MIT OR Apache-2.0 |
| [`idna`](https://github.com/servo/rust-url/tree/main/idna) | IDNA/UTS46 domain-to-ASCII (punycode) / domain-to-Unicode conversion, used internally by `url` for host processing | MIT OR Apache-2.0 |
| [`percent-encoding`](https://github.com/servo/rust-url/tree/main/percent_encoding) | Percent-encode/decode primitives, used internally by `url` and directly by this package | MIT OR Apache-2.0 |
| [`form_urlencoded`](https://github.com/servo/rust-url/tree/main/form_urlencoded) | `application/x-www-form-urlencoded` query-string encode/decode, used internally by `url`'s query-pairs API | MIT OR Apache-2.0 |
| [`psl`](https://github.com/addr-rs/psl) | Mozilla Public Suffix List, compiled to native Rust code at `psl`'s own build time (no runtime list fetch) | MIT OR Apache-2.0 |

## Transitive dependencies

| Crate | Role | License |
|---|---|---|
| `idna_adapter` | Backs `idna`'s Unicode processing | MIT OR Apache-2.0 |
| `icu_normalizer`, `icu_properties` | Unicode normalization/property tables behind `idna_adapter` | Unicode-3.0 (permissive, OSI-approved) |
| `smallvec`, `utf8_iter` | Small-vector / UTF-8 iteration helpers behind `idna` | MIT OR Apache-2.0 |
| `psl-types` | Shared `Domain`/`Suffix`/`Type` types behind `psl` | MIT OR Apache-2.0 |

## License verification

Every crate in the resolved tree is **MIT OR Apache-2.0** except
`icu_normalizer`/`icu_properties`, which carry **Unicode-3.0** — a permissive,
OSI-approved license (not copyleft). No crate is copyleft (no GPL/LGPL/AGPL/MPL)
anywhere in the tree. Verified against crates.io's published license metadata
for each crate's exact pinned version, cross-checked against each crate's
repository (`servo/rust-url`, `addr-rs/psl`).

The Axiom Rust service runtime (`tonic`, `prost`, `tokio`, `hyper`, `tower`,
`tracing`, and their dependencies) is likewise MIT / Apache-2.0 throughout.

## Determinism and offline operation

All URL/IDNA/percent-encoding crates are pinned to exact versions in
`Cargo.toml` for reproducible builds. The `psl` crate compiles Mozilla's
Public Suffix List into native code at ITS OWN build time — no node in this
package fetches a URL, a suffix list, or any other network resource at
runtime. Correctness is enforced in the test suite (`axiom test`) against
RFC 3986 §5.4 reference resolution vectors, the WHATWG URL Standard's own
worked examples, canonical RFC 3492 Punycode examples, and published Mozilla
Public Suffix List entries.
