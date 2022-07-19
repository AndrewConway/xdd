//! Compute Pattern avoiding permutations, as described in section 4.4 of
//!
//! [Yuma Inoue, Studies on Permutation Set Manipulation based on Decision Diagrams,
//! Doctor of Info. Sciences thesis, Hokkaido University, (2017).](https://eprints.lib.hokudai.ac.jp/dspace/handle/2115/65366?locale=en&lang=en)
//!
//!


use xdd::generating_function::GeneratingFunction;
use xdd::{NodeIndexWithMultiplicity, NoMultiplicity};
use xdd::permutation_diagrams::{LeftRotation, PermutationDecisionDiagramFactory, PermutedItem};

fn factorial(n:u64) -> u64 {
    let mut res = 1;
    for i in 2..=n {
        res*=i;
    }
    res
}

fn n_c_r(n:u64,r:u64) -> u64 {
    factorial(n)/factorial(r)/factorial(n-r)
}

type NodeIndex = NodeIndexWithMultiplicity<u32,NoMultiplicity>;

/// C is a set of permutations that distribute the k-prefix of a permutation π over the n choose k possible positions in π.
/// Algorithm 4.4.3 from YI's thesis.
fn compute_rot_pidd_c(factory:&mut PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>,n:PermutedItem,k:PermutedItem) -> NodeIndex {
    let mut p_j_minus_1 = vec![NodeIndex::TRUE;n as usize];
    for j in 1..=k {
        let mut p_j = vec![NodeIndex::FALSE;j as usize]; // P_{i,j} = p_j[i].
        for i in j..=n {
            let term2 = factory.left_rot(p_j_minus_1[(i-1) as usize],j,i);
            let p_ij = factory.or(p_j[(i-1) as usize],term2);
            p_j.push(p_ij);
        }
        p_j_minus_1=p_j;
    }
    p_j_minus_1[n as usize]
}

/// A is a set of all permutations whose k-prefix is ordered in increasing order.
/// Algorithm 4.4.4 from YI's thesis.
fn compute_rot_pidd_a(factory:&mut PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>,n:PermutedItem,k:PermutedItem) -> NodeIndex {
    let mut i_i_minus_1 = NodeIndexWithMultiplicity::TRUE;
    for i in k+1..=n {
        let mut i_i = i_i_minus_1;
        for j in 1..i {
            let rot = factory.left_rot(i_i_minus_1,j,i);
            i_i = factory.or(i_i,rot);
        }
        i_i_minus_1=i_i;
    }
    i_i_minus_1
}

#[test]
fn test_compute_rot_pidd_c() {
    fn test(n:PermutedItem,k:PermutedItem) {
        let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>::new(n as u16);
        let c = compute_rot_pidd_c(&mut factory,n,k);
        let renamer = factory.gc([c]);
        let c = renamer.rename(c).unwrap();
        let solutions : u64 = factory.number_solutions(c);
        assert_eq!(solutions,n_c_r(n as u64,k as u64));
        assert_eq!(factory.len(),((n-k)*k) as usize);
    }
    test(1,1);
    test(8,1);
    test(8,3);
    test(8,5);
}

#[test]
fn test_compute_rot_pidd_a() {
    fn test(n:PermutedItem,k:PermutedItem) {
        let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>::new(n as u16);
        let a = compute_rot_pidd_a(&mut factory,n,k);
        let renamer = factory.gc([a]);
        let a = renamer.rename(a).unwrap();
        let solutions : u64 = factory.number_solutions(a);
        assert_eq!(solutions,n_c_r(n as u64,k as u64)*factorial((n-k) as u64));
        assert_eq!(factory.len(),((n-k)*(n+k-1)/2) as usize);
    }
    test(1,1);
    test(8,1);
    test(8,3);
    test(8,5);
}

fn get_number_of_permutations_containing_a_given_pattern<G: GeneratingFunction>(n:PermutedItem,permutation:&[PermutedItem]) -> G {
    if n < permutation.len() as PermutedItem { return G::zero(); }
    let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,NoMultiplicity>::new(n as u16);
    let k = permutation.len() as PermutedItem;
    let a = compute_rot_pidd_a(&mut factory,n,k);
    let b = factory.compute_for_single_permutation(permutation);
    let c = compute_rot_pidd_c(&mut factory,n,k);
    let b_cross_a = factory.compose(b,a);
    let c_cross_b_cross_a = factory.compose(c,b_cross_a);
    println!("Terms created : {}",factory.len());
    factory.number_solutions(c_cross_b_cross_a)
}

fn num_avoiding_1324(n:PermutedItem) -> u64 {
    factorial(n as u64)-get_number_of_permutations_containing_a_given_pattern::<u64>(n,&[1,3,2,4])
}

#[test]
fn test_avoid1324() { // See https://oeis.org/A061552
    assert_eq!(1,num_avoiding_1324(1));
    assert_eq!(2,num_avoiding_1324(2));
    assert_eq!(6,num_avoiding_1324(3));
    assert_eq!(23,num_avoiding_1324(4));
    assert_eq!(103,num_avoiding_1324(5));
    assert_eq!(513,num_avoiding_1324(6));
    assert_eq!(2762,num_avoiding_1324(7));
    assert_eq!(15793,num_avoiding_1324(8));
    assert_eq!(94776,num_avoiding_1324(9));
    assert_eq!(591950,num_avoiding_1324(10));
    assert_eq!(3824112,num_avoiding_1324(11));
    assert_eq!(25431452,num_avoiding_1324(12));
    assert_eq!(173453058,num_avoiding_1324(13));
    assert_eq!(1209639642,num_avoiding_1324(14));
}
