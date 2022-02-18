#![no_std]
extern crate alloc;
extern crate hashbrown;

pub mod edge;
pub mod error;
pub mod node;
pub mod result;

use crate::edge::directed_edge::DirectedEdge;
use crate::edge::Edge;
use crate::error::GraphError;
use crate::node::Node;
use crate::result::Result;
use alloc::vec;
use alloc::vec::Vec;
use hashbrown::hash_map::Values;
use hashbrown::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DirectedGraph<TNode: Node, TEdge: Edge<Node = TNode>> {
    // Stores real data
    node: HashMap<TNode::NodeIndex, TNode>,
    edge: HashMap<DirectedEdge<TEdge>, TEdge>,
    weak_edge: HashMap<DirectedEdge<TEdge>, TEdge>,

    // Indexes to search nodes
    children: HashMap<TNode::NodeIndex, HashSet<TNode::NodeIndex>>,
    parent: HashMap<TNode::NodeIndex, TNode::NodeIndex>,
}

impl<TNode: Node, TEdge: Edge<Node = TNode>> DirectedGraph<TNode, TEdge> {
    pub fn new() -> Self {
        Self {
            node: HashMap::new(),
            edge: HashMap::new(),
            weak_edge: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
        }
    }

    pub fn nodes(&self) -> Values<TNode::NodeIndex, TNode> {
        self.node.values()
    }

    pub fn edges(&self) -> Values<DirectedEdge<TEdge>, TEdge> {
        self.edge.values()
    }

    pub fn add_node(&mut self, node: &TNode) -> () {
        // NOTE: *Last* inserted node overwhelms existing node when nodes with same name are inserted
        self.node.insert(node.index().clone(), node.clone());
        if !self.children.contains_key(&node.index()) {
            // Initialize children on first time
            self.children.insert(node.index().clone(), HashSet::new());
        }
    }

    pub fn add_edge(&mut self, edge: &TEdge) -> () {
        // Some times explicit node declarations are missed in original mutation graph node
        if self.get_node(&edge.parent()).is_none() {
            self.add_node(&TNode::implicit_new(&edge.parent()))
        }
        if self.get_node(&edge.child()).is_none() {
            self.add_node(&TNode::implicit_new(&edge.child()))
        }

        // Insert edge and update indexes avoiding making closed chains
        match (self.root_of(&edge.parent()), self.root_of(&edge.child())) {
            (Ok(left), Ok(right)) => {
                if left == right {
                    self.add_weak_edge(edge);
                    return;
                }
            }
            _ => (),
        }

        self.edge.insert(DirectedEdge::from(&edge), edge.clone());

        match self.children.get_mut(&edge.parent()) {
            Some(children) => {
                children.insert(edge.child().clone());
            }
            None => {
                self.children.insert(
                    edge.parent().clone(),
                    HashSet::from_iter([edge.child().clone()].iter().cloned()),
                );
            }
        };

        self.parent
            .insert(edge.child().clone(), edge.parent().clone());
    }

    pub fn add_weak_edge(&mut self, edge: &TEdge) {
        self.weak_edge
            .insert(DirectedEdge::from(&edge), edge.clone());
    }

    pub fn get_node(&self, node: &TNode::NodeIndex) -> Option<&TNode> {
        self.node.get(node)
    }

    pub fn get_edge(&self, arrow: &DirectedEdge<TEdge>) -> Option<&TEdge> {
        self.edge.get(arrow)
    }

    pub fn children_of(&self, parent: &TNode::NodeIndex) -> Option<&HashSet<TNode::NodeIndex>> {
        self.children.get(parent)
    }

    pub fn parent_of(&self, child: &TNode::NodeIndex) -> Option<&TNode::NodeIndex> {
        self.parent.get(child)
    }

    pub fn root_of<'a>(
        &'a self,
        node: &'a TNode::NodeIndex,
    ) -> Result<&'a TNode::NodeIndex, TNode> {
        if self.get_node(node).is_none() {
            return Err(GraphError::NodeNotExists(node.clone()));
        }
        match self.parent_of(node) {
            Some(parent) => self.root_of(parent),
            None => Ok(node),
        }
    }

    fn __rank_of(&self, node: &TNode::NodeIndex, rank: usize) -> Result<usize, TNode> {
        match self.parent_of(node) {
            Some(parent) => self.__rank_of(parent, rank + 1),
            None => Ok(rank), // If given node is root, then rank is 0.
        }
    }

    pub fn rank_of(&self, node: &TNode::NodeIndex) -> Result<usize, TNode> {
        self.__rank_of(node, 0)
    }

    pub fn predecessors_of(
        &self,
        node: &TNode::NodeIndex,
    ) -> Result<Vec<&TNode::NodeIndex>, TNode> {
        if self.get_node(node).is_none() {
            return Err(GraphError::NodeNotExists(node.clone()));
        }
        match self.parent_of(node) {
            Some(parent) => match self.predecessors_of(parent) {
                Ok(mut res) => {
                    res.push(parent);
                    Ok(res)
                }
                Err(why) => return Err(why),
            },
            None => Ok(vec![]),
        }
    }

    pub fn self_and_its_predecessors_of(
        &self,
        node: &TNode::NodeIndex,
    ) -> Result<Vec<&TNode::NodeIndex>, TNode> {
        let mut res = self.predecessors_of(node)?;
        match self.get_node(node) {
            Some(node) => res.push(node.index()),
            None => return Err(GraphError::NodeNotExists(node.clone())),
        }
        Ok(res)
    }

    pub fn leaves(&self) -> HashSet<&TNode::NodeIndex> {
        self.children
            .iter()
            .filter(|(_, v)| v.len() == 0)
            .map(|(k, _)| k)
            .collect()
    }

    pub fn roots(&self) -> HashSet<&TNode::NodeIndex> {
        self.node
            .keys()
            .filter(|v| self.parent_of(v).is_none())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::error::GraphError;
    use crate::node::node_index::NodeIndex;
    use crate::node::Node;
    use crate::{DirectedGraph, Edge};
    use alloc::string::String;
    use core::hash::Hash;
    use hashbrown::HashSet;
    use alloc::vec;
    use std::println;

    impl NodeIndex for String {}

    #[derive(Debug, Clone, Eq, PartialEq, Default, Hash)]
    struct TestGraphNode {
        index: String,
    }

    impl Node for TestGraphNode {
        type NodeIndex = String;

        fn implicit_new(index: &Self::NodeIndex) -> Self {
            Self::new(index)
        }

        fn index(&self) -> &Self::NodeIndex {
            &self.index
        }
    }

    impl TestGraphNode {
        pub fn new(index: &String) -> Self {
            Self {
                index: index.clone(),
            }
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq, Default)]
    struct TestGraphEdge {
        parent: String,
        child: String,
    }

    impl TestGraphEdge {
        pub fn new(parent: &String, child: &String) -> Self {
            Self {
                parent: parent.clone(),
                child: child.clone(),
            }
        }
    }

    impl Edge for TestGraphEdge {
        type Node = TestGraphNode;

        fn parent(&self) -> &<Self::Node as Node>::NodeIndex {
            &self.parent
        }

        fn child(&self) -> &<Self::Node as Node>::NodeIndex {
            &self.child
        }
    }

    #[test]
    fn test_mutation_graph_node() {
        let node_1_sha1 = String::from("node_1");
        let no_such_node_sha1 = String::from("no_such_node");

        let mut graph = DirectedGraph::<TestGraphNode, TestGraphEdge>::new();

        let node_1 = TestGraphNode::new(&node_1_sha1);
        graph.add_node(&node_1);

        assert_eq!(graph.get_node(&node_1_sha1), Some(&node_1));
        assert_eq!(graph.get_node(&no_such_node_sha1), None);
    }

    #[test]
    fn test_mutation_graph_edge() {
        let node_1_sha1 = String::from("node_1");
        let node_2_sha1 = String::from("node_2");
        let node_3_sha1 = String::from("node_3");
        let node_4_sha1 = String::from("node_4");
        let node_5_sha1 = String::from("node_5");
        let no_such_node_sha1 = String::from("no_such_node");

        let mut graph = DirectedGraph::new();
        /*
           (1)
           / \
         (2) (3)
              |
             (4)
              |
             (5)
        */
        graph.add_node(&TestGraphNode::new(&node_1_sha1));
        graph.add_node(&TestGraphNode::new(&node_2_sha1));
        graph.add_node(&TestGraphNode::new(&node_3_sha1));
        graph.add_node(&TestGraphNode::new(&node_4_sha1));
        graph.add_node(&TestGraphNode::new(&node_5_sha1));
        graph.add_edge(&TestGraphEdge::new(&node_1_sha1, &node_2_sha1));
        graph.add_edge(&TestGraphEdge::new(&node_1_sha1, &node_3_sha1));
        graph.add_edge(&TestGraphEdge::new(&node_3_sha1, &node_4_sha1));
        graph.add_edge(&TestGraphEdge::new(&node_4_sha1, &node_5_sha1));

        println!("[*] graph = {:#?}", graph);

        assert_eq!(graph.parent_of(&node_1_sha1), None);
        assert_eq!(graph.parent_of(&node_2_sha1), Some(&node_1_sha1));
        assert_eq!(graph.parent_of(&node_3_sha1), Some(&node_1_sha1));

        assert_eq!(graph.root_of(&node_1_sha1), Ok(&node_1_sha1));
        assert_eq!(graph.root_of(&node_4_sha1), Ok(&node_1_sha1));
        assert_eq!(
            graph.root_of(&no_such_node_sha1),
            Err(GraphError::NodeNotExists(no_such_node_sha1.clone()))
        );

        assert_eq!(
            graph.predecessors_of(&node_5_sha1),
            Ok(vec![&node_1_sha1, &node_3_sha1, &node_4_sha1])
        );

        assert_eq!(
            graph.leaves(),
            HashSet::from_iter(vec![&node_2_sha1, &node_5_sha1])
        );

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_sha1]));

        assert_eq!(graph.rank_of(&node_1_sha1), Ok(0));
        assert_eq!(graph.rank_of(&node_5_sha1), Ok(3));
    }

    #[test]
    fn test_mutation_graph_missing_explicit_node_decl() {
        let node_1_sha1 = String::from("node_1");
        let node_2_sha1 = String::from("node_2");
        let node_3_sha1 = String::from("node_3");

        let mut graph = DirectedGraph::new();
        /*
           (1)
           / \
         (2) (3)
        */
        graph.add_edge(&TestGraphEdge::new(&node_1_sha1, &node_2_sha1));
        graph.add_edge(&TestGraphEdge::new(&node_1_sha1, &node_3_sha1));

        assert_eq!(
            graph.nodes().map(|v| v.index()).collect::<HashSet<&String>>(),
            HashSet::from_iter([&node_1_sha1, &node_2_sha1, &node_3_sha1])
        );

        assert_eq!(
            graph.leaves(),
            HashSet::from_iter(vec![&node_2_sha1, &node_3_sha1])
        );

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_sha1]));
    }

    #[test]
    fn test_mutation_graph_cycle_graph() {
        let node_1_sha1 = String::from("node_1");
        let node_2_sha1 = String::from("node_2");
        let node_3_sha1 = String::from("node_3");

        let mut graph = DirectedGraph::new();
        /*
           (1)
           / \
         (2)-(3)
        */
        graph.add_edge(&TestGraphEdge::new(&node_1_sha1, &node_2_sha1));
        graph.add_edge(&TestGraphEdge::new(&node_2_sha1, &node_3_sha1));
        graph.add_edge(&TestGraphEdge::new(&node_3_sha1, &node_1_sha1));

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_sha1]));
    }
}
