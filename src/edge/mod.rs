pub mod basic_edge;
pub mod directed_edge;

use crate::node::Node;
use alloc::string::String;
use core::fmt::{Display, Debug};

pub trait Edge: Display + Debug + Clone + Ord + PartialOrd + Default {
    type Node: Node;
    fn parent(&self) -> &<Self::Node as Node>::NodeIndex;
    fn child(&self) -> &<Self::Node as Node>::NodeIndex;
    fn label(&self) -> &String;
}
