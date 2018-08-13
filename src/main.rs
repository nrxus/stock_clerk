extern crate chrono;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod dollars;
mod equity;
mod taxes;
mod user_data;

use dollars::Dollars;
use equity::Equity;
use taxes::TaxTable;
use user_data::UserData;

use chrono::{Local, NaiveDate};

use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Grant {
    price: Dollars,
    total: u16,
    start: NaiveDate,
}

fn main() -> Result<(), Box<Error>> {
    let tax_table: TaxTable = serde_json::from_str(include_str!("../taxes.json"))?;
    let user_data: UserData = serde_json::from_str(include_str!("../user_data.json"))?;
    let grants = user_data.grants;
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
        .map(|g| Equity::new(g, &today, stock_price))
        .collect();

    equities.iter().map(|e| e.vested).for_each(|v| {
        println!("Option:  ");
        println!("\tgross profit: {:>10}", v.gross_profit().to_string());
        println!("\ttax: {:>10}", v.tax().to_string());
        println!("\tnet profit: {:>10}", v.net_profit().to_string());
    });
    Ok(())
}
