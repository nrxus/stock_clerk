use serde::{Deserialize, Deserializer};

use std::{
    fmt::{self, Display, Formatter},
    ops::{Mul, Sub},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Dollars {
    whole: u32,
    cents: u8,
}

impl Dollars {
    pub fn new(amount: f64) -> Self {
        Dollars {
            whole: amount as u32,
            cents: (amount.fract() * 100.0).round() as u8,
        }
    }
}

impl<'de> Deserialize<'de> for Dollars {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        f64::deserialize(deserializer).map(Dollars::new)
    }
}

impl Display for Dollars {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "${}.{}", self.whole, self.cents)
    }
}

impl Sub for Dollars {
    type Output = Dollars;

    fn sub(self, subtrahend: Dollars) -> Dollars {
        let mut whole = self.whole - subtrahend.whole;
        let cents = self.cents as i8 - subtrahend.cents as i8;

        let cents = if cents < 0 {
            whole -= 1;
            (cents + 100)
        } else {
            cents
        } as u8;

        Dollars { whole, cents }
    }
}

impl Mul<u16> for Dollars {
    type Output = Dollars;

    fn mul(self, multiplier: u16) -> Dollars {
        let multiplier = multiplier as u32;
        let cents = multiplier * self.cents as u32;
        let whole = multiplier * self.whole;
        Dollars::from_parts(whole, cents)
    }
}

impl Mul<f64> for Dollars {
    type Output = Dollars;

    fn mul(self, multiplier: f64) -> Dollars {
        let cents = (multiplier * self.cents as f64) as u32;
        let whole = (multiplier * self.whole as f64) as u32;
        Dollars::from_parts(whole, cents)
    }
}

impl Dollars {
    fn from_parts(whole: u32, cents: u32) -> Self {
        Dollars {
            whole: whole + cents / 100,
            cents: (cents % 100) as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplies() {
        let count = 2343;
        let amount = Dollars::new(23.45);
        let actual = amount * count;
        assert_eq!(Dollars::new(54943.35), actual);
    }
}
