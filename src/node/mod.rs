pub mod basic_node;
pub mod metadata_node;
pub mod node_index;

use crate::node::node_index::NodeIndex;
use core::fmt::Display;
use core::fmt::Debug;
use core::hash::Hash;

pub trait Node: Display + Debug + Clone + Ord + PartialOrd + Hash + Default {
    type NodeIndex: NodeIndex;
    fn implicit_new(index: &Self::NodeIndex) -> Self;
    fn index(&self) -> &Self::NodeIndex;
}
