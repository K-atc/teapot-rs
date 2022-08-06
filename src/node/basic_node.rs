use crate::node::node_index::NodeIndex;
use crate::node::Node;
use std::fmt;
use core::fmt::Debug;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct BasicNode<T: NodeIndex> {
    index: T,
}

impl<T: NodeIndex> Node for BasicNode<T> {
    type NodeIndex = T;

    fn implicit_new(index: &Self::NodeIndex) -> Self {
        Self::new(index)
    }

    fn index(&self) -> &Self::NodeIndex {
        &self.index
    }
}

impl<T: NodeIndex> BasicNode<T> {
    pub fn new(index: &T) -> Self {
        Self {
            index: index.clone(),
        }
    }
}

impl<T: NodeIndex> fmt::Display for BasicNode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}
