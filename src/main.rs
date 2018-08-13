extern crate chrono;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod dollars;

use dollars::Dollars;

use chrono::{Datelike, Local, NaiveDate};

use std::error::Error;

trait AwardDate {
    fn percent_awarded(&self, date: impl Datelike) -> f64;
    fn months_until(&self, later: impl Datelike) -> u32;
}

impl<T: Datelike> AwardDate for T {
    fn percent_awarded(&self, date: impl Datelike) -> f64 {
        let elapsed = self.months_until(date);
        if elapsed < 12 {
            0.0
        } else {
            elapsed as f64 * 0.25 / 12.0
        }
    }

    fn months_until(&self, later: impl Datelike) -> u32 {
        let years = later.year() - self.year();
        let months = later.month() as i32 - self.month() as i32;
        (years * 12 + months) as u32
    }
}

#[derive(Debug, Clone, Copy)]
struct Stock {
    cost: Dollars,
    revenue: Dollars,
    count: u16,
}

impl Stock {
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
    vested: Stock,
    unvested: Stock,
}

#[derive(Debug, Deserialize)]
struct Grant {
    price: Dollars,
    total: u16,
    start: NaiveDate,
}

fn main() -> Result<(), Box<Error>> {
    let grants: Vec<Grant> = serde_json::from_str(include_str!("../user_data.json"))?;
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
                vested: Stock {
                    count: vested,
                    cost: g.price * vested,
                    revenue: stock_price * vested,
                },
                unvested: Stock {
                    count: unvested,
                    cost: g.price * unvested,
                    revenue: stock_price * unvested,
                },
            }
        })
        .collect();

    equities.iter().map(|e| e.vested).for_each(|v| {
        println!("Option:  ");
        println!("\tgross profit: {:>10}", v.gross_profit().to_string());
        println!("\ttax: {:>10}", v.tax().to_string());
        println!("\tnet profit: {:>10}", v.net_profit().to_string());
    });
    Ok(())
}
