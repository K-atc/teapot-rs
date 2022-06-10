pub mod basic_edge;
pub mod directed_edge;

use crate::node::Node;
use alloc::string::String;
use core::fmt::Debug;

pub trait Edge: Debug + Clone + Ord + PartialOrd + Default {
    type Node: Node;
    fn parent(&self) -> &<Self::Node as Node>::NodeIndex;
    fn child(&self) -> &<Self::Node as Node>::NodeIndex;
    fn label(&self) -> &String;
}
