use dollars::Dollars;
use enum_map::EnumMap;

#[derive(Debug, Enum, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum FilingStatus {
    Single,
    Married,
    MarriedSeparately,
    HeadOfHousehold,
}

struct TaxUser {
    income: Dollars,
    status: FilingStatus,
}

#[derive(Deserialize)]
pub struct TaxTable {
    #[serde(flatten)]
    info: EnumMap<FilingStatus, TaxInformation>,
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

impl TaxTable {
    fn bracket_for(&self, user: &TaxUser) -> &TaxBracket {
        let brackets = &self.info[user.status].brackets;
        brackets
            .windows(2)
            .find(|pair| pair[1].bracket_start > user.income)
            .map(|pair| &pair[0])
            .unwrap_or_else(|| &brackets[brackets.len() - 1])
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
            income: Dollars::new(160000.0),
            status: FilingStatus::Single,
        };

        let bracket = subject.bracket_for(&user);
        assert_eq!(bracket, &subject.single.brackets[4]);
    }

    #[test]
    fn tax_married() {
        let subject = subject();
        let user = TaxUser {
            income: Dollars::new(170000.0),
            status: FilingStatus::Married,
        };

        let bracket = subject.bracket_for(&user);
        assert_eq!(bracket, &subject.married.brackets[3]);
    }

    #[test]
    fn highest_bracket() {
        let subject = subject();
        let user = TaxUser {
            income: Dollars::new(500000.0),
            status: FilingStatus::HeadOfHousehold,
        };

        let bracket = subject.bracket_for(&user);
        assert_eq!(bracket, &subject.head_of_household.brackets[6]);
    }

    fn subject() -> TaxTable {
        serde_json::from_str(include_str!("../taxes.json")).expect("taxes.json could not be parsed")
    }
}
