#![allow(improper_ctypes)]

mod block;
pub mod cron;
pub mod cron_tx;
mod data_types;
mod defaults;
mod error;
mod meta_contract;
mod meta_contract_impl;
mod metadatas;
mod metadatas_impl;
mod result;
mod storage_impl;
mod transaction;
pub mod transaction_receipt;
pub mod transactions_impl;
mod utils;
mod validators;

// use cron::SerdeCron;
// use cron_tx::CronTx;
use data_types::{DataTypeClone, DataTypeFork, SerdeDataTypeFork};
use defaults::{
    CRON_ACTION_CREATE, CRON_STATUS_DISABLE, CRON_STATUS_ENABLE, ENCRYPTION_TYPE_ED25519,
    ENCRYPTION_TYPE_SECP256K1, METHOD_CRON, STATUS_PENDING, STATUS_SUCCESS,
};
use defaults::{METHOD_CLONE, METHOD_CONTRACT, METHOD_METADATA, STATUS_FAILED};
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::MountedBinaryResult;
use marine_rs_sdk::WasmLoggerBuilder;

use error::ServiceError::{
    self, InvalidMethod, InvalidOwner, InvalidSignature, NoEncryptionType, NoProgramId,
    NotSupportedEncryptionType, RecordFound,
};

use metadatas::{FinalMetadata, MetadataOrdering, MetadataQuery};
use result::{
    FdbClock, FdbCronTxResult, FdbCronTxsResult, FdbCronsResult, FdbMetaContractResult,
    FdbMetadataHistoryResult, FdbMetadatasResult, FdbTransactionResult, FdbTransactionsResult,
};
use result::{FdbMetadataResult, FdbResult};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use storage_impl::get_storage;
use transaction::{Transaction, TransactionOrdering, TransactionQuery, TransactionRequest};
use types::{IpfsDagGetResult, IpfsDagPutResult, IpfsPutResult};
use utils::hasher;
use validators::{
    validate_clone, validate_cron, validate_meta_contract, validate_metadata,
    validate_metadata_cron,
};

#[macro_use]
extern crate fstrings;

module_manifest!();

pub fn wrapped_try<F, T>(func: F) -> T
where
    F: FnOnce() -> T,
{
    func()
}

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Info)
        .build()
        .unwrap();

    let storage = get_storage();
    storage.create_meta_contract_table();
    storage.create_transactions_table();
    storage.create_metadatas_table();
    storage.create_cron_table();
    storage.create_cron_tx_table();
    storage.create_transaction_receipt_table();
}

#[marine]
pub fn publish(
    data_key: String,
    method: String,
    alias: String,
    public_key: String,
    program_id: String,
    signature: String,
    data: String,
    version: String,
    mcdata: String,
) -> FdbResult {
    let mut program_id = program_id;
    let mut error: Option<ServiceError> = None;
    let mut content = "".to_string();
    let storage = get_storage();

    if error.is_none() {
        if method != METHOD_CONTRACT
            && method != METHOD_METADATA
            && method != METHOD_CLONE
            && method != METHOD_CRON
        {
            error = Some(InvalidMethod(f!("invalid method: {method}")));
        }
    }

    let enc_verify = get_public_key_type(public_key.clone().as_str());
    if enc_verify.len() <= 0 {
        error = Some(ServiceError::InvalidEncryption(public_key.clone()));
    }

    if error.is_none() {
        if method.clone() == METHOD_METADATA {
            if program_id.clone().is_empty() {
                error = Some(NoProgramId());
            }

            if (error.is_none()) {
                let result = storage.get_owner_metadata(
                    data_key.clone(),
                    program_id.clone(),
                    public_key.clone(),
                    alias.clone(),
                    version.clone(),
                );

                match result {
                    Ok(metadata) => {
                        if metadata.public_key != public_key {
                            error = Some(InvalidOwner(f!("not owner of data_key: {public_key}")));
                        }

                        content = metadata.cid;
                    }
                    Err(ServiceError::RecordNotFound(_)) => {}
                    Err(e) => error = Some(e),
                }
            }
        } else if method.clone() == METHOD_CONTRACT {
            // to generate unique program_id
            program_id = hasher(f!("{}{}", data.clone(), public_key.clone()));
            // Check if the
            let result = storage.get_meta_contract(program_id.clone());
            match result {
                Ok(metacontract) => {
                    error = Some(ServiceError::RecordFound(program_id.clone()));
                    content = metacontract.program_id;
                }
                Err(e) => error = Some(e),
            }
            content = program_id.clone();
        } else if method.clone() == METHOD_CLONE {
            let data_clone_result: Result<DataTypeClone, serde_json::Error> =
                serde_json::from_str(&data.clone());

            match data_clone_result {
                Ok(data_clone) => {
                    let origin_metadata_result = storage.get_owner_metadata(
                        data_clone.origin_data_key.clone(),
                        data_clone.origin_program_id.clone(),
                        data_clone.origin_public_key.clone(),
                        data_clone.origin_alias.clone(),
                        data_clone.origin_version.clone(),
                    );

                    match origin_metadata_result {
                        Ok(_) => {}
                        Err(e) => error = Some(e),
                    }

                    if error.is_none() {
                        let new_metadata_result = storage.get_owner_metadata(
                            data_key.clone(),
                            data_clone.origin_program_id.clone(),
                            data_clone.origin_public_key.clone(),
                            data_clone.origin_alias.clone(),
                            data_clone.origin_version.clone(),
                        );

                        match new_metadata_result {
                            Ok(_) => error = Some(RecordFound(data_key.clone())),
                            Err(ServiceError::RecordNotFound(_)) => {}
                            Err(e) => error = Some(e),
                        }
                    }
                }
                Err(_) => {
                    error = Some(ServiceError::InvalidDataFormatForMethodType(method.clone()))
                }
            }
        } else if method.clone() == METHOD_CRON {
            let cron_result: Result<SerdeCron, serde_json::Error> = serde_json::from_str(&data);

            match cron_result {
                Ok(serde_cron) => {
                    if serde_cron.action == CRON_ACTION_CREATE {
                        if serde_cron.address.len() <= 0
                            || serde_cron.chain.len() <= 0
                            || serde_cron.topic.len() <= 0
                            || serde_cron.token_type.len() <= 0
                        {
                            error =
                                Some(ServiceError::InvalidDataFormatForMethodType(method.clone()))
                        } else {
                            let result = storage.search_cron(
                                serde_cron.address.clone(),
                                serde_cron.chain.clone(),
                                serde_cron.topic.clone(),
                            );

                            match result {
                                Ok(_) => {
                                    error = Some(RecordFound(f!(
                                    "{serde_cron.address} {serde_cron.chain} {serde_cron.topic}"
                                )))
                                }
                                Err(ServiceError::RecordNotFound(_)) => {}
                                Err(e) => error = Some(e),
                            }
                        }
                    } else {
                        if serde_cron.hash.is_empty()
                            || (serde_cron.status != CRON_STATUS_ACTIVE
                                && serde_cron.status != CRON_STATUS_DISABLE)
                        {
                            error =
                                Some(ServiceError::InvalidDataFormatForMethodType(method.clone()))
                        } else {
                            let result = storage.get_cron_by_hash(serde_cron.hash);
                            match result {
                                Ok(_) => {}
                                Err(e) => error = Some(e),
                            }
                        }
                    }
                }
                Err(e) => error = Some(ServiceError::InvalidDataFormatForMethodType(e.to_string())),
            }
        } else {
            error = Some(InvalidMethod(f!("{method}")));
        }
    }

    if error.is_none() {
        if enc_verify.clone().is_empty() {
            error = Some(NoEncryptionType())
        } else {
            if enc_verify.clone().ne(ENCRYPTION_TYPE_SECP256K1)
                && enc_verify.clone().ne(ENCRYPTION_TYPE_ED25519)
            {
                error = Some(NotSupportedEncryptionType(enc_verify.clone()));
            }
        }
    }

    if error.is_none() {
        let v = verify(
            public_key.clone(),
            signature.clone(),
            data.clone(),
            enc_verify.clone(),
        );

        if !v {
            error = Some(InvalidSignature(f!("not owner of data_key: {public_key}")));
        }
    }

    let cp = marine_rs_sdk::get_call_parameters();

    let now = SystemTime::now();
    let timestamp = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // Hash version
    let mut transaction = Transaction::new(
        program_id,
        cp.init_peer_id,
        cp.host_id,
        data_key,
        data,
        public_key,
        alias,
        timestamp.as_millis() as u64,
        method,
        hasher(version),
        mcdata,
        content,
    );

    if !error.is_none() {
        transaction.error_text = error.unwrap().to_string();
        transaction.status = STATUS_FAILED;
    }

    let _ = storage.write_transaction(transaction.clone());

    FdbResult {
        transaction_hash: transaction.hash,
    }
}

#[marine]
pub fn send_batch_transaction(txs: Vec<TransactionRequest>) -> Vec<FdbResult> {
    let mut results: Vec<FdbResult> = vec![];

    for tx in txs {
        let result = publish(
            tx.data_key,
            tx.method,
            tx.alias,
            tx.public_key,
            tx.program_id,
            tx.signature,
            tx.data,
            tx.version,
            tx.mcdata,
        );

        results.push(result);
    }
    results
}

#[marine]
pub fn send_cron_tx(
    hash: String,
    data_key: String,
    data: String,
    tx_block_number: u64,
    tx_hash: String,
    token_id: String,
) -> FdbCronTxResult {
    let mut error: Option<ServiceError> = None;
    let mut success = true;
    let mut proceed = false;
    let mut err_msg = "".to_string();
    let mut cron_tx = CronTx::default();
    let storage = get_storage().expect("Database non existance");

    let cron = storage.get_cron_by_hash(hash);

    match cron {
        Ok(cron_data) => {
            let logs = storage.get_cron_tx_by_tx_hash(
                tx_hash.clone(),
                cron_data.clone().address,
                cron_data.clone().chain,
                cron_data.clone().topic,
            );

            match logs {
                Ok(tx) => {
                    if tx.status == STATUS_FAILED {
                        proceed = true;
                    } else {
                        cron_tx = tx;
                    }
                }
                Err(ServiceError::RecordNotFound(_)) => {
                    proceed = true;
                }
                Err(e) => error = Some(e),
            }

            if proceed {
                let now = SystemTime::now();
                let timestamp = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

                cron_tx = CronTx::new(
                    cron_data.address,
                    cron_data.topic,
                    cron_data.token_type,
                    cron_data.chain,
                    cron_data.meta_contract_id,
                    timestamp.as_millis() as u64,
                    tx_block_number,
                    tx_hash,
                    STATUS_SUCCESS,
                    data,
                    "".to_string(),
                    token_id,
                    data_key,
                    cron_data.token_key,
                );

                let _ = storage.write_cron_tx(cron_tx.clone());
            }
        }
        Err(ServiceError::RecordNotFound(_)) => {}
        Err(e) => error = Some(e),
    }

    if !error.is_none() {
        success = false;
        err_msg = error.unwrap().to_string();
    }

    FdbCronTxResult {
        success,
        err_msg,
        cron_tx,
    }
}

#[marine]
pub fn get_transaction(hash: String) -> FdbTransactionResult {
    wrapped_try(|| get_storage()?.get_transaction(hash)).into()
}

#[marine]
pub fn get_success_transactions(from: i64, to: i64) -> FdbTransactionsResult {
    let mut ts: i64 = 0;
    if to == 0 {
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let milliseconds = timestamp.as_millis();
        ts = milliseconds as i64
    } else {
        ts = to
    }

    wrapped_try(|| get_storage()?.get_success_transactions(from, ts)).into()
}

#[marine]
pub fn get_node_clock() -> FdbClock {
    let now = SystemTime::now();
    let timestamp = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let milliseconds = timestamp.as_millis();

    FdbClock {
        timestamp: milliseconds as i64,
    }
}

#[marine]
pub fn get_metadata(
    data_key: String,
    program_id: String,
    public_key: String,
    alias: String,
    version: String,
) -> FdbMetadataResult {
    wrapped_try(|| {
        get_storage()?.get_owner_metadata(data_key, program_id, public_key, alias, version)
    })
    .into()
}

#[marine]
pub fn get_metadatas(data_key: String, version: String) -> FdbMetadatasResult {
    wrapped_try(|| get_storage()?.get_metadata_by_datakey_and_version(data_key, version)).into()
}

#[marine]
pub fn search_metadatas(
    query: Vec<MetadataQuery>,
    ordering: Vec<MetadataOrdering>,
    from: u32,
    to: u32,
) -> FdbMetadatasResult {
    wrapped_try(|| get_storage()?.search_metadatas(query, ordering, from, to)).into()
}

#[marine]
pub fn get_meta_contract(token_key: String) -> FdbMetaContractResult {
    wrapped_try(|| get_storage()?.get_meta_contract(token_key)).into()
}

#[marine]
pub fn get_meta_contract_by_id(meta_contract_id: String) -> FdbMetaContractResult {
    wrapped_try(|| get_storage()?.get_meta_contract_by_id(meta_contract_id)).into()
}

#[marine]
pub fn get_pending_transactions() -> FdbTransactionsResult {
    wrapped_try(|| get_storage()?.get_pending_transactions()).into()
}

#[marine]
pub fn get_transactions(
    query: Vec<TransactionQuery>,
    ordering: Vec<TransactionOrdering>,
    from: u32,
    to: u32,
) -> FdbTransactionsResult {
    wrapped_try(|| get_storage()?.get_transactions(query, ordering, from, to)).into()
}

#[marine]
pub fn get_active_crons() -> FdbCronsResult {
    wrapped_try(|| get_storage()?.get_active_crons()).into()
}

#[marine]
pub fn get_all_crons() -> FdbCronsResult {
    wrapped_try(|| get_storage()?.get_all_crons()).into()
}

#[marine]
pub fn get_all_cron_txs() -> FdbCronTxsResult {
    wrapped_try(|| get_storage()?.get_all_cron_txs()).into()
}

#[marine]
pub fn get_cron_tx_by_tx_hash(
    tx_hash: String,
    address: String,
    chain: String,
    topic: String,
) -> FdbCronTxResult {
    wrapped_try(|| get_storage()?.get_cron_tx_by_tx_hash(tx_hash, address, chain, topic)).into()
}

#[marine]
pub fn search_cron_tx(address: String, chain: String, topic: String) -> FdbCronTxsResult {
    wrapped_try(|| get_storage()?.search_cron_tx(address, chain, topic)).into()
}

#[marine]
pub fn get_cron_tx_latest_block(address: String, chain: String, topic: String) -> u64 {
    wrapped_try(|| {
        let storage = get_storage().expect("Internal error to database connector");
        let result = storage.get_cron_tx_latest_block(address, chain, topic);

        match result {
            Ok(log) => log.tx_block_number,
            Err(ServiceError::RecordNotFound(_)) => 0,
            Err(_) => 0,
        }
    })
    .into()
}

#[marine]
pub fn get_metadata_with_history(
    data_key: String,
    program_id: String,
    public_key: String,
    alias: String,
    version: String,
) -> FdbMetadataHistoryResult {
    wrapped_try(|| {
        let storage = get_storage().expect("Internal error to database connector");

        let result = storage.get_owner_metadata(data_key, program_id, public_key, alias, version);

        let metadata;
        let mut metadatas: Vec<String> = Vec::new();

        match result {
            Ok(m) => {
                metadata = m;
            }
            Err(e) => return Err(e),
        };

        let mut read_metadata_cid: String = metadata.cid.clone();

        while read_metadata_cid.len() > 0 {
            let result = get_ipld(read_metadata_cid.clone(), "".to_string(), 0);
            let val: Value = serde_json::from_str(&result.block).unwrap();

            let input = format!(r#"{}"#, val);
            metadatas.push(input);

            let previous_cid = val
                .get("previous")
                .and_then(|v| v.get("/"))
                .and_then(|v| v.as_str());

            if previous_cid.is_none() {
                break;
            } else {
                read_metadata_cid = previous_cid.unwrap().to_string();
            }
        }

        Ok(metadatas)
    })
    .into()
}

// *********** VALIDATOR *****************
#[marine]
pub fn bind_meta_contract(transaction_hash: String) {
    validate_meta_contract(transaction_hash);
}

#[marine]
pub fn set_metadata(
    transaction_hash: String,
    meta_contract_id: String,
    on_metacontract_result: bool,
    metadatas: Vec<FinalMetadata>,
    final_error_msg: String,
) {
    validate_metadata(
        transaction_hash,
        on_metacontract_result,
        metadatas,
        final_error_msg,
    );
}

#[marine]
pub fn set_metadata_cron(
    data_key: String,
    on_metacontract_result: bool,
    metadatas: Vec<FinalMetadata>,
) {
    validate_metadata_cron(data_key, on_metacontract_result, metadatas);
}

#[marine]
pub fn set_clone(
    transaction_hash: String,
    meta_contract_id: String,
    on_metacontract_result: bool,
    data: String,
    final_error_msg: String,
) {
    validate_clone(
        transaction_hash,
        meta_contract_id,
        on_metacontract_result,
        data,
        final_error_msg,
    );
}

#[marine]
pub fn set_cron(transaction_hash: String, data: String) {
    validate_cron(transaction_hash, data);
}

// *********** Deserializer *****************
#[marine]
pub fn deserialize_fork(data: String) -> DataTypeFork {
    let result: SerdeDataTypeFork = serde_json::from_str(&data).unwrap_or_default();

    DataTypeFork::from(result)
}

/************************ *********************/
#[marine]
#[link(wasm_import_module = "ipfsdag")]
extern "C" {
    #[link_name = "put_ipld"]
    pub fn put_ipld(
        content: String,
        previous_cid: String,
        transaction: String,
        api_multiaddr: String,
        timeout_sec: u64,
    ) -> IpfsDagPutResult;

    #[link_name = "get_ipld"]
    pub fn get_ipld(hash: String, api_multiaddr: String, timeout_sec: u64) -> IpfsDagGetResult;

    #[link_name = "put_contract"]
    pub fn put_contract(content: String, api_multiaddr: String, timeout_sec: u64) -> IpfsPutResult;
}

#[marine]
#[link(wasm_import_module = "crypto")]
extern "C" {
    #[link_name = "verify"]
    pub fn verify(public_key: String, signature: String, message: String, enc: String) -> bool;

    #[link_name = "get_public_key_type"]
    pub fn get_public_key_type(public_key: &str) -> String;
}

#[marine]
#[link(wasm_import_module = "host")]
extern "C" {
    pub fn curl(cmd: Vec<String>) -> MountedBinaryResult;
}
