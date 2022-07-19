use std::ops::RangeInclusive;
use clap::Parser;
use xdd::generating_function::GeneratingFunctionSplitByMultiplicity;
use xdd::permutation_diagrams::{factorial, LeftRotation, PermutationDecisionDiagramFactory};

/// Pattern avoiding permutations
///
/// Compute Pattern avoiding permutations, using an algorithm based on that in Yuma Inoue's thesis.
/// [Yuma Inoue, Studies on Permutation Set Manipulation based on Decision Diagrams,
/// Doctor of Info. Sciences thesis, Hokkaido University, (2017).](https://eprints.lib.hokudai.ac.jp/dspace/handle/2115/65366?locale=en&lang=en)
///
/// Computes, for each value of n, the number of permutations with k instances of a pattern for each k.
/// See https://oeis.org/A263771 as an example.
#[derive(Parser, Debug)]
#[clap(author="Andrew Conway", version, about, long_about = None)]
struct Args {
    /// The lengths of permutations to iterate for.
    #[clap(parse(try_from_str = xdd::util::parse_range_inclusive))]
    range : RangeInclusive<u32>,
}


fn main() {
    let args = Args::parse();
    let pattern = [1,3,2,4];

    for n in args.range {
        let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,u32>::new(n as u16);
        let containing = factory.permutations_containing_a_given_pattern(&pattern);
        println!("Terms created {}",factory.len());
        let num_containing : GeneratingFunctionSplitByMultiplicity::<u128> = factory.number_solutions(containing);
        let zero = factorial::<u128>(n as u32)-num_containing.0.iter().fold(0,|a,b|a + *b);
        print!("{}\t{}",n,zero);
        for v in num_containing.0 {
            print!("\t{}",v);
        }
        println!();
    }
}