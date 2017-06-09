use num_bigint::BigInt;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct RpNumber {
    whole: BigInt,
    fraction: Option<BigInt>,
    exponent: Option<u32>,
}

impl fmt::Display for RpNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.whole)?;

        if let Some(fraction) = self.fraction {
            write!(f, ".{}", fraction)?;
        }

        if let Some(exponent) = self.exponent {
            write!(f, "e{}", exponent)?;
        }

        Ok(())
    }
}
