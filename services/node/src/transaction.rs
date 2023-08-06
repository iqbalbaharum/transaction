use crate::defaults::STATUS_PENDING;
use marine_rs_sdk::marine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[marine]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub method: String,
    pub program_id: String,
    pub data_key: String,
    pub data: String,
    pub public_key: String,
    pub alias: String,
    pub timestamp: u64,
    pub version: String,
    pub mcdata: String,
}

#[marine]
#[derive(Debug, Default)]
pub struct TransactionRequest {
    pub data_key: String,
    pub program_id: String,
    pub alias: String,
    pub public_key: String,
    pub signature: String,
    pub data: String,
    pub method: String,
    pub version: String,
    pub mcdata: String,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionSubset {
    pub hash: String,
    pub timestamp: u64,
    pub meta_contract_id: String,
    pub method: String,
    pub value: String,
}

#[marine]
#[derive(Debug)]
pub struct TransactionQuery {
    pub column: String,
    pub query: String,
    pub op: String,
}

#[marine]
#[derive(Debug)]
pub struct TransactionOrdering {
    pub column: String,
    pub sort: String,
}

#[marine]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub hash: String,
    pub program_id: String,
    pub status: i64,
    pub timestamp: u64,
    pub error_text: String,
    pub data: String,
}

impl Transaction {
    pub fn new(
        program_id: String,
        from_peer_id: String,
        host_id: String,
        data_key: String,
        data: String,
        public_key: String,
        alias: String,
        timestamp: u64,
        method: String,
        version: String,
        mcdata: String,
        previous_data: String,
    ) -> Self {
        let hash = Self::generate_hash(
            program_id.clone(),
            data_key.clone(),
            data.clone(),
            public_key.clone(),
            alias.clone(),
            method.clone(),
            version.clone(),
            mcdata.clone(),
            previous_data,
        );

        Self {
            hash,
            method,
            program_id,
            data_key,
            data,
            public_key,
            alias,
            timestamp,
            version,
            mcdata,
        }
    }

    /**
     * Generating new transaction hash
     * Using the formula hash(tx datas + previous_data)
     * This hash would only prevent replay attach if the transaction content is similar from the current content.
     * A good example is health/mana level - There will be multiple duplicate data
     */
    pub fn generate_hash(
        program_id: String,
        data_key: String,
        data: String,
        public_key: String,
        alias: String,
        method: String,
        version: String,
        previous_content: String,
        mcdata: String,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(
            format!(
                "{}{}{}{}{}{}{}{}{}",
                program_id,
                data_key,
                data,
                public_key,
                alias,
                method,
                version,
                mcdata,
                previous_content
            )
            .as_bytes(),
        );
        bs58::encode(hasher.finalize()).into_string()
    }
}
