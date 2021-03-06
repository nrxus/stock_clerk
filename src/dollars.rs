use serde::{Deserialize, Deserializer};

use std::{
    fmt::{self, Display, Formatter},
    iter,
    ops::{Add, AddAssign, Div, Mul, Sub},
    u32,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Default)]
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

    pub fn max() -> Self {
        Dollars {
            whole: u32::MAX,
            cents: 99,
        }
    }

    pub fn zero() -> Self {
        Dollars::default()
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
        f.pad(&format!("${}.{:02}", self.whole, self.cents))
    }
}

impl iter::Sum for Dollars {
    fn sum<I: Iterator<Item = Dollars>>(iter: I) -> Dollars {
        iter.fold(Dollars::zero(), |a, b| a + b)
    }
}

impl Sub for Dollars {
    type Output = Dollars;

    fn sub(self, subtrahend: Dollars) -> Dollars {
        let whole = self.whole - subtrahend.whole;
        let cents = self.cents as i8 - subtrahend.cents as i8;

        if cents < 0 {
            Dollars {
                whole: whole - 1,
                cents: (cents + 100) as u8,
            }
        } else {
            Dollars {
                whole,
                cents: cents as u8,
            }
        }
    }
}

impl Add for Dollars {
    type Output = Dollars;

    fn add(self, adder: Dollars) -> Dollars {
        let whole = self.whole + adder.whole;
        let cents = self.cents + adder.cents;
        if cents > 99 {
            Dollars {
                whole: whole + 1,
                cents: cents - 100,
            }
        } else {
            Dollars { whole, cents }
        }
    }
}

impl AddAssign for Dollars {
    fn add_assign(&mut self, adder: Dollars) {
        *self = *self + adder
    }
}

impl Div for Dollars {
    type Output = f64;

    fn div(self, divisor: Dollars) -> f64 {
        self.as_float() / divisor.as_float()
    }
}

impl Mul<u16> for Dollars {
    type Output = Dollars;

    fn mul(self, multiplier: u16) -> Dollars {
        let multiplier = u32::from(multiplier);
        let cents = multiplier * u32::from(self.cents);
        let whole = multiplier * self.whole;
        Dollars::from_parts(whole, cents)
    }
}

impl Mul<f64> for Dollars {
    type Output = Dollars;

    fn mul(self, multiplier: f64) -> Dollars {
        let cents = Dollars::from_parts(0, (multiplier * f64::from(self.cents)) as u32);
        let whole = Dollars::new(multiplier * f64::from(self.whole));
        whole + cents
    }
}

impl Dollars {
    fn from_parts(whole: u32, cents: u32) -> Self {
        Dollars {
            whole: whole + cents / 100,
            cents: (cents % 100) as u8,
        }
    }

    fn as_float(self) -> f64 {
        f64::from(self.whole) + f64::from(self.cents) / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplies_integer() {
        let multiplier = 2343;
        let amount = Dollars::new(23.45);
        let actual = amount * multiplier;
        let expected = Dollars::new(54943.35);
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiplies_float() {
        let multiplier = 23.43;
        let amount = Dollars::new(32.67);
        let actual = amount * multiplier;
        let expected = Dollars::new(765.45); //765.4581 truncated
        assert_eq!(expected, actual);
    }

    #[test]
    fn divides_dollars() {
        let amount = Dollars::new(5.00);
        let divisor = Dollars::new(2.00);
        assert_eq!(amount / divisor, 2.5);
    }

    #[test]
    fn substracts() {
        let actual = Dollars::new(23.34) - Dollars::new(12.41);
        let expected = Dollars::new(10.93);
        assert_eq!(expected, actual);
    }

    #[test]
    fn adds() {
        let actual = Dollars::new(26.41) + Dollars::new(52.84);
        let expected = Dollars::new(79.25);
        assert_eq!(expected, actual);
    }

    #[test]
    fn compare() {
        assert!(Dollars::new(23.31) > Dollars::new(12.34));
        assert!(Dollars::new(23.29) < Dollars::new(23.31));
        assert!(Dollars::new(12.29) == Dollars::new(12.29));
    }
}
