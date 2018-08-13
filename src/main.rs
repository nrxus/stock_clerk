extern crate chrono;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate enum_map;

mod dollars;
mod equity;
mod tax_calculator;
mod taxes;
mod user_data;

use dollars::Dollars;
use equity::Equity;
use tax_calculator::TaxCalculator;
use taxes::{TaxTable, TaxUser};
use user_data::UserData;

use chrono::Local;

use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    let tax_table: TaxTable = serde_json::from_str(include_str!("../taxes.json"))?;
    let tax_calculator = TaxCalculator::new(tax_table);
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

    let user = TaxUser {
        income: user_data.income,
        status: user_data.filing_status,
    };

    equities.iter().map(|e| e.vested).for_each(|equity| {
        println!("Vested Equity:  ");
        println!("\tgross profit: {}", equity.gross_profit().to_string());
        println!(
            "\ttax: {}",
            tax_calculator.taxes_for_user(&user, equity.gross_profit())
        )
        // println!("\tnet profit: {:>10}", v.net_profit().to_string());
    });
    Ok(())
}
