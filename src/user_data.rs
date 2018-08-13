use dollars::Dollars;
use Grant;

#[derive(Deserialize)]
pub struct UserData {
    pub income: Dollars,
    pub filing_status: String,
    pub grants: Vec<Grant>,
}
