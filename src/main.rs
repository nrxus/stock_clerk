extern crate chrono;
extern crate reqwest;
#[macro_use]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate enum_map;
#[macro_use]
extern crate prettytable;
extern crate docopt;

mod clerk_calculator;
mod dollars;
mod equity;
mod taxes;
mod user_data;

use clerk_calculator::StockClerk;
use dollars::Dollars;
use taxes::TaxTable;
use user_data::UserData;

use chrono::{Local, NaiveDate, TimeZone};
use docopt::Docopt;

use std::{error::Error, fs::File, process::exit};

const USAGE: &str = "
Calculate Stock Information
Usage:
    stock_clerk -f FILE [options]
    stock_clerk -h

Options:
    -f, --file FILE       User data JSON file
    -d, --date DATE       Exercise date (YYYY-MM-DD). Defaults to today
    -p, --price PRICE     Share price. Defaults to the current share price
    -h, --help            Show this screen
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_file: String,
    flag_date: Option<NaiveDate>,
    flag_price: Option<Dollars>,
}

fn main() -> Result<(), Box<Error>> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.deserialize())
        .unwrap_or_else(|e| e.exit());
    let file = File::open(args.flag_file)?;
    let stock_price = args.flag_price.unwrap_or_else(fetch_price);
    let exercise_date = args
        .flag_date
        .map(|d| Local.from_local_date(&d).unwrap())
        .unwrap_or_else(Local::today);
    let tax_table: TaxTable = serde_json::from_str(include_str!("../taxes.json"))?;
    let user_data: UserData = serde_json::from_reader(file)?;
    let clerk = StockClerk {
        tax_table,
        exercise_date,
    };

    let calculation = clerk.calculate(&user_data, stock_price);
    println!("{}", calculation);
    Ok(())
}

fn fetch_price() -> Dollars {
    try_fetch_price().unwrap_or_else(|e| {
        eprintln!("could not fetch current PVTL stock price");
        eprintln!("try passing it in with --price PRICE");
        eprintln!("cause: {}", e);
        exit(1);
    })
}

fn try_fetch_price() -> Result<Dollars, String> {
    let http_client = reqwest::Client::new();
    let response = http_client
        .get("https://api.iextrading.com/1.0/stock/pvtl/price")
        .send()
        .and_then(|mut r| r.text())
        .map_err(|e| {
            format!(
                "Failed to make request for PVTL stock price\n. Cause: {}",
                e
            )
        })?;

    response.parse().map(Dollars::new).map_err(|e| {
        format!(
            "Failed to parse PVTL stock price: '{}'\n. Cause: {}",
            response, e
        )
    })
}
