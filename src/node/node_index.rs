use core::fmt::{Debug, Display};
use core::hash::Hash;

pub trait NodeIndex: Display + Debug + Clone + Eq + PartialEq + Ord + Hash + Sized {}
