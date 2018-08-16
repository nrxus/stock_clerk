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
extern crate failure;

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
use failure::ResultExt;

use std::fs::File;

type Result<T> = std::result::Result<T, failure::Context<String>>;

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

fn main() -> Result<()> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.deserialize())
        .unwrap_or_else(|e| e.exit());
    let input = args.validate()?;
    let tax_table: TaxTable = serde_json::from_str(include_str!("../taxes.json")).unwrap();
    let clerk = StockClerk {
        tax_table,
        exercise_date: input.exercise_date,
    };

    let calculation = clerk.calculate(&input.user_data, input.stock_price);
    println!("{}", calculation);
    Ok(())
}

struct ValidatedInput {
    user_data: UserData,
    stock_price: Dollars,
    exercise_date: Date<Local>,
}

impl Args {
    fn validate(self) -> Result<ValidatedInput> {
        let user_data = File::open(&self.flag_file)
            .context(format!("Failed to open file: '{}'", self.flag_file))
            .and_then(|f| {
                serde_json::from_reader(f)
                    .context(format!("Failed to parse file: '{}'", self.flag_file))
            })?;
        let stock_price = self
            .flag_price
            .ok_or(())
            .or_else(|_| fetch_price())
            .context(format!(
                "Failed to fetch current PVTL stock price.\nTry passing it with --price PRICE"
            ))?;
        let exercise_date = self
            .flag_date
            .map(|d| Local.from_local_date(&d).unwrap())
            .unwrap_or_else(Local::today);
        Ok(ValidatedInput {
            user_data,
            stock_price,
            exercise_date,
        })
    }
}

fn fetch_price() -> Result<Dollars> {
    let http_client = reqwest::Client::new();
    let response = http_client
        .get("https://api.iextrading.com/1.0/stock/pvtl/price")
        .send()
        .and_then(|mut r| r.text())
        .context(format!("Failed to make request for PVTL stock price"))?;

    response
        .parse()
        .map(Dollars::new)
        .context(format!("Failed to parse PVTL stock price: '{}'", response))
}
