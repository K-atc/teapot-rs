use crate::edge::directed_edge::DirectedEdge;
use crate::edge::Edge;
#[allow(unused_imports)]
use crate::error::GraphError;
use crate::io;
use crate::metrics;
use crate::node::Node;
use crate::result::Result;

use alloc::collections::binary_heap::BinaryHeap;
use alloc::collections::btree_map::Values;
use alloc::collections::BTreeMap;
use alloc::string::String;
#[allow(unused_imports)]
use alloc::vec;
#[allow(unused_imports)]
use alloc::vec::Vec;
use core::cmp::Reverse;
use core::fmt;
#[allow(unused_imports)]
use hashbrown::{HashMap, HashSet};
#[cfg(feature = "std")]
#[allow(unused_imports)]
use log::{info, trace};

/// DirectedGraph:
/// * assumes edge is *directed*.
/// * can hold nodes that satisfies:
///     * a given node can have only one root node
///     * ~~each of node has only one parent~~
/// * With `metrics` feature: avoids cycled path. A edge makes a cycle is to be ignored and it is treated as *weak edge* (See implementation of DirectedGraph::add_edge())
/// * Without `metrics` feature: can be hold cycled path.
#[derive(Debug, Clone)]
pub struct DirectedGraph<TEdge: Edge> {
    // Metadata
    /// Graph name
    name: String,

    // Stores real data
    node: BTreeMap<<TEdge::Node as Node>::NodeIndex, TEdge::Node>,
    edge: BTreeMap<DirectedEdge<TEdge>, TEdge>,
    weak_edge: BTreeMap<DirectedEdge<TEdge>, TEdge>,

    // Indexes to search nodes
    #[cfg(feature = "metrics")]
    children: BTreeMap<<TEdge::Node as Node>::NodeIndex, HashSet<<TEdge::Node as Node>::NodeIndex>>,
    #[cfg(feature = "metrics")]
    parent: BTreeMap<<TEdge::Node as Node>::NodeIndex, <TEdge::Node as Node>::NodeIndex>,
}

impl<TEdge: Edge> fmt::Display for DirectedGraph<TEdge> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\n")?;
        for edge in self.edge.keys() {
            write!(f, "\t{:?} -> {:?}\n", edge.parent(), edge.child())?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<TEdge: Edge> DirectedGraph<TEdge> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            // NOTE: Do not use `HashMap::new()`. `HashMap::with_capacity()` avoids assertion fail
            node: BTreeMap::new(),
            edge: BTreeMap::new(),
            weak_edge: BTreeMap::new(),
            #[cfg(feature = "metrics")]
            children: BTreeMap::new(),
            #[cfg(feature = "metrics")]
            parent: BTreeMap::new(),
        }
    }

    pub fn nodes(&self) -> Values<<TEdge::Node as Node>::NodeIndex, TEdge::Node> {
        self.node.values()
    }

    pub fn edges(&self) -> Values<DirectedEdge<TEdge>, TEdge> {
        self.edge.values()
    }

    pub fn add_node(&mut self, node: &TEdge::Node) -> () {
        // NOTE: *Last* inserted node overwhelms existing node when nodes with same name are inserted
        self.node.insert(node.index().clone(), node.clone());
        metrics! {
            if !self.children.contains_key(&node.index()) {
                // Initialize children on first time
                // NOTE: Do not use HashSet::new(). HashSet::with_capacity() avoids asertion fail related to SSE
                self.children
                    .insert(node.index().clone(), HashSet::with_capacity(8));
            }
        }
    }

    pub fn add_edge(&mut self, edge: &TEdge) -> () {
        // Some times explicit node declarations are missed in original mutation graph node
        if self.get_node(&edge.parent()).is_none() {
            self.add_node(&TEdge::Node::implicit_new(&edge.parent()))
        }
        if self.get_node(&edge.child()).is_none() {
            self.add_node(&TEdge::Node::implicit_new(&edge.child()))
        }

        if edge.parent() == edge.child() {
            // This is self loop
            self.add_weak_edge(edge);
            return;
        }

        trace!("add_edge({} : {} -> {})", edge, edge.parent(), edge.child());

        metrics! {
            // Insert edge and update indexes avoiding making closed chains
            // NOTE: Cannot support this workaround without `metrics` feature
            match (self.root_of(&edge.parent()), self.root_of(&edge.child())) {
                (Ok(left), Ok(right)) => {
                    if left == right {
                        self.add_weak_edge(edge);
                        return;
                    }
                }
                _ => (),
            }
        }

        self.edge.insert(DirectedEdge::from(&edge), edge.clone());

        metrics! {
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
        }

        metrics! {{
            // debug_assert_eq!(self.parent.contains_key(edge.child()), false);
            self.parent
                .insert(edge.child().clone(), edge.parent().clone());
        }}
    }

    pub fn add_weak_edge(&mut self, edge: &TEdge) {
        self.weak_edge
            .insert(DirectedEdge::from(&edge), edge.clone());
    }

    pub fn get_node(&self, node: &<TEdge::Node as Node>::NodeIndex) -> Option<&TEdge::Node> {
        self.node.get(node)
    }

    pub fn get_edge(&self, arrow: &DirectedEdge<TEdge>) -> Option<&TEdge> {
        self.edge.get(arrow)
    }

    #[cfg(feature = "metrics")]
    pub fn children_of(
        &self,
        parent: &<TEdge::Node as Node>::NodeIndex,
    ) -> Option<&HashSet<<TEdge::Node as Node>::NodeIndex>> {
        self.children.get(parent)
    }

    #[cfg(feature = "metrics")]
    pub fn parent_of(
        &self,
        child: &<TEdge::Node as Node>::NodeIndex,
    ) -> Option<&<TEdge::Node as Node>::NodeIndex> {
        self.parent.get(child)
    }

    #[cfg(feature = "metrics")]
    fn __root_of<'a>(
        &'a self,
        node: &'a <TEdge::Node as Node>::NodeIndex,
        ttl: usize,
    ) -> Result<&'a <TEdge::Node as Node>::NodeIndex, TEdge> {
        if ttl == 0 {
            return Err(GraphError::ReachedRecursionLimit);
        }
        if self.get_node(node).is_none() {
            return Err(GraphError::NodeNotExists(node.clone()));
        }
        match self.parent_of(node) {
            Some(parent) => self.__root_of(parent, ttl - 1),
            None => Ok(node),
        }
    }

    #[cfg(feature = "metrics")]
    pub fn root_of<'a>(
        &'a self,
        node: &'a <TEdge::Node as Node>::NodeIndex,
    ) -> Result<&'a <TEdge::Node as Node>::NodeIndex, TEdge> {
        self.__root_of(node, self.node.len())
    }

    #[cfg(feature = "metrics")]
    fn __rank_of(
        &self,
        node: &<TEdge::Node as Node>::NodeIndex,
        degree: usize,
    ) -> Result<usize, TEdge> {
        match self.parent_of(node) {
            Some(parent) => self.__rank_of(parent, degree + 1),
            None => Ok(degree), // If given node is root, then degree is 0.
        }
    }

    #[cfg(feature = "metrics")]
    pub fn rank_of(&self, node: &<TEdge::Node as Node>::NodeIndex) -> Result<usize, TEdge> {
        self.__rank_of(node, 0)
    }

    #[cfg(feature = "metrics")]
    pub fn predecessors_of(
        &self,
        node: &<TEdge::Node as Node>::NodeIndex,
    ) -> Result<Vec<&<TEdge::Node as Node>::NodeIndex>, TEdge> {
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

    #[cfg(feature = "metrics")]
    pub fn self_and_its_predecessors_of(
        &self,
        node: &<TEdge::Node as Node>::NodeIndex,
    ) -> Result<Vec<&<TEdge::Node as Node>::NodeIndex>, TEdge> {
        let mut res = self.predecessors_of(node)?;
        match self.get_node(node) {
            Some(node) => res.push(node.index()),
            None => return Err(GraphError::NodeNotExists(node.clone())),
        }
        Ok(res)
    }

    /// Checks if nodes *from* and *to* is on the same path
    #[cfg(feature = "metrics")]
    pub fn are_on_the_path(
        &self,
        from: &<TEdge::Node as Node>::NodeIndex,
        to: &<TEdge::Node as Node>::NodeIndex,
    ) -> bool {
        if from == to {
            return true;
        }
        match self.parent_of(from) {
            Some(parent) => {
                if parent == to {
                    true
                } else {
                    self.are_on_the_path(parent, to)
                }
            }
            None => false,
        }
    }

    /// Collects leaves (i.e. nodes that does not have children) from entire this graph
    #[cfg(feature = "metrics")]
    pub fn leaves(&self) -> HashSet<&<TEdge::Node as Node>::NodeIndex> {
        let mut result = HashSet::with_capacity(8); // NOTE: Do not use collect(); HashSet::with_capacity() avoids assertion fail in Intel Pin
        for child in self
            .children
            .iter()
            .filter(|(_, v)| v.len() == 0)
            .map(|(k, _)| k)
        {
            result.insert(child);
        }
        result
    }

    /// Collects leaves (i.e. nodes that does not have children) of given node
    #[cfg(feature = "metrics")]
    pub fn leaves_of<'a>(
        &'a self,
        node: &'a <TEdge::Node as Node>::NodeIndex,
    ) -> Result<HashSet<&'a <TEdge::Node as Node>::NodeIndex>, TEdge> {
        let mut result = HashSet::with_capacity(8);
        for leaf in self.leaves() {
            if self.are_on_the_path(leaf, node) {
                result.insert(leaf);
            }
        }
        Ok(result)
    }

    #[cfg(feature = "metrics")]
    pub fn roots(&self) -> HashSet<&<TEdge::Node as Node>::NodeIndex> {
        let mut result = HashSet::with_capacity(8); // NOTE: Do not use collect(); HashSet::with_capacity() avoids assertion fail in Intel Pin
        for root in self.node
            .keys()
            .filter(|v| self.parent_of(v).is_none()) {
            result.insert(root);
        }
        result
    }

    #[cfg(feature = "metrics")]
    pub fn is_root(&self, node: &<TEdge::Node as Node>::NodeIndex) -> Result<bool, TEdge> {
        Ok(self.root_of(node)? == node)
    }

    pub fn gml_write<T: io::Write>(&self, file: &mut T) -> Result<(), TEdge> {
        write!(file, "graph [\n")?;
        write!(file, "  directed 1\n")?;
        write!(file, "  name \"{}\"\n", self.name)?;

        let mut index_to_id = HashMap::with_capacity(self.node.len());

        {
            let heap: BinaryHeap<Reverse<&TEdge::Node>> =
                self.node.values().map(|v| Reverse(v)).collect();
            for (id, node) in heap.into_iter_sorted().enumerate() {
                index_to_id.insert(node.0.index(), id);

                write!(file, "  node [\n")?;
                write!(file, "    id {}\n", id)?;
                write!(file, "    label \"{}\"\n", node.0)?;
                // metrics! {{
                //     write!(file, "    rank {}\n", self.rank_of(index.0)?)?;
                //     write!(file, "    is_root {}\n", if self.is_root(index.0)? { 1 } else { 0 })?;
                // }}
                write!(file, "  ]\n")?;
            }
        }
        {
            let heap: BinaryHeap<Reverse<&TEdge>> =
                self.edge.values().map(|v| Reverse(v)).collect();
            for edge in heap.into_iter_sorted() {
                if let (Some(source), Some(target)) = (
                    index_to_id.get(edge.0.parent()),
                    index_to_id.get(edge.0.child()),
                ) {
                    write!(file, "  edge [\n")?;
                    write!(file, "    source {}\n", source)?;
                    write!(file, "    target {}\n", target)?;
                    write!(file, "    label \"{}\"\n", edge.0)?;
                    write!(file, "  ]\n")?;
                }
            }
        }
        write!(file, "]\n")?;

        Ok(())
    }

    pub fn dot_write<T: io::Write>(&self, file: &mut T) -> Result<(), TEdge> {
        write!(file, "digraph {{\n")?;

        let mut index_to_id = HashMap::with_capacity(self.node.len());

        {
            // Write nodes
            let heap: BinaryHeap<Reverse<&TEdge::Node>> =
                self.node.values().map(|v| Reverse(v)).collect();
            for (id, node) in heap.into_iter_sorted().enumerate() {
                index_to_id.insert(node.0.index(), id);

                write!(file, "  {} [label=\"{}\"]\n", id, node.0)?;
            }
        }
        {
            // Write edges
            let heap: BinaryHeap<Reverse<&TEdge>> =
                self.edge.values().map(|v| Reverse(v)).collect();
            for edge in heap.into_iter_sorted() {
                if let (Some(source), Some(target)) = (
                    index_to_id.get(edge.0.parent()),
                    index_to_id.get(edge.0.child()),
                ) {
                    write!(file, "  {} -> {} [label=\"{}\"]\n", source, target, edge.0)?;
                }
            }
        }
        write!(file, "}}\n")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::edge::basic_edge::BasicEdge;
    #[allow(unused_imports)]
    use crate::error::GraphError;
    use crate::graph::directed_graph::DirectedGraph;
    use crate::metrics;
    use crate::node::basic_node::BasicNode;
    use crate::node::node_index::NodeIndex;
    use crate::node::Node;

    use alloc::string::String;
    #[allow(unused_imports)]
    use alloc::vec;
    use alloc::vec::Vec;
    use difference::Changeset;
    use hashbrown::HashSet;
    use io::Read;
    #[cfg(feature = "std")]
    use std::{fs::File, io, println, str};

    type TestGraphNode = BasicNode<String>;
    type TestGraphEdge = BasicEdge<TestGraphNode>;

    #[test]
    fn test_directed_graph_node() {
        let node_1_index = String::from("node_1");
        let no_such_node_sha1 = String::from("no_such_node");

        let mut graph = DirectedGraph::<TestGraphEdge>::new(String::from("test"));

        let node_1 = TestGraphNode::new(&node_1_index);
        graph.add_node(&node_1);

        assert_eq!(graph.get_node(&node_1_index), Some(&node_1));
        assert_eq!(graph.get_node(&no_such_node_sha1), None);
    }

    #[test]
    #[cfg(feature = "metrics")]
    fn test_directed_graph_edge() {
        let node_1_index = String::from("node_1");
        let node_2_index = String::from("node_2");
        let node_3_index = String::from("node_3");
        let node_4_index = String::from("node_4");
        let node_5_index = String::from("node_5");
        let no_such_node_sha1 = String::from("no_such_node");

        let mut graph = DirectedGraph::new(String::from("test"));
        /*
           (1)
           / \
         (2) (3)
              |
             (4)
              |
             (5)
        */
        graph.add_node(&TestGraphNode::new(&node_1_index));
        graph.add_node(&TestGraphNode::new(&node_2_index));
        graph.add_node(&TestGraphNode::new(&node_3_index));
        graph.add_node(&TestGraphNode::new(&node_4_index));
        graph.add_node(&TestGraphNode::new(&node_5_index));
        graph.add_edge(&TestGraphEdge::new(
            &node_1_index,
            &node_2_index,
            String::from("1->2"),
        ));
        graph.add_edge(&TestGraphEdge::new(
            &node_1_index,
            &node_3_index,
            String::from("1->3"),
        ));
        graph.add_edge(&TestGraphEdge::new(
            &node_3_index,
            &node_4_index,
            String::from("3->4"),
        ));
        graph.add_edge(&TestGraphEdge::new(
            &node_4_index,
            &node_5_index,
            String::from("4->5"),
        ));

        println!("[*] graph = {:#?}", graph);

        assert_eq!(graph.parent_of(&node_1_index), None);
        assert_eq!(graph.parent_of(&node_2_index), Some(&node_1_index));
        assert_eq!(graph.parent_of(&node_3_index), Some(&node_1_index));

        assert!(graph.are_on_the_path(&node_2_index, &node_1_index));
        assert!(graph.are_on_the_path(&node_5_index, &node_1_index));

        assert_eq!(graph.root_of(&node_1_index), Ok(&node_1_index));
        assert_eq!(graph.root_of(&node_4_index), Ok(&node_1_index));
        assert_eq!(graph.root_of(&node_5_index), Ok(&node_1_index));
        assert_eq!(
            graph.root_of(&no_such_node_sha1),
            Err(GraphError::NodeNotExists(no_such_node_sha1.clone()))
        );

        assert_eq!(
            graph.predecessors_of(&node_5_index),
            Ok(vec![&node_1_index, &node_3_index, &node_4_index])
        );

        assert_eq!(
            graph.leaves(),
            HashSet::from_iter(vec![&node_2_index, &node_5_index])
        );
        assert_eq!(
            graph.leaves_of(&node_1_index),
            Ok(HashSet::from_iter([&node_2_index, &node_5_index]))
        );
        assert_eq!(
            graph.leaves_of(&node_3_index),
            Ok(HashSet::from_iter([&node_5_index]))
        );

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_index]));

        assert_eq!(graph.rank_of(&node_1_index), Ok(0));
        assert_eq!(graph.rank_of(&node_5_index), Ok(3));
    }

    #[test]
    fn test_directed_graph_missing_explicit_node_decl() {
        let node_1_index = String::from("node_1");
        let node_2_index = String::from("node_2");
        let node_3_index = String::from("node_3");

        let mut graph = DirectedGraph::new(String::from("test"));
        /*
           (1)
           / \
         (2) (3)
        */
        graph.add_edge(&TestGraphEdge::new(
            &node_1_index,
            &node_2_index,
            String::from("1->2"),
        ));
        graph.add_edge(&TestGraphEdge::new(
            &node_1_index,
            &node_3_index,
            String::from("1->3"),
        ));

        assert_eq!(
            graph
                .nodes()
                .map(|v| v.index())
                .collect::<HashSet<&String>>(),
            HashSet::from_iter([&node_1_index, &node_2_index, &node_3_index])
        );

        metrics! {
            assert_eq!(
                graph.leaves(),
                HashSet::from_iter(vec![&node_2_index, &node_3_index])
            );
        }

        metrics! {
            assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_index]));
        }
    }

    #[test]
    #[cfg(feature = "metrics")]
    fn test_directed_graph_cycle_graph() {
        let node_1_index = String::from("node_1");
        let node_2_index = String::from("node_2");
        let node_3_index = String::from("node_3");

        let mut graph = DirectedGraph::new(String::from("test"));
        /*
           (1)
           / \
         (2)-(3)
        */
        graph.add_edge(&TestGraphEdge::new(
            &node_1_index,
            &node_2_index,
            String::from("1->2"),
        ));
        graph.add_edge(&TestGraphEdge::new(
            &node_2_index,
            &node_3_index,
            String::from("2->3"),
        ));
        graph.add_edge(&TestGraphEdge::new(
            &node_3_index,
            &node_1_index,
            String::from("3->1"),
        ));

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_index]));
        assert_eq!(graph.root_of(&node_2_index), Ok(&node_1_index));
        assert_eq!(graph.root_of(&node_3_index), Ok(&node_1_index));
    }

    #[test]
    fn test_directed_graph_with_self_loop() {
        let node_1_index = String::from("node_1");
        let node_2_index = String::from("node_2");

        let mut graph = DirectedGraph::new(String::from("test"));
        /*
           (1)
            |
           (2)
         ↺
        */
        graph.add_edge(&TestGraphEdge::new(
            &node_1_index,
            &node_2_index,
            String::from("1->2"),
        ));
        graph.add_edge(&TestGraphEdge::new(
            &node_2_index,
            &node_2_index,
            String::from("2->2"),
        ));

        metrics! {
            assert_eq!(graph.parent_of(&node_2_index), Some(&node_1_index));
        }
    }

    #[test]
    fn test_directed_graph_real_sample() {
        type Edge = BasicEdge<BasicNode<u64>>;
        impl NodeIndex for u64 {}

        let mut graph = DirectedGraph::new(String::from("test"));
        graph.add_edge(&Edge::new(&0x421493, &0x41c2d9, String::from("1")));
        graph.add_edge(&Edge::new(&0x41c2d9, &0x402566, String::from("2")));
        graph.add_edge(&Edge::new(&0x41c33c, &0x421493, String::from("3")));
    }

    #[test]
    fn test_directed_graph_xxx_write() {
        let node_1_index = String::from("node_1");
        let node_2_index = String::from("node_2");
        let node_3_index = String::from("node_3");
        let node_4_index = String::from("node_4");

        let mut graph = DirectedGraph::new(String::from("test"));
        /*
           (1)
           / \
         (2) (3)
              |
             (4)
        */
        graph.add_edge(&TestGraphEdge::new(
            &node_1_index,
            &node_2_index,
            String::from("1->2"),
        ));
        graph.add_edge(&TestGraphEdge::new(
            &node_1_index,
            &node_3_index,
            String::from("1->3"),
        ));
        graph.add_edge(&TestGraphEdge::new(
            &node_3_index,
            &node_4_index,
            String::from("3->4"),
        ));

        {
            let mut out_gml = io::Cursor::new(Vec::new());
            assert!(graph.gml_write(&mut out_gml).is_ok());

            // #[cfg(feature = "metrics")]
            // let mut true_file = File::open("tests/test_directed_graph_gml_write.gml").unwrap();
            // #[cfg(not(feature = "metrics"))]
            let mut true_file =
                File::open("tests/test_directed_graph_xxx_write.minimal.gml").unwrap();
            let mut true_gml = Vec::new();
            assert!(true_file.read_to_end(&mut true_gml).is_ok());

            println!(
                "{}",
                Changeset::new(
                    str::from_utf8(true_gml.as_slice()).unwrap(),
                    str::from_utf8(out_gml.get_ref()).unwrap(),
                    ""
                )
            );

            assert_eq!(out_gml.get_ref(), &true_gml);
        }

        {
            let mut out_dot = io::Cursor::new(Vec::new());
            assert!(graph.dot_write(&mut out_dot).is_ok());

            let mut true_file =
                File::open("tests/test_directed_graph_xxx_write.minimal.dot").unwrap();
            let mut true_dot = Vec::new();
            assert!(true_file.read_to_end(&mut true_dot).is_ok());

            println!(
                "{}",
                Changeset::new(
                    str::from_utf8(true_dot.as_slice()).unwrap(),
                    str::from_utf8(out_dot.get_ref()).unwrap(),
                    ""
                )
            );

            assert_eq!(out_dot.get_ref(), &true_dot);
        }
    }

    #[test]
    fn test_directed_graph_multi_root() {
        let node_1_index = String::from("node_1");
        let node_2_index = String::from("node_2");
        let node_3_index = String::from("node_3");

        let mut graph = DirectedGraph::new(String::from("test"));
        /*
          (1) (3)
            \ /
            (2)
        */
        graph.add_edge(&TestGraphEdge::new(
            &node_1_index,
            &node_2_index,
            String::from("1->2"),
        ));
        graph.add_edge(&TestGraphEdge::new(
            &node_3_index,
            &node_2_index,
            String::from("3->2"),
        ));

        #[cfg(feature = "metrics")]
        {
            // assert!(graph.are_on_the_path(&node_2_index, &node_1_index)); // TODO: DirectedGraph does not support
            assert!(graph.are_on_the_path(&node_2_index, &node_3_index));
        }
    }
}
