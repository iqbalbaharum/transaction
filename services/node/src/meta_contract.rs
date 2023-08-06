use marine_rs_sdk::marine;
use serde::Deserialize;

#[marine]
#[derive(Debug, Default, Clone, Deserialize)]
pub struct MetaContract {
    pub program_id: String,
    pub public_key: String,
    pub cid: String,
}
