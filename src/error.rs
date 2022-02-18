use crate::node::Node;

#[derive(Debug, Eq, PartialEq)]
pub enum GraphError<TNode: Node> {
    NodeNotExists(TNode::NodeIndex),
}
