use std::ops::RangeInclusive;
use clap::Parser;
use xdd::generating_function::GeneratingFunctionSplitByMultiplicity;
use xdd::permutation::Permutation;
use xdd::permutation_diagrams::{factorial, LeftRotation, n_choose_r, PermutationDecisionDiagramFactory};
use std::str::FromStr;

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
    /// The pattern to count instances of.
    #[clap(parse(try_from_str = xdd::permutation::Permutation::from_str))]
    pattern : Permutation,
}


fn main() {
    let args = Args::parse();
    let mut triangle : OEISTriangle<u128> = Default::default();

    let pattern_len = args.pattern.sequence.len() as u32;
    for n in args.range {
        let mut factory = PermutationDecisionDiagramFactory::<LeftRotation,u32,u32>::new(n as u16);
        let containing = factory.permutations_containing_a_given_pattern(&args.pattern.sequence);
        println!("\nTerms created {}",factory.len());
        let mut num_containing : GeneratingFunctionSplitByMultiplicity::<u128> = factory.number_solutions(containing);
        let zero = factorial::<u128>(n as u32)-num_containing.0.iter().fold(0,|a,b|a + *b); // the number of elements that avoid the pattern.
        num_containing.0.insert(0,zero);
        print!("{}",n);
        for &v in &num_containing.0 {
            print!("\t{}",v);
        }
        println!();
        // make a format more suitable for OEIS.
        if n>pattern_len { num_containing.0.resize(n_choose_r::<usize>(n,pattern_len)+1,0); }
        triangle.push(num_containing.0);
        triangle.print_as_single_line();
        triangle.print_as_triangle();
    }
}

#[derive(Default)]
struct OEISTriangle<T> {
    triangle : Vec<Vec<T>>
}

impl <T:ToString+Clone> OEISTriangle<T> {
    pub fn print_as_single_line(&self) {
        let line = self.triangle.iter().flatten().map(|v|v.to_string()).collect::<Vec<_>>().join(",");
        println!("{}",line);
    }
    pub fn print_as_triangle(&self) {
        println!("Triangle begins:");
        for row in &self.triangle {
            println!("{}",row.iter().map(|v|v.to_string()).collect::<Vec<_>>().join(" "));
        }
    }
    pub fn push(&mut self,row : Vec<T>) { self.triangle.push(row); }
}
