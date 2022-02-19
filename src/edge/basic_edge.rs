use crate::edge::Edge;
use crate::node::basic_node::BasicNode;
use crate::node::node_index::NodeIndex;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct BasicEdge<T: NodeIndex> {
    parent: T,
    child: T,
}

impl<T: NodeIndex> BasicEdge<T> {
    pub fn new(parent: &T, child: &T) -> Self {
        Self {
            parent: parent.clone(),
            child: child.clone(),
        }
    }
}

impl<T: NodeIndex> Edge for BasicEdge<T> {
    type Node = BasicNode<T>;

    fn parent(&self) -> &T {
        &self.parent
    }

    fn child(&self) -> &T {
        &self.child
    }
}
