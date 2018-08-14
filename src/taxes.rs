use dollars::Dollars;
use enum_map::EnumMap;

use std::{cmp, f64, iter};

#[derive(Debug, Enum, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    Married,
    MarriedSeparately,
    HeadOfHousehold,
}

pub struct TaxUser {
    pub income: Dollars,
    pub status: FilingStatus,
}

#[derive(Deserialize)]
pub struct TaxTable {
    #[serde(flatten)]
    pub info: EnumMap<FilingStatus, TaxInformation>,
    #[serde(skip)]
    #[serde(default = "TaxBracket::max")]
    max_bracket: TaxBracket,
}

#[derive(Deserialize)]
pub struct TaxInformation {
    pub brackets: Vec<TaxBracket>,
    pub deduction: Dollars,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct TaxBracket {
    pub bracket_start: Dollars,
    pub rate: u8,
    pub capital_gain_rate: u8,
    pub base_amount: Dollars,
}

impl TaxBracket {
    fn max() -> Self {
        TaxBracket {
            bracket_start: Dollars::max(),
            rate: 100,
            capital_gain_rate: 100,
            base_amount: Dollars::max(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct TaxedAmount {
    pub rate: u8,
    pub amount: Dollars,
}

impl TaxTable {
    pub fn brackets_for<'s>(&'s self, user: &TaxUser) -> impl Iterator<Item = TaxedAmount> + 's {
        let info: &TaxInformation = &self.info[user.status];
        let income = user.income - info.deduction;

        let mut bracket_pairs = info
            .brackets
            .windows(2)
            .map(|pair| (&pair[0], &pair[1]))
            .rev()
            .peekable();

        let bracket_pairs =
            iter::once((bracket_pairs.peek().unwrap().1, &self.max_bracket)).chain(bracket_pairs);

        bracket_pairs
            .skip_while(move |(low, _)| low.bracket_start >= income)
            .map(move |(low, high)| {
                let high = cmp::min(high.bracket_start, income);
                TaxedAmount {
                    rate: low.rate,
                    amount: high - low.bracket_start,
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json;

    #[test]
    fn tax_single() {
        let subject = subject();
        let user = TaxUser {
            income: Dollars::new(170000.0),
            status: FilingStatus::Single,
        };

        let mut brackets = subject.brackets_for(&user);
        let taxed_value = brackets.next().unwrap();
        assert_eq!(taxed_value.rate, 32);
        assert_eq!(taxed_value.amount, Dollars::new(500.0));

        let taxed_value = brackets.next().unwrap();
        assert_eq!(taxed_value.rate, 24);
        assert_eq!(taxed_value.amount, Dollars::new(75000.0));
    }

    #[test]
    fn tax_married() {
        let subject = subject();
        let user = TaxUser {
            income: Dollars::new(170000.0),
            status: FilingStatus::Married,
        };

        let mut brackets = subject.brackets_for(&user);
        let taxed_value = brackets.next().unwrap();
        assert_eq!(taxed_value.rate, 22);
        assert_eq!(taxed_value.amount, Dollars::new(68600.0));

        let taxed_value = brackets.next().unwrap();
        assert_eq!(taxed_value.rate, 12);
        assert_eq!(taxed_value.amount, Dollars::new(58350.0));
    }

    #[test]
    fn edge() {
        let subject = subject();
        let user = TaxUser {
            income: Dollars::new(94500.0),
            status: FilingStatus::MarriedSeparately,
        };

        let mut brackets = subject.brackets_for(&user);
        let taxed_value = brackets.next().unwrap();
        assert_eq!(taxed_value.rate, 22);
        assert_eq!(taxed_value.amount, Dollars::new(43800.0));

        let taxed_value = brackets.next().unwrap();
        assert_eq!(taxed_value.rate, 12);
        assert_eq!(taxed_value.amount, Dollars::new(29175.0));
    }

    #[test]
    fn highest_bracket() {
        let subject = subject();
        let user = TaxUser {
            income: Dollars::new(520000.0),
            status: FilingStatus::HeadOfHousehold,
        };

        let mut brackets = subject.brackets_for(&user);
        let taxed_value = brackets.next().unwrap();
        assert_eq!(taxed_value.rate, 37);
        assert_eq!(taxed_value.amount, Dollars::new(2000.0));

        let taxed_value = brackets.next().unwrap();
        assert_eq!(taxed_value.rate, 35);
        assert_eq!(taxed_value.amount, Dollars::new(300000.0));
    }

    fn subject() -> TaxTable {
        serde_json::from_str(include_str!("../taxes.json")).expect("taxes.json could not be parsed")
    }
}
