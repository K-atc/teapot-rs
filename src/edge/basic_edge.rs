use crate::edge::Edge;
use crate::node::Node;
use alloc::fmt;
use alloc::string::String;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct BasicEdge<T: Node> {
    parent: <T as Node>::NodeIndex,
    child: <T as Node>::NodeIndex,
    label: String,
}

impl<T: Node> BasicEdge<T> {
    pub fn new(
        parent: &<T as Node>::NodeIndex,
        child: &<T as Node>::NodeIndex,
        label: String,
    ) -> Self {
        Self {
            parent: parent.clone(),
            child: child.clone(),
            label,
        }
    }
}

impl<T: Node> Edge for BasicEdge<T> {
    type Node = T;

    fn parent(&self) -> &<Self::Node as Node>::NodeIndex {
        &self.parent
    }

    fn child(&self) -> &<Self::Node as Node>::NodeIndex {
        &self.child
    }

    fn label(&self) -> &String {
        &self.label
    }
}

impl<T: Node> fmt::Display for BasicEdge<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}
