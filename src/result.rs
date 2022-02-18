use super::error::GraphError;

pub type Result<Ok, Node> = core::result::Result<Ok, GraphError<Node>>;
