/// Copyright 2022-2025 Andrew Conway. All rights reserved. See README.md for licensing. 

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::RangeInclusive;
use std::str::FromStr;


pub fn parse_range_inclusive<T:FromStr+Clone>(s:&str) -> Result<RangeInclusive<T>,ParseNumericRangeError<T::Err>> {
    Ok(if let Some((low,high)) = s.split_once("...") {
        let low = low.trim();
        let high = high.trim();
        let min = T::from_str(low)?;
        let max = T::from_str(high)?;
        RangeInclusive::new(min,max)
    } else { // must just be a number
        let n = T::from_str(s)?;
        RangeInclusive::new(n.clone(),n)
    })
}


pub enum ParseNumericRangeError<T> {
    InvalidFormat,
    Other(T)
}

impl<T:Display> Debug for ParseNumericRangeError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self)
    }
}

impl <T:Display> Error for ParseNumericRangeError<T> {

}

impl <T:Display> Display for ParseNumericRangeError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseNumericRangeError::InvalidFormat => write!(f,"Range should be a single number, or two numbers separated by ..."),
            ParseNumericRangeError::Other(t) => write!(f,"Range should be a single number, or two numbers separated by ...\nHad error {} interpreting a number",t)
        }
    }
}

impl <T> From<T> for ParseNumericRangeError<T> {
    fn from(t: T) -> Self {
        ParseNumericRangeError::Other(t)
    }
}
