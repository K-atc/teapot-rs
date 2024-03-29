use alloc::string::String;
use core::fmt::{Debug, Display};
use core::hash::Hash;

pub trait NodeIndex:
    Display + Debug + Clone + Eq + PartialEq + Ord + Hash + Default + Sized
{
}

impl NodeIndex for usize {}
impl NodeIndex for String {}
