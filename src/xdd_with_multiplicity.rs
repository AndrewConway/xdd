//! My own invention, decision diagrams with multiplicities.
//!
//! Where XDDs represent a set, the equivalent version with multiplicities represents a multiset.
//!

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::io::Write;
use std::ops::Range;
use crate::{NodeWithMultiplicity, NodeIndexWithMultiplicity, VariableIndex, NodeAddress, Multiplicity, NodeRenamingWithMuliplicity};
use crate::generating_function::{GeneratingFunctionWithMultiplicity};

/// Functions that any representation of an XDD must have, although some representations
/// will execute this more quickly than others, at the cost of more memory capacity.
pub trait XDDBase<A:NodeAddress,M:Multiplicity> {
    /// Get the node pointed to by a NodeIndex. panic if it does not exist.
    /// Do NOT call with the two special node indices NodeIndex::TRUE or NodeIndex::FALSE
    /// Also nodes should be sorted topologically - that is, node(x).hi>x and node(x).lo>x for all x.
    fn node(&self,index:A) -> NodeWithMultiplicity<A,M>;
    /// Get the node index for a node if it is already present.
    fn find_node_index(&self,node:NodeWithMultiplicity<A,M>) -> Option<A>;
    /// Add a node to the list, returning its new index.
    fn add_node(&mut self,node:NodeWithMultiplicity<A,M>) -> A;
    /// The number of nodes in this tree, not counting the two special node indices.
    fn len(&self) -> usize;

    /// Like add_node, but first check with find_node_index to see if it is already there. Also canonicalize multiplicities by removing gcd.
    fn add_node_if_not_present(&mut self,node:NodeWithMultiplicity<A,M>) -> NodeIndexWithMultiplicity<A,M> {
        let (node,multiplicity) = if M::MULTIPLICITIES_IRRELEVANT { (node,M::ONE) }
        else { // for uniqueness, want to make sure that there is no gcd of the hi and lo values.
            let (m_lo,m_hi,multiplicity) =
                if node.hi.is_false() { (M::ONE,M::ONE,node.lo.multiplicity) } // note that for the false node, multiplicity is irrelevant, and so gcd has to account for that.
                else if node.lo.is_false() { (M::ONE,M::ONE,node.hi.multiplicity) }
                else { M::gcd(node.lo.multiplicity,node.hi.multiplicity) };
            let node = NodeWithMultiplicity{ variable:node.variable, lo: NodeIndexWithMultiplicity{ address: node.lo.address, multiplicity: m_lo }, hi: NodeIndexWithMultiplicity{ address: node.hi.address, multiplicity: m_hi } };
            (node,multiplicity)
        };
        let address = self.find_node_index(node).unwrap_or_else(||self.add_node(node));
        NodeIndexWithMultiplicity{address,multiplicity}
    }

    /// Produce a DD that describes a single variable. That is, a DD that has just that variable leading to TRUE iff variable is true.
    /// * For a BDD, this is a simple function f(v,...)=v.
    /// * For a ZDD, this means a function f(v,...) = v & !(union of all other variables).
    fn single_variable(&mut self,variable:VariableIndex) -> NodeIndexWithMultiplicity<A,M> {
        self.add_node_if_not_present(NodeWithMultiplicity{variable,lo:NodeIndexWithMultiplicity::FALSE,hi:NodeIndexWithMultiplicity::TRUE})
    }

    /// Produce a ZDD that describes a single variable. That is, a ZDD that has all variables having no effect other than just that variable leading to TRUE iff variable is true.
    /// * For a ZDD, this is a simple function f(v,...)=v.
    /// * This is not a valid BDD.
    fn single_variable_zdd(&mut self,variable:VariableIndex,total_num_variables:u16) -> NodeIndexWithMultiplicity<A,M> {
        let mut index = NodeIndexWithMultiplicity::TRUE;
        for i in (0..total_num_variables).rev() {
            let v = VariableIndex(i);
            index = self.add_node_if_not_present(NodeWithMultiplicity {
                variable : v,
                lo: if v==variable { NodeIndexWithMultiplicity::FALSE } else { index },
                hi: index,
            });
        }
        index
    }

    /// Produce a BDD which is true iff exactly 1 of the given variables is true, regardless of other variables.
    /// The variables array must be sorted, smallest to highest.
    fn exactly_one_of_bdd(&mut self,variables:&[VariableIndex]) -> NodeIndexWithMultiplicity<A,M> {
        if variables.len()==0 { NodeIndexWithMultiplicity::FALSE } else {
            let mut right = NodeIndexWithMultiplicity::TRUE;
            let mut left = NodeIndexWithMultiplicity::FALSE;
            // The diagram that is needed has two parallel diagonal lines, one right, one left.
            // One is on the right if one has had exactly 1 variable, one is on the left if one has had 0 variables.
            for &variable in variables.into_iter().rev() {
                left = self.add_node_if_not_present(NodeWithMultiplicity{variable,lo:left,hi:right});
                if variable==variables[0] { return left; }
                right = self.add_node_if_not_present(NodeWithMultiplicity{variable,lo:right,hi:NodeIndexWithMultiplicity::FALSE});
            }
            panic!("Never got to the first variable.");
        }
    }

    fn zdd_variables_in_range_dont_matter(&mut self,base:NodeIndexWithMultiplicity<A,M>,range:Range<u16>) -> NodeIndexWithMultiplicity<A,M> {
        let mut res = base;
        for v in range.rev() {
            res=self.add_node_if_not_present(NodeWithMultiplicity{variable:VariableIndex(v),lo:res,hi:res});
        }
        res
    }

    /// Produce a ZDD which is true iff exactly 1 of the given variables is true, regardless of other variables.
    /// The variables array must be sorted, smallest to highest.
    fn exactly_one_of_zdd(&mut self,variables:&[VariableIndex],total_num_variables:u16) -> NodeIndexWithMultiplicity<A,M> {
        if variables.len()==0 { NodeIndexWithMultiplicity::FALSE } else {
            let mut right = NodeIndexWithMultiplicity::TRUE;
            let mut left = NodeIndexWithMultiplicity::FALSE;
            let mut dealt_with = total_num_variables;
            // The diagram that is needed has two parallel diagonal lines, one right, one left.
            // One is on the right if one has had exactly 1 variable, one is on the left if one has had 0 variables.
            for &variable in variables.into_iter().rev() {
                left = self.zdd_variables_in_range_dont_matter(left,variable.0+1..dealt_with);
                right = self.zdd_variables_in_range_dont_matter(right,variable.0+1..dealt_with);
                dealt_with = variable.0;
                left = self.add_node_if_not_present(NodeWithMultiplicity{variable,lo:left,hi:right});
                if variable==variables[0] { return self.zdd_variables_in_range_dont_matter(left,0..dealt_with); }
                right = self.add_node_if_not_present(NodeWithMultiplicity{variable,lo:right,hi:NodeIndexWithMultiplicity::FALSE});
            }
            panic!("Never got to the first variable.");
        }
    }

    /// make a function that is true if starting evaluating a ZDD starting from upto.
    /// This is a long chain of variables from upto (inclusive) to total_num_variables (exclusive)
    /// where each elememt points to the next with both hi and lo, and the final field is NodeIndex::TRUE
    /// TODO cache.
    fn true_regardless_of_variables_below_zdd(&mut self,upto:VariableIndex,total_num_variables:u16) -> NodeIndexWithMultiplicity<A,M> {
        let mut index = NodeIndexWithMultiplicity::TRUE;
        for i in (upto.0..total_num_variables).rev() {
            let v = VariableIndex(i as u16);
            index = self.add_node_if_not_present(NodeWithMultiplicity{
                variable : v,
                lo: index,
                hi: index,
            });
        }
        index
    }

    fn print_with_indentation(&self,index:NodeIndexWithMultiplicity<A,M>,indentation:usize) {
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
    fn print(&self,index:NodeIndexWithMultiplicity<A,M>) {
        self.print_with_indentation(index,0);
    }

    /// Evaluate as a BDD with given variables.
    fn evaluate_bdd(&self,index:NodeIndexWithMultiplicity<A,M>,variables:&[bool]) -> bool {
        let mut index = index;
        while !index.is_sink() {
            let node = self.node(index.address);
            index = if variables[node.variable.0 as usize] {node.hi} else {node.lo}
        }
        index.is_true()
    }

    /// Evaluate as a ZDD with given variables.
    fn evaluate_zdd(&self,index:NodeIndexWithMultiplicity<A,M>,variables:&[bool]) -> bool {
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
    fn not_bdd(&mut self,index:NodeIndexWithMultiplicity<A,M>,cache : &mut HashMap<A,A>) -> NodeIndexWithMultiplicity<A,M> {
        if index.is_true() { NodeIndexWithMultiplicity::FALSE }
        else if index.is_false() { NodeIndexWithMultiplicity::TRUE }
        else if let Some(&res) = cache.get(&index.address) { NodeIndexWithMultiplicity{address:res,multiplicity:M::ONE} }
        else {
            let node = self.node(index.address);
            let newnode = NodeWithMultiplicity {
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
    fn not_zdd(&mut self,index:NodeIndexWithMultiplicity<A,M>,upto:VariableIndex,total_number_variables:u16,cache : &mut HashMap<(A,VariableIndex),A>) -> NodeIndexWithMultiplicity<A,M> {
        //println!("not_zdd({},{},{})",index,upto,total_number_variables);
        // else if index.is_true() { self.create_zdd_any_variables_below_given_variable_true(upto,total_number_variables) }
        let key = (index.address,upto);
        if let Some(&res) = cache.get(&key) { NodeIndexWithMultiplicity{address:res,multiplicity:M::ONE} }
        else {
            let res={
                let mut upper_bound = total_number_variables;
                let mut index = {
                    if index.is_false() { NodeIndexWithMultiplicity::TRUE }
                    else if index.is_true() { NodeIndexWithMultiplicity::FALSE }
                    else {
                        let node = self.node(index.address);
                        upper_bound = node.variable.0;
                        let new_upto = VariableIndex(node.variable.0+1);
                        let newnode = NodeWithMultiplicity {
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
                    index = self.add_node_if_not_present(NodeWithMultiplicity {
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
    fn create_node_bdd<K:Eq+Hash>(&mut self,lo:NodeIndexWithMultiplicity<A,M>,hi:NodeIndexWithMultiplicity<A,M>,variable:VariableIndex,key:K,cache:&mut HashMap<K,NodeIndexWithMultiplicity<A,M>>) -> NodeIndexWithMultiplicity<A,M> {
        let res = if lo==hi { lo } else {
            self.add_node_if_not_present(NodeWithMultiplicity{variable,lo,hi})
        };
        cache.insert(key,res);
        res
    }

    /// Make a node representing index1 and index2 (and in the logical sense, a.k.a. ∧ or &&)
    ///
    /// If multiplicities are involved, this is a Product operation. That is, the multiplicity of a value in the result is the product of the multiplicities of the value in the inputs.
    fn mul_bdd(&mut self,index1:NodeIndexWithMultiplicity<A,M>,index2:NodeIndexWithMultiplicity<A,M>,cache : &mut HashMap<(NodeIndexWithMultiplicity<A,M>,NodeIndexWithMultiplicity<A,M>),NodeIndexWithMultiplicity<A,M>>) -> NodeIndexWithMultiplicity<A,M> {
        if index1.is_false() || index2.is_false() { NodeIndexWithMultiplicity::FALSE }
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

    fn node_incorporating_multiplicity(&self,index:NodeIndexWithMultiplicity<A,M>) -> NodeWithMultiplicity<A,M> {
        let node = self.node(index.address);
        NodeWithMultiplicity{
            variable: node.variable,
            lo: node.lo.multiply(index.multiplicity),
            hi: node.hi.multiply(index.multiplicity)
        }
    }

    /// Make a node representing index1 + index2 (like OR, but with taking multiplicities into account)
    /// For non-trivial multiplicities, this is the *Sum* operator, not the *Union* operator.
    ///
    /// In particular, the sum_bdd(f,g)(x) has multiplicity equal to the sum of the multiplicity of f(x) and g(x).
    fn sum_bdd(&mut self,index1:NodeIndexWithMultiplicity<A,M>,index2:NodeIndexWithMultiplicity<A,M>,cache : &mut HashMap<(NodeIndexWithMultiplicity<A,M>,NodeIndexWithMultiplicity<A,M>),NodeIndexWithMultiplicity<A,M>>) -> NodeIndexWithMultiplicity<A,M> {
        if index1.address==index2.address { NodeIndexWithMultiplicity{address:index1.address,multiplicity:M::combine_or(index1.multiplicity,index2.multiplicity)} }
        else if index1.is_false() { index2 }
        else if index2.is_false() { index1 }
        else if M::MULTIPLICITIES_IRRELEVANT && (index1.is_true() || index2.is_true()) { NodeIndexWithMultiplicity::TRUE }
            // if one of the two is true, then need to add true to both sides of the other to get multiplicities correct. The above line is just an optimization for that case.
        else {
            let (index1,index2) = if (M::SYMMETRIC_OR && index1.address < index2.address) || index1.address.is_true() {(index2,index1)} else {(index1,index2)};
            let key = (index1,index2);
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node_incorporating_multiplicity(index1);
                let node2 = if index2.is_true() {NodeWithMultiplicity{
                    variable: node1.variable,
                    lo: NodeIndexWithMultiplicity { address: A::TRUE, multiplicity: index2.multiplicity },
                    hi: NodeIndexWithMultiplicity { address: A::TRUE, multiplicity: index2.multiplicity }
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
    fn and_zdd_true(&mut self,index:NodeIndexWithMultiplicity<A,M>) -> NodeIndexWithMultiplicity<A,M> {
        let mut index = index;
        while !index.is_sink() {
            index = self.node(index.address).lo.multiply(index.multiplicity);
        }
        index
    }

    /// Create a node for a zdd (or find existing) for variable variable with lo and hi choices, and store it in the provided cache.
    /// Uniqueifies - sees if the hi is false, in which case just produce lo, and looks for existing nodes.
    fn create_node_zdd<K:Eq+Hash>(&mut self,lo:NodeIndexWithMultiplicity<A,M>,hi:NodeIndexWithMultiplicity<A,M>,variable:VariableIndex,key:K,cache:&mut HashMap<K,NodeIndexWithMultiplicity<A,M>>) -> NodeIndexWithMultiplicity<A,M> {
        let res = if hi.is_false() { lo } else {
            self.add_node_if_not_present(NodeWithMultiplicity{variable,lo,hi})
        };
        cache.insert(key,res);
        res
    }
    /// Make a node representing index1 and index2 (and in the logical sense, a.k.a. ∧ or &&)
    ///
    /// If multiplicities are involved, this is a Product operation. That is, the multiplicity of a value in the result is the product of the multiplicities of the value in the inputs.
    fn mul_zdd(&mut self,index1:NodeIndexWithMultiplicity<A,M>,index2:NodeIndexWithMultiplicity<A,M>,cache : &mut HashMap<(NodeIndexWithMultiplicity<A,M>,NodeIndexWithMultiplicity<A,M>),NodeIndexWithMultiplicity<A,M>>) -> NodeIndexWithMultiplicity<A,M> {
        if index1.is_false() || index2.is_false() { NodeIndexWithMultiplicity::FALSE }
        else if index1.is_true() { self.and_zdd_true(index2).multiply(index1.multiplicity) }
        else if index2.is_true() { self.and_zdd_true(index1).multiply(index2.multiplicity) }
        else if M::MULTIPLICITIES_IRRELEVANT && index1==index2 { index1.multiply(index2.multiplicity) } // a&a is not a in presence of multiplicities. Or even a multiple of a.
        else {
            let key = if index1.address < index2.address {(index1,index2)} else {(index2,index1)};
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node_incorporating_multiplicity(index1);
                let node2 = self.node_incorporating_multiplicity(index2);
                let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1,NodeIndexWithMultiplicity::FALSE)};
                let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2,NodeIndexWithMultiplicity::FALSE)};
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
    fn sum_zdd(&mut self,index1:NodeIndexWithMultiplicity<A,M>,index2:NodeIndexWithMultiplicity<A,M>,cache : &mut HashMap<(NodeIndexWithMultiplicity<A,M>,NodeIndexWithMultiplicity<A,M>),NodeIndexWithMultiplicity<A,M>>) -> NodeIndexWithMultiplicity<A,M> {
        if index1.address==index2.address { NodeIndexWithMultiplicity{address:index1.address,multiplicity:M::combine_or(index1.multiplicity,index2.multiplicity)} }
        else if index1.is_false() { index2 }
        else if index2.is_false() { index1 }
        // if one of the two is true, then need to add true to both sides of the other to get multiplicities correct. The above line is just an optimization for that case.
        else {
            let key = if (M::SYMMETRIC_OR && index1.address < index2.address) || index1.address.is_true() {(index1,index2)} else {(index2,index1)};
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node_incorporating_multiplicity(index1);
                let node2 = if index2.is_true() {NodeWithMultiplicity{
                    variable: node1.variable,
                    lo: NodeIndexWithMultiplicity { address: A::TRUE, multiplicity: index2.multiplicity },
                    hi: NodeIndexWithMultiplicity::FALSE
                }} else {self.node_incorporating_multiplicity(index2)};
                let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1,NodeIndexWithMultiplicity::FALSE)};
                let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2,NodeIndexWithMultiplicity::FALSE)};
                let lo = self.sum_zdd(lo1,lo2,cache);
                let hi = self.sum_zdd(hi1,hi2,cache);
                self.create_node_zdd(lo,hi,if node1.variable <= node2.variable { node1.variable } else {node2.variable},key,cache)
            }
        }
    }



    /// Create generating functions for nodes 0 inclusive to length exclusive.
    /// This is easy because of the topological sort.
    /// Return an array such that res[node] = the variable used at the time and the generating function.
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

    fn number_solutions<G:GeneratingFunctionWithMultiplicity<M>,const BDD:bool>(&self,index:NodeIndexWithMultiplicity<A,M>,num_variables:u16) -> G {
        let work = self.all_number_solutions::<G,BDD>(index.address.as_usize()+1,num_variables);
        let found = work[index.address.as_usize()].clone();
        let before_multiplicity = if BDD {
            let level = if index.is_sink() { VariableIndex(num_variables) } else { self.node(index.address).variable };
            found.deal_with_variable_range_being_indeterminate(VariableIndex(0),level)
        } else { found };
        before_multiplicity.multiply(index.multiplicity)
    }

    fn number_solutions_bdd<G:GeneratingFunctionWithMultiplicity<M>>(&self,index:NodeIndexWithMultiplicity<A,M>,num_variables:u16) -> G { self.number_solutions::<G,true>(index,num_variables) }
    fn number_solutions_zdd<G:GeneratingFunctionWithMultiplicity<M>>(&self,index:NodeIndexWithMultiplicity<A,M>,num_variables:u16) -> G { self.number_solutions::<G,false>(index,num_variables) }

    /// Do garbage collection. Provide the items one wants to keep, and get rid of anything not in the transitive dependencies of keep.
    /// Returns a renamer from old nodes to new nodes.
    fn gc(&mut self,keep:impl IntoIterator<Item=NodeIndexWithMultiplicity<A,M>>) -> NodeRenamingWithMuliplicity<A>;

    fn make_dot_file<W:Write,F:Fn(VariableIndex)->String>(&self,writer:&mut W,name:impl Display,start_nodes:&[(NodeIndexWithMultiplicity<A,M>,Option<String>)],namer:F) -> std::io::Result<()> {
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
}



/// A list of all the nodes.
/// This is a compact representation of nodes that is all that is needed to serialize/deserialize,
/// although it is not ideal for many operations that need hash table look-ups.
/// In particular find_node_index is slow.
///
/// Note that the two special indices are not explicitly stored.
#[derive(Clone,Eq, PartialEq,Default)]
pub struct NodeList<A:NodeAddress,M:Multiplicity> {
    pub(crate) nodes : Vec<NodeWithMultiplicity<A,M>>,
}

impl <A:NodeAddress,M:Multiplicity> XDDBase<A,M> for NodeList<A,M> {
    fn node(&self, index: A) -> NodeWithMultiplicity<A,M> { self.nodes[index.as_usize()-2] }
    fn find_node_index(&self, node: NodeWithMultiplicity<A,M>) -> Option<A> {
        self.nodes.iter().position(|n|*n==node).map(|i|(i+2).try_into().map_err(|_|()).expect("Too many nodes for given address length"))
    }
    fn add_node(&mut self, node: NodeWithMultiplicity<A,M>) -> A {
        self.nodes.push(node);
        (1+self.nodes.len()).try_into().map_err(|_|()).unwrap()
    }

    fn len(&self) -> usize { self.nodes.len() }

    /// Do garbage collection. Provide the items one wants to keep, and get rid of anything not in the transitive dependencies of keep.
    /// Returns a renamer such that v[old_node.0] is what v maps in to. If nothing, then map into NodeIndex::JUNK.
    fn gc(&mut self,keep:impl IntoIterator<Item=NodeIndexWithMultiplicity<A,M>>) -> NodeRenamingWithMuliplicity<A> {
        // First pass. Use map to say what to keep, use A::FALSE as a placeholder meaning the address is not used, and A::TRUE as a placeholder meaning the address is used.
        let mut map : Vec<A> = vec![A::FALSE;self.len()+2];
        fn do_keep<A:NodeAddress,M:Multiplicity>(nodes:&Vec<NodeWithMultiplicity<A,M>>,map:&mut Vec<A>,n:NodeIndexWithMultiplicity<A,M>) {
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
                self.nodes[len]=NodeWithMultiplicity {
                    variable: old_node.variable,
                    lo: NodeIndexWithMultiplicity{ address: map[old_node.lo.address.as_usize()], multiplicity:old_node.lo.multiplicity},
                    hi: NodeIndexWithMultiplicity{ address: map[old_node.hi.address.as_usize()], multiplicity:old_node.hi.multiplicity},
                };
                len=len+1;
            }
        }
        self.nodes.truncate(len);
        // now convert map into the final result.
        NodeRenamingWithMuliplicity(map)
    }

}

/// An extension to NodeList that contains a cache from nodes to indices that is constantly
/// kept up to date.
#[derive(Clone,Eq, PartialEq,Default)]
pub struct NodeListWithFastLookup<A:NodeAddress,M:Multiplicity> {
    pub(crate) nodes : NodeList<A,M>,
    pub(crate) node_to_index : HashMap<NodeWithMultiplicity<A,M>,A>,
}

impl <A:NodeAddress,M:Multiplicity> XDDBase<A,M> for NodeListWithFastLookup<A,M> {
    fn node(&self, index: A) -> NodeWithMultiplicity<A,M> { self.nodes.node(index) }
    fn find_node_index(&self, node: NodeWithMultiplicity<A,M>) -> Option<A> {
        self.node_to_index.get(&node).cloned()
    }

    fn add_node(&mut self, node: NodeWithMultiplicity<A,M>) -> A {
        let res = self.nodes.add_node(node);
        self.node_to_index.insert(node, res);
        res
    }
    fn len(&self) -> usize { self.nodes.len() }

    fn gc(&mut self, keep: impl IntoIterator<Item=NodeIndexWithMultiplicity<A,M>>) -> NodeRenamingWithMuliplicity<A> {
        let map = self.nodes.gc(keep);
        self.node_to_index.clear();
        for (i,node) in self.nodes.nodes.iter().enumerate() {
            self.node_to_index.insert(*node,(i+2).try_into().map_err(|_|()).unwrap());
        }
        map
    }
}
