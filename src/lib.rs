//! xDD is a library for binary decision diagrams - either BDD or XDD.
//!
//! These were largely described by Minato, although this library is designed using Knuth's notations in "The Art of Computer Programming" volume 4 fascicle 1, "Binary decision diagrams"
//!
//! The library differs from the BDD library on crates.io in being targetted at combinatorics and having features essential for combinatorics like generating function generation.
//! It also uses external factories for generating xDDs, which improves efficiency for generating structures with lots of reuse which tends to arise in combinatorics.
//! It also supports ZDDs as well as BDDs.
//!
//! It supports 16 bits for variables and 32 bits for pointers, limiting it to trees of 4 billion nodes.
//! This may be changed in a newer version to a larger number.
//!

pub mod xdd_representations;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// The identifier of a variable. Variable 0 is the highest one in the diagram.
#[derive(Copy, Clone,Eq, PartialEq,Hash,Ord, PartialOrd,Debug)]
pub struct VariableIndex(pub u16);


impl Display for VariableIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.0)
    }
}

/// The identifier of a node on the tree (effectively a pointer).
///
/// Two of these have special meanings:
/// * Node 0 is the sink representing FALSE/0/⊥
/// * Node 1 is the sink representing TRUE/1/⊤
#[derive(Copy, Clone,Eq, PartialEq,Hash,Debug)]
pub struct NodeIndex(u32);

impl Display for NodeIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.0)
    }
}

impl NodeIndex {
    /// Special sink index that indicates the result is false. Sometimes called 0 or Bottom or ⊥.
    /// Do not use this node index for anything else.
    pub const FALSE : NodeIndex = NodeIndex(0);
    /// Special sink index that indicates the result is true. Sometimes called 1 or Top or ⊤.
    /// Do not use this node index for anything else.
    pub const TRUE : NodeIndex = NodeIndex(1);

    /// See if the node index is one of the two special sink nodes.
    pub fn is_sink(self) -> bool { self.0<=1 }
    /// See if the node index is the special FALSE sink node.
    pub fn is_false(self) -> bool { self.0==Self::FALSE.0 }
    /// See if the node index is the special TRUE sink node.
    pub fn is_true(self) -> bool { self.0==Self::TRUE.0 }
}

/// A node in a BDD.
///
/// # Meaning
/// If the variable is true, go to the hi node, else go to the low node.
#[derive(Copy, Clone,Eq, PartialEq,Hash)]
pub struct Node {
    pub variable : VariableIndex,
    pub lo : NodeIndex,
    pub hi : NodeIndex,
}

impl Node {
    /// Produce a node that describes a single variable. That is, a DD that has just that variable leading to TRUE iff variable is true
    pub fn single_variable(variable:VariableIndex) -> Node {
        Node{
            variable,
            lo: NodeIndex::FALSE,
            hi: NodeIndex::TRUE
        }
    }

}

/// A free standing decision tree.
pub struct FreestandingXDD {
    pub start : NodeIndex,
    pub nodes : Vec<Node>,
}

pub struct DDFactory {
    pub(crate) nodes : Vec<Node>,
    pub(crate) node_to_index : HashMap<Node,NodeIndex>,
}

pub struct XDDInFactory<'a> {
    start : NodeIndex,
    factory : &'a DDFactory,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
