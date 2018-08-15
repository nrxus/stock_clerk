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
};

pub struct StockCalculation {
    grants: HashMap<Grant, Equity>,
    taxes: Vec<TaxedAmount>,
    share_value: Dollars,
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
        let profits = grants.iter().map(|(_, e)| e.vested.gross_profit()).sum();
        let tax_user = TaxUser {
            income: user.income + profits,
            status: user.filing_status,
        };
        let taxes = self
            .tax_table
            .brackets_for(&tax_user)
            .scan(profits, |untaxed_profits, taxed_value| {
                if *untaxed_profits == Dollars::new(0.0) {
                    None
                } else {
                    let amount = cmp::min(*untaxed_profits, taxed_value.amount);
                    *untaxed_profits = *untaxed_profits - amount;
                    Some(TaxedAmount {
                        amount: amount,
                        rate: taxed_value.rate,
                    })
                }
            })
            .collect();

        StockCalculation {
            grants,
            share_value,
            taxes,
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

        let (vested_rows, vested_profit) = rows_for_grants(vested.into_iter());
        vested_table.extend(vested_rows);
        unvested_table.extend(rows_for_grants(unvested.into_iter()).0);

        let mut taxes_owed = Dollars::new(0.0);
        let taxes_table = {
            let mut table = table!(["Rate", "Taxed Amount", "Owed Taxed"]);
            self.taxes
                .iter()
                .inspect(|t| taxes_owed += t.taxes())
                .map(|t| row![t.rate, t.amount, t.taxes()])
                .for_each(|r| {
                    table.add_row(r);
                });
            table
        };

        vested_table.add_row(row![r => "", "", "", "Taxes", taxes_table]);
        vested_table.add_row(row![r => "", "", "", "Net Profit", vested_profit - taxes_owed]);

        writeln!(f, "SHARE VALUE: {}\n", self.share_value)?;
        f.write_str("VESTED OPTIONS\n")?;
        vested_table.fmt(f)?;
        f.write_str("\n")?;
        f.write_str("UNVESTED OPTIONS\n")?;
        unvested_table.fmt(f)?;
        Ok(())
    }
}

fn rows_for_grants<'a>(
    equities: impl Iterator<Item = (&'a Grant, &'a Stock)>,
) -> (Vec<Row>, Dollars) {
    let mut count = 0;
    let mut cost = Dollars::new(0.0);
    let mut revenue = Dollars::new(0.0);

    let mut rows: Vec<_> = equities
        .inspect(|(_, s)| {
            count += s.count;
            cost += s.cost;
            revenue += s.revenue;
        })
        .map(|(g, s)| row![r => g.start, s.count, s.cost, s.revenue, s.gross_profit()])
        .collect();

    let profit = revenue - cost;
    rows.push(row![r => "Total", count, cost, revenue, revenue - cost]);
    (rows, profit)
}
