use std::fmt::Debug;
use crate::VariableIndex;

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


impl GeneratingFunction for u64 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
    fn add(self, other: Self) -> Self { self+other }
    fn variable_set(self, _variable: VariableIndex) -> Self { self }
}