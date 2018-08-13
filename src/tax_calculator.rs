use dollars::Dollars;
use taxes::{TaxTable, TaxUser};

pub struct TaxCalculator {
    table: TaxTable,
}

impl TaxCalculator {
    pub fn new(table: TaxTable) -> Self {
        TaxCalculator { table }
    }

    pub fn taxes_for_user(&self, user: &TaxUser, gross_profit: Dollars) -> Dollars {
        let bracket = self.table.bracket_for(user);

        gross_profit * (bracket.rate as f64 / 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json;

    #[test]
    fn calculates_tax_single() {
        let tax_table: TaxTable = serde_json::from_str(include_str!("../taxes.json"))
            .expect("taxes.json could not be parsed");
        let subject = TaxCalculator::new(tax_table);
        let user = TaxUser {
            income: Dollars::new(160000.0),
            status: FilingStatus::Single,
        };

        let taxed_amount = subject.taxes_for_user(&user, Dollars::new(35000.0));
        assert_eq!(Dollars::new(35000.0) * 0.32, taxed_amount);
    }

    #[test]
    fn calculates_tax_married() {
        let tax_table: TaxTable = serde_json::from_str(include_str!("../taxes.json"))
            .expect("taxes.json could not be parsed");
        let subject = TaxCalculator::new(tax_table);
        let user = TaxUser {
            income: Dollars::new(170000.0),
            status: FilingStatus::Married,
        };

        let taxed_amount = subject.taxes_for_user(&user, Dollars::new(40000.0));
        assert_eq!(Dollars::new(40000.0) * 0.24, taxed_amount);
    }

    #[test]
    fn highest_bracket() {
        let tax_table: TaxTable = serde_json::from_str(include_str!("../taxes.json"))
            .expect("taxes.json could not be parsed");
        let subject = TaxCalculator::new(tax_table);
        let user = TaxUser {
            income: Dollars::new(500000.0),
            status: FilingStatus::HeadOfHousehold,
        };

        let taxed_amount = subject.taxes_for_user(&user, Dollars::new(40000.0));
        assert_eq!(Dollars::new(40000.0) * 0.37, taxed_amount);
    }
}
