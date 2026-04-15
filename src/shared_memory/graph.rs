//! shared_memory/graph.rs - Knowledge Graph for shared memory system
//!
//! Provides a simple knowledge graph structure for storing relationships
//! between concepts, agents, and search results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub metadata: HashMap<String, String>,
}

/// Types of nodes in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Concept,
    Agent,
    SearchResult,
    Pattern,
    Context,
}

/// An edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f64,
}

/// Knowledge graph for shared memory
#[derive(Debug, Clone, Default)]
pub struct KnowledgeGraph {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    pub fn get_node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.get(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn neighbors(&self, node_id: &str) -> Vec<&GraphNode> {
        self.edges
            .iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .filter_map(|e| {
                let neighbor_id = if e.source == node_id {
                    &e.target
                } else {
                    &e.source
                };
                self.nodes.get(neighbor_id.as_str())
            })
            .collect()
    }

    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({
            "node_count": self.nodes.len(),
            "edge_count": self.edges.len(),
            "node_types": self.node_type_counts(),
        })
    }

    fn node_type_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for node in self.nodes.values() {
            let key = format!("{:?}", node.node_type);
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph() {
        let graph = KnowledgeGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_get_node() {
        let mut graph = KnowledgeGraph::new();
        let node = GraphNode {
            id: "node1".to_string(),
            label: "Node 1".to_string(),
            node_type: NodeType::Concept,
            metadata: HashMap::new(),
        };
        graph.add_node(node.clone());
        assert_eq!(graph.node_count(), 1);
        let retrieved = graph.get_node("node1").unwrap();
        assert_eq!(retrieved.label, "Node 1");
        assert!(graph.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = KnowledgeGraph::new();
        let edge = GraphEdge {
            source: "node1".to_string(),
            target: "node2".to_string(),
            relation: "related_to".to_string(),
            weight: 1.0,
        };
        graph.add_edge(edge);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_neighbors() {
        let mut graph = KnowledgeGraph::new();
        let node1 = GraphNode {
            id: "node1".to_string(),
            label: "Node 1".to_string(),
            node_type: NodeType::Concept,
            metadata: HashMap::new(),
        };
        let node2 = GraphNode {
            id: "node2".to_string(),
            label: "Node 2".to_string(),
            node_type: NodeType::Concept,
            metadata: HashMap::new(),
        };
        let node3 = GraphNode {
            id: "node3".to_string(),
            label: "Node 3".to_string(),
            node_type: NodeType::Concept,
            metadata: HashMap::new(),
        };
        graph.add_node(node1);
        graph.add_node(node2);
        graph.add_node(node3);

        graph.add_edge(GraphEdge {
            source: "node1".to_string(),
            target: "node2".to_string(),
            relation: "link".to_string(),
            weight: 1.0,
        });
        graph.add_edge(GraphEdge {
            source: "node3".to_string(),
            target: "node1".to_string(),
            relation: "link".to_string(),
            weight: 1.0,
        });

        let neighbors = graph.neighbors("node1");
        assert_eq!(neighbors.len(), 2);
        let ids: Vec<String> = neighbors.iter().map(|n| n.id.clone()).collect();
        assert!(ids.contains(&"node2".to_string()));
        assert!(ids.contains(&"node3".to_string()));
    }

    #[test]
    fn test_stats() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(GraphNode {
            id: "node1".to_string(),
            label: "Node 1".to_string(),
            node_type: NodeType::Concept,
            metadata: HashMap::new(),
        });
        graph.add_edge(GraphEdge {
            source: "node1".to_string(),
            target: "node2".to_string(),
            relation: "link".to_string(),
            weight: 1.0,
        });

        let stats = graph.stats();
        assert_eq!(stats["node_count"], 1);
        assert_eq!(stats["edge_count"], 1);
        assert_eq!(stats["node_types"]["Concept"], 1);
    }
}
