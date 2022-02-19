use super::error::GraphError;
use crate::edge::Edge;

pub type Result<Ok, TEdge> = core::result::Result<Ok, GraphError<<TEdge as Edge>::Node>>;
