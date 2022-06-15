use std::collections::HashMap;
use crate::{Node, NodeIndex, VariableIndex};

/// Functions that any representation of an XDD must have, although some representations
/// will execute this more quickly than others, at the cost of more memory capacity.
pub trait XDDBase {
    /// Get the node pointed to by a NodeIndex. panic if it does not exist.
    /// Do NOT call with the two special node indices NodeIndex::TRUE or NodeIndex::FALSE
    fn node(&self,index:NodeIndex) -> Node;
    /// Get the node index for a node if it is already present.
    fn find_node_index(&self,node:Node) -> Option<NodeIndex>;
    /// Add a node to the list, returning its new index.
    fn add_node(&mut self,node:Node) -> NodeIndex;
    /// The number of nodes in this tree, not counting the two special node indices.
    fn len(&self) -> usize;

    /// Like add_node, but first check with find_node_index to see if it is already there.
    fn add_node_if_not_present(&mut self,node:Node) -> NodeIndex {
        self.find_node_index(node).unwrap_or_else(||self.add_node(node))
    }

    /// Produce a DD that describes a single variable. That is, a DD that has just that variable leading to TRUE iff variable is true. This has different meanings for a BDD and ZDD.
    fn single_variable(&mut self,variable:VariableIndex) -> NodeIndex {
        self.add_node_if_not_present(Node::single_variable(variable))
    }

    fn print_with_indentation(&self,index:NodeIndex,indentation:usize) {
        print!("{: <1$}", "", indentation);
        if index.is_sink() { println!("{}",if index.is_true() {1} else {0}); }
        else {
            let node = self.node(index);
            println!("if variable {}",node.variable);
            self.print_with_indentation(node.hi,indentation+1);
            print!("{: <1$}", "", indentation);
            self.print_with_indentation(node.lo,indentation+1);
            println!("else")
        }
    }
    fn print(&self,index:NodeIndex) {
        self.print_with_indentation(index,0);
    }

    /// Evaluate as a BDD with given variables.
    fn evaluate_bdd(&self,index:NodeIndex,variables:&[bool]) -> bool {
        let mut index = index;
        while !index.is_sink() {
            let node = self.node(index);
            index = if variables[node.variable.0 as usize] {node.hi} else {node.lo}
        }
        index.is_true()
    }

    /// Make a node representing the negation of the function represented by the input node. A.k.a. ~ or !.
    /// TODO support caching of not.
    fn not_bdd(&mut self,index:NodeIndex) -> NodeIndex {
        if index.is_true() { NodeIndex::FALSE }
        else if index.is_false() { NodeIndex::TRUE }
        else {
            let node = self.node(index);
            let newnode = Node {
                variable: node.variable,
                lo: self.not_bdd(node.lo),
                hi: self.not_bdd(node.hi),
            };
            self.add_node_if_not_present(newnode)
        }
    }

    /// Make a node representing index1 and index2 (and in the logical sense, a.k.a. ∧ or &&)
    /// TODO support general ops, and support caching of operations
    fn and_bdd(&mut self,index1:NodeIndex,index2:NodeIndex) -> NodeIndex {
        if index1.is_false() || index2.is_false() { NodeIndex::FALSE }
        else if index1.is_true() { index2 }
        else if index2.is_true() { index1 }
        else {
            let node1 = self.node(index1);
            let node2 = self.node(index2);
            let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1,index1)};
            let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2,index2)};
            let lo = self.and_bdd(lo1,lo2);
            let hi = self.and_bdd(hi1,hi2);
            if lo==hi { lo } else {
                let variable = if node1.variable <= node2.variable { node1.variable } else {node2.variable};
                self.add_node_if_not_present(Node{variable,lo,hi})
            }
        }
    }
}



/// A list of all the nodes.
/// This is a compact representation of nodes that is all that is needed to serialize/deserialize,
/// although it is not ideal for many operations that need hash table look-ups.
/// In particular find_node_index is slow.
///
/// Note that the two special indices are not explicitly stored.
#[derive(Clone,Eq, PartialEq,Default)]
pub struct NodeList {
    pub(crate) nodes : Vec<Node>,
}

impl XDDBase for NodeList {
    fn node(&self, index: NodeIndex) -> Node { self.nodes[(index.0-2) as usize] }
    fn find_node_index(&self, node: Node) -> Option<NodeIndex> {
        self.nodes.iter().position(|n|*n==node).map(|i|NodeIndex((i+2) as u32))
    }
    fn add_node(&mut self, node: Node) -> NodeIndex {
        self.nodes.push(node);
        NodeIndex(1+self.nodes.len() as u32)
    }

    fn len(&self) -> usize { self.nodes.len() }
}

/// An extension to NodeList that contains a cache from nodes to indices that is constantly
/// kept up to date.
#[derive(Clone,Eq, PartialEq,Default)]
pub struct NodeListWithFastLookup {
    pub(crate) nodes : NodeList,
    pub(crate) node_to_index : HashMap<Node,NodeIndex>,
}

impl XDDBase for NodeListWithFastLookup {
    fn node(&self, index: NodeIndex) -> Node { self.nodes.node(index) }
    fn find_node_index(&self, node: Node) -> Option<NodeIndex> {
        self.node_to_index.get(&node).cloned()
    }

    fn add_node(&mut self, node: Node) -> NodeIndex {
        let res = self.nodes.add_node(node);
        self.node_to_index.insert(node, res);
        res
    }
    fn len(&self) -> usize { self.nodes.len() }
}
