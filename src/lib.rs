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

use std::collections::HashMap;

/// The identifier of a variable. Variable 0 is the highest one in the diagram.
pub struct VariableIndex(u16);

/// The identifier of a node on the tree (effectively a pointer).
///
/// Two of these have special meanings:
/// * Node 0 is the sink representing FALSE/0/⊥
/// * Node 1 is the sink representing TRUE/1/⊤
pub struct NodeIndex(u32);

/// A node in a BDD.
///
/// # Meaning
/// If the variable is true, go to the hi node, else go to the low node.
pub struct Node {
    pub variable : VariableIndex,
    pub lo : NodeIndex,
    pub hi : NodeIndex,
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
