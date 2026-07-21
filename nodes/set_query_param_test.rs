// Separate test file: nodes/set_query_param_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/set_query_param_test.rs"] mod
// set_query_param_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::QueryParamInput;
    use crate::set_query_param::set_query_param;
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

    fn qi(url: &str, key: &str, value: &str) -> QueryParamInput {
        QueryParamInput { url: url.to_string(), key: key.to_string(), value: value.to_string() }
    }

    #[test]
    fn test_replaces_all_duplicate_occurrences() {
        let ax = test_context();
        let out = set_query_param(&ax, qi("https://example.com/?a=1&b=2&a=3", "a", "9")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "https://example.com/?b=2&a=9");
    }

    #[test]
    fn test_sets_new_key_on_bare_url() {
        let ax = test_context();
        let out = set_query_param(&ax, qi("https://example.com/", "x", "1")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "https://example.com/?x=1");
    }

    #[test]
    fn test_other_keys_untouched_and_order_preserved() {
        let ax = test_context();
        let out = set_query_param(&ax, qi("https://example.com/?b=1&a=old&c=3", "a", "new")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.url, "https://example.com/?b=1&c=3&a=new");
    }

    #[test]
    fn test_empty_key_is_structured_error() {
        let ax = test_context();
        let out = set_query_param(&ax, qi("https://example.com/", "", "v")).unwrap();
        assert_eq!(out.error, "EMPTY_KEY");
    }

    #[test]
    fn test_malformed_url_is_structured_error() {
        let ax = test_context();
        let out = set_query_param(&ax, qi("not a url", "a", "1")).unwrap();
        assert_eq!(out.error, "PARSE_ERROR");
    }
}
