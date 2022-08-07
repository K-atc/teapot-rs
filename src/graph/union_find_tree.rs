use crate::edge::basic_edge::BasicEdge;
use crate::error::GraphError;
use crate::node::Node;
use crate::result::Result;
use hashbrown::HashMap;

#[derive(Debug, Clone)]
pub struct UnionFindTree<TNode: Node> {
    parent: HashMap<TNode::NodeIndex, TNode::NodeIndex>, // Child --> Parent
}

impl<TNode: Node> UnionFindTree<TNode> {
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
        }
    }

    pub fn number_of_nodes(&self) -> usize {
        self.parent.len()
    }

    pub fn add<'a>(&mut self, x: &'a TNode::NodeIndex) -> &'a TNode::NodeIndex {
        self.parent.insert(x.clone(), x.clone());
        x
    }

    pub fn find(&self, child: &TNode::NodeIndex) -> Result<TNode::NodeIndex, BasicEdge<TNode>> {
        match self.parent.get(&child) {
            Some(parent) => {
                if parent == child {
                    Ok(child.clone())
                } else {
                    Ok(self.find(parent)?)
                }
            }
            None => Err(GraphError::NodeNotExists(child.clone())),
        }
    }

    pub fn unite(
        &mut self,
        x: &TNode::NodeIndex,
        y: &TNode::NodeIndex,
    ) -> Result<(), BasicEdge<TNode>> {
        if !self.parent.contains_key(x) {
            self.add(x);
        }
        if !self.parent.contains_key(x) {
            self.add(x);
        }
        let x = self.find(&x)?;
        let y = self.find(&y)?;

        if x == y {
            return Ok(());
        }

        self.parent.insert(y, x);
        Ok(())
    }

    pub fn same(
        &self,
        x: &TNode::NodeIndex,
        y: &TNode::NodeIndex,
    ) -> Result<bool, BasicEdge<TNode>> {
        Ok(self.find(x)? == self.find(y)?)
    }
}

#[cfg(test)]
mod test {
    use super::UnionFindTree;
    use crate::node::{basic_node::BasicNode, Node};

    #[test]
    fn test_union_find_tree() {
        let node_1 = BasicNode::<usize>::new(&1);
        let node_2 = BasicNode::<usize>::new(&2);
        let node_3 = BasicNode::<usize>::new(&3);
        let node_4 = BasicNode::<usize>::new(&4);

        #[allow(non_snake_case)]
        let mut T = UnionFindTree::<BasicNode<usize>>::new();

        // Graph updates
        T.add(&node_1.index());
        T.add(&node_2.index());
        T.add(&node_3.index());
        T.add(&node_4.index());
        assert_eq!(T.unite(&node_1.index(), &node_2.index()), Ok(()));
        assert_eq!(T.unite(&node_1.index(), &node_3.index()), Ok(()));

        // Introspection
        assert_eq!(T.find(&node_2.index()), Ok(1));
        assert_eq!(T.find(&node_4.index()), Ok(4));
        assert_eq!(T.same(&node_2.index(), &node_3.index()), Ok(true));
        assert_eq!(T.same(&node_1.index(), &node_4.index()), Ok(false));
    }
}
