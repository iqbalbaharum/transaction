use marine_rs_sdk::marine;

#[marine]
#[derive(Debug, Default, Clone)]
pub struct MetaContract {
    pub program_id: String,
    pub public_key: String,
    pub cid: String,
}
