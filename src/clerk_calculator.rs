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
        let income = user.income + profits;

        let brackets = self.tax_table.info[user.filing_status]
            .brackets
            .iter()
            .rev()
            .skip_while(|bracket| bracket.bracket_start > income);

        let taxes = brackets
            .scan(profits, |untaxed_profits, bracket| {
                if *untaxed_profits == Dollars::new(0.0) {
                    None
                } else if user.income > bracket.bracket_start {
                    let next = *untaxed_profits;
                    *untaxed_profits = Dollars::new(0.0);
                    Some(next)
                } else {
                    let remaining_delta = bracket.bracket_start - user.income;
                    let next = *untaxed_profits - remaining_delta;
                    *untaxed_profits = remaining_delta;
                    Some(next)
                }.map(|amount| (bracket.rate, amount))
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
