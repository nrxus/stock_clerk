use std::ops::{Mul, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Dollars {
    whole: u32,
    cents: u8,
}

impl Dollars {
    pub fn new(amount: f64) -> Self {
        Dollars {
            whole: amount as u32,
            cents: (amount.fract() * 100.0) as u8,
        }
    }

    fn from_parts(whole: u32, cents: u32) -> Self {
        Dollars {
            whole: whole + cents / 100,
            cents: (cents % 100) as u8,
        }
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
        let cents = (self.cents as u32) * multiplier;
        let whole = self.whole * multiplier;
        Dollars::from_parts(whole, cents)
    }
}

impl Mul<f64> for Dollars {
    type Output = Dollars;

    fn mul(self, multiplier: f64) -> Dollars {
        let cents = (self.cents as f64 * multiplier) as u32;
        let whole = (self.whole as f64 * multiplier) as u32;
        Dollars::from_parts(whole, cents)
    }
}
