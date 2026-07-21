// Separate test file: nodes/join_url_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/join_url_test.rs"] mod
// join_url_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::JoinInput;
    use crate::join_url::join_url;
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

    fn ji(base: &str, relative: &str) -> JoinInput {
        JoinInput { base: base.to_string(), relative: relative.to_string() }
    }

    // Independent oracle: RFC 3986 §5.4.1 "Normal Examples" — the canonical,
    // published reference-resolution test vectors for base URI
    // "http://a/b/c/d;p?q", verbatim from the RFC, not derived from this code.
    const BASE: &str = "http://a/b/c/d;p?q";
    const NORMAL_EXAMPLES: &[(&str, &str)] = &[
        ("g:h", "g:h"),
        ("g", "http://a/b/c/g"),
        ("./g", "http://a/b/c/g"),
        ("g/", "http://a/b/c/g/"),
        ("/g", "http://a/g"),
        ("//g", "http://g/"),
        ("?y", "http://a/b/c/d;p?y"),
        ("g?y", "http://a/b/c/g?y"),
        ("#s", "http://a/b/c/d;p?q#s"),
        ("g#s", "http://a/b/c/g#s"),
        ("g?y#s", "http://a/b/c/g?y#s"),
        (";x", "http://a/b/c/;x"),
        ("g;x", "http://a/b/c/g;x"),
        ("", "http://a/b/c/d;p?q"),
        (".", "http://a/b/c/"),
        ("./", "http://a/b/c/"),
        ("..", "http://a/b/"),
        ("../", "http://a/b/"),
        ("../g", "http://a/b/g"),
        ("../..", "http://a/"),
        ("../../", "http://a/"),
        ("../../g", "http://a/g"),
    ];

    #[test]
    fn test_rfc3986_normal_examples() {
        let ax = test_context();
        for (relative, expected) in NORMAL_EXAMPLES {
            let out = join_url(&ax, ji(BASE, relative)).unwrap();
            assert_eq!(out.error, "", "relative {:?}", relative);
            assert_eq!(&out.url, expected, "relative {:?}", relative);
        }
    }

    // Independent oracle: RFC 3986 §5.4.2 "Abnormal Examples" — climbing
    // above the root clamps at the root rather than erroring or escaping it.
    #[test]
    fn test_rfc3986_abnormal_climb_above_root() {
        let ax = test_context();
        let out = join_url(&ax, ji(BASE, "../../../g")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "http://a/g");
        let out2 = join_url(&ax, ji(BASE, "../../../../g")).unwrap();
        assert_eq!(out2.error, "");
        assert_eq!(out2.url, "http://a/g");
    }

    // An absolute reference is returned normalized, ignoring the base.
    #[test]
    fn test_absolute_reference_ignores_base() {
        let ax = test_context();
        let out = join_url(&ax, ji("https://example.com/a/b", "https://other.example/x")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "https://other.example/x");
    }

    #[test]
    fn test_empty_base_is_structured_error() {
        let ax = test_context();
        let out = join_url(&ax, ji("", "g")).unwrap();
        assert_eq!(out.error, "EMPTY_BASE");
    }

    #[test]
    fn test_malformed_base_is_structured_error() {
        let ax = test_context();
        let out = join_url(&ax, ji("not a url", "g")).unwrap();
        assert_eq!(out.error, "BASE_PARSE_ERROR");
    }

    #[test]
    fn test_deterministic() {
        let ax = test_context();
        let a = join_url(&ax, ji(BASE, "../g")).unwrap();
        let b = join_url(&ax, ji(BASE, "../g")).unwrap();
        assert_eq!(a.url, b.url);
    }
}
