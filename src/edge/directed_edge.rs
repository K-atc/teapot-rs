use super::Edge;
use crate::Node;
use core::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct DirectedEdge<TEdge: Edge> {
    pub(crate) parent: <TEdge::Node as Node>::NodeIndex,
    pub(crate) child: <TEdge::Node as Node>::NodeIndex,
}

impl<TEdge: Edge> Hash for DirectedEdge<TEdge> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.parent.hash(state);
        self.child.hash(state);
    }
}

impl<TEdge: Edge> PartialEq for DirectedEdge<TEdge> {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.child == other.child
    }
}

impl<TEdge: Edge> Eq for DirectedEdge<TEdge> {}

impl<TEdge: Edge> DirectedEdge<TEdge> {
    pub fn new(
        parent: &<TEdge::Node as Node>::NodeIndex,
        child: &<TEdge::Node as Node>::NodeIndex,
    ) -> Self {
        Self {
            parent: parent.clone(),
            child: child.clone(),
        }
    }

    pub fn from(edge: &TEdge) -> Self {
        Self {
            parent: edge.parent().clone(),
            child: edge.child().clone(),
        }
    }
}
