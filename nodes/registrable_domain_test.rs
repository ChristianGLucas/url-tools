// Separate test file: nodes/registrable_domain_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/registrable_domain_test.rs"] mod
// registrable_domain_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::UrlInput;
    use crate::registrable_domain::registrable_domain;
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

    // Independent oracle: "co.uk" is a published ICANN-section entry in the
    // Mozilla Public Suffix List (publicsuffix.org) — a fact external to
    // this code and this package's own dependency choice.
    #[test]
    fn test_two_label_icann_suffix_from_full_url() {
        let ax = test_context();
        let out = registrable_domain(&ax, ui("https://www.example.co.uk/path")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.registrable_domain, "example.co.uk");
        assert_eq!(out.suffix, "co.uk");
        assert!(out.is_icann);
        assert!(out.is_known);
    }

    #[test]
    fn test_bare_hostname_input() {
        let ax = test_context();
        let out = registrable_domain(&ax, ui("www.example.com")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.registrable_domain, "example.com");
        assert_eq!(out.suffix, "com");
        assert!(out.is_icann);
    }

    // Independent oracle: "github.io" is a published PRIVATE-section entry
    // in the Mozilla Public Suffix List (used by GitHub Pages), a fact that
    // is publicly documented on publicsuffix.org independent of this code.
    #[test]
    fn test_private_section_suffix_is_not_icann() {
        let ax = test_context();
        let out = registrable_domain(&ax, ui("https://example.github.io/")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.registrable_domain, "example.github.io");
        assert_eq!(out.suffix, "github.io");
        assert!(!out.is_icann);
        assert!(out.is_known);
    }

    #[test]
    fn test_case_insensitive_matching() {
        let ax = test_context();
        let out = registrable_domain(&ax, ui("WWW.EXAMPLE.COM")).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(out.registrable_domain, "example.com");
    }

    // "localhost" has no entry in the Public Suffix List. Per the published
    // PSL algorithm's own default rule ("If no rules match, the prevailing
    // rule is '*'" — publicsuffix.org/list/), an unmatched single label
    // still comes back as *a* suffix (is_known=false marks it unrecognized,
    // not absent) with no registrable domain above it.
    #[test]
    fn test_localhost_has_no_recognized_suffix() {
        let ax = test_context();
        let out = registrable_domain(&ax, ui("localhost")).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.is_known);
        assert!(!out.is_icann);
        assert_eq!(out.suffix, "localhost");
        assert_eq!(out.registrable_domain, "");
    }

    #[test]
    fn test_empty_input_is_structured_error() {
        let ax = test_context();
        let out = registrable_domain(&ax, ui("")).unwrap();
        assert_eq!(out.error, "EMPTY_INPUT");
    }

    #[test]
    fn test_deterministic() {
        let ax = test_context();
        let a = registrable_domain(&ax, ui("https://www.example.co.uk/")).unwrap();
        let b = registrable_domain(&ax, ui("https://www.example.co.uk/")).unwrap();
        assert_eq!(a.registrable_domain, b.registrable_domain);
        assert_eq!(a.suffix, b.suffix);
    }
}
