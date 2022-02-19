use crate::io;
use crate::node::Node;
use alloc::format;
use alloc::string::String;
use core::convert::From;

#[derive(Debug, Eq, PartialEq)]
pub enum GraphError<TNode: Node> {
    NodeNotExists(TNode::NodeIndex),
    IoError(String),
}

impl<TNode: Node> From<io::Error> for GraphError<TNode> {
    fn from(error: io::Error) -> Self {
        Self::IoError(format!("{}", error))
    }
}
