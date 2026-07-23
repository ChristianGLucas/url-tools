// Separate test file: nodes/percent_decode_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/percent_decode_test.rs"] mod
// percent_decode_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::TextInput;
    use crate::percent_decode::percent_decode;
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

    fn ti(value: &str) -> TextInput { TextInput { value: value.to_string() } }

    // Independent oracle: the ASCII table — 0x68='h', 0x65='e', 0x6C='l',
    // 0x6F='o' — not derived from this code.
    #[test]
    fn test_ascii_hex_escapes() {
        let ax = test_context();
        let out = percent_decode(&ax, ti("%68%65%6C%6C%6F")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.value, "hello");
    }

    #[test]
    fn test_space_escape() {
        let ax = test_context();
        let out = percent_decode(&ax, ti("a%20b")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.value, "a b");
    }

    // Independent oracle: the Unicode UTF-8 encoding of U+2713 CHECK MARK is
    // the fixed byte sequence E2 9C 93, per the Unicode standard.
    #[test]
    fn test_multibyte_utf8_escape() {
        let ax = test_context();
        let out = percent_decode(&ax, ti("%E2%9C%93")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.value, "\u{2713}");
    }

    #[test]
    fn test_no_escapes_passes_through_unchanged() {
        let ax = test_context();
        let out = percent_decode(&ax, ti("hello world")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.value, "hello world");
    }

    // 0xFF 0xFE is not a valid UTF-8 byte sequence under any interpretation.
    #[test]
    fn test_invalid_utf8_is_structured_error_not_lossy() {
        let ax = test_context();
        let out = percent_decode(&ax, ti("%FF%FE")).unwrap();
        assert_eq!(out.error, "INVALID_UTF8");
        assert_eq!(out.value, "");
    }

    // Round-trips with PercentEncode's "component" mode.
    #[test]
    fn test_round_trip_with_percent_encode_component_mode() {
        use crate::percent_encode::percent_encode;
        use crate::gen::messages::PercentEncodeInput;
        let ax = test_context();
        let original = "hello world! / ünïcödé";
        let encoded = percent_encode(&ax, PercentEncodeInput {
            value: original.to_string(),
            component: "component".to_string(),
        }).unwrap();
        let decoded = percent_decode(&ax, ti(&encoded.value)).unwrap();
        assert_eq!(decoded.value, original);
    }
}
