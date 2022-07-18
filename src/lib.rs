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
pub mod xdd_with_multiplicity;

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::io::Write;
use std::ops::Rem;
use num::{Integer, Unsigned, Zero};
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

/// The identifier of a node on the tree (effectively a pointer), along with an associated multiplicity (number of times represented, for a multiset).
///
/// Two of these have special meanings:
/// * Node address 0 is the sink representing FALSE/0/⊥
/// * Node address 1 is the sink representing TRUE/1/⊤
///
/// A is the type of addresses, typically u32 or usize.
/// M is the type of multiplicies, typically some unsigned integer
#[derive(Copy, Clone,Eq, PartialEq,Hash,Debug)]
pub struct NodeIndexWithMultiplicity<A:NodeAddress,M:Multiplicity> {
    address : A,
    multiplicity : M,
}


impl <A:NodeAddress,M:Multiplicity> NodeIndexWithMultiplicity<A,M> {
    /// Special sink index that indicates the result is false. Sometimes called 0 or Bottom or ⊥.
    /// Do not use this node index for anything else.
    pub const FALSE : Self = NodeIndexWithMultiplicity{address:A::FALSE,multiplicity:M::ONE};
    /// Special sink index that indicates the result is true. Sometimes called 1 or Top or ⊤.
    /// Do not use this node index for anything else.
    pub const TRUE : Self = NodeIndexWithMultiplicity{address:A::TRUE,multiplicity:M::ONE};

    /// See if the node index is one of the two special sink nodes.
    pub fn is_sink(self) -> bool { self.is_false()||self.is_true() } // could be made more efficient by <2, but requires further restrictions on A.
    /// See if the node index is the special FALSE sink node.
    pub fn is_false(self) -> bool { self.address==A::FALSE }
    /// See if the node index is the special TRUE sink node.
    pub fn is_true(self) -> bool { self.address==A::TRUE }

    pub fn multiply(self,m:M) -> Self { NodeIndexWithMultiplicity{address:self.address,multiplicity:M::multiply(self.multiplicity,m)}}
}


pub trait NodeAddress : TryInto<usize>+Copy+Eq+PartialOrd+Hash+TryFrom<usize>+Display+Debug {
    const ZERO : Self;
    const ONE : Self;
    /// Special sink index that indicates the result is false. Sometimes called 0 or Bottom or ⊥.
    /// Do not use this node index for anything else.
    const FALSE : Self = Self::ZERO;
    /// Special sink index that indicates the result is true. Sometimes called 1 or Top or ⊤.
    /// Do not use this node index for anything else.
    const TRUE : Self = Self::ONE;

    /// See if the node index is one of the two special sink nodes.
    fn is_sink(self) -> bool { self.is_false() || self.is_true() } // can specialize to <2 for speed maybe?
    /// See if the node index is the special FALSE sink node.
    fn is_false(self) -> bool { self==Self::FALSE }
    /// See if the node index is the special TRUE sink node.
    fn is_true(self) -> bool { self==Self::TRUE }

    fn as_usize(self) -> usize { self.try_into().map_err(|_|()).expect("Should be able to convert a NodeAddress into a usize")}
}

impl NodeAddress for usize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl NodeAddress for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl NodeAddress for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}

pub trait Multiplicity : Copy+Eq+Hash+Display+Debug {
    const ONE : Self;
    /// True iff the multiplicities are not used. Useful for optimizations.
    const MULTIPLICITIES_IRRELEVANT: bool;
    /// True iff a or b = b or a (usually true).
    const SYMMETRIC_OR : bool = true;
    /// combine two multiplicities that are subject to a logical OR. Typically addition.
    fn combine_or(a:Self,b:Self) -> Self;
    fn multiply(a:Self,b:Self) -> Self;
    /// Given a and b, compute g=gcd(a,b) and return (a/g,b/g,g).
    fn gcd(a:Self,b:Self) -> (Self,Self,Self);
    fn is_unity(self) -> bool { self==Self::ONE }
}

#[derive(Copy, Clone,Eq, PartialEq,Hash,Debug,Default)]
pub struct NoMultiplicity {}

impl Display for NoMultiplicity {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result { Ok(()) }
}

impl Multiplicity for NoMultiplicity {
    const ONE: Self = NoMultiplicity{};
    const MULTIPLICITIES_IRRELEVANT: bool = true;

    fn combine_or(_a: Self, _b: Self) -> Self { NoMultiplicity{} }
    fn multiply(_a: Self, _b: Self) -> Self { NoMultiplicity{} }
    fn gcd(_a: Self, _b: Self) -> (Self, Self, Self) { (NoMultiplicity{},NoMultiplicity{},NoMultiplicity{}) }
    fn is_unity(self) -> bool { true }
}

fn compute_gcd<T:Rem<T,Output=T>+Ord+Copy+Unsigned+Integer+Zero>(a:T,b:T) -> T {
    let (mut min,mut max) = if a<b { (a,b) } else { (b,a) };
    while min>T::zero() {
        let remainder = max%min;
        max=min;
        min=remainder;
    }
    max
}

impl Multiplicity for u32 {
    const ONE: Self = 1;
    const MULTIPLICITIES_IRRELEVANT: bool = false;

    fn combine_or(a: Self, b: Self) -> Self { a+b }
    fn multiply(a: Self, b: Self) -> Self { a*b }
    fn gcd(a: Self, b: Self) -> (Self, Self, Self) {
        let g = compute_gcd(a,b);
        (a/g,b/g,g)
    }
}

impl Display for NodeIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.0)
    }
}

impl <A:NodeAddress,M:Multiplicity> Display for NodeIndexWithMultiplicity<A,M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}*{}",self.multiplicity,self.address)
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

/// A node in a version of a BDD with multiplicity.
///
/// A BDD can be considered a set of combinations of the variables. A multiset is
/// a set where elements can be in the set multiple times. This represents that case;
/// the multiplicity is the number of times that element should be counted.
///
/// Two nested multiplicities should be multiplied together.
///
/// M is the type of the multiplicity.
#[derive(Copy, Clone,Eq, PartialEq,Hash)]
pub struct NodeWithMultiplicity<A:NodeAddress,M:Multiplicity> {
    pub variable : VariableIndex,
    pub lo : NodeIndexWithMultiplicity<A,M>,
    pub hi : NodeIndexWithMultiplicity<A,M>,
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


pub struct NodeRenamingWithMuliplicity<A:NodeAddress>(Vec<A>);

impl <A:NodeAddress> NodeRenamingWithMuliplicity<A> {
    pub fn rename<M:Multiplicity>(&self,index:NodeIndexWithMultiplicity<A,M>) -> Option<NodeIndexWithMultiplicity<A,M>> {
        let res = self.0[index.address.as_usize()];
        if res==A::FALSE && index.address!=A::FALSE { None } else { Some(NodeIndexWithMultiplicity{address:res,multiplicity:index.multiplicity}) }
    }
}
