use crate::edge::Edge;
use crate::node::basic_node::BasicNode;
use crate::node::node_index::NodeIndex;
use alloc::string::String;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct BasicEdge<T: NodeIndex> {
    parent: T,
    child: T,
    label: String,
}

impl<T: NodeIndex> BasicEdge<T> {
    pub fn new(parent: &T, child: &T, label: String) -> Self {
        Self {
            parent: parent.clone(),
            child: child.clone(),
            label,
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

    fn label(&self) -> &String {
        &self.label
    }
}
