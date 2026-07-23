// Separate test file: nodes/parse_url_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/parse_url_test.rs"] mod
// parse_url_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::UrlInput;
    use crate::parse_url::parse_url;
    use std::collections::HashMap;

    struct TestLogger;
    impl AxiomLogger for TestLogger {
        fn debug(&self, _m: &str, _a: &HashMap<&str, String>) {}
        fn info(&self, _m: &str, _a: &HashMap<&str, String>) {}
        fn warn(&self, _m: &str, _a: &HashMap<&str, String>) {}
        fn error(&self, _m: &str, _a: &HashMap<&str, String>) {}
    }
    struct TestSecrets;
    impl AxiomSecrets for TestSecrets {
        fn get(&self, _n: &str) -> (String, bool) { (String::new(), false) }
        fn status(&self, _n: &str) -> SecretStatus { SecretStatus::Unset }
    }
    struct EmptyFlow { pos: FlowPosition }
    impl FlowReflection for EmptyFlow {
        fn nodes(&self) -> &[ReflectionNode] { &[] }
        fn edges(&self) -> &[ReflectionEdge] { &[] }
        fn loop_edges(&self) -> &[ReflectionEdge] { &[] }
        fn position(&self) -> &FlowPosition { &self.pos }
        fn graph_id(&self) -> &str { "" }
    }
    struct TestReflection { flow: EmptyFlow }
    impl Reflection for TestReflection { fn flow(&self) -> &dyn FlowReflection { &self.flow } }
    struct TestFlowMut;
    impl FlowMutation for TestFlowMut {
        fn add_node(&self, _p: &str, _v: &str, _c: Option<CanvasPosition>) -> u32 { 0 }
        fn add_edge(&self, _s: u32, _d: u32, _c: Option<EdgeCondition>) {}
    }
    struct TestMutation { flow: TestFlowMut }
    impl Mutation for TestMutation { fn flow(&self) -> &dyn FlowMutation { &self.flow } }

    struct TestContext {
        log: TestLogger, secrets: TestSecrets,
        reflection: TestReflection, mutation: TestMutation,
    }
    impl AxiomContext for TestContext {
        fn log(&self) -> &dyn AxiomLogger { &self.log }
        fn secrets(&self) -> &dyn AxiomSecrets { &self.secrets }
        fn execution_id(&self) -> &str { "test-execution-id" }
        fn flow_id(&self) -> &str { "test-flow-id" }
        fn tenant_id(&self) -> &str { "test-tenant-id" }
        fn reflection(&self) -> &dyn Reflection { &self.reflection }
        fn mutation(&self) -> &dyn Mutation { &self.mutation }
    }
    fn test_context() -> TestContext {
        TestContext {
            log: TestLogger, secrets: TestSecrets,
            reflection: TestReflection { flow: EmptyFlow { pos: FlowPosition::default() } },
            mutation: TestMutation { flow: TestFlowMut },
        }
    }

    fn ui(url: &str) -> UrlInput { UrlInput { url: url.to_string() } }

    // Independent oracle: the WHATWG URL Standard's own worked example for a
    // fully-populated URL (scheme/userinfo/host/port/path/query/fragment) —
    // https://url.spec.whatwg.org/#example-url-components — not derived from
    // this code.
    #[test]
    fn test_full_component_breakdown() {
        let ax = test_context();
        let out = parse_url(&ax, ui("https://user:pass@example.com:8080/path/to/page?query=string#hash")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.scheme, "https");
        assert_eq!(out.username, "user");
        assert_eq!(out.password, "pass");
        assert!(out.has_password);
        assert_eq!(out.host, "example.com");
        assert_eq!(out.host_type, "domain");
        assert!(out.has_port);
        assert_eq!(out.port, 8080);
        assert_eq!(out.effective_port, 8080);
        assert_eq!(out.path, "/path/to/page");
        assert_eq!(out.path_segments, vec!["path", "to", "page"]);
        assert_eq!(out.query, "query=string");
        assert!(out.has_query);
        assert_eq!(out.query_pairs.len(), 1);
        assert_eq!(out.query_pairs[0].key, "query");
        assert_eq!(out.query_pairs[0].value, "string");
        assert_eq!(out.fragment, "hash");
        assert!(out.has_fragment);
        assert_eq!(out.origin, "https://example.com:8080");
        assert!(!out.cannot_be_a_base);
    }

    // Independent oracle: the WHATWG URL Standard's default-port table (http
    // -> 80) — a well-known-port URL omits it from has_port/port but
    // port_or_known_default still resolves it.
    #[test]
    fn test_default_port_not_explicit() {
        let ax = test_context();
        let out = parse_url(&ax, ui("http://example.com/")).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.has_port);
        assert_eq!(out.port, 0);
        assert_eq!(out.effective_port, 80);
    }

    // Independent oracle: RFC 3986 §5.2.4 dot-segment removal — "/a/b/../c"
    // resolves to "/a/c" by the standard algorithm, independent of our code.
    #[test]
    fn test_dot_segments_resolved_on_parse() {
        let ax = test_context();
        let out = parse_url(&ax, ui("https://example.com/a/b/../c")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.path, "/a/c");
    }

    // Independent oracle: WHATWG URL serialization lowercases scheme and
    // domain host but does NOT alter path casing.
    #[test]
    fn test_scheme_and_host_lowercased_path_preserved() {
        let ax = test_context();
        let out = parse_url(&ax, ui("HTTPS://EXAMPLE.COM/PathCase")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.scheme, "https");
        assert_eq!(out.host, "example.com");
        assert_eq!(out.path, "/PathCase");
    }

    // A "cannot-be-a-base" URL (mailto:) has no host/authority and an
    // opaque ("null") origin — a defining WHATWG URL Standard property.
    #[test]
    fn test_cannot_be_a_base_url() {
        let ax = test_context();
        let out = parse_url(&ax, ui("mailto:test@example.com")).unwrap();
        assert_eq!(out.error, "");
        assert!(out.cannot_be_a_base);
        assert_eq!(out.host, "");
        assert_eq!(out.origin, "null");
    }

    #[test]
    fn test_ipv4_and_ipv6_host_types() {
        let ax = test_context();
        let v4 = parse_url(&ax, ui("http://192.168.1.1:8080/")).unwrap();
        assert_eq!(v4.host_type, "ipv4");
        assert_eq!(v4.host, "192.168.1.1");
        let v6 = parse_url(&ax, ui("http://[2001:db8::1]/")).unwrap();
        assert_eq!(v6.host_type, "ipv6");
        assert_eq!(v6.host, "[2001:db8::1]");
    }

    #[test]
    fn test_no_query_no_fragment_flags() {
        let ax = test_context();
        let out = parse_url(&ax, ui("https://example.com/path")).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.has_query);
        assert_eq!(out.query, "");
        assert!(!out.has_fragment);
        assert_eq!(out.fragment, "");
    }

    #[test]
    fn test_empty_url_is_structured_error() {
        let ax = test_context();
        let out = parse_url(&ax, ui("")).unwrap();
        assert_eq!(out.error, "EMPTY_URL");
        assert_eq!(out.url, "");
    }

    #[test]
    fn test_malformed_url_is_structured_error() {
        let ax = test_context();
        let out = parse_url(&ax, ui("not a url at all")).unwrap();
        assert_eq!(out.error, "PARSE_ERROR");
    }

    #[test]
    fn test_deterministic() {
        let ax = test_context();
        let a = parse_url(&ax, ui("https://example.com/a?b=c")).unwrap();
        let b = parse_url(&ax, ui("https://example.com/a?b=c")).unwrap();
        assert_eq!(a.url, b.url);
        assert_eq!(a.query_pairs.len(), b.query_pairs.len());
    }
}
