use core::fmt::Debug;
use core::hash::Hash;

pub trait NodeIndex: Debug + Clone + Eq + PartialEq + Hash + Sized {}
