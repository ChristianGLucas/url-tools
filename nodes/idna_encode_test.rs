// Separate test file: nodes/idna_encode_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/idna_encode_test.rs"] mod
// idna_encode_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::IdnaInput;
    use crate::idna_encode::idna_encode;
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

    fn di(domain: &str) -> IdnaInput { IdnaInput { domain: domain.to_string() } }

    // Independent oracle: "münchen.de" -> "xn--mnchen-3ya.de" is the
    // textbook worked Punycode/IDNA example (see RFC 3492 and the Punycode
    // Wikipedia article) — a fixed, published conversion, not derived from
    // this code.
    #[test]
    fn test_canonical_punycode_example() {
        let ax = test_context();
        let out = idna_encode(&ax, di("münchen.de")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.value, "xn--mnchen-3ya.de");
    }

    #[test]
    fn test_already_ascii_domain_passes_through() {
        let ax = test_context();
        let out = idna_encode(&ax, di("example.com")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.value, "example.com");
    }

    #[test]
    fn test_ascii_domain_is_lowercased() {
        let ax = test_context();
        let out = idna_encode(&ax, di("EXAMPLE.COM")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.value, "example.com");
    }

    #[test]
    fn test_empty_domain_is_structured_error() {
        let ax = test_context();
        let out = idna_encode(&ax, di("")).unwrap();
        assert_eq!(out.error, "EMPTY_DOMAIN");
    }

    // Independent oracle: RFC 3492 Punycode decoding — a label already
    // prefixed "xn--" is decoded and re-validated; "xn--a" has no valid
    // extended (delta) sequence after the delimiter, so it is malformed
    // punycode by the algorithm's own decode rules, not by anything this
    // package computes.
    #[test]
    fn test_malformed_existing_punycode_label_is_structured_error() {
        let ax = test_context();
        let out = idna_encode(&ax, di("xn--a.com")).unwrap();
        assert_eq!(out.error, "IDNA_ERROR");
        assert_eq!(out.value, "");
    }

    #[test]
    fn test_deterministic() {
        let ax = test_context();
        let a = idna_encode(&ax, di("münchen.de")).unwrap();
        let b = idna_encode(&ax, di("münchen.de")).unwrap();
        assert_eq!(a.value, b.value);
    }
}
