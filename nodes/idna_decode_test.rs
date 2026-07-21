// Separate test file: nodes/idna_decode_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/idna_decode_test.rs"] mod
// idna_decode_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::IdnaInput;
    use crate::idna_decode::idna_decode;
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

    // Independent oracle: same canonical Punycode/IDNA worked example as
    // idna_encode's test, applied in reverse (RFC 3492 / Punycode reference
    // example), not derived from this code.
    #[test]
    fn test_canonical_punycode_example_reverse() {
        let ax = test_context();
        let out = idna_decode(&ax, di("xn--mnchen-3ya.de")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.value, "münchen.de");
    }

    #[test]
    fn test_no_punycode_labels_passes_through() {
        let ax = test_context();
        let out = idna_decode(&ax, di("example.com")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.value, "example.com");
    }

    #[test]
    fn test_empty_domain_is_structured_error() {
        let ax = test_context();
        let out = idna_decode(&ax, di("")).unwrap();
        assert_eq!(out.error, "EMPTY_DOMAIN");
    }

    // Round-trip: encoding then decoding an already-Unicode domain returns
    // the original.
    #[test]
    fn test_round_trip_with_idna_encode() {
        use crate::idna_encode::idna_encode;
        let ax = test_context();
        let ascii = idna_encode(&ax, di("café.example")).unwrap();
        assert_eq!(ascii.error, "");
        let unicode = idna_decode(&ax, di(&ascii.value)).unwrap();
        assert_eq!(unicode.error, "");
        assert_eq!(unicode.value, "café.example");
    }

    #[test]
    fn test_deterministic() {
        let ax = test_context();
        let a = idna_decode(&ax, di("xn--mnchen-3ya.de")).unwrap();
        let b = idna_decode(&ax, di("xn--mnchen-3ya.de")).unwrap();
        assert_eq!(a.value, b.value);
    }
}
