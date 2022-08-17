use crate::node::Node;
use core::cmp::min;
use alloc::collections::BTreeMap;
#[cfg(feature = "std")]
#[allow(unused_imports)]
use log::{info, trace};

#[derive(Debug, Clone)]
pub struct UnionFindTree<TNode: Node> {
    parent: BTreeMap<TNode::NodeIndex, TNode::NodeIndex>, // Child --> Parent
}

impl<TNode: Node> UnionFindTree<TNode> {
    pub fn new() -> Self {
        Self {
            parent: BTreeMap::new(),
        }
    }

    pub fn number_of_nodes(&self) -> usize {
        self.parent.len()
    }

    pub fn find(&self, child: &TNode::NodeIndex) -> TNode::NodeIndex {
        trace!("find({:?})", child);

        match self.parent.get(&child) {
            Some(parent) => {
                if parent == child {
                    child.clone()
                } else {
                    self.find(parent)
                }
            }
            None => child.clone(),
        }
    }

    pub fn unite(&mut self, x: &TNode::NodeIndex, y: &TNode::NodeIndex) -> () {
        let root_x = self.find(&x);
        let root_y = self.find(&y);

        trace!("unite({:?}, {:?}) = {:?}", x, y, min(&root_x, &root_y));

        if root_x == root_y {
            return;
        }

        if root_x < root_y {
            // NOTE: Smaller node is parent
            self.parent.insert(root_y, root_x);
        } else {
            self.parent.insert(root_x, root_y);
        }
    }

    pub fn same(&self, x: &TNode::NodeIndex, y: &TNode::NodeIndex) -> bool {
        self.find(x) == self.find(y)
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
        assert_eq!(T.unite(&node_1.index(), &node_2.index()), ());
        assert_eq!(T.unite(&node_1.index(), &node_3.index()), ());
        assert_eq!(T.unite(&node_2.index(), &node_3.index()), ()); // Make a cycle

        // Introspection
        assert_eq!(T.find(&node_2.index()), 1);
        assert_eq!(T.find(&node_4.index()), 4);
        assert_eq!(T.same(&node_2.index(), &node_3.index()), true);
        assert_eq!(T.same(&node_1.index(), &node_4.index()), false);
    }

    #[test]
    fn test_union_find_tree_reversed() {
        let node_1 = BasicNode::<usize>::new(&1);
        let node_2 = BasicNode::<usize>::new(&2);
        let node_3 = BasicNode::<usize>::new(&3);

        #[allow(non_snake_case)]
        let mut T = UnionFindTree::<BasicNode<usize>>::new();

        // Graph updates
        assert_eq!(T.unite(&node_2.index(), &node_1.index()), ());
        assert_eq!(T.unite(&node_3.index(), &node_1.index()), ());
        assert_eq!(T.unite(&node_3.index(), &node_2.index()), ()); // Make a cycle

        // Introspection
        assert_eq!(T.find(&node_2.index()), 1);
        assert_eq!(T.same(&node_2.index(), &node_3.index()), true);
    }
}
