use dollars::Dollars;
use taxes::FilingStatus;

use chrono::NaiveDate;

#[derive(Deserialize)]
pub struct UserData {
    pub income: Dollars,
    pub filing_status: FilingStatus,
    pub grants: Vec<Grant>,
}

#[derive(Debug, Deserialize)]
pub struct Grant {
    pub price: Dollars,
    pub total: u16,
    pub start: NaiveDate,
}
