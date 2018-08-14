use dollars::Dollars;
use equity::Equity;
use taxes::{TaxTable, TaxUser};
use user_data::{Grant, UserData};

use chrono::{Date, Local};

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

pub struct StockCalculation {
    grants: HashMap<Grant, Equity>,
    cost: StockCosts,
}

struct StockCosts {
    immediate: Dollars,
    taxes: HashMap<u8, Dollars>,
}

pub struct StockClerk {
    pub tax_table: TaxTable,
    pub exercise_date: Date<Local>,
}

impl StockClerk {
    pub fn calculate(&self, user: &UserData, stock_price: Dollars) -> StockCalculation {
        let grants: HashMap<_, _> = user
            .grants
            .iter()
            .map(|g| (g.clone(), Equity::new(g, &self.exercise_date, stock_price)))
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
                } else if *untaxed_profits <= taxed_value.amount {
                    Some(*untaxed_profits)
                } else {
                    Some(taxed_value.amount)
                }.map(|taxed_amount| {
                    *untaxed_profits = *untaxed_profits - taxed_amount;
                    (taxed_value.rate, taxed_amount)
                })
            })
            .collect();

        StockCalculation {
            grants,
            cost: StockCosts {
                immediate: immediate_cost,
                taxes,
            },
        }
    }
}

impl Display for StockCalculation {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "Grants:")?;
        for (g, e) in &self.grants {
            writeln!(f, "  - {}", g)?;
            writeln!(f, "{}", e)?;
        }
        writeln!(f, "Buying All Vested:")?;
        writeln!(f, "  cost: {}", self.cost.immediate)?;
        writeln!(f, "  taxes: {:?}", self.cost.taxes)?;

        Ok(())
    }
}
