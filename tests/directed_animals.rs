//! Count directed animals, comparing the XDD approach to a traditional recursive memoization approach.
//!
//! Each variable in the XDD corresponds to the presence or absence of a given site.
//! To get up to length n, need the triangular lattice with coordinates (x,y) such that x>=0, y>=0, and x+y<n
//! We need to number sites. Do a triangular counting. Diagonal d=x+y has numbers starting from d*(d+1)/2.
//! Site (x,y) has number x+d*(d+1)/2, where d=x+y.
//!
//! The site constraint means that for each site (x,y) other than (0,0),
//! at least one of the prior sites (x-1,y) and (x,y-1) must be present as long as it is present.
//! That is, the function we need to compute is the intersection (logical and) of
//! one term for each site other than the origin being (x-1,y) | (x,y-1) | !(x,y)


use std::collections::HashMap;
use xdd::{BDDFactory, DecisionDiagramFactory, NodeIndex, NoMultiplicity, VariableIndex, ZDDFactory};
use xdd::generating_function::{SingleVariableGeneratingFunctionFixedLength};

#[test]
fn count_memoization() {
    fn count_work(cache:&mut HashMap<(u64,u64,u32),u64>,sig_this_row:u64,sig_next_row:u64,n:u32) -> u64 {
        if n==0 { 1 }
        else if sig_this_row==0 {
            if sig_next_row == 0 { 0 } else { count_work(cache, sig_next_row, 0, n) }
        } else {
            let key = (sig_this_row,sig_next_row,n);
            cache.get(&key).cloned().unwrap_or_else(||{
                let next_choice = sig_this_row&(1+!sig_this_row); // single bit, can't be zero.
                let removed_choice = sig_this_row& !next_choice; // removed but.
                let choice0 = count_work(cache,removed_choice,sig_next_row,n);
                let choice1 = count_work(cache,removed_choice,sig_next_row|(3*next_choice),n-1);
                let res = choice0+choice1;
                cache.insert(key,res);
                res
            })
        }
    }
    let mut cache = HashMap::new();
    for i in 0..21 {
        println!("{} : {}",i,count_work(&mut cache,1,0,i));
    }
    println!("Used {} cache entries",cache.len());
}

fn variable_number(x:u16,y:u16) -> VariableIndex {
    let d = x+y;
    VariableIndex(x+(d*(d+1))/2)
}




/// Count using a decision diagram
fn count_xdd<F: DecisionDiagramFactory<u32,NoMultiplicity>>() {
    let terms_wanted = 13;
    let num_variables = variable_number(0,terms_wanted).0;
    let mut factory = F::new(num_variables);
    let mut function : Option<NodeIndex<u32,NoMultiplicity>> = None;
    for x in 0..terms_wanted {
        for y in 0..(terms_wanted-x) {
            // println!("Working on node ({},{})",x,y);
            // std::io::stdout().flush().unwrap();
            if x>0 || y>0 {
                let variable_here = factory.single_variable(variable_number(x,y));
                let not_variable_here = factory.not(variable_here);
                let left = if x>0 { factory.single_variable(variable_number(x-1,y)) } else { NodeIndex::FALSE };
                let below = if y>0 { factory.single_variable(variable_number(x,y-1)) } else { NodeIndex::FALSE };
                let prior = factory.or(left,below);
                let term = factory.or(prior,not_variable_here);
                function = Some(if let Some(f) = function {factory.and(term,f)} else {term});
            }
        }
    }
    //factory.print(function.unwrap());
    let result = factory.number_solutions::<SingleVariableGeneratingFunctionFixedLength::<16>>(function.unwrap());
    println!("{:?}",result);
    assert_eq!(1,result.0[0]);
    assert_eq!(1,result.0[1]);
    assert_eq!(2,result.0[2]);
    assert_eq!(5,result.0[3]);
    assert_eq!(13,result.0[4]);
    assert_eq!(35,result.0[5]);
    assert_eq!(96,result.0[6]);
    assert_eq!(267,result.0[7]);
    assert_eq!(750,result.0[8]);
    assert_eq!(2123,result.0[9]);
    assert_eq!(6046,result.0[10]);
    assert_eq!(17303,result.0[11]);
    assert_eq!(49721,result.0[12]);
    assert_eq!(143365,result.0[13]);
    //assert_eq!(414584,result.0[14]);
    //assert_eq!(1201917,result.0[15]);
    let original_size = factory.len();
    factory.gc([function.unwrap()]);
    println!("Used {} nodes ({} after GC)",original_size,factory.len());
}

#[test]
fn count_bdd() {
    count_xdd::<BDDFactory<u32,NoMultiplicity>>()
}

#[test]
fn count_zdd() {
    count_xdd::<ZDDFactory<u32,NoMultiplicity>>()
}
