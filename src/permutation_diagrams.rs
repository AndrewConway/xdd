//! Implement πDD (as described by S Minato) and Rot-πDD (as described by Y Inoue)
//!
//! Shin-ichi Minato. πDD: A New Decision Diagram for Efficient Problem Solving
//! in Permutation Space. In Theory and Applications of Satisfiability Testing - SAT
//! 2011 - 14th International Conference, SAT 2011, Ann Arbor, MI, USA, June 19-
//! 22, 2011. Proceedings, volume 6695 of Lecture Notes in Computer Science, pages
//! 90–104. Springer, Berlin, Heidelberg, 2011
//!
//! [Yuma Inoue, Studies on Permutation Set Manipulation based on Decision Diagrams,
//! Doctor of Info. Sciences thesis, Hokkaido University, (2017).](https://eprints.lib.hokudai.ac.jp/dspace/handle/2115/65366?locale=en&lang=en)
//!


use std::collections::HashMap;


use std::fmt::{Display, Formatter};
use std::io::Write;
use std::marker::PhantomData;
use std::ops::{Div, Index, MulAssign};
use num::Num;
use crate::{DecisionDiagramFactory, Node, NodeIndex, NodeRenaming, VariableIndex, ZDDFactory, NodeAddress, Multiplicity, GeneratingFunctionWithMultiplicity};
use crate::permutation::PermutedItem;
use crate::xdd_with_multiplicity::XDDBase;



/// This is a placeholder indicating that a permutation element should be considered a swap, as described in
///
/// Shin-ichi Minato. πDD: A New Decision Diagram for Efficient Problem Solving
/// in Permutation Space. In Theory and Applications of Satisfiability Testing - SAT
/// 2011 - 14th International Conference, SAT 2011, Ann Arbor, MI, USA, June 19-
/// 22, 2011. Proceedings, volume 6695 of Lecture Notes in Computer Science, pages
/// 90–104. Springer, Berlin, Heidelberg, 2011
///
/// # Swaps (transpositions)
/// A permutation can be encoded by transpositions τ(a,b) which swaps elements a and b
/// of the permutation. For uniqueness, require b>a. There is clearly no point having a=b.
///
/// Order transpositions such that
/// τ(a,b) < τ(c,d) iff b>d or (b==d and a<c). Note that this seems an unusual order
/// and the sign on the b>d is intended.
/// This ordering is used for two purposes:
///  * To define a unique mapping from a set of transpositions to a permutation.
///    If τ1<τ2<...<τk then { τ1,τ2,...,τk } represents the permutation τk· … ·τ2·τ1
///  * To order the variables in a ZDD. For an n element permutation there are n(n-1)/2
///    transpositions and thus variables.
///
/// We define the canonical decomposition of a permutation into transpositions
/// as in definition 2.1.6 in Yuna's thesis (slightly edited):
///
/// Transposition decomposition of a permutation π is a sequence of
/// transpositions recursively computed as follows: If π is an identity permutation, we
/// return an empty sequence. Otherwise, let x be the maximum unfixed element, that is x≠π(x). Then
/// π′ = π·τ(x,π(x)) is recursively decomposed and compose τ(x,π(x)) to the right of the obtained
/// composition.
///
/// The canonical decomposition of a permutation can only have one τ(x,?) for each x as constructed.
/// Clearly this maps one to one to the n! permutations.
///
/// This means that we have to make sure that all πDDs are valid; they do not contain any
/// solutions containing two different τ(x,?) with the same x.
#[derive(Copy, Clone,Eq, PartialEq,Debug)]
pub struct Swap {}

/// This is a placeholder indicating that a permutation element should be considered a left rotation, as described in YI's thesis:
///
/// [Yuma Inoue, Studies on Permutation Set Manipulation based on Decision Diagrams,
/// Doctor of Info. Sciences thesis, Hokkaido University, (2017).](https://eprints.lib.hokudai.ac.jp/dspace/handle/2115/65366?locale=en&lang=en)
///
/// # Left Rotations
///
/// A left-rotation ρ(i,j) (i < j) is a permutation (1,…,i-1,i+1,i+2,…,j−1,j,i,j+1,…,n).
/// That is element i is placed in position j, and all elements between i+1 and j inclusive
/// are shifted left one place.
///
/// The canonical decomposition of a permutation is similar to swaps - find the right
/// most non fixed element, and make it correct, and then proceed recursively. This
/// ensures that, like swaps, there can only be one τ(x,?) for each x.
///
/// Ordering is the same as for swaps, as is the constraint of valid πDDs.
///
#[derive(Copy, Clone,Eq, PartialEq,Debug)]
pub struct LeftRotation {}

/// A permutation can be encoded as a set of variables by defining a basis of permutations
/// such that each permutations is encoded by exactly one composition of a set of such
/// variables ordered in a canonical manner.
///
/// The I field is the interpretation, and should be [Swap] or [LeftRotation]. Ideally this would
/// be a const generic enum.
#[derive(Copy, Clone,Eq, PartialEq,Debug)]
pub struct PermutationElement<I> {
    elem1 : PermutedItem,
    /// elem2 is always >= elem1.
    elem2 : PermutedItem,
    _placeholder : PhantomData<I>
}

impl <I> PermutationElement<I> {
    pub fn new(i:PermutedItem,j:PermutedItem) -> Self {
        assert!(i<j);
        PermutationElement{
            elem1: i,
            elem2: j,
            _placeholder: Default::default()
        }
    }
}

impl PermutationElement<LeftRotation> {
    /// Get a collection of LeftRotations that produce the given permutation.
    /// panics if the permutation argument is not a permutation of 1..n.
    /// # Example (see example 4.1.2 in Yuma Inoue's thesis)
    /// ```
    /// use xdd::permutation_diagrams::{LeftRotation, PermutationElement};
    /// let res = PermutationElement::<LeftRotation>::get_permutation(&[4, 3, 1, 5, 2]);
    /// assert_eq!(res,vec![PermutationElement::new(2,5),PermutationElement::new(1,3),PermutationElement::new(1,2)])
    /// ```
    pub fn get_permutation(permutation:&[PermutedItem]) -> Vec<Self> {
        let n = permutation.len();
        let mut res = Vec::new();
        let mut sofar : Vec<PermutedItem> = (1..=n as PermutedItem).collect();
        for j in (0..n).rev() {
            // make sure element j is correct.
            if permutation[j]!=sofar[j] {
                let position = sofar.iter().position(|&e|e==permutation[j]).expect("Input was not a permutation");
                assert!(position<j);
                res.push(PermutationElement::new((position+1) as PermutedItem,(j+1) as PermutedItem)); // +1 as PermutedItem is 1 based, and position and j are 0 based.
                let extracted = sofar.remove(position);
                sofar.insert(j,extracted);
            }
        }
        res
    }
}

/// Convert ASCII digits in a string to subscripts.
fn subscript(s:String) -> String {
    s.chars().map(|c|if c.is_ascii_digit() {char::from_u32(c as u32-'0' as u32+'₀' as u32).unwrap_or(c)} else {c}).collect()
}

impl Display for PermutationElement<Swap> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"τ{},{}",subscript(self.elem1.to_string()),subscript(self.elem2.to_string()))
    }
}
impl Display for PermutationElement<LeftRotation> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"ρ{},{}",subscript(self.elem1.to_string()),subscript(self.elem2.to_string()))
    }
}

/// Store the details of the map between ZDD variables and their interpretation as swaps or rotations.
pub struct PermutationEncodingAsVariables<I> {
    /// the length of the permutation
    pub n : PermutedItem,
    /// variable i has this element.
    /// It will be a list like (1,4),(2,4),(3,4),(1,3),(2,3),(1,2)
    pub elements : Vec<PermutationElement<I>>
}

impl <I> PermutationEncodingAsVariables<I> {
    /// Make a description of the variables used in a πDD, given the number of items to be permuted.
    /// # Example
    /// ```
    /// use xdd::permutation_diagrams::{PermutationElement, PermutationEncodingAsVariables, Swap};
    /// use xdd::VariableIndex;
    /// let enc = PermutationEncodingAsVariables::<Swap>::new(4);
    /// assert_eq!(PermutationElement::<Swap>::new(1,4),enc.elements[0]);
    /// assert_eq!(PermutationElement::<Swap>::new(2,4),enc.elements[1]);
    /// assert_eq!(PermutationElement::<Swap>::new(3,4),enc.elements[2]);
    /// assert_eq!(PermutationElement::<Swap>::new(1,3),enc.elements[3]);
    /// assert_eq!(PermutationElement::<Swap>::new(2,3),enc[VariableIndex(4)]);
    /// assert_eq!(PermutationElement::<Swap>::new(1,2),enc[VariableIndex(5)]);
    /// ```
    pub fn new(n:PermutedItem) -> Self {
        let mut elements = Vec::new();
        for j in (2..=n).rev() {
            for i in 1..j {
                elements.push(PermutationElement::new(i,j));
            }
        }
        PermutationEncodingAsVariables{n,elements}
    }
    /// The number of variables used.
    /// # Example
    /// ```
    /// use xdd::permutation_diagrams::{PermutationEncodingAsVariables, Swap};
    /// let enc = PermutationEncodingAsVariables::<Swap>::new(4);
    /// assert_eq!(enc.num_variables(),6)
    /// ```
    pub fn num_variables(&self) -> u16 { self.elements.len() as u16 }
    /// Get the variable for element (i,j)
    /// # Example
    /// ```
    /// use xdd::permutation_diagrams::{PermutationEncodingAsVariables, Swap};
    /// use xdd::VariableIndex;
    /// let enc = PermutationEncodingAsVariables::<Swap>::new(4);
    /// assert_eq!(enc.variable(1,4),VariableIndex(0));
    /// assert_eq!(enc.variable(2,4),VariableIndex(1));
    /// assert_eq!(enc.variable(3,4),VariableIndex(2));
    /// assert_eq!(enc.variable(1,3),VariableIndex(3));
    /// assert_eq!(enc.variable(2,3),VariableIndex(4));
    /// assert_eq!(enc.variable(1,2),VariableIndex(5));
    /// ```
    pub fn variable(&self,i:PermutedItem,j:PermutedItem) -> VariableIndex {
        let rows = self.n-j; // the number of rows of variables with the same j skipped. The first will have length n-1, the second n-2...the last will have (n-rows) elements.
        let elements_in_rows = (self.n-1+self.n-rows)*rows/2; // the number of elements in the skipped rows.
        VariableIndex((i-1+elements_in_rows) as u16)
    }
}

impl <I> Index<VariableIndex> for PermutationEncodingAsVariables<I> {
    type Output = PermutationElement<I>;

    fn index(&self, index: VariableIndex) -> &Self::Output {
        &self.elements[index.0 as usize]
    }
}

pub struct PermutationDecisionDiagramFactory<I,A:NodeAddress,M:Multiplicity> {
    pub zdd : ZDDFactory<A,M>,
    pub vars : PermutationEncodingAsVariables<I>,
    i_cache : HashMap<(NodeIndex<A,M>, VariableIndex), NodeIndex<A,M>>, // cache of the "I" operation
    compose_cache : HashMap<(NodeIndex<A,M>, NodeIndex<A,M>), NodeIndex<A,M>>, // cache of the compose/cross product operation
}

impl <I,A:NodeAddress,M:Multiplicity> PermutationDecisionDiagramFactory<I,A,M> {
    /// Note that the argument to new is different to the usual interpretation
    /// of DDs. The argument is the number of elements in the permutation. The
    /// total number of variables will be (num_elements_in_permutation-1)(num_elements_in_permutation-2)/2.
    pub fn new(num_elements_in_permutation: u16) -> Self {
        let vars = PermutationEncodingAsVariables::new(num_elements_in_permutation as PermutedItem);
        PermutationDecisionDiagramFactory{ zdd: ZDDFactory::new(vars.num_variables()), vars, i_cache:Default::default(), compose_cache: Default::default() }
    }

    // Standard DD operations just delegate to the underlying ZDD. But does not implement DecisionDiagramFactory as it is not really one.

    pub fn and(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>) -> NodeIndex<A,M> { self.zdd.and(index1, index2) }
    pub fn or(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>) -> NodeIndex<A,M> { self.zdd.or(index1, index2) }
    pub fn not(&mut self, index: NodeIndex<A,M>) -> NodeIndex<A,M> { self.zdd.not(index) }
    pub fn number_solutions<G: GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A,M>) -> G { self.zdd.number_solutions::<G>(index) }
    pub fn single_variable(&mut self, variable: VariableIndex) -> NodeIndex<A,M> { self.zdd.single_variable(variable) }
    pub fn len(&self) -> usize { self.zdd.len() }
    pub fn exactly_one_of(&mut self, variables: &[VariableIndex]) -> NodeIndex<A,M> { self.zdd.exactly_one_of(variables) }

    pub fn gc(&mut self, keep: impl IntoIterator<Item=NodeIndex<A,M>>) -> NodeRenaming<A> {
        self.i_cache.clear();
        self.compose_cache.clear();
        self.zdd.gc(keep)
    }
    /*
        fn make_dot_file<W:Write,F:Fn(VariableIndex)->String>(&self,writer:&mut W,name:impl Display,start_nodes:&[(NodeIndex,Option<String>)],namer:F) -> std::io::Result<()> {
            self.zdd.make_dot_file(writer,name,start_nodes,namer)
        }*/
}

impl <I,A:NodeAddress,M:Multiplicity> PermutationDecisionDiagramFactory<I,A,M> {
    /// create a new node, or use existing if not present.
    fn create(&mut self, variable:VariableIndex, lo: NodeIndex<A,M>, hi: NodeIndex<A,M>) -> NodeIndex<A,M> {
        self.zdd.nodes.add_node_if_not_present(Node {variable,lo,hi})
    }

}

impl <I,A:NodeAddress,M:Multiplicity> PermutationDecisionDiagramFactory<I,A,M> where PermutationElement<I>:Display {
    pub fn make_dot_file_default_names<W:Write>(&self, writer:&mut W, name:impl Display, start_nodes:&[(NodeIndex<A,M>, Option<String>)]) -> std::io::Result<()> {
        self.zdd.make_dot_file(writer,name,start_nodes,|v|self.vars[v].to_string())
    }
}
impl <A:NodeAddress,M:Multiplicity> PermutationDecisionDiagramFactory<Swap,A,M> {
    /// Perform the SWAP operation on a πDD. That is, convert the permutations
    /// considered by the tree starting at node to another one with the addition
    /// of the transposition τ(i,j)
    ///
    /// This must be done in a way to preserve the validity of the tree.
    /// # Example
    /// ```
    /// use xdd::{NodeIndex, NoMultiplicity};
    /// use xdd::permutation_diagrams::{PermutationDecisionDiagramFactory, Swap};
    /// let mut factory = PermutationDecisionDiagramFactory::<Swap,u32,NoMultiplicity>::new(4);
    /// assert_eq!(factory.len(),0);
    /// let swap13 = factory.swap(NodeIndex::TRUE,1,3);
    /// assert_eq!(factory.len(),1);
    /// assert_eq!(NodeIndex::TRUE,factory.swap(swap13,1,3));
    /// let swap13_34 = factory.swap(swap13,3,4);
    /// let swap13_34_14 = factory.swap(swap13_34,1,4);
    /// let swap34 = factory.swap(NodeIndex::TRUE,3,4);
    /// factory.make_dot_file_default_names(&mut std::fs::File::create("swap.gv").unwrap(),"dd",&[(swap13,Some("swap13".to_string())),(swap13_34,Some("swap13_34".to_string())),(swap13_34_14,Some("swap13_34_14".to_string())),(swap34,Some("swap34".to_string()))]);
    /// assert_eq!(swap34,swap13_34_14);
    /// ```
    pub fn swap(&mut self, node_index: NodeIndex<A,M>, i: PermutedItem, j: PermutedItem) -> NodeIndex<A,M> {
        if i == j { node_index } else if i > j { self.swap(node_index, j, i) } else {
            assert!(i < j);
            if node_index.is_false() { NodeIndex::FALSE } else if node_index.is_true() { self.create(self.vars.variable(i, j), NodeIndex::FALSE, NodeIndex::TRUE).multiply(node_index.multiplicity) } else {
                let variable = self.vars.variable(i, j);
                let node = self.zdd.nodes.node_incorporating_multiplicity(node_index);
                let node_variable = self.vars[node.variable];
                if node_variable.elem2 < j { self.create(variable, NodeIndex::FALSE, node_index) } // this is something lower down the diagram than the variable.
                else {
                    let cache_key = (node_index, variable);
                    if let Some(cached_answer) = self.i_cache.get(&cache_key) { *cached_answer } else {
                        let lo = self.swap(node.lo, i, j); // if we don't use node_variable, simple.
                        let hi1 = self.swap(node.hi, i, if j == node_variable.elem2 { node_variable.elem1 } else { j });
                        let hi = self.swap(hi1, if node_variable.elem1 == j { i } else if node_variable.elem1 == i { j } else { node_variable.elem1 }, node_variable.elem2);
                        let res = self.or(lo, hi);
                        self.i_cache.insert(cache_key, res);
                        res
                    }
                }
            }
        }
    }

    /// Perform the compose action on a πDD. That is, if p,q represents a set of permutations P,Q respectively,
    /// then make { p·q | p∈P, q∈Q }
    /// # Example
    /// ```
    /// use xdd::{NodeIndex, NoMultiplicity};
    /// use xdd::permutation_diagrams::{PermutationDecisionDiagramFactory, Swap};
    /// let mut factory = PermutationDecisionDiagramFactory::<Swap,u32,NoMultiplicity>::new(4);
    /// let swap12 = factory.swap(NodeIndex::TRUE,1,2);
    /// let swap23 = factory.swap(NodeIndex::TRUE,2,3);
    /// let two = factory.or(swap12,swap23);
    /// assert_eq!(2,factory.number_solutions::<u64>(two));
    /// let two_times_two = factory.compose(two,two);
    /// assert_eq!(3,factory.number_solutions::<u64>(two_times_two));
    /// let swap14 = factory.swap(NodeIndex::TRUE,1,4);
    /// let maybe_swap14 = factory.or(swap14,NodeIndex::TRUE);
    /// let some_mix = factory.compose(maybe_swap14,two_times_two);
    /// assert_eq!(6,factory.number_solutions::<u64>(some_mix));
    /// let s_n = factory.construct_all_permutations();
    /// assert_eq!(24,factory.number_solutions::<u64>(s_n));
    /// assert_eq!(s_n,factory.compose(s_n,s_n));
    /// assert_eq!(s_n,factory.compose(s_n,some_mix));
    /// assert_eq!(s_n,factory.compose(some_mix,s_n));
    /// ```
    pub fn compose(&mut self, p: NodeIndex<A,M>, q: NodeIndex<A,M>) -> NodeIndex<A,M> {
        if p.is_false() || q.is_false() { NodeIndex::FALSE } else if p.is_true() { q.multiply(p.multiplicity) } else if q.is_true() { p.multiply(q.multiplicity) } else {
            let cache_key = (p,q);
            if let Some(cached_answer) = self.compose_cache.get(&cache_key) { *cached_answer } else {
                let q_node = self.zdd.nodes.node_incorporating_multiplicity(q);
                let q_var = self.vars[q_node.variable];
                let lo = self.compose(p, q_node.lo);
                let hi = self.compose(p, q_node.hi);
                let hi = self.swap(hi, q_var.elem1, q_var.elem2);
                let res = self.or(lo, hi);
                self.compose_cache.insert(cache_key,res);
                res
            }
        }
    }

    /// Construct the set of all permutations.
    /// # Example
    /// ```
    /// use xdd::NoMultiplicity;
    /// use xdd::permutation_diagrams::{PermutationDecisionDiagramFactory, Swap};
    /// let mut factory = PermutationDecisionDiagramFactory::<Swap,u32,NoMultiplicity>::new(4);
    /// let s_n = factory.construct_all_permutations();
    /// factory.make_dot_file_default_names(&mut std::fs::File::create("S_4.gv").unwrap(),"Sn",&[(s_n,None)]);
    /// assert_eq!(24,factory.number_solutions::<u64>(s_n));
    /// ```
    pub fn construct_all_permutations(&mut self) -> NodeIndex<A,M> {
        let mut res = NodeIndex::TRUE;
        for i in 1..=self.vars.n {
            let prev = res;
            for j in 1..i {
                let extras = self.swap(prev,j,i);
                res=self.or(res,extras);
            }
        }
        res
    }
}

impl <A:NodeAddress,M:Multiplicity>  PermutationDecisionDiagramFactory<LeftRotation,A,M> {
    /// Perform the SWAP operation on a Rot-πDD. That is, convert the permutations
    /// considered by the tree starting at node to another one with the addition
    /// of the left rotation ρ(i,j)
    ///
    /// This must be done in a way to preserve the validity of the tree.
    /// See algorithm 4.1.1 in Yuma Inoue's thesis. (although the result, line 24/25 is done via a call to left_rot and OR which saves some generalization of checking l and r..
    /// # Example
    /// ```
    /// use xdd::{NodeIndex, NoMultiplicity};
    /// use xdd::permutation_diagrams::{PermutationDecisionDiagramFactory, LeftRotation};
    /// let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>::new(4);
    /// assert_eq!(factory.len(),0);
    /// let rot13 = factory.left_rot(NodeIndex::TRUE,1,3);
    /// assert_eq!(factory.len(),1);
    /// let rot13_13 = factory.left_rot(rot13,1,3);  // rotating by 13 twice is the same as 2-3 and then 1-2.
    /// let rot13_13_13 = factory.left_rot(rot13_13,1,3);  // rotating by 13 3 times is the identity.
    /// let rot23 = factory.left_rot(NodeIndex::TRUE,2,3);
    /// let rot23_12 = factory.left_rot(rot23,1,2);
    /// let rot12 = factory.left_rot(NodeIndex::TRUE,1,2);
    /// let rot12_23 = factory.left_rot(rot12,2,3);
    /// factory.make_dot_file_default_names(&mut std::fs::File::create("rot.gv").unwrap(),"dd",&[(rot13,Some("rot13".to_string())),(rot13_13,Some("rot13_13".to_string())),(rot13_13_13,Some("rot13_13_13".to_string())),(rot23,Some("rot23".to_string())),(rot23_12,Some("rot23_12".to_string())),(rot12_23,Some("rot12_23".to_string()))]);
    /// assert_eq!(rot13,rot23_12);
    /// assert_eq!(rot13_13,rot12_23);
    /// assert_eq!(NodeIndex::TRUE,factory.left_rot(rot13_13,1,3));
    /// ```
    pub fn left_rot(&mut self, node_index: NodeIndex<A,M>, l: PermutedItem, r: PermutedItem) -> NodeIndex<A,M> {
        if l == r { node_index } else if l > r { self.left_rot(node_index, r, l) } else {
            assert!(l < r);
            assert!(r <=self.vars.n);
            if node_index.is_false() { NodeIndex::FALSE } else if node_index.is_true() { self.create(self.vars.variable(l, r), NodeIndex::FALSE, NodeIndex::TRUE).multiply(node_index.multiplicity) } else {
                let variable = self.vars.variable(l, r);
                let node = self.zdd.nodes.node_incorporating_multiplicity(node_index);
                let node_variable = self.vars[node.variable]; // in YI's notation, x=node_variable.elem1, y=node_variable.elem2.
                if node_variable.elem2 < r { self.create(variable, NodeIndex::FALSE, node_index) } // this is something lower down the diagram than the variable.
                else {
                    let cache_key = (node_index, variable);
                    if let Some(cached_answer) = self.i_cache.get(&cache_key) { *cached_answer } else {
                        // Let P = ( ρ_x,y , P_0 , P_1 ). = P_0 + P_1.left_rot(x,y)
                        // so result = P_0.left_rot(l,r) + P_1.left_rot(x,y).left_rot(l,r)
                        // theorem 4.1.1 in Yuma Inoue's theis says ρ_x,y ρ_l,r can be transformed into the form of ρ_l′,r′ ρ_x′,y with r′< y
                        let lo = self.left_rot(node.lo, l, r); // if we don't use node_variable, simple. This is P′0 in YI's notation.
                        // hi computed here is P′1 in YI's notation
                        let (x_prime,hi) = if r <node_variable.elem1 { (node_variable.elem1, self.left_rot(node.hi, l, r))}
                        else if r ==node_variable.elem1 { (l, node.hi) }
                        else if l<=node_variable.elem1 { (node_variable.elem1+1,self.left_rot(node.hi, l, r -1))}
                        else { (node_variable.elem1,self.left_rot(node.hi, l-1, r -1)) };
                        // now the algorithm 4.1.1 says the result should be (ρ_x′,y , P′0 , P′1) (where y = node_variable.elem2). But I don't understand why ρ_x′,y comes before anything in P'0, and I need to deal with x'=y or x'>y, which is dealt with by the following code in a general way without significant cost if true anyway.
                        let hi = self.left_rot(hi,x_prime,node_variable.elem2);
                        let res = self.or(lo,hi);
                        self.i_cache.insert(cache_key, res);
                        res
                    }
                }
            }
        }
    }

    /// Perform the compose action on a Rot-πDD. That is, if p,q represents a set of permutations P,Q respectively,
    /// then make { p·q | p∈P, q∈Q }
    /// # Example
    /// ```
    /// use xdd::{NodeIndex, NoMultiplicity};
    /// use xdd::permutation_diagrams::{PermutationDecisionDiagramFactory, LeftRotation};
    /// let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>::new(4);
    /// let rot12 = factory.left_rot(NodeIndex::TRUE,1,2);
    /// let rot23 = factory.left_rot(NodeIndex::TRUE,2,3);
    /// let two = factory.or(rot12,rot23);
    /// assert_eq!(2,factory.number_solutions::<u64>(two));
    /// let two_times_two = factory.compose(two,two);
    /// assert_eq!(3,factory.number_solutions::<u64>(two_times_two));
    /// let rot14 = factory.left_rot(NodeIndex::TRUE,1,4);
    /// let maybe_rot14 = factory.or(rot14,NodeIndex::TRUE);
    /// let some_mix = factory.compose(maybe_rot14,two_times_two);
    /// assert_eq!(6,factory.number_solutions::<u64>(some_mix));
    /// let s_n = factory.construct_all_permutations();
    /// assert_eq!(24,factory.number_solutions::<u64>(s_n));
    /// assert_eq!(s_n,factory.compose(s_n,s_n));
    /// assert_eq!(s_n,factory.compose(s_n,some_mix));
    /// assert_eq!(s_n,factory.compose(some_mix,s_n));
    /// ```
    pub fn compose(&mut self, p: NodeIndex<A,M>, q: NodeIndex<A,M>) -> NodeIndex<A,M> {
        if p.is_false() || q.is_false() { NodeIndex::FALSE } else if p.is_true() { q.multiply(p.multiplicity) } else if q.is_true() { p.multiply(q.multiplicity) } else {
            let cache_key = (p,q);
            if let Some(cached_answer) = self.compose_cache.get(&cache_key) { *cached_answer } else {
                let q_node = self.zdd.nodes.node_incorporating_multiplicity(q);
                let q_var = self.vars[q_node.variable];
                let lo = self.compose(p, q_node.lo);
                let hi = self.compose(p, q_node.hi);
                let hi = self.left_rot(hi, q_var.elem1, q_var.elem2);
                let res = self.or(lo, hi);
                self.compose_cache.insert(cache_key,res);
                res
            }
        }
    }

    /// Construct the set of all permutations.
    /// # Example
    /// ```
    /// use xdd::NoMultiplicity;
    /// use xdd::permutation_diagrams::{PermutationDecisionDiagramFactory, LeftRotation};
    /// let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>::new(4);
    /// let s_n = factory.construct_all_permutations();
    /// factory.make_dot_file_default_names(&mut std::fs::File::create("rot_S_4.gv").unwrap(),"Sn",&[(s_n,None)]);
    /// assert_eq!(24,factory.number_solutions::<u64>(s_n));
    /// ```
    pub fn construct_all_permutations(&mut self) -> NodeIndex<A,M> {
        let mut res = NodeIndex::TRUE;
        for i in 1..=self.vars.n {
            let prev = res;
            for j in 1..i {
                let extras = self.left_rot(prev,j,i);
                res=self.or(res,extras);
            }
        }
        res
    }

    /// Get a set containing the single specified permutation.
    /// panics if the permutation argument is not a permutation of 1..n.
    pub fn compute_for_single_permutation(&mut self,permutation:&[PermutedItem]) -> NodeIndex<A,M> {
        let decomposition = PermutationElement::<LeftRotation>::get_permutation(permutation);
        let mut res = NodeIndex::TRUE;
        for e in decomposition.iter().rev() {
            res = self.left_rot(res,e.elem1,e.elem2);
        }
        res
    }

    /// Compute the set of permutations that distribute the k-prefix of a permutation π over the n choose k possible positions in the first n elements of π.
    ///
    /// Algorithm 4.4.3 from YI's thesis to compute the set "C" used in enumeration pattern avoiding permutations.
    ///
    /// In principle, n could be less than the length of the permutation, but it would typically be the length of the permutation.
    ///
    /// # Example - show that it produces the correct number of elements and correct size of ZDD.
    /// ```
    /// use xdd::NoMultiplicity;
    /// use xdd::permutation_diagrams::{factorial, LeftRotation, n_choose_r, PermutationDecisionDiagramFactory, PermutedItem};
    /// fn test(n:PermutedItem,k:PermutedItem) {
    ///     let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>::new(n as u16);
    ///     let c = factory.permutations_distributing_k_prefix_over_n_elements(n,k);
    ///     let renamer = factory.gc([c]);
    ///     let c = renamer.rename(c).unwrap();
    ///     let solutions : u64 = factory.number_solutions(c);
    ///     assert_eq!(solutions,n_choose_r::<u64>(n as u32,k as u32));
    ///     assert_eq!(factory.len(),((n-k)*k) as usize);
    /// }
    /// test(1,1);
    /// test(8,1);
    /// test(8,3);
    /// test(8,5);
    /// ```
    ///
    /// # Example - show that it produces just 1 of each element.
    ///
    /// ```
    /// use xdd::generating_function::GeneratingFunctionSplitByMultiplicity;
    /// use xdd::NoMultiplicity;
    /// use xdd::permutation_diagrams::{LeftRotation, PermutationDecisionDiagramFactory};
    /// let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,u32>::new(6);
    /// let a = factory.permutations_distributing_k_prefix_over_n_elements(6,2);
    /// assert_eq!(GeneratingFunctionSplitByMultiplicity(vec![15u64]),factory.number_solutions(a));
    /// ```
    pub fn permutations_distributing_k_prefix_over_n_elements(&mut self, n:PermutedItem, k:PermutedItem) -> NodeIndex<A,M> {
        assert!(k<=n);
        assert!(n<=self.vars.n);
        let mut p_j_minus_1 = vec![NodeIndex::TRUE; n as usize];
        for j in 1..=k {
            let mut p_j = vec![NodeIndex::FALSE; j as usize]; // P_{i,j} = p_j[i].
            for i in j..=n {
                let term2 = self.left_rot(p_j_minus_1[(i-1) as usize],j,i);
                let p_ij = self.or(p_j[(i-1) as usize],term2);
                p_j.push(p_ij);
            }
            p_j_minus_1=p_j;
        }
        p_j_minus_1[n as usize]
    }

    /// Compute the set of all permutations whose k-prefix is ordered in increasing order.
    ///
    /// Algorithm 4.4.4 from YI's thesis to compute the set "A" used in enumeration pattern avoiding permutations.
    ///
    /// In principle, n could be less than the length of the permutation, but it would typically be the length of the permutation.
    ///
    /// # Example - show that it produces the correct number of elements and correct size of ZDD.
    /// ```
    /// use xdd::NoMultiplicity;
    /// use xdd::permutation_diagrams::{factorial, LeftRotation, n_choose_r, PermutationDecisionDiagramFactory, PermutedItem};
    /// fn test(n:PermutedItem,k:PermutedItem) {
    ///     let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>::new(n as u16);
    ///     let a = factory.permutations_with_ordered_k_prefix(n,k);
    ///     let renamer = factory.gc([a]);
    ///     let a = renamer.rename(a).unwrap();
    ///     let solutions : u64 = factory.number_solutions(a);
    ///     assert_eq!(solutions,n_choose_r::<u64>(n as u32,k as u32)*factorial::<u64>((n-k) as u32));
    ///     assert_eq!(factory.len(),((n-k)*(n+k-1)/2) as usize);
    /// }
    /// test(1,1);
    /// test(8,1);
    /// test(8,3);
    /// test(8,5);
    /// ```
    ///
    /// # Example - show that it produces just 1 of each element.
    ///
    /// ```
    /// use xdd::generating_function::GeneratingFunctionSplitByMultiplicity;
    /// use xdd::NoMultiplicity;
    /// use xdd::permutation_diagrams::{LeftRotation, PermutationDecisionDiagramFactory};
    /// let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,u32>::new(6);
    /// let a = factory.permutations_with_ordered_k_prefix(6,2);
    /// assert_eq!(GeneratingFunctionSplitByMultiplicity(vec![360u64]),factory.number_solutions(a));
    /// ```
    ///
    pub fn permutations_with_ordered_k_prefix(&mut self, n:PermutedItem, k:PermutedItem) -> NodeIndex<A,M> {
        assert!(k<=n);
        assert!(n<=self.vars.n);
        let mut i_i_minus_1 = NodeIndex::TRUE;
        for i in k+1..=n {
            let mut i_i = i_i_minus_1;
            for j in 1..i {
                let rot = self.left_rot(i_i_minus_1,j,i);
                i_i = self.or(i_i,rot);
            }
            i_i_minus_1=i_i;
        }
        i_i_minus_1
    }

    /// Compute the permutations containing a given pattern, as in as described in section 4.4 of YI's thesis:
    ///
    /// [Yuma Inoue, Studies on Permutation Set Manipulation based on Decision Diagrams,
    /// Doctor of Info. Sciences thesis, Hokkaido University, (2017).](https://eprints.lib.hokudai.ac.jp/dspace/handle/2115/65366?locale=en&lang=en)
    ///
    /// # Example - compute the number of patterns avoiding 1324 [OEIS](https://oeis.org/A061552)
    ///
    /// ```
    /// use xdd::NoMultiplicity;
    /// use xdd::permutation_diagrams::{factorial, LeftRotation, PermutationDecisionDiagramFactory, PermutedItem};
    /// fn num_avoiding_1324(n:PermutedItem) -> u64 {
    ///     let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>::new(n as u16);
    ///     let containing = factory.permutations_containing_a_given_pattern(&[1,3,2,4]);
    ///     let num_containing : u64 = factory.number_solutions(containing);
    ///     factorial::<u64>(n as u32)-num_containing
    /// }
    /// assert_eq!(1,num_avoiding_1324(1));
    /// assert_eq!(2,num_avoiding_1324(2));
    /// assert_eq!(6,num_avoiding_1324(3));
    /// assert_eq!(23,num_avoiding_1324(4));
    /// assert_eq!(103,num_avoiding_1324(5));
    /// assert_eq!(513,num_avoiding_1324(6));
    /// assert_eq!(2762,num_avoiding_1324(7));
    /// assert_eq!(15793,num_avoiding_1324(8));
    /// assert_eq!(94776,num_avoiding_1324(9));
    /// assert_eq!(591950,num_avoiding_1324(10));
    /// assert_eq!(3824112,num_avoiding_1324(11));
    /// assert_eq!(25431452,num_avoiding_1324(12));
    /// assert_eq!(173453058,num_avoiding_1324(13));
    /// assert_eq!(1209639642,num_avoiding_1324(14));
    /// ```
    ///
    pub fn permutations_containing_a_given_pattern(&mut self,permutation:&[PermutedItem]) -> NodeIndex<A,M> {
        let n = self.vars.n;
        if n < permutation.len() as PermutedItem { return NodeIndex::FALSE; }
        let k = permutation.len() as PermutedItem;
        let a = self.permutations_with_ordered_k_prefix(n, k);
        let b = self.compute_for_single_permutation(permutation);
        let c = self.permutations_distributing_k_prefix_over_n_elements(n, k);
        let b_cross_a = self.compose(b,a);
        let c_cross_b_cross_a = self.compose(c,b_cross_a);
        c_cross_b_cross_a
    }

}

//
// General utility functions to do with permutations
//


/// Compute the factorial of a number.
///
/// # Example
/// ```
/// use xdd::permutation_diagrams::factorial;
/// assert_eq!(120u64,factorial(5));
/// assert_eq!(2432902008176640000u128,factorial(20));
/// assert_eq!("265252859812191058636308480000000",factorial::<num::bigint::BigUint>(30).to_str_radix(10));
/// ```
pub fn factorial<T:Num+MulAssign+TryFrom<u32>>(n:u32) -> T {
    let mut res = T::one();
    for i in 2..=n {
        let i : T = i.try_into().map_err(|_|()).expect("Could not convert u32 to factorial result type");
        res*=i;
    }
    res
}

/// Compute n choose r.
/// That is, n!/(n-r)!/r!
///
/// It is computed in the obvious somewhat inefficient manner; it needs an integer type large enough
/// to hold n!
///
/// # Example
/// ```
/// use xdd::permutation_diagrams::n_choose_r;
/// assert_eq!(10u64,n_choose_r(5,2));
/// assert_eq!(190u128,n_choose_r(20,2));
/// ```
pub fn n_choose_r<T:Num+MulAssign+TryFrom<u32>+Div>(n:u32,r:u32) -> T {
    factorial::<T>(n)/factorial::<T>(r)/factorial::<T>(n-r)
}
