pub mod directed_edge;

use crate::Node;
use core::fmt::Debug;

pub trait Edge: Debug + Clone + Default {
    type Node: Node;
    fn parent(&self) -> &<Self::Node as Node>::NodeIndex;
    fn child(&self) -> &<Self::Node as Node>::NodeIndex;
}
