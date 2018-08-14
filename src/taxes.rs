use dollars::Dollars;
use enum_map::EnumMap;

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
    pub fn bracket_for(&self, user: &TaxUser) -> &TaxBracket {
        let brackets = &self.info[user.status].brackets;
        brackets
            .windows(2)
            .find(|pair| user.income <= pair[1].bracket_start)
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
        assert_eq!(bracket, &subject.info[FilingStatus::Single].brackets[4]);
    }

    #[test]
    fn tax_married() {
        let subject = subject();
        let user = TaxUser {
            income: Dollars::new(170000.0),
            status: FilingStatus::Married,
        };

        let bracket = subject.bracket_for(&user);
        assert_eq!(bracket, &subject.info[FilingStatus::Married].brackets[3]);
    }

    #[test]
    fn edge() {
        let subject = subject();
        let user = TaxUser {
            income: Dollars::new(82500.0),
            status: FilingStatus::MarriedSeparately,
        };
        let bracket = subject.bracket_for(&user);
        assert_eq!(
            bracket,
            &subject.info[FilingStatus::MarriedSeparately].brackets[2]
        );
    }

    #[test]
    fn highest_bracket() {
        let subject = subject();
        let user = TaxUser {
            income: Dollars::new(510000.0),
            status: FilingStatus::HeadOfHousehold,
        };

        let bracket = subject.bracket_for(&user);
        assert_eq!(
            bracket,
            &subject.info[FilingStatus::HeadOfHousehold].brackets[6]
        );
    }

    fn subject() -> TaxTable {
        serde_json::from_str(include_str!("../taxes.json")).expect("taxes.json could not be parsed")
    }
}
