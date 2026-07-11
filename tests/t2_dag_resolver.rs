//! Acceptance checks AC5-AC8 (sprints/s1/sprint-plans/test-plan.md).
//!
//! Fixture graph (a has no deps; edges point from a unit to its deps):
//!   a <- b <- c <- d, and d also depends directly on a.
//! So: b deps=[a], c deps=[b], d deps=[a, c].

use array_test::dag::{Dag, DagError};
use std::collections::BTreeSet;

fn fixture_units() -> Vec<(&'static str, Vec<String>)> {
    vec![
        ("a", vec![]),
        ("b", vec!["a".to_string()]),
        ("c", vec!["b".to_string()]),
        ("d", vec!["a".to_string(), "c".to_string()]),
    ]
}

fn refs<'a>(units: &'a [(&'static str, Vec<String>)]) -> Vec<(&'static str, &'a [String])> {
    units
        .iter()
        .map(|(id, deps)| (*id, deps.as_slice()))
        .collect()
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(std::string::ToString::to_string).collect()
}

#[test]
fn given_a_two_unit_cycle_should_be_rejected() {
    let cyclic = vec![("x", vec!["y".to_string()]), ("y", vec!["x".to_string()])];

    let result = Dag::build(refs(&cyclic));

    assert!(matches!(result, Err(DagError::Cycle(_))));
}

#[test]
fn given_a_self_cycle_should_be_rejected() {
    let cyclic = vec![("x", vec!["x".to_string()])];

    let result = Dag::build(refs(&cyclic));

    assert!(matches!(result, Err(DagError::Cycle(_))));
}

#[test]
fn given_a_dependency_on_an_unknown_unit_should_be_rejected() {
    let bad = vec![("a", vec!["ghost".to_string()])];

    let result = Dag::build(refs(&bad));

    assert_eq!(
        result.unwrap_err(),
        DagError::UnknownDependency("a".to_string(), "ghost".to_string())
    );
}

#[test]
fn given_the_fixture_graph_forward_closure_of_d_should_be_its_transitive_deps() {
    let units = fixture_units();
    let dag = Dag::build(refs(&units)).unwrap();

    let actual = dag.closure("d");
    let expected = set(&["a", "b", "c"]);

    assert_eq!(actual, expected);
}

#[test]
fn given_the_fixture_graph_forward_closure_of_a_should_be_empty() {
    let units = fixture_units();
    let dag = Dag::build(refs(&units)).unwrap();

    let actual = dag.closure("a");
    let expected: BTreeSet<String> = BTreeSet::new();

    assert_eq!(actual, expected);
}

#[test]
fn given_the_fixture_graph_impact_closure_of_a_should_be_its_transitive_dependents() {
    let units = fixture_units();
    let dag = Dag::build(refs(&units)).unwrap();

    let actual = dag.impact("a");
    let expected = set(&["b", "c", "d"]);

    assert_eq!(actual, expected);
}

#[test]
fn given_the_fixture_graph_impact_closure_of_d_should_be_empty() {
    let units = fixture_units();
    let dag = Dag::build(refs(&units)).unwrap();

    let actual = dag.impact("d");
    let expected: BTreeSet<String> = BTreeSet::new();

    assert_eq!(actual, expected);
}

#[test]
fn given_an_unknown_unit_id_closures_should_be_empty_not_erroring() {
    let units = fixture_units();
    let dag = Dag::build(refs(&units)).unwrap();

    assert_eq!(dag.closure("ghost"), BTreeSet::new());
    assert_eq!(dag.impact("ghost"), BTreeSet::new());
}

#[test]
fn given_the_fixture_graph_topo_order_should_place_deps_before_dependents() {
    let units = fixture_units();
    let dag = Dag::build(refs(&units)).unwrap();

    let order = dag.topo_order();
    let pos = |id: &str| order.iter().position(|u| u == id).unwrap();

    // Edges: b->a, c->b, d->a, d->c. Deps must come first.
    assert!(pos("a") < pos("b"));
    assert!(pos("a") < pos("d"));
    assert!(pos("b") < pos("c"));
    assert!(pos("c") < pos("d"));
}

#[test]
fn given_the_same_units_topo_order_should_be_identical_across_builds() {
    let units = fixture_units();
    let dag1 = Dag::build(refs(&units)).unwrap();
    let dag2 = Dag::build(refs(&units)).unwrap();

    assert_eq!(dag1.topo_order(), dag2.topo_order());
}

#[test]
fn given_the_same_units_dag_json_should_serialize_deterministically() {
    let units = fixture_units();
    let dag1 = Dag::build(refs(&units)).unwrap();
    let dag2 = Dag::build(refs(&units)).unwrap();

    let json1 = serde_json::to_string(&dag1.to_json()).unwrap();
    let json2 = serde_json::to_string(&dag2.to_json()).unwrap();

    assert_eq!(json1, json2);
}

#[test]
fn given_the_fixture_graph_dag_json_should_list_each_units_direct_deps_sorted() {
    let units = fixture_units();
    let dag = Dag::build(refs(&units)).unwrap();

    let json = dag.to_json();

    assert_eq!(json["a"], serde_json::json!([]));
    assert_eq!(json["b"], serde_json::json!(["a"]));
    assert_eq!(json["c"], serde_json::json!(["b"]));
    assert_eq!(json["d"], serde_json::json!(["a", "c"]));
}
