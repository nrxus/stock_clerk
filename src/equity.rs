use dollars::Dollars;
use user_data::Grant;

use chrono::Datelike;

#[derive(Debug)]
pub struct Equity {
    pub vested: Stock,
    pub unvested: Stock,
}

impl Equity {
    pub fn new(grant: &Grant, date: &impl Datelike, stock_price: Dollars) -> Self {
        let vested = (grant.start.percent_awarded(date) * f64::from(grant.total)) as u16;
        let unvested = grant.total - vested;
        Equity {
            vested: Stock::new(vested, grant.price, stock_price),
            unvested: Stock::new(unvested, grant.price, stock_price),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Stock {
    cost: Dollars,
    revenue: Dollars,
    count: u16,
}

impl Stock {
    fn new(count: u16, grant_price: Dollars, stock_price: Dollars) -> Self {
        Stock {
            count,
            cost: grant_price * count,
            revenue: stock_price * count,
        }
    }

    pub fn gross_profit(&self) -> Dollars {
        self.revenue - self.cost
    }
}

trait AwardDate {
    fn percent_awarded(&self, date: &impl Datelike) -> f64;
    fn months_until(&self, later: &impl Datelike) -> u32;
}

impl<T: Datelike> AwardDate for T {
    fn percent_awarded(&self, date: &impl Datelike) -> f64 {
        let elapsed = self.months_until(date);
        if elapsed < 12 {
            0.0
        } else {
            f64::from(elapsed) * 0.25 / 12.0
        }
    }

    fn months_until(&self, later: &impl Datelike) -> u32 {
        let years = later.year() - self.year();
        let months = later.month() as i32 - self.month() as i32;
        (years * 12 + months) as u32
    }
}
