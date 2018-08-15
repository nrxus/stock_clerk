use dollars::Dollars;
use equity::{Equity, Stock};
use taxes::{TaxTable, TaxUser, TaxedAmount};
use user_data::{Grant, UserData};

use chrono::{Date, Local};
use prettytable::row::Row;

use std::{
    cmp,
    collections::HashMap,
    fmt::{self, Display, Formatter},
    iter,
};

pub struct StockCalculation {
    grants: HashMap<Grant, Equity>,
    total: Equity,
    taxes: Vec<TaxedAmount>,
    share_value: Dollars,
    exercise_date: Date<Local>,
}

pub struct StockClerk {
    pub tax_table: TaxTable,
    pub exercise_date: Date<Local>,
}

impl StockClerk {
    pub fn calculate(&self, user: &UserData, share_value: Dollars) -> StockCalculation {
        let grants: HashMap<_, _> = user
            .grants
            .iter()
            .map(|g| (g.clone(), Equity::new(g, &self.exercise_date, share_value)))
            .collect();
        let total: Equity = grants.iter().map(|(_, e)| e.clone()).sum();
        let profits = total.vested.gross_profit();

        let tax_user = TaxUser {
            income: user.income + profits,
            status: user.filing_status,
        };
        let taxes = self
            .tax_table
            .brackets_for(&tax_user)
            .scan(profits, |untaxed_profits, taxed_value| {
                if *untaxed_profits == Dollars::zero() {
                    None
                } else {
                    let amount = cmp::min(*untaxed_profits, taxed_value.amount);
                    *untaxed_profits = *untaxed_profits - amount;
                    Some(TaxedAmount {
                        amount,
                        rate: taxed_value.rate,
                    })
                }
            })
            .collect();

        StockCalculation {
            total,
            grants,
            share_value,
            taxes,
            exercise_date: self.exercise_date,
        }
    }
}

impl Display for StockCalculation {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let mut vested_table =
            table!([c => "Grant", "# of Shares", "Cost", "Revenue", "Gross Profit"]);
        let mut unvested_table = vested_table.clone();

        let (vested, unvested): (Vec<_>, Vec<_>) = self
            .grants
            .iter()
            .map(|(g, e)| ((g, &e.vested), (g, &e.unvested)))
            .unzip();

        let vested_rows = rows_for_grants(&self.total.vested, vested.into_iter());
        vested_table.extend(vested_rows);
        let unvested_rows = rows_for_grants(&self.total.unvested, unvested.into_iter());
        unvested_table.extend(unvested_rows);

        let mut taxes_owed = Dollars::zero();
        let taxes_table = {
            let mut table = table!(["Rate", "Taxed Amount", "Owed Taxed"]);
            self.taxes
                .iter()
                .inspect(|t| taxes_owed += t.taxes())
                .map(|t| row![r => t.rate, t.amount, t.taxes()])
                .for_each(|r| {
                    table.add_row(r);
                });
            table
        };
        let total = &self.total.vested;
        let cost = total.cost + taxes_owed;
        let to_sell = (cost / self.share_value).ceil() as u32;

        f.write_str("### INPUTS ###\n")?;
        writeln!(f, "Share Value: {}", self.share_value)?;
        writeln!(f, "Exercise Date: {}", self.exercise_date)?;
        f.write_str("\n### VESTED OPTIONS ###\n")?;
        vested_table.fmt(f)?;
        writeln!(f, "\n## TAXES ##")?;
        taxes_table.fmt(f)?;

        writeln!(f, "\n## RESULTS ##")?;
        table!(["Cost + Taxes", "Net Profit", "Min Zero-Cost Sell"],
               [r => cost, total.revenue - cost, to_sell])
            .fmt(f)?;

        f.write_str("\n### UNVESTED OPTIONS ###\n")?;
        unvested_table.fmt(f)?;
        Ok(())
    }
}

fn rows_for_grants<'a>(
    total: &Stock,
    equities: impl Iterator<Item = (&'a Grant, &'a Stock)>,
) -> Vec<Row> {
    equities
        .map(|(g, s)| (g.start.to_string(), s))
        .chain(iter::once(("Total".to_string(), total)))
        .map(|(id, s)| row![r => id, s.count, s.cost, s.revenue, s.gross_profit()])
        .collect()
}
