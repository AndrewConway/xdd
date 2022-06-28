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
pub mod generating_function;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::generating_function::GeneratingFunction;
use crate::xdd_representations::{NodeListWithFastLookup, XDDBase};

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

/// A object that can function as a decision diagram factory, doing stuff quickly.
pub trait DecisionDiagramFactory {
    /// Compute a diagram being the logical and of index1 and index2.
    fn and(&mut self,index1:NodeIndex,index2:NodeIndex) -> NodeIndex;
    /// Compute a diagram being the logical or of index1 and index2.
    fn or(&mut self,index1:NodeIndex,index2:NodeIndex) -> NodeIndex;
    /// Compute a diagram being the logical not of index1 and index2.
    fn not(&mut self,index:NodeIndex) -> NodeIndex;
    /// Enumerate the solutions to the given generating function.
    fn number_solutions<G:GeneratingFunction>(&self,index:NodeIndex) -> G;
    /// Produce a DD that describes a single variable. That is, a DD that has all variables having no effect other than just that variable leading to TRUE iff variable is true.
    fn single_variable(&mut self,variable:VariableIndex) -> NodeIndex;
    /// Get the number of nodes in the DD.
    fn len(&self) -> usize;
}

/// A factory that can do efficient operations on BDDs.
pub struct BDDFactory {
    nodes : NodeListWithFastLookup,
    and_cache : HashMap<(NodeIndex,NodeIndex),NodeIndex>,
    or_cache : HashMap<(NodeIndex,NodeIndex),NodeIndex>,
    not_cache : HashMap<NodeIndex,NodeIndex>,
    num_variables : u16,
}
impl BDDFactory {
    pub fn new(num_variables:u16) -> Self {
        BDDFactory{
            nodes: Default::default(),
            and_cache: Default::default(),
            or_cache: Default::default(),
            not_cache: Default::default(),
            num_variables
        }
    }
}
impl DecisionDiagramFactory for BDDFactory {
    fn and(&mut self, index1: NodeIndex, index2: NodeIndex) -> NodeIndex {
        self.nodes.and_bdd(index1,index2,&mut self.and_cache)
    }

    fn or(&mut self, index1: NodeIndex, index2: NodeIndex) -> NodeIndex {
        self.nodes.or_bdd(index1,index2,&mut self.or_cache)
    }

    fn not(&mut self, index:NodeIndex) -> NodeIndex {
        self.nodes.not_bdd(index,&mut self.not_cache)
    }

    fn number_solutions<G: GeneratingFunction>(&self, index: NodeIndex) -> G {
        self.nodes.number_solutions::<G,true>(index,self.num_variables)
    }

    fn single_variable(&mut self, variable: VariableIndex) -> NodeIndex {
        self.nodes.single_variable(variable)
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }
}


/// A factory that can do efficient operations on BDDs.
pub struct ZDDFactory {
    nodes : NodeListWithFastLookup,
    and_cache : HashMap<(NodeIndex,NodeIndex),NodeIndex>,
    or_cache : HashMap<(NodeIndex,NodeIndex),NodeIndex>,
    not_cache : HashMap<(NodeIndex,VariableIndex),NodeIndex>,
    num_variables : u16,
}

impl ZDDFactory {
    pub fn new(num_variables:u16) -> Self {
        ZDDFactory{
            nodes: Default::default(),
            and_cache: Default::default(),
            or_cache: Default::default(),
            not_cache: Default::default(),
            num_variables
        }
    }
}
impl DecisionDiagramFactory for ZDDFactory {
    fn and(&mut self, index1: NodeIndex, index2: NodeIndex) -> NodeIndex {
        self.nodes.and_zdd(index1,index2,&mut self.and_cache)
    }

    fn or(&mut self, index1: NodeIndex, index2: NodeIndex) -> NodeIndex {
        self.nodes.or_zdd(index1,index2,&mut self.or_cache)
    }

    fn not(&mut self, index:NodeIndex) -> NodeIndex {
        self.nodes.not_zdd(index,VariableIndex(0),self.num_variables,&mut self.not_cache)
    }

    fn number_solutions<G: GeneratingFunction>(&self, index: NodeIndex) -> G {
        self.nodes.number_solutions::<G,false>(index,self.num_variables)
    }

    fn single_variable(&mut self, variable: VariableIndex) -> NodeIndex {
        self.nodes.single_variable_zdd(variable,self.num_variables)
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }
}






#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
