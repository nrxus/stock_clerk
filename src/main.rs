extern crate chrono;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate enum_map;

mod clerk_calculator;
mod dollars;
mod equity;
mod taxes;
mod user_data;

use clerk_calculator::StockClerk;
use dollars::Dollars;
use taxes::TaxTable;
use user_data::UserData;

use chrono::Local;

use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    let tax_table: TaxTable = serde_json::from_str(include_str!("../taxes.json"))?;
    let user_data: UserData = serde_json::from_str(include_str!("../user_data.json"))?;
    let today = Local::today();
    let clerk = StockClerk {
        tax_table,
        exercise_date: today,
    };
    let http_client = reqwest::Client::new();
    let stock_price = http_client
        .get("https://api.iextrading.com/1.0/stock/pvtl/price")
        .send()?
        .text()?
        .parse()
        .map(Dollars::new)?;

    let calculation = clerk.calculate(&user_data, stock_price);
    println!("{}", calculation);
    Ok(())
}
