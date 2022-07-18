use std::fmt::Debug;
use std::ops::Mul;
use num::Integer;
use crate::{NoMultiplicity, VariableIndex};

/// A Generating Function is some aggregate of the variables. This could be:
///  * An integer, being the number of solutions.
pub trait GeneratingFunction : Sized + Clone + Debug {
    /// The base value for NodeIndex::FALSE
    fn zero() -> Self;
    /// The base value for NodeIndex::TRUE
    fn one() -> Self;
    /// Add two GFs.
    fn add(self,other:Self) -> Self;
    /// Effect of having this variable set. For a simple count, nothing. For a generating function, shift one left.
    fn variable_set(self,variable:VariableIndex) -> Self;
    /// Effect of having this variable set. Generally nothing
    fn variable_not_set(self,_variable:VariableIndex) -> Self {self}
    /// Effect of having this variable either set or not set.
    fn deal_with_variable_being_indeterminate(self,variable:VariableIndex) -> Self {
        let v1 = self.clone().variable_set(variable);
        let v2 = self.variable_not_set(variable);
        v1.add(v2)
    }
    /// Effect of variables inclusive_start..exclusive_end being indeterminate
    fn deal_with_variable_range_being_indeterminate(self,inclusive_start:VariableIndex,exclusive_end:VariableIndex) -> Self {
        let mut res = self;
        for v in (inclusive_start.0 .. exclusive_end.0).rev() {
            res = res.deal_with_variable_being_indeterminate(VariableIndex(v));
        }
        res
    }
}

/// A generating function that can also multiply itself by some constant.
pub trait GeneratingFunctionWithMultiplicity<M> : GeneratingFunction {
    /// multiply self by M.
    fn multiply(self,multiple:M) -> Self;
}

impl <G:GeneratingFunction> GeneratingFunctionWithMultiplicity<NoMultiplicity> for G {
    fn multiply(self, _multiple: NoMultiplicity) -> Self { self }
}

/// A simple generating function that separates counts by the number of variables set.
impl GeneratingFunction for u64 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn add(self, other: Self) -> Self { self+other }
    fn variable_set(self, _variable: VariableIndex) -> Self { self }
}

/// A simple generating function that separates counts by the number of variables set.
impl GeneratingFunction for u128 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn add(self, other: Self) -> Self { self+other }
    fn variable_set(self, _variable: VariableIndex) -> Self { self }
}

impl <G:GeneratingFunction,I:Into<G>+Ord> GeneratingFunctionWithMultiplicity<I> for G // The requirement on Ord is to prevent a possible clash with NoMultiplicity.
    where G:Mul<G,Output=G>,
{

    fn multiply(self, multiple: I) -> Self {
        self*multiple.into()
    }
}



#[derive(Clone,Eq, PartialEq,Debug)]
pub struct SingleVariableGeneratingFunction(pub Vec<u64>);

impl GeneratingFunction for SingleVariableGeneratingFunction {
    fn zero() -> Self {
        SingleVariableGeneratingFunction(vec![])
    }

    fn one() -> Self {
        SingleVariableGeneratingFunction(vec![1])
    }

    fn add(self, other: Self) -> Self {
        let SingleVariableGeneratingFunction(mut res) = self;
        let SingleVariableGeneratingFunction(other) = other;
        for i in 0..other.len() {
            let v = other[i];
            if res.len()>i { res[i]+=v } else { res.push(v) }
        }
        SingleVariableGeneratingFunction(res)
    }

    /// shift up by one
    fn variable_set(self, _variable: VariableIndex) -> Self {
        let SingleVariableGeneratingFunction(mut res) = self;
        if res.len()>0 { res.insert(0,0); }
        SingleVariableGeneratingFunction(res)
    }
}

impl <M:Copy+Integer+TryInto<u64>> GeneratingFunctionWithMultiplicity<M> for SingleVariableGeneratingFunction {
    fn multiply(self, multiple: M) -> Self {
        let mut res = self;
        let multiple : u64 = multiple.try_into().map_err(|_|()).expect("Could not convert multiplicity into generating function element type");
        for i in 0..res.0.len() {
            res.0[i]*=multiple;
        }
        res
    }
}

#[derive(Clone,Eq, PartialEq,Debug)]
/// a generating function with a fixed maximum length
pub struct SingleVariableGeneratingFunctionFixedLength<const L:usize>(pub Vec<u64>);

impl <const L:usize> GeneratingFunction for SingleVariableGeneratingFunctionFixedLength<L> {
    fn zero() -> Self {
        SingleVariableGeneratingFunctionFixedLength::<L>(vec![])
    }

    fn one() -> Self {
        SingleVariableGeneratingFunctionFixedLength::<L>(vec![1])
    }

    fn add(self, other: Self) -> Self {
        let SingleVariableGeneratingFunctionFixedLength(mut res) = self;
        let SingleVariableGeneratingFunctionFixedLength(other) = other;
        for i in 0..other.len() {
            let v = other[i];
            if res.len()>i { res[i]+=v } else { res.push(v) }
        }
        SingleVariableGeneratingFunctionFixedLength::<L>(res)
    }

    /// shift up by one
    fn variable_set(self, _variable: VariableIndex) -> Self {
        let SingleVariableGeneratingFunctionFixedLength(mut res) = self;
        if res.len()>0 { res.insert(0,0); }
        if res.len()>L { res.pop(); }
        SingleVariableGeneratingFunctionFixedLength::<L>(res)
    }
}
