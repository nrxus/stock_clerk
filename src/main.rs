extern crate chrono;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
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

use chrono::{Date, Local, NaiveDate, TimeZone};
use docopt::Docopt;

use std::{error::Error, fs::File, process::exit};

const USAGE: &'static str = "
Calculate Stock Information
Usage:
    stock_clerk (-f | --file) <file> [--date=DATE] [--price=PRICE]
Options:
    -f FILE, --file FILE       User Data JSON
    --date=DATE                Exercise Date (YYYY-MM-DD). Defaults to today
    --price=PRICE              Share Price. Defaults to the current share price
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_file: String,
    flag_date: Option<NaiveDate>,
    flag_price: Option<Dollars>,
}

fn main() -> Result<(), Box<Error>> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.deserialize())
        .unwrap_or_else(|e| e.exit());
    let file = File::open(args.arg_file)?;
    let stock_price = args.flag_price.unwrap_or_else(|| {
        let http_client = reqwest::Client::new();
        let response = http_client
            .get("https://api.iextrading.com/1.0/stock/pvtl/price")
            .send()
            .and_then(|mut r| r.text())
            .unwrap_or_else(|e| {
                eprintln!("could not fetch current PVTL stock price");
                eprintln!("try passing it in with -price=<PRICE>");
                eprintln!("got error: {}", e.to_string());
                exit(1);
            });
        response.parse().map(Dollars::new).unwrap_or_else(|e| {
            eprintln!("could not parse fetched PVTL stock price: '{}'", response);
            eprintln!("try passing it in with -price=<PRICE>");
            eprintln!("got error: {}", e.to_string());
            exit(1);
        })
    });
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
