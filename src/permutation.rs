/// Copyright 2022-2025 Andrew Conway. All rights reserved. See README.md for licensing. 

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Index;
use std::str::FromStr;

pub type PermutedItem = u32;

/// A permutation π = (π(1),π(2),…,π(n)) on n elements
/// Note that indices are 1 based to match general convention for permutations!
#[derive(Clone,Debug,Eq, PartialEq)]
pub struct Permutation {
    /// The representation permutation of the integers 1..n
    pub sequence : Vec<PermutedItem>
}

impl Index<PermutedItem> for Permutation {
    type Output = PermutedItem;
    fn index(&self, index: PermutedItem) -> &Self::Output { &self.sequence[(index-1) as usize] }
}

impl Permutation {
    /// The number of elements being permuted.
    /// /// # Example
    /// ```
    /// use xdd::permutation::Permutation;
    /// let x = Permutation { sequence: vec![4,5,2,1,3] };
    /// assert_eq!(5,x.n());
    /// ```
    pub fn n(&self) -> usize { self.sequence.len() }

    /// Apply one permutation to the other.
    /// The composition of two permutations π and σ is π·σ = ( σ(π(1)),…,σ(π(n)) )
    /// # Example
    /// ```
    /// use xdd::permutation::Permutation;
    /// let x = Permutation { sequence: vec![4,5,2,1,3] };
    /// let y = Permutation { sequence: vec![4,1,3,5,2] };
    /// assert_eq!(vec![5,2,1,4,3],x.compose(&y).sequence);
    /// ```
    pub fn compose(&self,other:&Permutation) -> Permutation {
        Permutation{
            sequence : self.sequence.iter().map(|&i|other[i]).collect()
        }
    }

    /// Check to see if this is a valid permutation.
    ///
    /// That means it must contain each number 1..n once.
    /// # Example
    /// ```
    /// use xdd::permutation::Permutation;
    /// let x = Permutation { sequence: vec![4,5,2,1,3] };
    /// assert!(x.is_permutation());
    /// let y = Permutation { sequence: vec![4,1,3,5,1] };
    /// assert_eq!(false,y.is_permutation()); // has two 1s.
    /// ```
    pub fn is_permutation(&self) -> bool {
        let n = self.n();
        let mut num = vec![0;n];
        for &e in &self.sequence {
            if e==0 || e as usize>n { return false; }
            num[e as usize-1]+=1;
        }
        num.iter().all(|&e|e==1)
    }
}

/// A permutation can be expressed two ways:
/// * As a comma separated set n of unique integers 1..n, such as `1,3,4,2`
/// * The same, except without the commas such as `1324`. Only works for n<10.
impl FromStr for Permutation {
    type Err = ParsePermutationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let strings: Vec<&str> = if s.contains(',') {
            s.split(",").collect()
        } else {
            let mut v:Vec<&str> = s.split("").skip(1).collect();// get rid of the leading "" that splitting on "" causes.
            v.pop(); // get rid of the trailing "" that splitting on "" causes.
            v
        };
        let mut sequence : Vec<PermutedItem> = vec![];
        for s in strings {
            sequence.push(s.parse().map_err(|_|ParsePermutationError::NumberFormat(s.to_string()))?);
        }
        let res = Permutation{sequence};
        if res.is_permutation() { Ok(res) } else { Err(ParsePermutationError::NotPermutation) }
    }
}

#[derive(Clone,Debug)]
pub enum ParsePermutationError {
    NumberFormat(String),
    NotPermutation,
}

impl Error for ParsePermutationError { }

impl Display for ParsePermutationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsePermutationError::NumberFormat(s) => write!(f,"Could not interpret '{}' as a number",s),
            ParsePermutationError::NotPermutation => write!(f,"Not a permutation"),
        }
    }
}
