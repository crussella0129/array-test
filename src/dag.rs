//! Integration DAG resolver (ARCHITECTURE.md §1.3).
//!
//! Edges are declared `deps` only — this is what keeps integration cells at O(edges)
//! instead of O(2^units) (ARCHITECTURE.md §5). "Down" is a unit's dependency closure;
//! "backwards" is its reverse-dependency (impact) closure.

use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{Dfs, Reversed};
use serde_json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DagError {
    #[error("unit '{0}' declares a dependency on unknown unit '{1}'")]
    UnknownDependency(String, String),
    #[error("dependency cycle detected involving unit '{0}'")]
    Cycle(String),
}

#[derive(Debug)]
pub struct Dag {
    graph: DiGraph<String, ()>,
    index_of: HashMap<String, NodeIndex>,
}

impl Dag {
    /// Build the DAG from `(unit_id, deps)` pairs. Every id in every `deps` list must
    /// itself be a unit in `units`, and the resulting graph must be acyclic.
    pub fn build<'a>(
        units: impl IntoIterator<Item = (&'a str, &'a [String])>,
    ) -> Result<Dag, DagError> {
        let mut units: Vec<(&str, &[String])> = units.into_iter().collect();
        units.sort_by(|a, b| a.0.cmp(b.0));

        let mut graph = DiGraph::new();
        let mut index_of = HashMap::new();
        for (id, _) in &units {
            let idx = graph.add_node((*id).to_string());
            index_of.insert((*id).to_string(), idx);
        }

        for (id, deps) in &units {
            let mut deps_sorted: Vec<&String> = deps.iter().collect();
            deps_sorted.sort();
            let from = index_of[*id];
            for dep in deps_sorted {
                let to = *index_of
                    .get(dep)
                    .ok_or_else(|| DagError::UnknownDependency((*id).to_string(), dep.clone()))?;
                graph.add_edge(from, to, ());
            }
        }

        if let Err(cycle) = toposort(&graph, None) {
            let node = graph[cycle.node_id()].clone();
            return Err(DagError::Cycle(node));
        }

        Ok(Dag { graph, index_of })
    }

    /// "down": the transitive dependency closure of `id` (not including `id` itself).
    /// Empty if `id` is unknown.
    pub fn closure(&self, id: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        if let Some(&start) = self.index_of.get(id) {
            let mut dfs = Dfs::new(&self.graph, start);
            dfs.next(&self.graph); // the start node itself
            while let Some(nx) = dfs.next(&self.graph) {
                out.insert(self.graph[nx].clone());
            }
        }
        out
    }

    /// "backwards" / impact: the transitive reverse-dependency closure of `id` (not
    /// including `id` itself) — every unit that would need re-confirming if `id` changed.
    /// Empty if `id` is unknown.
    pub fn impact(&self, id: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        if let Some(&start) = self.index_of.get(id) {
            let mut dfs = Dfs::new(Reversed(&self.graph), start);
            dfs.next(Reversed(&self.graph)); // the start node itself
            while let Some(nx) = dfs.next(Reversed(&self.graph)) {
                out.insert(self.graph[nx].clone());
            }
        }
        out
    }

    /// Deterministic serialization: sorted unit ids, sorted deps lists (`dag.json`,
    /// ARCHITECTURE.md §8).
    pub fn to_json(&self) -> serde_json::Value {
        let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (id, &idx) in &self.index_of {
            let mut deps: Vec<String> = self
                .graph
                .neighbors(idx)
                .map(|n| self.graph[n].clone())
                .collect();
            deps.sort();
            map.insert(id.clone(), deps);
        }
        serde_json::to_value(map).expect("BTreeMap<String, Vec<String>> always serializes")
    }
}
