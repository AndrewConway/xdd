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
pub mod permutation_diagrams;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::Write;
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
    /// Special entry meaning garbage collected. Only encountered in the result of the gc() function.
    pub(crate) const JUNK : NodeIndex = NodeIndex(u32::MAX);

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
    /// Make a new decision diagram with the stated number of variables.
    fn new(num_variables:u16) -> Self;
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
    /// Do garbage collection. Provide the items one wants to keep, and get rid of anything not in the transitive dependencies of keep.
    /// Returns a vector v such that v[old_node.0] is what v maps in to. If nothing, then map into NodeIndex::JUNK.
    fn gc(&mut self,keep:impl IntoIterator<Item=NodeIndex>) -> NodeRenaming;
    /// Produce a DD which is true iff exactly 1 of the given variables is true, regardless of other variables.
    /// The variables array must be sorted, smallest to highest.
    fn exactly_one_of(&mut self,variables:&[VariableIndex]) -> NodeIndex;
    /// Do an "and" of lots of functions.
    fn poly_and(&mut self,indices:&[NodeIndex]) -> Option<NodeIndex> {
        let mut res : Option<NodeIndex> = None;
        for n in indices {
            if let Some(ni) = res {
                res=Some(self.and(*n,ni));
            } else {
                res=Some(*n);
            }
        }
        res
    }
    /// write a graph file to the given writer with a given name showing the DD starting from start_nodes.
    fn make_dot_file<W:Write,F:Fn(VariableIndex)->String>(&self,writer:&mut W,name:impl Display,start_nodes:&[(NodeIndex,Option<String>)],namer:F) -> std::io::Result<()>;
}

/// A factory that can do efficient operations on BDDs.
pub struct BDDFactory {
    nodes : NodeListWithFastLookup,
    and_cache : HashMap<(NodeIndex,NodeIndex),NodeIndex>,
    or_cache : HashMap<(NodeIndex,NodeIndex),NodeIndex>,
    not_cache : HashMap<NodeIndex,NodeIndex>,
    num_variables : u16,
}

impl DecisionDiagramFactory for BDDFactory {
    fn new(num_variables:u16) -> Self {
        BDDFactory{
            nodes: Default::default(),
            and_cache: Default::default(),
            or_cache: Default::default(),
            not_cache: Default::default(),
            num_variables
        }
    }
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

    fn gc(&mut self,keep:impl IntoIterator<Item=NodeIndex>) -> NodeRenaming {
        self.and_cache.clear();
        self.or_cache.clear();
        self.not_cache.clear();
        self.nodes.gc(keep)
    }

    fn exactly_one_of(&mut self, variables: &[VariableIndex]) -> NodeIndex {
        self.nodes.exactly_one_of_bdd(variables)
    }

    fn make_dot_file<W:Write,F:Fn(VariableIndex)->String>(&self,writer:&mut W,name:impl Display,start_nodes:&[(NodeIndex,Option<String>)],namer:F) -> std::io::Result<()> {
        self.nodes.make_dot_file(writer,name,start_nodes,namer)
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


impl DecisionDiagramFactory for ZDDFactory {
    fn new(num_variables:u16) -> Self {
        ZDDFactory{
            nodes: Default::default(),
            and_cache: Default::default(),
            or_cache: Default::default(),
            not_cache: Default::default(),
            num_variables
        }
    }

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

    fn gc(&mut self,keep:impl IntoIterator<Item=NodeIndex>) -> NodeRenaming {
        self.and_cache.clear();
        self.or_cache.clear();
        self.not_cache.clear();
        self.nodes.gc(keep)
    }

    fn exactly_one_of(&mut self, variables: &[VariableIndex]) -> NodeIndex {
        self.nodes.exactly_one_of_zdd(variables,self.num_variables)
    }

    fn make_dot_file<W:Write,F:Fn(VariableIndex)->String>(&self,writer:&mut W,name:impl Display,start_nodes:&[(NodeIndex,Option<String>)],namer:F) -> std::io::Result<()> {
        self.nodes.make_dot_file(writer,name,start_nodes,namer)
    }
}

pub struct NodeRenaming(Vec<NodeIndex>);

impl NodeRenaming {
    pub fn rename(&self,index:NodeIndex) -> Option<NodeIndex> {
        let res = self.0[index.0 as usize];
        if res==NodeIndex::JUNK { None } else { Some(res) }
    }
}
