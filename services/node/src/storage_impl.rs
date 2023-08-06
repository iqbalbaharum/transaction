use crate::cron::Cron;
use crate::cron_tx::CronTx;
use crate::curl;
use crate::defaults::{SQL_EXECUTE, SQL_QUERY};
use crate::error::ServiceError;
use crate::meta_contract::MetaContract;
use crate::metadatas::Metadata;
use crate::transaction::{Transaction, TransactionReceipt};
use eyre::Result;
use marine_rs_sdk::MountedBinaryResult;
use serde::Deserialize;

pub struct Storage {}

#[derive(Deserialize)]
pub(crate) struct RQLiteResponse {
    results: Vec<RQLiteResult>,
}
#[derive(Deserialize)]
pub(crate) struct RQLiteResult {
    last_insert_id: Option<i64>,
    rows_affected: Option<i64>,
    error: Option<String>,
    pub rows: Option<Vec<Row>>,
}

#[derive(Deserialize)]
pub(crate) enum Row {
    MetaContract(MetaContract),
    Metadata(Metadata),
    Transaction(Transaction),
    TransactionReceipt(TransactionReceipt),
    Cron(Cron),
    CronTx(CronTx),
}

#[inline]
pub(crate) fn get_storage() -> Storage {
    Storage {}
}

impl Storage {
    pub(crate) fn execute(query: String) -> Result<RQLiteResult, ServiceError> {
        let statement = vec![
            "-XPOST".to_string(),
            SQL_EXECUTE.to_string(),
            "-H".to_string(),
            "Content-Type: application/json".to_string(),
            "-d".to_string(),
            "[".to_string(),
            query,
            "]".to_string(),
        ];

        let result = curl(statement);
        Self::unwrap_mounted_binary_result(result)
    }

    pub(crate) fn read(query: String) -> Result<RQLiteResult, ServiceError> {
        let args = vec![
            "-G".to_string(),
            SQL_QUERY.to_string(),
            "Content-Type: application/json".to_string(),
            "--data-urlencode".to_string(),
            format!("q={}", query).to_string(),
        ];

        let result = curl(args);
        Self::unwrap_mounted_binary_result(result)
    }

    pub fn unwrap_mounted_binary_result(
        result: MountedBinaryResult,
    ) -> Result<RQLiteResult, ServiceError> {
        let response: RQLiteResponse = serde_json::from_slice(&result.stdout)?;
        if let Some(result) = response.results.into_iter().next() {
            if let Some(error) = result.error {
                return Err(ServiceError::InternalError(error));
            } else {
                return Ok(result);
            }
        }

        Err(ServiceError::InternalError("Invalid response".to_string()))
    }
}
