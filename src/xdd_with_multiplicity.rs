//! My own invention, decision diagrams with multiplicities.
//!
//! Where XDDs represent a set, the equivalent version with multiplicities represents a multiset.
//!

/// Copyright 2022-2025 Andrew Conway. All rights reserved. See README.md for licensing. 


use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::io::Write;
use std::marker::PhantomData;
use std::ops::Range;
use crate::{Node, NodeIndex, VariableIndex, NodeAddress, Multiplicity, NodeRenaming};
use crate::generating_function::{GeneratingFunctionWithMultiplicity};



/// Functions that any representation of an XDD must have, although some representations
/// will execute this more quickly than others, at the cost of more memory capacity.
pub trait XDDBase<A:NodeAddress,M:Multiplicity> {
    /// Get the node pointed to by a NodeIndex. panic if it does not exist.
    /// Do NOT call with the two special node indices NodeIndex::TRUE or NodeIndex::FALSE
    /// Also nodes should be sorted topologically - that is, node(x).hi>x and node(x).lo>x for all x.
    fn node(&self,index:A) -> Node<A,M>;
    /// Get the node index for a node if it is already present.
    fn find_node_index(&self, node: Node<A,M>) -> Option<A>;
    /// Add a node to the list, returning its new index.
    fn add_node(&mut self, node: Node<A,M>) -> A;
    /// The number of nodes in this tree, not counting the two special node indices.
    fn len(&self) -> usize;

    /// Like add_node, but first check with find_node_index to see if it is already there. Also canonicalize multiplicities by removing gcd.
    fn add_node_if_not_present(&mut self, node: Node<A,M>) -> NodeIndex<A,M> {
        let (node,multiplicity) = if M::MULTIPLICITIES_IRRELEVANT { (node,M::ONE) }
        else { // for uniqueness, want to make sure that there is no gcd of the hi and lo values.
            let (m_lo,m_hi,multiplicity) =
                if node.hi.is_false() { (M::ONE,M::ONE,node.lo.multiplicity) } // note that for the false node, multiplicity is irrelevant, and so gcd has to account for that.
                else if node.lo.is_false() { (M::ONE,M::ONE,node.hi.multiplicity) }
                else { M::gcd(node.lo.multiplicity,node.hi.multiplicity) };
            let node = Node { variable:node.variable, lo: NodeIndex { address: node.lo.address, multiplicity: m_lo }, hi: NodeIndex { address: node.hi.address, multiplicity: m_hi } };
            (node,multiplicity)
        };
        let address = self.find_node_index(node).unwrap_or_else(||self.add_node(node));
        NodeIndex {address,multiplicity}
    }

    /// Produce a DD that describes a single variable. That is, a DD that has just that variable leading to TRUE iff variable is true.
    /// * For a BDD, this is a simple function f(v,...)=v.
    /// * For a ZDD, this means a function f(v,...) = v & !(union of all other variables).
    fn single_variable(&mut self,variable:VariableIndex) -> NodeIndex<A,M> {
        self.add_node_if_not_present(Node {variable,lo: NodeIndex::FALSE,hi: NodeIndex::TRUE})
    }

    /// Produce a ZDD that describes a single variable. That is, a ZDD that has all variables having no effect other than just that variable leading to TRUE iff variable is true.
    /// * For a ZDD, this is a simple function f(v,...)=v.
    /// * This is not a valid BDD.
    fn single_variable_zdd(&mut self,variable:VariableIndex,total_num_variables:u16) -> NodeIndex<A,M> {
        let mut index = NodeIndex::TRUE;
        for i in (0..total_num_variables).rev() {
            let v = VariableIndex(i);
            index = self.add_node_if_not_present(Node {
                variable : v,
                lo: if v==variable { NodeIndex::FALSE } else { index },
                hi: index,
            });
        }
        index
    }

    /// Produce a BDD which is true iff exactly 1 of the given variables is true, regardless of other variables.
    /// The variables array must be sorted, smallest to highest.
    fn exactly_one_of_bdd(&mut self,variables:&[VariableIndex]) -> NodeIndex<A,M> {
        if variables.len()==0 { NodeIndex::FALSE } else {
            let mut right = NodeIndex::TRUE;
            let mut left = NodeIndex::FALSE;
            // The diagram that is needed has two parallel diagonal lines, one right, one left.
            // One is on the right if one has had exactly 1 variable, one is on the left if one has had 0 variables.
            for &variable in variables.into_iter().rev() {
                left = self.add_node_if_not_present(Node {variable,lo:left,hi:right});
                if variable==variables[0] { return left; }
                right = self.add_node_if_not_present(Node {variable,lo:right,hi: NodeIndex::FALSE});
            }
            panic!("Never got to the first variable.");
        }
    }

    /// Produce a BDD which is true iff exactly n of the given variables is true, regardless of other variables.
    /// The variables array must be sorted, smallest to highest.
    fn exactly_n_of_bdd(&mut self,n:usize,variables:&[VariableIndex],cache:&mut HashMap<(usize,usize),NodeIndex<A,M>>) -> NodeIndex<A,M> {
        if n>variables.len() { NodeIndex::FALSE }
        else if variables.len()==0 { return NodeIndex::TRUE }
        else if let Some(existing) = cache.get(&(n,variables.len()))  { *existing  }
        else {
            // deal with first variable being true
            let hi = if n>0 {self.exactly_n_of_bdd(n-1,&variables[1..],cache)} else {NodeIndex::FALSE};
            // deal with first variable being false
            let lo = self.exactly_n_of_bdd(n,&variables[1..],cache);
            let res = if lo==hi {lo} else {self.add_node_if_not_present(Node {variable:variables[0],lo,hi})};
            cache.insert((n,variables.len()),res);
            res
        }
    }


    fn zdd_variables_in_range_dont_matter(&mut self, base: NodeIndex<A,M>, range:Range<u16>) -> NodeIndex<A,M> {
        let mut res = base;
        for v in range.rev() {
            res=self.add_node_if_not_present(Node {variable:VariableIndex(v),lo:res,hi:res});
        }
        res
    }

    /// Produce a ZDD which is true iff exactly 1 of the given variables is true, regardless of other variables.
    /// The variables array must be sorted, smallest to highest.
    fn exactly_one_of_zdd(&mut self,variables:&[VariableIndex],total_num_variables:u16) -> NodeIndex<A,M> {
        if variables.len()==0 { NodeIndex::FALSE } else {
            let mut right = NodeIndex::TRUE;
            let mut left = NodeIndex::FALSE;
            let mut dealt_with = total_num_variables;
            // The diagram that is needed has two parallel diagonal lines, one right, one left.
            // One is on the right if one has had exactly 1 variable, one is on the left if one has had 0 variables.
            for &variable in variables.into_iter().rev() {
                left = self.zdd_variables_in_range_dont_matter(left,variable.0+1..dealt_with);
                right = self.zdd_variables_in_range_dont_matter(right,variable.0+1..dealt_with);
                dealt_with = variable.0;
                left = self.add_node_if_not_present(Node {variable,lo:left,hi:right});
                if variable==variables[0] { return self.zdd_variables_in_range_dont_matter(left,0..dealt_with); }
                right = self.add_node_if_not_present(Node {variable,lo:right,hi: NodeIndex::FALSE});
            }
            panic!("Never got to the first variable.");
        }
    }

    /// make a function that is true if starting evaluating a ZDD starting from upto.
    /// This is a long chain of variables from upto (inclusive) to total_num_variables (exclusive)
    /// where each elememt points to the next with both hi and lo, and the final field is NodeIndex::TRUE
    /// TODO cache.
    fn true_regardless_of_variables_below_zdd(&mut self,upto:VariableIndex,total_num_variables:u16) -> NodeIndex<A,M> {
        let mut index = NodeIndex::TRUE;
        for i in (upto.0..total_num_variables).rev() {
            let v = VariableIndex(i as u16);
            index = self.add_node_if_not_present(Node {
                variable : v,
                lo: index,
                hi: index,
            });
        }
        index
    }

    fn print_with_indentation(&self, index: NodeIndex<A,M>, indentation:usize) {
        print!("{: <1$}", "", indentation);
        if index.is_sink() { println!("{}",if index.is_true() {1} else {0}); }
        else {
            let node = self.node(index.address);
            println!("if variable {}",node.variable);
            self.print_with_indentation(node.hi,indentation+1);
            println!("{: <1$}else", "", indentation);
            self.print_with_indentation(node.lo,indentation+1);
        }
    }
    fn print(&self,index: NodeIndex<A,M>) {
        self.print_with_indentation(index,0);
    }

    /// Evaluate as a BDD with given variables.
    fn evaluate_bdd(&self, index: NodeIndex<A,M>, variables:&[bool]) -> bool {
        let mut index = index;
        while !index.is_sink() {
            let node = self.node(index.address);
            index = if variables[node.variable.0 as usize] {node.hi} else {node.lo}
        }
        index.is_true()
    }

    /// Evaluate as a ZDD with given variables.
    fn evaluate_zdd(&self, index: NodeIndex<A,M>, variables:&[bool]) -> bool {
        let mut up_to_variable = VariableIndex(0);
        let mut index = index;
        while !index.is_sink() {
            let node = self.node(index.address);
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
    ///
    /// Multiplicity of all terms in result is 1.
    fn not_bdd(&mut self, index: NodeIndex<A,M>, cache : &mut HashMap<A,A>) -> NodeIndex<A,M> {
        if index.is_true() { NodeIndex::FALSE }
        else if index.is_false() { NodeIndex::TRUE }
        else if let Some(&res) = cache.get(&index.address) { NodeIndex {address:res,multiplicity:M::ONE} }
        else {
            let node = self.node(index.address);
            let newnode = Node {
                variable: node.variable,
                lo: self.not_bdd(node.lo,cache),
                hi: self.not_bdd(node.hi,cache),
            };
            let res = self.add_node_if_not_present(newnode);
            cache.insert(index.address,res.address);
            res
        }
    }

    /// Make a node representing the negation of the function represented by the input node interpreted as a ZDD. A.k.a. ~ or !.
    /// upto should be be VariableIndex(0) unless you want to ignore variables less than it.
    /// TODO extend caching.
    ///
    /// Multiplicity of all terms in result is 1.
    fn not_zdd(&mut self, index: NodeIndex<A,M>, upto:VariableIndex, total_number_variables:u16, cache : &mut HashMap<(A, VariableIndex),A>) -> NodeIndex<A,M> {
        //println!("not_zdd({},{},{})",index,upto,total_number_variables);
        // else if index.is_true() { self.create_zdd_any_variables_below_given_variable_true(upto,total_number_variables) }
        let key = (index.address,upto);
        if let Some(&res) = cache.get(&key) { NodeIndex {address:res,multiplicity:M::ONE} }
        else {
            let res={
                let mut upper_bound = total_number_variables;
                let mut index = {
                    if index.is_false() { NodeIndex::TRUE }
                    else if index.is_true() { NodeIndex::FALSE }
                    else {
                        let node = self.node(index.address);
                        upper_bound = node.variable.0;
                        let new_upto = VariableIndex(node.variable.0+1);
                        let newnode = Node {
                            variable: node.variable,
                            lo: self.not_zdd(node.lo,new_upto,total_number_variables,cache),
                            hi: self.not_zdd(node.hi,new_upto,total_number_variables,cache),
                        };
                        if newnode.hi.is_false() { newnode.lo }
                        else { self.add_node_if_not_present(newnode) }
                    }
                };
                for i in (upto.0..upper_bound).rev() {
                    let hi = self.true_regardless_of_variables_below_zdd(VariableIndex(i+1),total_number_variables);
                    index = self.add_node_if_not_present(Node {
                        variable : VariableIndex(i),
                        lo: index,
                        hi,
                    });
                }
                index
            };
            cache.insert(key,res.address);
            res
        }
    }

    /// Create a node for a zdd (or find existing) for variable variable with lo and hi choices, and store it in the provided cache.
    /// Uniqueifies - sees if the hi and lo are same, in which case just produce lo, and looks for existing nodes.
    fn create_node_bdd<K:Eq+Hash>(&mut self, lo: NodeIndex<A,M>, hi: NodeIndex<A,M>, variable:VariableIndex, key:K, cache:&mut HashMap<K, NodeIndex<A,M>>) -> NodeIndex<A,M> {
        let res = if lo==hi { lo } else {
            self.add_node_if_not_present(Node {variable,lo,hi})
        };
        cache.insert(key,res);
        res
    }

    /// Make a node representing index1 and index2 (and in the logical sense, a.k.a. ∧ or &&)
    ///
    /// If multiplicities are involved, this is a Product operation. That is, the multiplicity of a value in the result is the product of the multiplicities of the value in the inputs.
    fn mul_bdd(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>, cache : &mut HashMap<(NodeIndex<A,M>, NodeIndex<A,M>), NodeIndex<A,M>>) -> NodeIndex<A,M> {
        if index1.is_false() || index2.is_false() { NodeIndex::FALSE }
        else if index1.is_true() { index2.multiply(index1.multiplicity) }
        else if index2.is_true() { index1.multiply(index2.multiplicity) }
        else if M::MULTIPLICITIES_IRRELEVANT && index1.address==index2.address { index1.multiply(index2.multiplicity) } // a&a is not a in presence of multiplicities. Or even a multiple of a.
        else {
            let key = if index1.address < index2.address {(index1,index2)} else {(index2,index1)};
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node_incorporating_multiplicity(index1);
                let node2 = self.node_incorporating_multiplicity(index2);
                let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1,index1)};
                let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2,index2)};
                let lo = self.mul_bdd(lo1,lo2,cache);
                let hi = self.mul_bdd(hi1,hi2,cache);
                self.create_node_bdd(lo,hi,if node1.variable <= node2.variable { node1.variable } else {node2.variable},key,cache)
            }
        }
    }

    fn node_incorporating_multiplicity(&self, index: NodeIndex<A,M>) -> Node<A,M> {
        let node = self.node(index.address);
        Node {
            variable: node.variable,
            lo: node.lo.multiply(index.multiplicity),
            hi: node.hi.multiply(index.multiplicity)
        }
    }

    /// Make a node representing index1 + index2 (like OR, but with taking multiplicities into account)
    /// For non-trivial multiplicities, this is the *Sum* operator, not the *Union* operator.
    ///
    /// In particular, the sum_bdd(f,g)(x) has multiplicity equal to the sum of the multiplicity of f(x) and g(x).
    fn sum_bdd(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>, cache : &mut HashMap<(NodeIndex<A,M>, NodeIndex<A,M>), NodeIndex<A,M>>) -> NodeIndex<A,M> {
        if index1.address==index2.address { NodeIndex {address:index1.address,multiplicity:M::combine_or(index1.multiplicity, index2.multiplicity)} }
        else if index1.is_false() { index2 }
        else if index2.is_false() { index1 }
        else if M::MULTIPLICITIES_IRRELEVANT && (index1.is_true() || index2.is_true()) { NodeIndex::TRUE }
            // if one of the two is true, then need to add true to both sides of the other to get multiplicities correct. The above line is just an optimization for that case.
        else {
            let (index1,index2) = if (M::SYMMETRIC_OR && index1.address < index2.address) || index1.address.is_true() {(index2,index1)} else {(index1,index2)};
            let key = (index1,index2);
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node_incorporating_multiplicity(index1);
                let node2 = if index2.is_true() {
                    Node {
                    variable: node1.variable,
                    lo: NodeIndex { address: A::TRUE, multiplicity: index2.multiplicity },
                    hi: NodeIndex { address: A::TRUE, multiplicity: index2.multiplicity }
                }} else {self.node_incorporating_multiplicity(index2)};
                let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1,index1)};
                let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2,index2)};
                let lo = self.sum_bdd(lo1,lo2,cache);
                let hi = self.sum_bdd(hi1,hi2,cache);
                self.create_node_bdd(lo,hi,if node1.variable <= node2.variable { node1.variable } else {node2.variable},key,cache)
            }
        }
    }


    /// compute index as a ZDD anded with NodeIndex::TRUE, which means take all lo branches on index1.
    fn and_zdd_true(&mut self, index: NodeIndex<A,M>) -> NodeIndex<A,M> {
        let mut index = index;
        while !index.is_sink() {
            index = self.node(index.address).lo.multiply(index.multiplicity);
        }
        index
    }

    /// Create a node for a zdd (or find existing) for variable variable with lo and hi choices, and store it in the provided cache.
    /// Uniqueifies - sees if the hi is false, in which case just produce lo, and looks for existing nodes.
    fn create_node_zdd<K:Eq+Hash>(&mut self, lo: NodeIndex<A,M>, hi: NodeIndex<A,M>, variable:VariableIndex, key:K, cache:&mut HashMap<K, NodeIndex<A,M>>) -> NodeIndex<A,M> {
        let res = if hi.is_false() { lo } else {
            self.add_node_if_not_present(Node {variable,lo,hi})
        };
        cache.insert(key,res);
        res
    }
    /// Make a node representing index1 and index2 (and in the logical sense, a.k.a. ∧ or &&)
    ///
    /// If multiplicities are involved, this is a Product operation. That is, the multiplicity of a value in the result is the product of the multiplicities of the value in the inputs.
    fn mul_zdd(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>, cache : &mut HashMap<(NodeIndex<A,M>, NodeIndex<A,M>), NodeIndex<A,M>>) -> NodeIndex<A,M> {
        if index1.is_false() || index2.is_false() { NodeIndex::FALSE }
        else if index1.is_true() { self.and_zdd_true(index2).multiply(index1.multiplicity) }
        else if index2.is_true() { self.and_zdd_true(index1).multiply(index2.multiplicity) }
        else if M::MULTIPLICITIES_IRRELEVANT && index1==index2 { index1.multiply(index2.multiplicity) } // a&a is not a in presence of multiplicities. Or even a multiple of a.
        else {
            let key = if index1.address < index2.address {(index1,index2)} else {(index2,index1)};
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node_incorporating_multiplicity(index1);
                let node2 = self.node_incorporating_multiplicity(index2);
                let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1, NodeIndex::FALSE)};
                let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2, NodeIndex::FALSE)};
                let lo = self.mul_zdd(lo1,lo2,cache);
                let hi = self.mul_zdd(hi1,hi2,cache);
                self.create_node_zdd(lo,hi,if node1.variable <= node2.variable { node1.variable } else {node2.variable},key,cache)
            }
        }
    }


    /// Make a node representing index1 + index2 (like OR, but with taking multiplicities into account)
    /// For non-trivial multiplicities, this is the *Sum* operator, not the *Union* operator.
    ///
    /// In particular, the sum_bdd(f,g)(x) has multiplicity equal to the sum of the multiplicity of f(x) and g(x).
    /// Make a node representing index1 and index2 (and in the logical sense, a.k.a. ∧ or &&)
    fn sum_zdd(&mut self, index1: NodeIndex<A,M>, index2: NodeIndex<A,M>, cache : &mut HashMap<(NodeIndex<A,M>, NodeIndex<A,M>), NodeIndex<A,M>>) -> NodeIndex<A,M> {
        if index1.address==index2.address { NodeIndex {address:index1.address,multiplicity:M::combine_or(index1.multiplicity, index2.multiplicity)} }
        else if index1.is_false() { index2 }
        else if index2.is_false() { index1 }
        // if one of the two is true, then need to add true to both sides of the other to get multiplicities correct. The above line is just an optimization for that case.
        else {
            let (index1,index2) = if (M::SYMMETRIC_OR && index1.address < index2.address) || index1.address.is_true() {(index2,index1)} else {(index1,index2)};
            let key = (index1,index2);
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node_incorporating_multiplicity(index1);
                let node2 = if index2.is_true() {
                    Node {
                    variable: node1.variable,
                    lo: NodeIndex { address: A::TRUE, multiplicity: index2.multiplicity },
                    hi: NodeIndex::FALSE
                }} else {self.node_incorporating_multiplicity(index2)};
                let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1, NodeIndex::FALSE)};
                let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2, NodeIndex::FALSE)};
                let lo = self.sum_zdd(lo1,lo2,cache);
                let hi = self.sum_zdd(hi1,hi2,cache);
                self.create_node_zdd(lo,hi,if node1.variable <= node2.variable { node1.variable } else {node2.variable},key,cache)
            }
        }
    }



    /// Create generating functions for nodes 0 inclusive to length exclusive.
    /// This is easy because of the topological sort.
    /// Return an array such that `res[node]` = the variable used at the time and the generating function.
    fn all_number_solutions<G:GeneratingFunctionWithMultiplicity<M>,const BDD:bool>(&self,length:usize,num_variables:u16) -> Vec<G> {
        let mut res = Vec::new();
        res.push(G::zero());
        res.push(G::one());
        for i in 2..length {
            let node = self.node(i.try_into().map_err(|_|()).unwrap());
            let next_variable = VariableIndex(node.variable.0+1);
            //println!("Computing {} lo={} hi={} variable={}",i,node.lo,node.hi,node.variable);
            let lo_g = res[node.lo.address.as_usize()].clone();
            let lo_g = if M::MULTIPLICITIES_IRRELEVANT || node.lo.multiplicity.is_unity() { lo_g } else { lo_g.multiply(node.lo.multiplicity) };
            let lo_level = if node.lo.is_sink() { VariableIndex(num_variables) } else { self.node(node.lo.address).variable };
            //println!("   lo_g={:?}, lo_level={}",lo_g,lo_level);
            let lo = if BDD {lo_g.deal_with_variable_range_being_indeterminate(next_variable,lo_level)} else {lo_g};
            let lo = lo.variable_not_set(node.variable);
            let hi_g = res[node.hi.address.as_usize()].clone();
            let hi_g = if M::MULTIPLICITIES_IRRELEVANT || node.hi.multiplicity.is_unity() { hi_g } else { hi_g.multiply(node.hi.multiplicity) };
            let hi_level = if node.hi.is_sink() { VariableIndex(num_variables) } else { self.node(node.hi.address).variable };
            //println!("   hi_g={:?}, hi_level={}",hi_g,hi_level);
            let hi = if BDD {hi_g.deal_with_variable_range_being_indeterminate(next_variable,hi_level)} else {hi_g};
            let hi = hi.variable_set(node.variable);
            //println!(" GF lo = {:?},   GF hi = {:?}",lo,hi);
            res.push(lo.add(hi));
        }
        //println!("{:?}",res);
        res
    }

    /// Create generating functions for nodes 0 inclusive to length exclusive.
    /// This is easy because of the topological sort.
    /// 
    /// Return a structure that could be used to select all solutions.
    /// 
    fn find_all_solutions<G: GeneratingFunctionWithMultiplicity<M>, const BDD: bool>(&self, index: NodeIndex<A, M>, num_variables:u16) -> SolutionFinder<A, M, Self, G, false> {
        let num_solutions_by_node = self.all_number_solutions::<G,BDD>(index.address.as_usize()+1,num_variables);
        SolutionFinder {
            num_solutions_by_node,
            number_of_variables_true_in_minimum_solution: vec![],
            xdd: &self,
            _phantom_a: Default::default(),
            _phantom_m: Default::default(),
            num_variables,
            start_node_index: index,
            is_bdd: BDD,
        }
    }
    fn number_solutions<G:GeneratingFunctionWithMultiplicity<M>,const BDD:bool>(&self, index: NodeIndex<A,M>, num_variables:u16) -> G {
        let work = self.all_number_solutions::<G,BDD>(index.address.as_usize()+1,num_variables);
        let found = work[index.address.as_usize()].clone();
        let before_multiplicity = if BDD {
            let level = if index.is_sink() { VariableIndex(num_variables) } else { self.node(index.address).variable };
            found.deal_with_variable_range_being_indeterminate(VariableIndex(0),level)
        } else { found };
        before_multiplicity.multiply(index.multiplicity)
    }

    fn number_solutions_bdd<G:GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A,M>, num_variables:u16) -> G { self.number_solutions::<G,true>(index, num_variables) }
    fn number_solutions_zdd<G:GeneratingFunctionWithMultiplicity<M>>(&self, index: NodeIndex<A,M>, num_variables:u16) -> G { self.number_solutions::<G,false>(index, num_variables) }

    /// Do garbage collection. Provide the items one wants to keep, and get rid of anything not in the transitive dependencies of keep.
    /// Returns a renamer from old nodes to new nodes.
    fn gc(&mut self, keep:impl IntoIterator<Item=NodeIndex<A,M>>) -> NodeRenaming<A>;

    fn make_dot_file<W:Write,F:Fn(VariableIndex)->String>(&self, writer:&mut W, name:impl Display, start_nodes:&[(NodeIndex<A,M>, Option<String>)], namer:F) -> std::io::Result<()> {
        //let namer = |i:VariableIndex| i.to_string();
        fn munge_label(s:&str) -> String { // see if html label.
            if s.starts_with('<') && s.ends_with('>') {s.to_string()} else { format!("\"{}\"",s) }
        }
        writeln!(writer,"digraph {} {{",name)?;
        let mut pending = Vec::new();
        for (entry_index,(node,nlabel)) in start_nodes.iter().enumerate() {
            writeln!(writer,"  e{} -> n{} [label=\"{}\"]",entry_index,node.address,node.multiplicity)?;
            pending.push(node.address);
            if let Some(label) = nlabel {
                writeln!(writer,"  e{} [label={}, shape=invtrapezium];",entry_index,munge_label(label))?;
            }
        }
        let mut done : HashSet<A> = Default::default();
        while let Some(index)=pending.pop() {
            if !(index.is_sink() || done.contains(&index)) {
                let node = self.node(index);
                writeln!(writer,"  n{} [label={}, xlabel={}];",index,munge_label(&namer(node.variable)),index)?;
                writeln!(writer,"  n{} -> n{} [style=dotted,label=\"{}\"];",index,node.lo.address,node.lo.multiplicity)?;
                writeln!(writer,"  n{} -> n{} [label=\"{}\"];",index,node.hi.address,node.hi.multiplicity)?;
                done.insert(index);
                pending.push(node.lo.address);
                pending.push(node.hi.address);
            }
        }
        writeln!(writer,"  n0 [label=\"0\",shape=box]")?;
        writeln!(writer,"  n1 [label=\"1\",shape=box]")?;
        writeln!(writer,"}}")?;
        Ok(())
    }

    /// Like find_all_solutions() except only produce those solutions that have a minimal
    /// number of the input arguments be true.
    ///
    /// That is, if f(a,b) = a or b, then find_all_solutions() will return `a`, `b` and `a,b` whereas
    /// this will just return `a` and `b`.
    ///
    /// Return a structure that could be used to select all solutions.
    ///
    fn find_all_solutions_with_minimal_true_arguments<G: GeneratingFunctionWithMultiplicity<M>, const BDD: bool>(&self, index: NodeIndex<A, M>, num_variables:u16) -> SolutionFinder<A, M, Self, G, true> {
        let length = index.address.as_usize()+1;
        let mut num_solutions_by_node = vec![G::zero(),G::one()]; // bottom node has no solutions, top node has 1.
        let mut number_of_variables_true_in_minimum_solution : Vec<usize> = vec![usize::MAX,0]; // bottom node has no solutions, top node is trivially solved.
        for i in 2..length {
            let node = self.node(i.try_into().map_err(|_|()).unwrap());
            let number_of_variables_lo = number_of_variables_true_in_minimum_solution[node.lo.address.as_usize()];
            let number_of_variables_hi = number_of_variables_true_in_minimum_solution[node.hi.address.as_usize()];
            let number_of_variables_here = if number_of_variables_hi==usize::MAX { number_of_variables_lo } else { number_of_variables_lo.min( number_of_variables_hi+1 )};
            number_of_variables_true_in_minimum_solution.push(number_of_variables_here);
            let use_lo = number_of_variables_lo==number_of_variables_here;
            let use_hi = number_of_variables_hi!=usize::MAX && number_of_variables_hi+1==number_of_variables_here;
            // let next_variable = VariableIndex(node.variable.0+1);
            //println!("Computing {} lo={} hi={} variable={}",i,node.lo,node.hi,node.variable);
            let lo = if use_lo {
                let lo_g = num_solutions_by_node[node.lo.address.as_usize()].clone();
                let lo_g = if M::MULTIPLICITIES_IRRELEVANT || node.lo.multiplicity.is_unity() { lo_g } else { lo_g.multiply(node.lo.multiplicity) };
                // let lo_level = if node.lo.is_sink() { VariableIndex(num_variables) } else { self.node(node.lo.address).variable };
                //println!("   lo_g={:?}, lo_level={}",lo_g,lo_level);
                let lo = lo_g; // if BDD {lo_g.deal_with_variable_range_being_indeterminate(next_variable,lo_level)} else {lo_g};
                lo.variable_not_set(node.variable)
            } else { G::zero() };
            let hi = if use_hi {
                let hi_g = num_solutions_by_node[node.hi.address.as_usize()].clone();
                let hi_g = if M::MULTIPLICITIES_IRRELEVANT || node.hi.multiplicity.is_unity() { hi_g } else { hi_g.multiply(node.hi.multiplicity) };
                // let hi_level = if node.hi.is_sink() { VariableIndex(num_variables) } else { self.node(node.hi.address).variable };
                //println!("   hi_g={:?}, hi_level={}",hi_g,hi_level);
                let hi = hi_g; // if BDD {hi_g.deal_with_variable_range_being_indeterminate(next_variable,hi_level)} else {hi_g};
                hi.variable_set(node.variable)
            } else {G::zero()};
            //println!(" GF lo = {:?},   GF hi = {:?}",lo,hi);
            num_solutions_by_node.push(lo.add(hi));
        }
        SolutionFinder {
            num_solutions_by_node,
            number_of_variables_true_in_minimum_solution,
            xdd: &self,
            _phantom_a: Default::default(),
            _phantom_m: Default::default(),
            num_variables,
            start_node_index: index,
            is_bdd: BDD,
        }
    }

    /// Determine the smallest number of variables that have to be true to make a node true, for nodes 0 inclusive to length exclusive.
    ///
    ///
    /// This is easy because of the topological sort.
    /// Return an array such that res[node] = the number of variables that are true to make the node true. usize::MAX means no solution.
    fn all_shortest_solutions(&self,length:usize) -> Vec<usize> {
        let mut res : Vec<usize> = Vec::new();
        res.push(usize::MAX); // bottom node has no solutions
        if length>1 {res.push(0);} // top node is trivially solved.
        for i in 2..length {
            let node = self.node(i.try_into().map_err(|_|()).unwrap());
            let lo = res[node.lo.address.as_usize()];
            let hi = res[node.hi.address.as_usize()];
            let current = if hi==usize::MAX { lo } else { lo.min( hi+1 )};
            res.push(current);
        }
        res
    }

    fn find_satisfying_solution_with_minimum_number_of_variables(&self, index: NodeIndex<A, M>) -> Option<Vec<VariableIndex>> {
        let work = self.all_shortest_solutions(index.address.as_usize()+1);
        if work[work.len()-1]==usize::MAX { None } else {
            let mut res = vec![];
            let mut current_index = index.address;
            while !current_index.is_sink() {
                // work backwards through work.
                let node = self.node(current_index);
                let lo = work[node.lo.address.as_usize()];
                let hi = work[node.hi.address.as_usize()];
                if hi==usize::MAX || lo<=hi+1 { // lo node is best route
                    current_index=node.lo.address;
                } else {
                    res.push(node.variable);
                    current_index=node.hi.address;
                }
            }
            Some(res)
        }
    }

}

pub struct SolutionFinder<'a,A:NodeAddress,M:Multiplicity,XDD:XDDBase<A,M> + ?Sized,G:GeneratingFunctionWithMultiplicity<M>,const WANT_MIN_VARS:bool> {
    num_solutions_by_node : Vec<G>,
    /// * If we want all solutions, then empty. 
    /// * If we only want solutions with the minimum number of variables true, then the best number of varaibles at each point (or usize::MAX if not defined)
    number_of_variables_true_in_minimum_solution : Vec<usize>,
    xdd : &'a XDD,
    _phantom_a : PhantomData<A>,
    _phantom_m : PhantomData<M>,
    num_variables : u16,
    start_node_index : NodeIndex<A, M>,
    is_bdd:bool,
}

impl <'a,A:NodeAddress,M:Multiplicity,XDD:XDDBase<A,M>,G:GeneratingFunctionWithMultiplicity<M>,const WANT_MIN_VARS:bool> SolutionFinder<'a,A,M,XDD,G,WANT_MIN_VARS> {

    fn number_solutions_starting_from_variable(&self,index:NodeIndex<A,M>, starting_variable:VariableIndex) -> G {
        let found = self.num_solutions_by_node[index.address.as_usize()].clone();
        let before_multiplicity = if self.is_bdd && !WANT_MIN_VARS {
            let level = if index.is_sink() { VariableIndex(self.num_variables) } else { self.xdd.node(index.address).variable };
            found.deal_with_variable_range_being_indeterminate(starting_variable,level)
        } else { found };
        before_multiplicity.multiply(index.multiplicity)
    }
    
    /// Compute the number of solutions. That is, the number of 
    pub fn number_solutions(&self) -> G {
        self.number_solutions_starting_from_variable(self.start_node_index,VariableIndex(0))
    }
    
}

impl <'a,A:NodeAddress,M:Multiplicity,XDD:XDDBase<A,M>,G:GeneratingFunctionWithMultiplicity<M> + Ord,const WANT_MIN_VARS:bool> SolutionFinder<'a,A,M,XDD,G,WANT_MIN_VARS> {

    /// Get a solution to the problem as a list of variables that are true in the solution.
    /// 
    /// solution_index specifies WHICH of the solutions to return. 0<= solution_index<number_solutions.
    /// If you were to write out a truth table, skip solution_index true results and get the next true result.
    /// 
    /// if solution_index<=number_solutions return the list, otherwise an error.
    pub fn get_ith_solution(&self, solution_index:G) -> Option<Vec<VariableIndex>> {
        if solution_index>=self.number_solutions() { return None; }
        let mut current_index = self.start_node_index;
        let mut res = vec![];
        let mut bypassed : G = G::zero(); // the number of solutions before the point we are at. Should always be <=solution_index.
        let mut scale : M = M::ONE; // multiply the number on left by this.
        let mut up_to_variable = VariableIndex(0);
        loop {
            // work backwards through work.
            scale = M::multiply(scale,current_index.multiplicity);
            if self.is_bdd && !WANT_MIN_VARS {
                let mut num_here = self.num_solutions_by_node[current_index.address.as_usize()].clone().multiply(scale);
                let mut num_here_doubling = vec![];
                let end_variable_index = if current_index.is_sink() { VariableIndex(self.num_variables) } else { self.xdd.node(current_index.address).variable };
                for i in (up_to_variable.0..end_variable_index.0).rev() {
                    num_here_doubling.push(num_here.clone());
                    num_here=num_here.deal_with_variable_being_indeterminate(VariableIndex(i));
                }
                for i in up_to_variable.0..end_variable_index.0 {
                    let num_in_question = num_here_doubling.pop().unwrap();
                    let bypassed_if_i_true = bypassed.clone().add(num_in_question.clone());
                    if bypassed_if_i_true<=solution_index {
                        bypassed=bypassed_if_i_true;
                        res.push(VariableIndex(i));
                    }
                }
                up_to_variable = VariableIndex(end_variable_index.0+1);
            }
            assert!(bypassed<=solution_index);
            assert!(bypassed.clone().add(self.num_solutions_by_node[current_index.address.as_usize()].clone().multiply(scale))>solution_index); // ,"{bypassed}+{}<=solution_index={solution_index}",self.num_solutions_by_node[current_index.address.as_usize()].clone().multiply(scale));
            if current_index.is_sink() { break; }
            let node = self.xdd.node(current_index.address);
            // if we are constrained by wanting the minimum number of variables, then we may not be able to go left.
            let could_go_left = (!WANT_MIN_VARS) || {
                let lo = self.number_of_variables_true_in_minimum_solution[node.lo.address.as_usize()];
                let hi = self.number_of_variables_true_in_minimum_solution[node.hi.address.as_usize()];
                hi==usize::MAX || lo<=hi+1
            };
            //println!("Node {} bypassed={bypassed} left={could_go_left} left={} right={}",current_index.address.as_usize(),self.number_solutions_starting_from_variable(node.lo,up_to_variable),self.number_solutions_starting_from_variable(node.hi,up_to_variable));
            // if we can go left, then do go left depending on the number of solutions to the left and i.
            let will_go_left = if could_go_left {
                let num_on_left = self.number_solutions_starting_from_variable(node.lo,up_to_variable).multiply(scale);
                let bypassed_if_choose_right = bypassed.clone().add(num_on_left);
                if bypassed_if_choose_right<=solution_index {
                    bypassed = bypassed_if_choose_right;
                    false
                } else { true }
            } else {false};
            //if will_go_left {println!("will go left");} else { println!("will go right"); }
            let new_index = if will_go_left { node.lo } else { res.push(node.variable); node.hi };
            current_index=new_index;
        }
        assert!(current_index.is_true());
        Some(res)
    }

}

/// A list of all the nodes.
/// This is a compact representation of nodes that is all that is needed to serialize/deserialize,
/// although it is not ideal for many operations that need hash table look-ups.
/// In particular find_node_index is slow.
///
/// Note that the two special indices are not explicitly stored.
#[derive(Clone,Eq, PartialEq)]
pub struct NodeList<A:NodeAddress,M:Multiplicity> {
    pub(crate) nodes : Vec<Node<A,M>>,
}

impl <A:NodeAddress,M:Multiplicity> Default for NodeList<A,M> {
    fn default() -> Self {
        NodeList{nodes:vec![]}
    }
}

impl <A:NodeAddress,M:Multiplicity> XDDBase<A,M> for NodeList<A,M> {
    fn node(&self, index: A) -> Node<A,M> { self.nodes[index.as_usize()-2] }
    fn find_node_index(&self, node: Node<A,M>) -> Option<A> {
        self.nodes.iter().position(|n|*n==node).map(|i|(i+2).try_into().map_err(|_|()).expect("Too many nodes for given address length"))
    }
    fn add_node(&mut self, node: Node<A,M>) -> A {
        self.nodes.push(node);
        (1+self.nodes.len()).try_into().map_err(|_|()).unwrap()
    }

    fn len(&self) -> usize { self.nodes.len() }

    /// Do garbage collection. Provide the items one wants to keep, and get rid of anything not in the transitive dependencies of keep.
    /// Returns a renamer such that v[old_node.0] is what v maps in to. If nothing, then map into NodeIndex::JUNK.
    fn gc(&mut self, keep:impl IntoIterator<Item=NodeIndex<A,M>>) -> NodeRenaming<A> {
        // First pass. Use map to say what to keep, use A::FALSE as a placeholder meaning the address is not used, and A::TRUE as a placeholder meaning the address is used.
        let mut map : Vec<A> = vec![A::FALSE;self.len()+2];
        fn do_keep<A:NodeAddress,M:Multiplicity>(nodes:&Vec<Node<A,M>>, map:&mut Vec<A>, n: NodeIndex<A,M>) {
            let address = n.address.as_usize();
            if map[address]!=A::TRUE {
                map[address]=A::TRUE;
                let node = nodes[address-2];
                do_keep(nodes,map,node.lo);
                do_keep(nodes,map,node.hi);
            }
        }
        map[0]=A::TRUE; // FALSE
        map[1]=A::TRUE; // TRUE
        for k in keep.into_iter() {
            do_keep(&self.nodes,&mut map,k);
        }
        // Now set values to actual values rather than dummy boolean placeholders.
        map[0]=A::FALSE;
        map[1]=A::TRUE;
        let mut len:usize = 0;
        for i in 2..map.len() {
            let into = map[i];
            if into==A::TRUE { // should keep this address
                map[i]=(len+2).try_into().map_err(|_|()).unwrap();
                let old_node = self.nodes[i-2];
                self.nodes[len]= Node {
                    variable: old_node.variable,
                    lo: NodeIndex { address: map[old_node.lo.address.as_usize()], multiplicity:old_node.lo.multiplicity},
                    hi: NodeIndex { address: map[old_node.hi.address.as_usize()], multiplicity:old_node.hi.multiplicity},
                };
                len=len+1;
            }
        }
        self.nodes.truncate(len);
        // now convert map into the final result.
        NodeRenaming(map)
    }

}

/// An extension to NodeList that contains a cache from nodes to indices that is constantly
/// kept up to date.
#[derive(Clone,Eq, PartialEq)]
pub struct NodeListWithFastLookup<A:NodeAddress,M:Multiplicity> {
    pub(crate) nodes : NodeList<A,M>,
    pub(crate) node_to_index : HashMap<Node<A,M>,A>,
}

impl <A:NodeAddress,M:Multiplicity> Default for NodeListWithFastLookup<A,M> {
    fn default() -> Self {
        NodeListWithFastLookup{ nodes: NodeList::default(), node_to_index: Default::default() }
    }
}

impl <A:NodeAddress,M:Multiplicity> XDDBase<A,M> for NodeListWithFastLookup<A,M> {
    fn node(&self, index: A) -> Node<A,M> { self.nodes.node(index) }
    fn find_node_index(&self, node: Node<A,M>) -> Option<A> {
        self.node_to_index.get(&node).cloned()
    }

    fn add_node(&mut self, node: Node<A,M>) -> A {
        let res = self.nodes.add_node(node);
        self.node_to_index.insert(node, res);
        res
    }
    fn len(&self) -> usize { self.nodes.len() }

    fn gc(&mut self, keep: impl IntoIterator<Item=NodeIndex<A,M>>) -> NodeRenaming<A> {
        let map = self.nodes.gc(keep);
        self.node_to_index.clear();
        for (i,node) in self.nodes.nodes.iter().enumerate() {
            self.node_to_index.insert(*node,(i+2).try_into().map_err(|_|()).unwrap());
        }
        map
    }
}
