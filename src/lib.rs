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
//! 
/// Copyright 2022-2025 Andrew Conway. All rights reserved. See README.md for licensing. 

pub mod generating_function;
pub mod permutation_diagrams;
pub mod xdd_with_multiplicity;
pub mod util;
pub mod permutation;

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::io::Write;
use std::ops::Rem;
use num::{Integer, Unsigned, Zero};
use crate::generating_function::GeneratingFunctionWithMultiplicity;
use crate::xdd_with_multiplicity::{SolutionFinder};

/// The identifier of a variable. Variable 0 is the highest one in the diagram.
#[derive(Copy, Clone,Eq, PartialEq,Hash,Ord, PartialOrd,Debug)]
pub struct VariableIndex(pub u16);


impl Display for VariableIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.0)
    }
}


/// The identifier of a node on the tree (effectively a pointer), along with an associated multiplicity (number of times represented, for a multiset).
///
/// Two of these have special meanings:
/// * Node address 0 is the sink representing FALSE/0/⊥
/// * Node address 1 is the sink representing TRUE/1/⊤
///
/// A is the type of addresses, typically u32 or usize.
/// M is the type of multiplicies, typically some unsigned integer
#[derive(Copy, Clone,Eq, PartialEq,Hash,Debug)]
pub struct NodeIndex<A:NodeAddress,M:Multiplicity> {
    address : A,
    multiplicity : M,
}


impl <A:NodeAddress,M:Multiplicity> NodeIndex<A,M> {
    /// Special sink index that indicates the result is false. Sometimes called 0 or Bottom or ⊥.
    /// Do not use this node index for anything else.
    pub const FALSE : Self = NodeIndex {address:A::FALSE,multiplicity:M::ONE};
    /// Special sink index that indicates the result is true. Sometimes called 1 or Top or ⊤.
    /// Do not use this node index for anything else.
    pub const TRUE : Self = NodeIndex {address:A::TRUE,multiplicity:M::ONE};

    /// See if the node index is one of the two special sink nodes.
    pub fn is_sink(self) -> bool { self.is_false()||self.is_true() } // could be made more efficient by <2, but requires further restrictions on A.
    /// See if the node index is the special FALSE sink node.
    pub fn is_false(self) -> bool { self.address==A::FALSE }
    /// See if the node index is the special TRUE sink node.
    pub fn is_true(self) -> bool { self.address==A::TRUE }

    pub fn multiply(self,m:M) -> Self { NodeIndex {address:self.address,multiplicity:M::multiply(self.multiplicity, m)}}
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


impl <A:NodeAddress,M:Multiplicity> Display for NodeIndex<A,M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}*{}",self.multiplicity,self.address)
    }
}

/// A node in a version of a binary decision tree with optional multiplicity.
///
/// A BDD can be considered a set of combinations of the variables. A multiset is
/// a set where elements can be in the set multiple times. This represents that case;
/// the multiplicity is the number of times that element should be counted.
///
/// Two nested multiplicities should be multiplied together.
///
/// M is the type of the multiplicity.
///
/// # Meaning
/// If the variable is true, go to the hi node, else go to the low node.
#[derive(Copy, Clone,Eq, PartialEq,Hash)]
pub struct Node<A:NodeAddress,M:Multiplicity> {
    pub variable : VariableIndex,
    pub lo : NodeIndex<A,M>,
    pub hi : NodeIndex<A,M>,
}

/// A object that can function as a decision diagram factory, doing stuff quickly.
pub trait DecisionDiagramFactory<A:NodeAddress,M:Multiplicity> {
    /// Make a new decision diagram with the stated number of variables.
    fn new(num_variables:u16) -> Self;
    /// Compute a diagram being the logical and of index1 and index2.
    fn and(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>) -> NodeIndex<A,M>;
    /// Compute a diagram being the logical or of index1 and index2.
    fn or(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>) -> NodeIndex<A,M>;
    /// Compute a diagram being the logical not of index1 and index2.
    fn not(&mut self, index: NodeIndex<A,M>) -> NodeIndex<A,M>;
    /// Enumerate the solutions to the given generating function.
    fn number_solutions<G:GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A,M>) -> G;
    /// A more powerful version of number_solutions that can list said solutions or find arbitrary ones. 
    /// ```
    /// use xdd::{ZDDFactory, DecisionDiagramFactory, NoMultiplicity, VariableIndex};
    /// let mut factory = ZDDFactory::<u32,NoMultiplicity>::new(2);
    /// let v0 = factory.single_variable(VariableIndex(0));
    /// // we have made a function that is true iff v0 is true, there is one other irrelevant 
    /// // variable v1, so there should be 2 solutions.
    /// let solutions_where_v0_is_true = factory.find_all_solutions::<u64>(v0,2);
    /// assert_eq!(2,solutions_where_v0_is_true.number_solutions());
    /// // solutions are listed by the lexicographic order that they would be in a truth table
    /// assert_eq!(Some(vec![VariableIndex(0)]),solutions_where_v0_is_true.get_ith_solution(0));
    /// assert_eq!(Some(vec![VariableIndex(0),VariableIndex(1)]),solutions_where_v0_is_true.get_ith_solution(1));
    /// assert_eq!(None,solutions_where_v0_is_true.get_ith_solution(2));
    /// ```
    fn find_all_solutions<G: GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A, M>, num_variables:u16) -> SolutionFinder<A, M, xdd_with_multiplicity::NodeListWithFastLookup<A,M>, G, false>;
    /// Like find_all_solutions, but only find the best (or equal best) solution where "best"
    /// means has the fewest number of true variables as arguments.
    /// 
    /// ```
    /// use xdd::{ZDDFactory, DecisionDiagramFactory, NoMultiplicity, VariableIndex};
    /// let mut factory = ZDDFactory::<u32,NoMultiplicity>::new(2);
    /// let v0 = factory.single_variable(VariableIndex(0));
    /// // we have made a function that is true iff v0 is true, there is one other irrelevant 
    /// // variable v1, so there should be 2 solutions. But only the one with v1=false is minimal,
    /// // so only 1 solutions comes up here.
    /// let solutions_where_v0_is_true = factory.find_all_solutions_with_minimal_true_arguments::<u64>(v0,2);
    /// assert_eq!(1,solutions_where_v0_is_true.number_solutions());
    /// // solutions are listed by the lexicographic order that they would be in a truth table
    /// assert_eq!(Some(vec![VariableIndex(0)]),solutions_where_v0_is_true.get_ith_solution(0));
    /// assert_eq!(None,solutions_where_v0_is_true.get_ith_solution(1));
    /// ```
    fn find_all_solutions_with_minimal_true_arguments<G: GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A, M>, num_variables:u16) -> SolutionFinder<A, M, xdd_with_multiplicity::NodeListWithFastLookup<A,M>, G, true>;
    
    /// Produce a DD that describes a single variable. That is, a DD that has all variables having no effect other than just that variable leading to TRUE iff variable is true.
    fn single_variable(&mut self,variable:VariableIndex) -> NodeIndex<A,M>;
    /// Get the number of nodes in the DD.
    fn len(&self) -> usize;
    /// Do garbage collection. Provide the items one wants to keep, and get rid of anything not in the transitive dependencies of keep.
    /// Returns a vector v such that v[old_node.0] is what v maps in to. If nothing, then map into NodeIndex::JUNK.
    fn gc(&mut self, keep:impl IntoIterator<Item=NodeIndex<A,M>>) -> NodeRenaming<A>;
    /// Produce a DD which is true iff exactly 1 of the given variables is true, regardless of other variables.
    /// The variables array must be sorted, smallest to highest.
    fn exactly_one_of(&mut self,variables:&[VariableIndex]) -> NodeIndex<A,M>;
    /// Do an "and" of lots of functions.
    fn poly_and(&mut self, indices:&[NodeIndex<A,M>]) -> Option<NodeIndex<A,M>> {
        let mut res : Option<NodeIndex<A,M>> = None;
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
    /// Requires
    /// * a writer for where to store the result
    /// * a name (heading), typically a string.
    /// * a slice of nodes and optional associated names for the start points of interest for the diagram. Often there is just one of these, but often more are useful.
    /// * a namer function from a VariableIndex to a String.
    fn make_dot_file<W:Write,F:Fn(VariableIndex)->String>(&self, writer:&mut W, name:impl Display, start_nodes:&[(NodeIndex<A,M>, Option<String>)], namer:F) -> std::io::Result<()>;


    /// Find a set of boolean assignments to the variable that makes the result true (or non-zero in the case of multiplicities) if one exists.
    ///
    /// If multiple exist, choose the one with the fewest variables set to `true`.
    ///
    /// If multiple of those exist, choose the one not using variable 0, should it exist.
    /// If multiple of those exist, choose the one not using variable 1, should it exist.
    /// etc.
    ///
    /// If successful, return a vector of the used variables, sorted in ascending order.
    fn find_satisfying_solution_with_minimum_number_of_variables(&self,index: NodeIndex<A,M>) -> Option<Vec<VariableIndex>>;

}



/// A factory that can do efficient operations on BDDs.
pub struct BDDFactory<A:NodeAddress,M:Multiplicity> {
    nodes : xdd_with_multiplicity::NodeListWithFastLookup<A,M>,
    and_cache : HashMap<(NodeIndex<A,M>, NodeIndex<A,M>), NodeIndex<A,M>>,
    or_cache : HashMap<(NodeIndex<A,M>, NodeIndex<A,M>), NodeIndex<A,M>>,
    not_cache : HashMap<A,A>,
    num_variables : u16,
}

impl <A:NodeAddress+Default,M:Multiplicity> DecisionDiagramFactory<A,M> for BDDFactory<A,M> {

    fn new(num_variables:u16) -> Self {
        BDDFactory {
            nodes: Default::default(),
            and_cache: Default::default(),
            or_cache: Default::default(),
            not_cache: Default::default(),
            num_variables
        }
    }
    fn and(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.mul_bdd(index1,index2,&mut self.and_cache)
    }

    fn or(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.sum_bdd(index1,index2,&mut self.or_cache)
    }

    fn not(&mut self, index: NodeIndex<A,M>) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.not_bdd(index,&mut self.not_cache)
    }

    fn number_solutions<G: GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A,M>) -> G {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.number_solutions::<G,true>(index,self.num_variables)
    }

    fn find_all_solutions<G: GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A, M>, num_variables:u16) -> SolutionFinder<A, M, xdd_with_multiplicity::NodeListWithFastLookup<A,M>, G, false> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.find_all_solutions::<G,true>(index,num_variables)        
    }

    fn find_all_solutions_with_minimal_true_arguments<G: GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A, M>, num_variables:u16) -> SolutionFinder<A, M, xdd_with_multiplicity::NodeListWithFastLookup<A,M>, G, true> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.find_all_solutions_with_minimal_true_arguments::<G,true>(index,num_variables)
    }

    fn single_variable(&mut self, variable: VariableIndex) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.single_variable(variable)
    }

    fn len(&self) -> usize {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.len()
    }

    fn gc(&mut self, keep:impl IntoIterator<Item=NodeIndex<A,M>>) -> NodeRenaming<A> {
        self.and_cache.clear();
        self.or_cache.clear();
        self.not_cache.clear();
        use xdd_with_multiplicity::XDDBase;
        self.nodes.gc(keep)
    }

    fn exactly_one_of(&mut self, variables: &[VariableIndex]) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.exactly_one_of_bdd(variables)
    }

    fn make_dot_file<W:Write,F:Fn(VariableIndex)->String>(&self, writer:&mut W, name:impl Display, start_nodes:&[(NodeIndex<A,M>, Option<String>)], namer:F) -> std::io::Result<()> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.make_dot_file(writer,name,start_nodes,namer)
    }

    fn find_satisfying_solution_with_minimum_number_of_variables(&self, index: NodeIndex<A, M>) -> Option<Vec<VariableIndex>> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.find_satisfying_solution_with_minimum_number_of_variables(index)
    }
}

impl <A:NodeAddress+Default,M:Multiplicity> BDDFactory<A,M> {
    /// Produce a BDD which is true iff exactly n of the given variables is true, regardless of other variables.
    /// The variables array must be sorted, smallest to highest.
    ///
    /// The intention is to put this into DecisionDiagramFactory when a similar one is done for ZDDs.
    pub fn exactly_n_of(&mut self,n:usize, variables: &[VariableIndex]) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        let mut cache = HashMap::new();
        self.nodes.exactly_n_of_bdd(n,variables,&mut cache)
    }
}

/// A factory that can do efficient operations on BDDs.
pub struct ZDDFactory<A:NodeAddress,M:Multiplicity> {
    nodes : xdd_with_multiplicity::NodeListWithFastLookup<A,M>,
    and_cache : HashMap<(NodeIndex<A,M>, NodeIndex<A,M>), NodeIndex<A,M>>,
    or_cache : HashMap<(NodeIndex<A,M>, NodeIndex<A,M>), NodeIndex<A,M>>,
    not_cache : HashMap<(A,VariableIndex),A>,
    num_variables : u16,
}

impl <A:NodeAddress,M:Multiplicity> DecisionDiagramFactory<A,M> for ZDDFactory<A,M> {

    fn new(num_variables:u16) -> Self {
        ZDDFactory {
            nodes: Default::default(),
            and_cache: Default::default(),
            or_cache: Default::default(),
            not_cache: Default::default(),
            num_variables
        }
    }
    fn and(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.mul_zdd(index1,index2,&mut self.and_cache)
    }

    fn or(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.sum_zdd(index1,index2,&mut self.or_cache)
    }

    fn not(&mut self, index: NodeIndex<A,M>) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.not_zdd(index,VariableIndex(0),self.num_variables,&mut self.not_cache)

    }

    fn number_solutions<G: GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A,M>) -> G {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.number_solutions::<G,false>(index,self.num_variables)
    }

    fn find_all_solutions<G: GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A, M>, num_variables:u16) -> SolutionFinder<A, M, xdd_with_multiplicity::NodeListWithFastLookup<A,M>, G, false> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.find_all_solutions::<G,false>(index,num_variables)
    }

    fn find_all_solutions_with_minimal_true_arguments<G: GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A, M>, num_variables:u16) -> SolutionFinder<A, M, xdd_with_multiplicity::NodeListWithFastLookup<A,M>, G, true> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.find_all_solutions_with_minimal_true_arguments::<G,false>(index,num_variables)
    }

    fn single_variable(&mut self, variable: VariableIndex) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.single_variable_zdd(variable,self.num_variables) // TODO
    }

    fn len(&self) -> usize {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.len()
    }

    fn gc(&mut self, keep:impl IntoIterator<Item=NodeIndex<A,M>>) -> NodeRenaming<A> {
        self.and_cache.clear();
        self.or_cache.clear();
        self.not_cache.clear();
        use xdd_with_multiplicity::XDDBase;
        self.nodes.gc(keep)
    }

    fn exactly_one_of(&mut self, variables: &[VariableIndex]) -> NodeIndex<A,M> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.exactly_one_of_zdd(variables,self.num_variables)
    }

    fn make_dot_file<W:Write,F:Fn(VariableIndex)->String>(&self, writer:&mut W, name:impl Display, start_nodes:&[(NodeIndex<A,M>, Option<String>)], namer:F) -> std::io::Result<()> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.make_dot_file(writer,name,start_nodes,namer)
    }

    fn find_satisfying_solution_with_minimum_number_of_variables(&self, index: NodeIndex<A, M>) -> Option<Vec<VariableIndex>> {
        use xdd_with_multiplicity::XDDBase;
        self.nodes.find_satisfying_solution_with_minimum_number_of_variables(index)
    }

}



pub struct NodeRenaming<A:NodeAddress>(Vec<A>);

impl <A:NodeAddress> NodeRenaming<A> {
    pub fn rename<M:Multiplicity>(&self, index: NodeIndex<A,M>) -> Option<NodeIndex<A,M>> {
        let res = self.0[index.address.as_usize()];
        if res==A::FALSE && index.address!=A::FALSE { None } else { Some(NodeIndex {address:res,multiplicity:index.multiplicity}) }
    }
}
