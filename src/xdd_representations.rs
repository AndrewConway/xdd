use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;
use crate::{Node, NodeIndex, NodeRenaming, VariableIndex};
use crate::generating_function::GeneratingFunction;

/// Functions that any representation of an XDD must have, although some representations
/// will execute this more quickly than others, at the cost of more memory capacity.
pub trait XDDBase {
    /// Get the node pointed to by a NodeIndex. panic if it does not exist.
    /// Do NOT call with the two special node indices NodeIndex::TRUE or NodeIndex::FALSE
    /// Also nodes should be sorted topologically - that is, node(x).hi>x and node(x).lo>x for all x.
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
    fn single_variable_zdd(&mut self,variable:VariableIndex,total_num_variables:u16) -> NodeIndex {
        let mut index = NodeIndex::TRUE;
        for i in (0..total_num_variables).rev() {
            let v = VariableIndex(i);
            index = self.add_node_if_not_present(Node{
                variable : v,
                lo: if v==variable { NodeIndex::FALSE } else { index },
                hi: index,
            });
        }
        index
    }

    /// Produce a BDD which is true iff exactly 1 of the given variables is true, regardless of other variables.
    /// The variables array must be sorted, smallest to highest.
    fn exactly_one_of_bdd(&mut self,variables:&[VariableIndex]) -> NodeIndex {
        if variables.len()==0 { NodeIndex::FALSE } else {
            let mut right = NodeIndex::TRUE;
            let mut left = NodeIndex::FALSE;
            // The diagram that is needed has two parallel diagonal lines, one right, one left.
            // One is on the right if one has had exactly 1 variable, one is on the left if one has had 0 variables.
            for &variable in variables.into_iter().rev() {
                left = self.add_node_if_not_present(Node{variable,lo:left,hi:right});
                if variable==variables[0] { return left; }
                right = self.add_node_if_not_present(Node{variable,lo:right,hi:NodeIndex::FALSE});
            }
            panic!("Never got to the first variable.");
        }
    }

    fn zdd_variables_in_range_dont_matter(&mut self,base:NodeIndex,range:Range<u16>) -> NodeIndex {
        let mut res = base;
        for v in range.rev() {
            res=self.add_node_if_not_present(Node{variable:VariableIndex(v),lo:res,hi:res});
        }
        res
    }

    /// Produce a ZDD which is true iff exactly 1 of the given variables is true, regardless of other variables.
    /// The variables array must be sorted, smallest to highest.
    fn exactly_one_of_zdd(&mut self,variables:&[VariableIndex],total_num_variables:u16) -> NodeIndex {
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
                left = self.add_node_if_not_present(Node{variable,lo:left,hi:right});
                if variable==variables[0] { return self.zdd_variables_in_range_dont_matter(left,0..dealt_with); }
                right = self.add_node_if_not_present(Node{variable,lo:right,hi:NodeIndex::FALSE});
            }
            panic!("Never got to the first variable.");
        }
    }

    /// make a function that is true if starting evaluating a ZDD starting from upto.
    /// This is a long chain of variables from upto (inclusive) to total_num_variables (exclusive)
    /// where each elememt points to the next with both hi and lo, and the final field is NodeIndex::TRUE
    /// TODO cache.
    fn true_regardless_of_variables_below_zdd(&mut self,upto:VariableIndex,total_num_variables:u16) -> NodeIndex {
        let mut index = NodeIndex::TRUE;
        for i in (upto.0..total_num_variables).rev() {
            let v = VariableIndex(i as u16);
            index = self.add_node_if_not_present(Node{
                variable : v,
                lo: index,
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
            println!("{: <1$}else", "", indentation);
            self.print_with_indentation(node.lo,indentation+1);
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
    fn not_bdd(&mut self,index:NodeIndex,cache : &mut HashMap<NodeIndex,NodeIndex>) -> NodeIndex {
        if index.is_true() { NodeIndex::FALSE }
        else if index.is_false() { NodeIndex::TRUE }
        else if let Some(&res) = cache.get(&index) { res }
        else {
            let node = self.node(index);
            let newnode = Node {
                variable: node.variable,
                lo: self.not_bdd(node.lo,cache),
                hi: self.not_bdd(node.hi,cache),
            };
            let res = self.add_node_if_not_present(newnode);
            cache.insert(index,res);
            res
        }
    }

    /// Make a node representing the negation of the function represented by the input node interpreted as a ZDD. A.k.a. ~ or !.
    /// upto should be be VariableIndex(0) unless you want to ignore variables less than it.
    /// TODO extend caching.
    fn not_zdd(&mut self,index:NodeIndex,upto:VariableIndex,total_number_variables:u16,cache : &mut HashMap<(NodeIndex,VariableIndex),NodeIndex>) -> NodeIndex {
        //println!("not_zdd({},{},{})",index,upto,total_number_variables);
        // else if index.is_true() { self.create_zdd_any_variables_below_given_variable_true(upto,total_number_variables) }
        let key = (index,upto);
        if let Some(&res) = cache.get(&key) { res }
        else {
            let res={
                let mut upper_bound = total_number_variables;
                let mut index = {
                    if index.is_false() { NodeIndex::TRUE }
                    else if index.is_true() { NodeIndex::FALSE }
                    else {
                        let node = self.node(index);
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
                    index = self.add_node_if_not_present(Node{
                        variable : VariableIndex(i),
                        lo: index,
                        hi,
                    });
                }
                index
            };
            cache.insert(key,res);
            res
        }
    }

    /// Create a node for a zdd (or find existing) for variable variable with lo and hi choices, and store it in the provided cache.
    /// Uniqueifies - sees if the hi and lo are same, in which case just produce lo, and looks for existing nodes.
    fn create_node_bdd<K:Eq+Hash>(&mut self,lo:NodeIndex,hi:NodeIndex,variable:VariableIndex,key:K,cache:&mut HashMap<K,NodeIndex>) -> NodeIndex {
        let res = if lo==hi { lo } else {
            self.add_node_if_not_present(Node{variable,lo,hi})
        };
        cache.insert(key,res);
        res
    }

    /// Make a node representing index1 and index2 (and in the logical sense, a.k.a. ∧ or &&)
    fn and_bdd(&mut self,index1:NodeIndex,index2:NodeIndex,cache : &mut HashMap<(NodeIndex,NodeIndex),NodeIndex>) -> NodeIndex {
        if index1.is_false() || index2.is_false() { NodeIndex::FALSE }
        else if index1.is_true() { index2 }
        else if index2.is_true() { index1 }
        else if index1==index2 { index1 }
        else {
            let key = if index1.0 < index2.0 {(index1,index2)} else {(index2,index1)};
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node(index1);
                let node2 = self.node(index2);
                let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1,index1)};
                let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2,index2)};
                let lo = self.and_bdd(lo1,lo2,cache);
                let hi = self.and_bdd(hi1,hi2,cache);
                self.create_node_bdd(lo,hi,if node1.variable <= node2.variable { node1.variable } else {node2.variable},key,cache)
            }
        }
    }

    /// Make a node representing index1 and index2 (and in the logical sense, a.k.a. ∧ or &&)
    fn or_bdd(&mut self,index1:NodeIndex,index2:NodeIndex,cache : &mut HashMap<(NodeIndex,NodeIndex),NodeIndex>) -> NodeIndex {
        if index1.is_true() || index2.is_true() { NodeIndex::TRUE }
        else if index1.is_false() { index2 }
        else if index2.is_false() { index1 }
        else if index1==index2 { index1 }
        else {
            let key = if index1.0 < index2.0 {(index1,index2)} else {(index2,index1)};
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node(index1);
                let node2 = self.node(index2);
                let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1,index1)};
                let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2,index2)};
                let lo = self.or_bdd(lo1,lo2,cache);
                let hi = self.or_bdd(hi1,hi2,cache);
                self.create_node_bdd(lo,hi,if node1.variable <= node2.variable { node1.variable } else {node2.variable},key,cache)
            }
        }
    }


    /// compute index as a ZDD anded with NodeIndex::TRUE, which means take all lo branches on index1.
    fn and_zdd_true(&mut self,index:NodeIndex) -> NodeIndex {
        let mut index = index;
        while !index.is_sink() {
            index = self.node(index).lo;
        }
        index
    }

    /// Create a node for a zdd (or find existing) for variable variable with lo and hi choices, and store it in the provided cache.
    /// Uniqueifies - sees if the hi is false, in which case just produce lo, and looks for existing nodes.
    fn create_node_zdd<K:Eq+Hash>(&mut self,lo:NodeIndex,hi:NodeIndex,variable:VariableIndex,key:K,cache:&mut HashMap<K,NodeIndex>) -> NodeIndex {
        let res = if hi.is_false() { lo } else {
            self.add_node_if_not_present(Node{variable,lo,hi})
        };
        cache.insert(key,res);
        res
    }
    /// Make a node representing index1 and index2 (and in the logical sense, a.k.a. ∧ or &&)
    fn and_zdd(&mut self,index1:NodeIndex,index2:NodeIndex,cache : &mut HashMap<(NodeIndex,NodeIndex),NodeIndex>) -> NodeIndex {
        if index1.is_false() || index2.is_false() { NodeIndex::FALSE }
        else if index1.is_true() { self.and_zdd_true(index2) }
        else if index2.is_true() { self.and_zdd_true(index1) }
        else if index1==index2 { index1 }
        else {
            let key = if index1.0 < index2.0 {(index1,index2)} else {(index2,index1)};
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node(index1);
                let node2 = self.node(index2);
                let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1,NodeIndex::FALSE)};
                let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2,NodeIndex::FALSE)};
                let lo = self.and_zdd(lo1,lo2,cache);
                let hi = self.and_zdd(hi1,hi2,cache);
                self.create_node_zdd(lo,hi,if node1.variable <= node2.variable { node1.variable } else {node2.variable},key,cache)
            }
        }
    }

    /// compute index as a ZDD ored with NodeIndex::TRUE, which means or all lo branches on index1.
    fn or_zdd_true(&mut self,index:NodeIndex) -> NodeIndex {
        if index.is_sink() { NodeIndex::TRUE }
        else {
            let node=self.node(index);
            let lo = self.or_zdd_true(node.lo);
            if lo==node.lo { index } // this check is just a (possibly) common path optimization to avoid hash lookup.
            else {
                self.add_node_if_not_present(Node{
                    variable: node.variable,
                    lo,
                    hi:node.hi,
                })
            }
        }
    }


    /// Make a node representing index1 and index2 (and in the logical sense, a.k.a. ∧ or &&)
    fn or_zdd(&mut self,index1:NodeIndex,index2:NodeIndex,cache : &mut HashMap<(NodeIndex,NodeIndex),NodeIndex>) -> NodeIndex {
        if index1.is_false() { index2 }
        else if index2.is_false() { index1 }
        else if index1.is_true() { self.or_zdd_true(index2) }
        else if index2.is_true() { self.or_zdd_true(index1) }
        else if index1==index2 { index1 }
        else {
            let key = if index1.0 < index2.0 {(index1,index2)} else {(index2,index1)};
            if let Some(&res) = cache.get(&key) { res }
            else {
                let node1 = self.node(index1);
                let node2 = self.node(index2);
                let (lo1,hi1) = if node1.variable <= node2.variable { (node1.lo,node1.hi)} else {(index1,NodeIndex::FALSE)};
                let (lo2,hi2) = if node2.variable <= node1.variable { (node2.lo,node2.hi)} else {(index2,NodeIndex::FALSE)};
                let lo = self.or_zdd(lo1,lo2,cache);
                let hi = self.or_zdd(hi1,hi2,cache);
                self.create_node_zdd(lo,hi,if node1.variable <= node2.variable { node1.variable } else {node2.variable},key,cache)
            }
        }
    }



    /// Create generating functions for nodes 0 inclusive to length exclusive.
    /// This is easy because of the topological sort.
    /// Return an array such that res[node] = the variable used at the time and the generating function.
    fn all_number_solutions<G:GeneratingFunction,const BDD:bool>(&self,length:usize,num_variables:u16) -> Vec<G> {
        let mut res = Vec::new();
        res.push(G::zero());
        res.push(G::one());
        for i in 2..length {
            let node = self.node(NodeIndex(i as u32));
            let next_variable = VariableIndex(node.variable.0+1);
            //println!("Computing {} lo={} hi={} variable={}",i,node.lo,node.hi,node.variable);
            let lo_g = res[node.lo.0 as usize].clone();
            let lo_level = if node.lo.is_sink() { VariableIndex(num_variables) } else { self.node(node.lo).variable };
            //println!("   lo_g={:?}, lo_level={}",lo_g,lo_level);
            let lo = if BDD {lo_g.deal_with_variable_range_being_indeterminate(next_variable,lo_level)} else {lo_g};
            let lo = lo.variable_not_set(node.variable);
            let hi_g = res[node.hi.0 as usize].clone();
            let hi_level = if node.hi.is_sink() { VariableIndex(num_variables) } else { self.node(node.hi).variable };
            //println!("   hi_g={:?}, hi_level={}",hi_g,hi_level);
            let hi = if BDD {hi_g.deal_with_variable_range_being_indeterminate(next_variable,hi_level)} else {hi_g};
            let hi = hi.variable_set(node.variable);
            //println!(" GF lo = {:?},   GF hi = {:?}",lo,hi);
            res.push(lo.add(hi));
        }
        //println!("{:?}",res);
        res
    }

    fn number_solutions<G:GeneratingFunction,const BDD:bool>(&self,index:NodeIndex,num_variables:u16) -> G {
        let work = self.all_number_solutions::<G,BDD>(index.0 as usize+1,num_variables);
        let found = work[index.0 as usize].clone();
        if BDD {
            let level = if index.is_sink() { VariableIndex(num_variables) } else { self.node(index).variable };
            found.deal_with_variable_range_being_indeterminate(VariableIndex(0),level)
        } else { found }
    }

    fn number_solutions_bdd<G:GeneratingFunction>(&self,index:NodeIndex,num_variables:u16) -> G { self.number_solutions::<G,true>(index,num_variables) }
    fn number_solutions_zdd<G:GeneratingFunction>(&self,index:NodeIndex,num_variables:u16) -> G { self.number_solutions::<G,false>(index,num_variables) }

    /// Do garbage collection. Provide the items one wants to keep, and get rid of anything not in the transitive dependencies of keep.
    /// Returns a renamer from old nodes to new nodes.
    fn gc(&mut self,keep:impl IntoIterator<Item=NodeIndex>) -> NodeRenaming;

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

    /// Do garbage collection. Provide the items one wants to keep, and get rid of anything not in the transitive dependencies of keep.
    /// Returns a renamer such that v[old_node.0] is what v maps in to. If nothing, then map into NodeIndex::JUNK.
    fn gc(&mut self,keep:impl IntoIterator<Item=NodeIndex>) -> NodeRenaming {
        let mut map = vec![NodeIndex::JUNK;self.len()+2];
        // initially map is used to detect what to keep.
        const KEEP: NodeIndex = NodeIndex::TRUE;
        fn do_keep(nodes:&Vec<Node>,map:&mut Vec<NodeIndex>,n:NodeIndex) {
            if map[n.0 as usize]!=KEEP {
                map[n.0 as usize]=KEEP;
                let node = nodes[(n.0-2) as usize];
                do_keep(nodes,map,node.lo);
                do_keep(nodes,map,node.hi);
            }
        }
        map[0]=KEEP; // FALSE
        map[1]=KEEP; // TRUE
        for k in keep.into_iter() {
            do_keep(&self.nodes,&mut map,k);
        }
        map[0]=NodeIndex::FALSE;
        map[1]=NodeIndex::TRUE;
        let mut len:usize = 0;
        for i in 2..map.len() {
            let into = map[i];
            if into==KEEP {
                map[i]=NodeIndex((len+2) as u32);
                let old_node = self.nodes[i-2];
                self.nodes[len]=Node {
                    variable: old_node.variable,
                    lo: map[old_node.lo.0 as usize],
                    hi: map[old_node.hi.0 as usize],
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

    fn gc(&mut self, keep: impl IntoIterator<Item=NodeIndex>) -> NodeRenaming {
        let map = self.nodes.gc(keep);
        self.node_to_index.clear();
        for (i,node) in self.nodes.nodes.iter().enumerate() {
            self.node_to_index.insert(*node,NodeIndex((i+2) as u32));
        }
        map
    }
}
