// Separate test file: nodes/remove_query_param_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/remove_query_param_test.rs"] mod
// remove_query_param_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::RemoveQueryParamInput;
    use crate::remove_query_param::remove_query_param;
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

    fn ri(url: &str, key: &str) -> RemoveQueryParamInput {
        RemoveQueryParamInput { url: url.to_string(), key: key.to_string() }
    }

    #[test]
    fn test_removes_all_duplicate_occurrences() {
        let ax = test_context();
        let out = remove_query_param(&ax, ri("https://example.com/?a=1&b=2&a=3", "a")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "https://example.com/?b=2");
    }

    #[test]
    fn test_removing_only_param_drops_question_mark() {
        let ax = test_context();
        let out = remove_query_param(&ax, ri("https://example.com/?a=1", "a")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "https://example.com/");
    }

    #[test]
    fn test_removing_absent_key_is_a_no_op() {
        let ax = test_context();
        let out = remove_query_param(&ax, ri("https://example.com/?a=1", "z")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "https://example.com/?a=1");
    }

    #[test]
    fn test_empty_key_is_structured_error() {
        let ax = test_context();
        let out = remove_query_param(&ax, ri("https://example.com/?a=1", "")).unwrap();
        assert_eq!(out.error, "EMPTY_KEY");
    }

    #[test]
    fn test_malformed_url_is_structured_error() {
        let ax = test_context();
        let out = remove_query_param(&ax, ri("not a url", "a")).unwrap();
        assert_eq!(out.error, "PARSE_ERROR");
    }
}
