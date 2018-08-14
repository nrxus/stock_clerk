use dollars::Dollars;
use taxes::FilingStatus;

use chrono::NaiveDate;
use std::fmt::{self, Display, Formatter};

#[derive(Deserialize)]
pub struct UserData {
    pub income: Dollars,
    pub filing_status: FilingStatus,
    pub grants: Vec<Grant>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Grant {
    pub price: Dollars,
    pub total: u16,
    pub start: NaiveDate,
}

impl Display for Grant {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} ({} options at {} per option):", self.start, self.total, self.price)
    }
}
