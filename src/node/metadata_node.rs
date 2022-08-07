use crate::node::node_index::NodeIndex;
use crate::node::Node;
use alloc::fmt;

use core::fmt::Debug;
use core::fmt::Display;
use core::hash::Hash;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct MetadataNode<T: NodeIndex, TMetadata: Debug> {
    index: T,
    metadata: TMetadata,
}

impl<
        T: NodeIndex,
        TMetadata: Display + Debug + Clone + Eq + PartialEq + Ord + PartialOrd + Default + Hash,
    > Node for MetadataNode<T, TMetadata>
{
    type NodeIndex = T;

    fn implicit_new(index: &Self::NodeIndex) -> Self {
        Self::new(index, &TMetadata::default())
    }

    fn index(&self) -> &Self::NodeIndex {
        &self.index
    }
}

impl<T: NodeIndex, TMetadata: Debug + Clone> MetadataNode<T, TMetadata> {
    pub fn new(index: &T, medatada: &TMetadata) -> Self {
        Self {
            index: index.clone(),
            metadata: medatada.clone(),
        }
    }

    pub fn metadata(&self) -> &TMetadata {
        &self.metadata
    }
}

impl<T: NodeIndex, TMetadata: Debug + Display> fmt::Display for MetadataNode<T, TMetadata> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.metadata, self.index)
    }
}
