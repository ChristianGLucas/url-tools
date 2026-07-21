// Separate test file: nodes/normalize_url_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/normalize_url_test.rs"] mod
// normalize_url_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::UrlInput;
    use crate::normalize_url::normalize_url;
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

    // Independent oracle: WHATWG URL serialization — default port (80 for
    // http) is omitted, scheme and host are lowercased, path case is
    // preserved. All are documented, code-independent facts of the standard.
    #[test]
    fn test_default_port_omitted_and_case_normalized() {
        let ax = test_context();
        let out = normalize_url(&ax, ui("HTTP://EXAMPLE.COM:80/Path")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "http://example.com/Path");
    }

    // Independent oracle: RFC 3986 §5.2.4 dot-segment removal.
    #[test]
    fn test_dot_segments_normalized() {
        let ax = test_context();
        let out = normalize_url(&ax, ui("https://example.com/a/./b/../c")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "https://example.com/a/c");
    }

    // A non-default port must be preserved.
    #[test]
    fn test_non_default_port_preserved() {
        let ax = test_context();
        let out = normalize_url(&ax, ui("http://example.com:8080/x")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "http://example.com:8080/x");
    }

    #[test]
    fn test_empty_url_is_structured_error() {
        let ax = test_context();
        let out = normalize_url(&ax, ui("")).unwrap();
        assert_eq!(out.error, "EMPTY_URL");
        assert_eq!(out.url, "");
    }

    #[test]
    fn test_malformed_url_is_structured_error() {
        let ax = test_context();
        let out = normalize_url(&ax, ui("://bad")).unwrap();
        assert_eq!(out.error, "PARSE_ERROR");
        assert_eq!(out.url, "");
    }

    #[test]
    fn test_deterministic() {
        let ax = test_context();
        let a = normalize_url(&ax, ui("HTTPS://Example.COM/a?b=c#d")).unwrap();
        let b = normalize_url(&ax, ui("HTTPS://Example.COM/a?b=c#d")).unwrap();
        assert_eq!(a.url, b.url);
        assert_eq!(a.url, "https://example.com/a?b=c#d");
    }
}
