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

    /// Produce a DD that describes a single variable. That is, a DD that has just that variable leading to TRUE iff variable is true.
    /// * For a BDD, this is a simple function f(v,...)=v.
    /// * For a ZDD, this means a function f(v,...) = v & !(union of all other variables).
    fn single_variable(&mut self,variable:VariableIndex) -> NodeIndex {
        self.add_node_if_not_present(Node::single_variable(variable))
    }

    /// Produce a ZDD that describes a single variable. That is, a ZDD that has all variables having no effect other than just that variable leading to TRUE iff variable is true.
    /// * For a ZDD, this is a simple function f(v,...)=v.
    /// * This is not a valid BDD.
    fn single_variable_zdd(&mut self,variable:VariableIndex,total_num_variables:usize) -> NodeIndex {
        let mut index = NodeIndex::TRUE;
        for i in (0..total_num_variables).rev() {
            let v = VariableIndex(i as u16);
            index = self.add_node_if_not_present(Node{
                variable : v,
                lo: if v==variable { NodeIndex::FALSE } else { index },
                hi: index,
            });
        }
        index
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

    /// Evaluate as a ZDD with given variables.
    fn evaluate_zdd(&self,index:NodeIndex,variables:&[bool]) -> bool {
        let mut up_to_variable = VariableIndex(0);
        let mut index = index;
        while !index.is_sink() {
            let node = self.node(index);
            while up_to_variable!=node.variable {
                if variables[up_to_variable.0 as usize] { return false; }
                else { up_to_variable=VariableIndex(up_to_variable.0+1); }
            }
            up_to_variable=VariableIndex(node.variable.0+1);
            index = if variables[node.variable.0 as usize] {node.hi} else {node.lo}
        }
        while (up_to_variable.0 as usize) < variables.len() {
            if variables[up_to_variable.0 as usize] { return false; }
            else { up_to_variable=VariableIndex(up_to_variable.0+1); }
        }
        index.is_true()
    }
/*
    /// Create a partial ZDD containing a chain of all variables from upto (inclusive) to total_number_variables (exclusive)
    /// producing true iff at least one variable is true.
    fn create_zdd_any_variables_below_given_variable_true(&mut self,start_from:VariableIndex,total_number_variables:usize) -> NodeIndex {
        let mut index = NodeIndex::FALSE;
        for i in (start_from.0..total_number_variables as u16).rev() {
            index = self.add_node_if_not_present(Node{
                variable : VariableIndex(i),
                lo: index,
                hi: NodeIndex::TRUE,
            });
        }
        index
    }
*/
    /// Make a node representing the negation of the function represented by the input node interpreted as a BDD. A.k.a. ~ or !.
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

    /// Make a node representing the negation of the function represented by the input node interpreted as a ZDD. A.k.a. ~ or !.
    /// upto should be be VariableIndex(0) unless you want to ignore variables less than it.
    /// TODO support caching of not.
    fn not_zdd(&mut self,index:NodeIndex,upto:VariableIndex,total_number_variables:u16) -> NodeIndex {
        if index.is_false() { NodeIndex::TRUE }
        // else if index.is_true() { self.create_zdd_any_variables_below_given_variable_true(upto,total_number_variables) }
        else {
            let mut upper_bound = total_number_variables;
            let mut index = {
                if index.is_true() { NodeIndex::FALSE }
                else {
                    let node = self.node(index);
                    upper_bound = node.variable.0;
                    let new_upto = VariableIndex(node.variable.0+1);
                    let newnode = Node {
                        variable: node.variable,
                        lo: self.not_zdd(node.lo,new_upto,total_number_variables),
                        hi: self.not_zdd(node.hi,new_upto,total_number_variables),
                    };
                    self.add_node_if_not_present(newnode)
                }
            };
            for i in (upto.0..upper_bound).rev() {
                index = self.add_node_if_not_present(Node{
                    variable : VariableIndex(i),
                    lo: index,
                    hi: NodeIndex::TRUE,
                });
            }
            index
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
