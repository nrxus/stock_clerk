extern crate chrono;
extern crate reqwest;
extern crate serde;

mod dollars;

use dollars::Dollars;

use chrono::{Date, Datelike, Local, TimeZone};

use std::error::Error;

trait AwardDate {
    fn percent_awarded(&self, date: Date<Local>) -> f64;
    fn months_until(&self, later: Date<Local>) -> u32;
}

impl AwardDate for Date<Local> {
    fn percent_awarded(&self, date: Date<Local>) -> f64 {
        let elapsed = self.months_until(date);
        if elapsed < 12 {
            0.0
        } else {
            elapsed as f64 * 0.25 / 12.0
        }
    }

    fn months_until(&self, later: Date<Local>) -> u32 {
        let years = later.year() - self.year();
        let months = later.month() as i32 - self.month() as i32;
        (years * 12 + months) as u32
    }
}

#[derive(Debug)]
struct Value {
    cost: Dollars,
    revenue: Dollars,
}

impl Value {
    fn gross_profit(&self) -> Dollars {
        self.revenue - self.cost
    }

    fn net_profit(&self) -> Dollars {
        self.gross_profit() - self.tax()
    }

    fn tax(&self) -> Dollars {
        self.gross_profit() * 0.25
    }
}

#[derive(Debug)]
struct Equity {
    vested: Value,
    unvested: Value,
}

#[derive(Debug)]
struct Grant {
    price: Dollars,
    total: u16,
    start: Date<Local>,
}

fn main() -> Result<(), Box<Error>> {
    let grants = vec![
        Grant {
            price: Dollars::new(8.16),
            total: 3750,
            start: Local.ymd(2016, 02, 08),
        },
        Grant {
            price: Dollars::new(9.90),
            total: 3875,
            start: Local.ymd(2017, 08, 08),
        },
    ];
    let today = Local::today();

    let http_client = reqwest::Client::new();
    let stock_price = http_client
        .get("https://api.iextrading.com/1.0/stock/pvtl/price")
        .send()?
        .text()?
        .parse()
        .map(Dollars::new)?;

    let equities: Vec<_> = grants
        .iter()
        .map(|g| {
            let vested = (g.start.percent_awarded(today) * g.total as f64) as u16;
            let unvested = g.total - vested;
            Equity {
                vested: Value {
                    cost: g.price * vested,
                    revenue: stock_price * vested,
                },
                unvested: Value {
                    cost: g.price * unvested,
                    revenue: stock_price * unvested,
                },
            }
        })
        .map(|e| e.vested.gross_profit())
        .collect();

    println!("grants: {:#?}", grants);
    println!("equities: {:#?}", equities);
    Ok(())
}
