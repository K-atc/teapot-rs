use crate::edge::directed_edge::DirectedEdge;
use crate::edge::Edge;
use crate::error::GraphError;
use crate::io;
use crate::node::Node;
use crate::result::Result;
use alloc::collections::binary_heap::BinaryHeap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Reverse;
use hashbrown::hash_map::Values;
use hashbrown::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DirectedGraph<TEdge: Edge> {
    // Metadata
    name: String,

    // Stores real data
    node: HashMap<<TEdge::Node as Node>::NodeIndex, TEdge::Node>,
    edge: HashMap<DirectedEdge<TEdge>, TEdge>,
    weak_edge: HashMap<DirectedEdge<TEdge>, TEdge>,

    // Indexes to search nodes
    children: HashMap<<TEdge::Node as Node>::NodeIndex, HashSet<<TEdge::Node as Node>::NodeIndex>>,
    parent: HashMap<<TEdge::Node as Node>::NodeIndex, <TEdge::Node as Node>::NodeIndex>,
}

impl<TEdge: Edge> DirectedGraph<TEdge> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            node: HashMap::new(),
            edge: HashMap::new(),
            weak_edge: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
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
        if !self.children.contains_key(&node.index()) {
            // Initialize children on first time
            self.children.insert(node.index().clone(), HashSet::new());
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

    pub fn get_node(&self, node: &<TEdge::Node as Node>::NodeIndex) -> Option<&TEdge::Node> {
        self.node.get(node)
    }

    pub fn get_edge(&self, arrow: &DirectedEdge<TEdge>) -> Option<&TEdge> {
        self.edge.get(arrow)
    }

    pub fn children_of(
        &self,
        parent: &<TEdge::Node as Node>::NodeIndex,
    ) -> Option<&HashSet<<TEdge::Node as Node>::NodeIndex>> {
        self.children.get(parent)
    }

    pub fn parent_of(
        &self,
        child: &<TEdge::Node as Node>::NodeIndex,
    ) -> Option<&<TEdge::Node as Node>::NodeIndex> {
        self.parent.get(child)
    }

    pub fn root_of<'a>(
        &'a self,
        node: &'a <TEdge::Node as Node>::NodeIndex,
    ) -> Result<&'a <TEdge::Node as Node>::NodeIndex, TEdge> {
        if self.get_node(node).is_none() {
            return Err(GraphError::NodeNotExists(node.clone()));
        }
        match self.parent_of(node) {
            Some(parent) => self.root_of(parent),
            None => Ok(node),
        }
    }

    fn __degree_of(
        &self,
        node: &<TEdge::Node as Node>::NodeIndex,
        degree: usize,
    ) -> Result<usize, TEdge> {
        match self.parent_of(node) {
            Some(parent) => self.__degree_of(parent, degree + 1),
            None => Ok(degree), // If given node is root, then degree is 0.
        }
    }

    pub fn degree_of(&self, node: &<TEdge::Node as Node>::NodeIndex) -> Result<usize, TEdge> {
        self.__degree_of(node, 0)
    }

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

    pub fn leaves(&self) -> HashSet<&<TEdge::Node as Node>::NodeIndex> {
        self.children
            .iter()
            .filter(|(_, v)| v.len() == 0)
            .map(|(k, _)| k)
            .collect()
    }

    pub fn roots(&self) -> HashSet<&<TEdge::Node as Node>::NodeIndex> {
        self.node
            .keys()
            .filter(|v| self.parent_of(v).is_none())
            .collect()
    }

    pub fn gml_write<T: io::Write>(&self, file: &mut T) -> Result<(), TEdge> {
        write!(file, "graph [\n")?;
        write!(file, "  directed 1\n")?;
        write!(file, "  name \"{}\"\n", self.name)?;

        let mut index_to_id = HashMap::new();

        {
            let heap: BinaryHeap<Reverse<&<TEdge::Node as Node>::NodeIndex>> =
                self.node.keys().map(|v| Reverse(v)).collect();
            for (id, index) in heap.into_iter_sorted().enumerate() {
                index_to_id.insert(index.0, id);

                write!(file, "  node [\n")?;
                write!(file, "    id {}\n", id)?;
                write!(file, "    label \"{}\"\n", index.0)?;
                write!(file, "  ]\n")?;
            }
        }
        {
            let heap: BinaryHeap<Reverse<&DirectedEdge<TEdge>>> =
                self.edge.keys().map(|v| Reverse(v)).collect();
            for edge in heap.into_iter_sorted() {
                if let (Some(source), Some(target)) = (
                    index_to_id.get(edge.0.parent()),
                    index_to_id.get(edge.0.child()),
                ) {
                    write!(file, "  edge [\n")?;
                    write!(file, "    source {}\n", source)?;
                    write!(file, "    target {}\n", target)?;
                    write!(file, "  ]\n")?;
                }
            }
        }
        write!(file, "]\n")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::edge::basic_edge::BasicEdge;
    use crate::error::GraphError;
    use crate::graph::directed_graph::DirectedGraph;
    use crate::node::basic_node::BasicNode;
    use crate::node::node_index::NodeIndex;
    use crate::node::Node;
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;
    use difference::Changeset;
    use hashbrown::HashSet;
    use io::Read;
    use std::fs::File;
    use std::io;
    use std::println;
    use std::str;

    impl NodeIndex for String {}

    type TestGraphNode = BasicNode<String>;
    type TestGraphEdge = BasicEdge<String>;

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
        graph.add_edge(&TestGraphEdge::new(&node_1_index, &node_2_index));
        graph.add_edge(&TestGraphEdge::new(&node_1_index, &node_3_index));
        graph.add_edge(&TestGraphEdge::new(&node_3_index, &node_4_index));
        graph.add_edge(&TestGraphEdge::new(&node_4_index, &node_5_index));

        println!("[*] graph = {:#?}", graph);

        assert_eq!(graph.parent_of(&node_1_index), None);
        assert_eq!(graph.parent_of(&node_2_index), Some(&node_1_index));
        assert_eq!(graph.parent_of(&node_3_index), Some(&node_1_index));

        assert_eq!(graph.root_of(&node_1_index), Ok(&node_1_index));
        assert_eq!(graph.root_of(&node_4_index), Ok(&node_1_index));
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

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_index]));

        assert_eq!(graph.degree_of(&node_1_index), Ok(0));
        assert_eq!(graph.degree_of(&node_5_index), Ok(3));
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
        graph.add_edge(&TestGraphEdge::new(&node_1_index, &node_2_index));
        graph.add_edge(&TestGraphEdge::new(&node_1_index, &node_3_index));

        assert_eq!(
            graph
                .nodes()
                .map(|v| v.index())
                .collect::<HashSet<&String>>(),
            HashSet::from_iter([&node_1_index, &node_2_index, &node_3_index])
        );

        assert_eq!(
            graph.leaves(),
            HashSet::from_iter(vec![&node_2_index, &node_3_index])
        );

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_index]));
    }

    #[test]
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
        graph.add_edge(&TestGraphEdge::new(&node_1_index, &node_2_index));
        graph.add_edge(&TestGraphEdge::new(&node_2_index, &node_3_index));
        graph.add_edge(&TestGraphEdge::new(&node_3_index, &node_1_index));

        assert_eq!(graph.roots(), HashSet::from_iter(vec![&node_1_index]));
    }

    #[test]
    fn test_directed_graph_gml_write() {
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
        graph.add_edge(&TestGraphEdge::new(&node_1_index, &node_2_index));
        graph.add_edge(&TestGraphEdge::new(&node_1_index, &node_3_index));
        graph.add_edge(&TestGraphEdge::new(&node_3_index, &node_4_index));

        let mut out_gml = io::Cursor::new(Vec::new());
        assert!(graph.gml_write(&mut out_gml).is_ok());

        let mut true_file = File::open("tests/test_directed_graph_gml_write.gml").unwrap();
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
}
