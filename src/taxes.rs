use dollars::Dollars;

#[derive(Deserialize)]
pub struct TaxTable {
    pub single: TaxInformation,
    pub married: TaxInformation,
    pub married_separately: TaxInformation,
    pub head_of_household: TaxInformation,
}

#[derive(Deserialize)]
pub struct TaxInformation {
    pub brackets: Vec<TaxBracket>,
    pub deduction: Dollars,
}

#[derive(Deserialize)]
pub struct TaxBracket {
    pub bracket_max: Dollars,
    pub rate: u8,
    pub capital_gain_rate: u8,
    pub base_amount: Dollars,
}
