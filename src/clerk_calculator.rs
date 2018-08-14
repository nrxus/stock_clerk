use dollars::Dollars;
use equity::Equity;
use taxes::{TaxTable, TaxUser, TaxedAmount};
use user_data::{Grant, UserData};

use chrono::{Date, Local};

use std::{
    cmp,
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

pub struct StockCalculation {
    grants: HashMap<Grant, Equity>,
    cost: StockCosts,
    share_value: Dollars,
}

struct StockCosts {
    immediate: Dollars,
    taxes: Vec<TaxedAmount>,
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
        let immediate_cost = grants.iter().map(|(_, e)| e.vested.cost).sum();
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
            cost: StockCosts {
                immediate: immediate_cost,
                taxes,
            },
        }
    }
}

impl Display for StockCalculation {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "Share Value: {}", self.share_value)?;
        writeln!(f, "Grants:")?;
        for (g, e) in &self.grants {
            writeln!(f, "  - {}", g)?;
            writeln!(f, "{}", e)?;
        }
        writeln!(f, "Buying All Vested:")?;
        writeln!(f, "  Cost: {}", self.cost.immediate)?;
        writeln!(f, "  Taxes:")?;
        for taxed in &self.cost.taxes {
            writeln!(
                f,
                "    {}% * {} = {}",
                taxed.rate,
                taxed.amount,
                taxed.taxes()
            )?;
        }
        writeln!(f, "Selling All Vested:")?;
        let gross_profit: Dollars = self
            .grants
            .iter()
            .map(|(_, e)| e.vested.gross_profit())
            .sum();
        let total_taxes: Dollars = self.cost.taxes.iter().map(TaxedAmount::taxes).sum();
        writeln!(f, "  {:<13} {:>10}", "Gross Profit:", gross_profit)?;
        writeln!(f, "  {:<13} {:>10}", "Net Profit:", gross_profit - total_taxes)?;

        Ok(())
    }
}
