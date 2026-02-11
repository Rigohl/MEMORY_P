//! src/shared_memory/graph.rs - Sistema de Grafo de Conocimiento (Nodos e Interconexiones)

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::error::Result;
use crate::shared_memory::types::ContextId;
use dashmap::DashMap;
use std::sync::Arc;

/// Tipo de relación entre nodos
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationType {
    Prerequisite,
    RelatedTo,
    PartOf,
    ConflictsWith,
    DerivedFrom,
}

/// Nodo en el grafo de memoria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNode {
    pub id: ContextId,
    pub label: String,
    pub weight: f64,
    pub metadata: HashMap<String, String>,
}

/// Conexión entre nodos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEdge {
    pub from: ContextId,
    pub to: ContextId,
    pub relation: RelationType,
    pub strength: f64,
}

/// Grafo de memoria relacional
pub struct RelationalMemoryGraph {
    nodes: Arc<DashMap<ContextId, MemoryNode>>,
    edges: Arc<DashMap<(ContextId, ContextId, RelationType), MemoryEdge>>,
    adjacency: Arc<DashMap<ContextId, HashSet<(ContextId, RelationType)>>>,
}

impl RelationalMemoryGraph {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(DashMap::new()),
            edges: Arc::new(DashMap::new()),
            adjacency: Arc::new(DashMap::new()),
        }
    }

    /// Añade o actualiza un nodo
    pub fn add_node(&self, node: MemoryNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Crea una interconexión entre nodos
    pub fn connect(&self, from: ContextId, to: ContextId, relation: RelationType, strength: f64) {
        let edge = MemoryEdge {
            from: from.clone(),
            to: to.clone(),
            relation: relation.clone(),
            strength,
        };
        self.edges.insert((from.clone(), to.clone(), relation.clone()), edge);

        self.adjacency.entry(from).or_insert_with(HashSet::new).insert((to, relation));
    }

    /// Obtiene todos los nodos relacionados
    pub fn get_related(&self, id: &ContextId) -> Vec<MemoryNode> {
        if let Some(targets) = self.adjacency.get(id) {
            targets.iter()
                .filter_map(|(to_id, _)| self.nodes.get(to_id).map(|n| n.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Auto-gestión: Fortalece conexiones activas y debilita las olvidadas
    pub async fn optimize_graph(&self) {
        // Lógica de decaimiento y fortalecimiento
        for mut edge_ref in self.edges.iter_mut() {
            let edge = edge_ref.value_mut();
            edge.strength *= 0.95; // Decaimiento natural
        }

        // Eliminar conexiones extremadamente débiles
        self.edges.retain(|_, edge| edge.strength > 0.01);
    }

    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({
            "node_count": self.nodes.len(),
            "edge_count": self.edges.len(),
            "average_strength": self.edges.iter().map(|e| e.strength).sum::<f64>() / self.edges.len().max(1) as f64
        })
    }
}
