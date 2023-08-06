use marine_rs_sdk::marine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
#[marine]
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Metadata {
    pub hash: String,
    pub data_key: String,
    pub program_id: String,
    pub alias: String,
    pub cid: String,
    pub version: String,
    pub public_key: String,
    pub loose: bool,
}

impl Metadata {
    pub fn new(
        data_key: String,
        program_id: String,
        alias: String,
        cid: String,
        public_key: String,
        version: String,
        loose: bool,
    ) -> Self {
        let hash = Self::generate_hash(
            data_key.clone(),
            program_id.clone(),
            alias.clone(),
            public_key.clone(),
            version.clone(),
        );

        Self {
            hash,
            data_key,
            program_id,
            alias,
            cid,
            public_key,
            version,
            loose,
        }
    }

    pub fn generate_hash(
        data_key: String,
        program_id: String,
        alias: String,
        public_key: String,
        version: String,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(
            format!(
                "{}{}{}{}{}",
                data_key, program_id, alias, public_key, version
            )
            .as_bytes(),
        );
        bs58::encode(hasher.finalize()).into_string()
    }
}

#[marine]
#[derive(Debug, Clone)]
pub struct FinalMetadata {
    pub public_key: String,
    pub program_id: String,
    pub alias: String,
    pub content: String,
    pub version: String,
}

#[marine]
#[derive(Debug)]
pub struct MetadataQuery {
    pub column: String,
    pub query: String,
    pub op: String,
}

#[marine]
#[derive(Debug)]
pub struct MetadataOrdering {
    pub column: String,
    pub sort: String,
}
