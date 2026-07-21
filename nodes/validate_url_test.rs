// Separate test file: nodes/validate_url_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/validate_url_test.rs"] mod
// validate_url_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::UrlInput;
    use crate::validate_url::validate_url;
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

    #[test]
    fn test_valid_url_reports_valid_and_normalized() {
        let ax = test_context();
        let out = validate_url(&ax, ui("HTTPS://EXAMPLE.COM:443/path?q=1")).unwrap();
        assert!(out.valid);
        assert_eq!(out.error, "");
        assert_eq!(out.url, "https://example.com/path?q=1");
    }

    #[test]
    fn test_invalid_url_is_a_normal_result_not_an_error_fault() {
        let ax = test_context();
        let out = validate_url(&ax, ui("not a url at all")).unwrap();
        assert!(!out.valid);
        assert_eq!(out.error, "PARSE_ERROR");
        assert_eq!(out.url, "");
    }

    #[test]
    fn test_empty_url_is_invalid() {
        let ax = test_context();
        let out = validate_url(&ax, ui("")).unwrap();
        assert!(!out.valid);
        assert_eq!(out.error, "EMPTY_URL");
    }

    // ftp is a WHATWG "special scheme", recognized and valid.
    #[test]
    fn test_ftp_scheme_is_valid() {
        let ax = test_context();
        let out = validate_url(&ax, ui("ftp://example.com/file.txt")).unwrap();
        assert!(out.valid);
        assert_eq!(out.error, "");
    }

    #[test]
    fn test_deterministic() {
        let ax = test_context();
        let a = validate_url(&ax, ui("https://example.com/")).unwrap();
        let b = validate_url(&ax, ui("https://example.com/")).unwrap();
        assert_eq!(a.valid, b.valid);
        assert_eq!(a.url, b.url);
    }
}
