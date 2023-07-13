use crate::block::Block;
use crate::cron::{Cron, SerdeCron};
use crate::data_types::DataTypeClone;
use crate::defaults::{
    CRON_ACTION_CREATE, CRON_ACTION_UPDATE, CRON_ACTION_UPDATE_STATUS, CRON_STATUS_ACTIVE,
};
use crate::metadatas::{FinalMetadata, Metadata};
use crate::transaction::TransactionSubset;
use crate::{defaults::STATUS_FAILED, defaults::STATUS_SUCCESS};
use crate::{error::ServiceError, error::ServiceError::*};
use crate::{get_ipld, put, put_ipld};
use crate::{meta_contract::MetaContract, storage_impl::get_storage};

/**
 * Validated meta contract method type
 * Upon success, upload the contract to IPFS
 */
pub fn validate_meta_contract(transaction_hash: String) {
    let error: Option<ServiceError> = None;

    let storage = get_storage().expect("Internal error to database connector");

    // Get data
    let mut transaction = storage.get_transaction(transaction_hash).unwrap().clone();

    // push the data to ipfs
    let result = put(transaction.data, "".to_string(), 0);

    log::info!("putting file to ipfs: {}", result.cid);

    // update metadata table
    let meta_contract = MetaContract {
        program_id: transaction.program_id,
        cid: result.cid,
        public_key: transaction.public_key,
    };

    let meta_result = storage.write_meta_contract(meta_contract);

    log::info!("{:?}", meta_result);

    if !error.is_none() {
        transaction.error_text = error.unwrap().to_string();
        transaction.status = STATUS_FAILED;
    } else {
        transaction.status = STATUS_SUCCESS;
        transaction.error_text = "".to_string();
    }

    let _ = storage.update_transaction_status(
        transaction.hash.clone(),
        transaction.status.clone(),
        transaction.error_text.clone(),
    );
}

pub fn validate_metadata(
    transaction_hash: String,
    meta_contract_id: String,
    on_metacontract_result: bool,
    metadatas: Vec<FinalMetadata>,
    final_error_msg: String,
) {
}
pub fn validate_metadata_cron(
    data_key: String,
    on_metacontract_result: bool,
    metadatas: Vec<FinalMetadata>,
) {
}

pub fn validate_clone(
    transaction_hash: String,
    meta_contract_id: String,
    on_metacontract_result: bool,
    data: String,
    final_error_msg: String,
) {
}

pub fn validate_cron(transaction_hash: String, data: String) {}

// /**
//  * Validated "metadata" method type
//  */
// pub fn validate_metadata(
//     transaction_hash: String,
//     meta_contract_id: String,
//     on_metacontract_result: bool,
//     metadatas: Vec<FinalMetadata>,
//     final_error_msg: String,
// ) {
//     let storage = get_storage().expect("Internal error to database connector");
//     let mut transaction = storage.get_transaction(transaction_hash).unwrap().clone();

//     if !on_metacontract_result {
//         transaction.status = STATUS_FAILED;
//         if final_error_msg.is_empty() {
//             transaction.error_text = "Metadata not updateable".to_string();
//         } else {
//             transaction.error_text = final_error_msg;
//         }
//     } else {
//         for data in metadatas {
//             let result = storage.get_owner_metadata(
//                 transaction.data_key.clone(),
//                 data.program_id.clone(),
//                 data.public_key.clone(),
//                 data.alias.clone(),
//                 data.version.clone(),
//             );

//             log::info!("{:?}", result);

//             match result {
//                 Ok(metadata) => {
//                     transaction.status = STATUS_SUCCESS;

//                     let tx = TransactionSubset {
//                         hash: transaction.hash.clone(),
//                         timestamp: transaction.timestamp.clone(),
//                         meta_contract_id: meta_contract_id.clone(),
//                         method: transaction.method.clone(),
//                         value: "".to_string(),
//                     };

//                     let tx_serde = serde_json::to_string(&tx).unwrap();

//                     let result_ipfs_dag_put =
//                         put_block(data.content, metadata.cid, tx_serde, "".to_string(), 0);
//                     let content_cid = result_ipfs_dag_put.cid;

//                     let _ = storage.update_cid(
//                         metadata.data_key,
//                         metadata.alias,
//                         metadata.public_key,
//                         content_cid,
//                     );
//                 }
//                 Err(ServiceError::RecordNotFound(_)) => {
//                     transaction.status = STATUS_SUCCESS;

//                     let tx = TransactionSubset {
//                         hash: transaction.hash.clone(),
//                         timestamp: transaction.timestamp.clone(),
//                         meta_contract_id: meta_contract_id.clone(),
//                         method: transaction.method.clone(),
//                         value: "".to_string(),
//                     };

//                     let tx_serde = serde_json::to_string(&tx).unwrap();

//                     let result_ipfs_dag_put =
//                         put_block(data.content, "".to_string(), tx_serde, "".to_string(), 0);
//                     let content_cid = result_ipfs_dag_put.cid;

//                     let metadata = Metadata::new(
//                         transaction.data_key.clone(),
//                         data.alias.clone(),
//                         content_cid,
//                         data.public_key.clone(),
//                     );

//                     let _ = storage.write_metadata(metadata);
//                 }
//                 Err(e) => {
//                     transaction.error_text = e.to_string();
//                     transaction.status = STATUS_FAILED;
//                 }
//             };
//         }
//     }

//     let _ = storage.update_transaction_status(
//         transaction.hash.clone(),
//         transaction.status.clone(),
//         transaction.error_text.clone(),
//     );
// }

// /**
//  * Validated "metadata cron" method type
//  */
// pub fn validate_metadata_cron(
//     data_key: String,
//     on_metacontract_result: bool,
//     metadatas: Vec<FinalMetadata>,
// ) {
//     let storage = get_storage().expect("Internal error to database connector");

//     if on_metacontract_result {
//         for data in metadatas {
//             let result = storage.get_owner_metadata_by_datakey_and_alias(
//                 data_key.clone(),
//                 data.public_key.clone(),
//                 data.alias.clone(),
//             );

//             log::info!("{:?}", result);

//             match result {
//                 Ok(_) => {}
//                 Err(ServiceError::RecordNotFound(_)) => {
//                     let result_ipfs_dag_put = put_block(
//                         data.content,
//                         "".to_string(),
//                         "{}".to_string(),
//                         "".to_string(),
//                         0,
//                     );
//                     let content_cid = result_ipfs_dag_put.cid;

//                     let metadata = Metadata::new(
//                         data_key.clone(),
//                         data.alias.clone(),
//                         content_cid,
//                         data.public_key.clone(),
//                     );

//                     let _ = storage.write_metadata(metadata);
//                 }
//                 Err(_) => {}
//             };
//         }
//     }
// }

// /**
//  * Validated "clone" method type
//  * Fetch the origin metadata content from Block and clone it to the new metadata
//  */
// pub fn validate_clone(
//     transaction_hash: String,
//     meta_contract_id: String,
//     on_metacontract_result: bool,
//     data: String,
//     final_error_msg: String,
// ) {
//     let storage = get_storage().expect("Internal error to database connector");
//     let mut transaction = storage.get_transaction(transaction_hash).unwrap().clone();

//     if !on_metacontract_result {
//         transaction.status = STATUS_FAILED;
//         if final_error_msg.is_empty() {
//             transaction.error_text = "Metadata not forkable".to_string();
//         } else {
//             transaction.error_text = final_error_msg;
//         }
//     } else {
//         let data_clone: DataTypeClone = serde_json::from_str(&data.clone()).unwrap();

//         let origin_metadata = storage
//             .get_owner_metadata_by_datakey_and_alias(
//                 data_clone.origin_data_key.clone(),
//                 data_clone.origin_public_key.clone(),
//                 data_clone.origin_alias.clone(),
//             )
//             .unwrap();

//         let tx = TransactionSubset {
//             hash: transaction.hash.clone(),
//             timestamp: transaction.timestamp.clone(),
//             meta_contract_id: meta_contract_id.clone(),
//             method: transaction.method.clone(),
//             value: serde_json::to_string(&data_clone).unwrap(),
//         };

//         let tx_serde = serde_json::to_string(&tx).unwrap();

//         let ipfs_get_result = get(origin_metadata.cid, "".to_string(), 0);

//         let block: Block = serde_json::from_str(&ipfs_get_result.block).unwrap();
//         let content = block.content;

//         let result_ipfs_dag_put = put_block(
//             serde_json::to_string(&content).unwrap(),
//             "".to_string(),
//             tx_serde,
//             "".to_string(),
//             0,
//         );

//         let metadata = Metadata::new(
//             transaction.data_key.clone(),
//             origin_metadata.alias.clone(),
//             result_ipfs_dag_put.cid,
//             origin_metadata.public_key.clone(),
//         );

//         let _ = storage.write_metadata(metadata);

//         transaction.status = STATUS_SUCCESS;
//     }

//     let _ = storage.update_transaction_status(
//         transaction.hash.clone(),
//         transaction.status.clone(),
//         transaction.error_text.clone(),
//     );
// }

// /**
//  * Validated "cron" method type
//  */
// pub fn validate_cron(transaction_hash: String, data: String) {
//     let mut status = STATUS_SUCCESS;
//     let mut error_text = "".to_string();

//     let storage = get_storage().expect("Internal error to database connector");
//     let mut transaction = storage.get_transaction(transaction_hash).unwrap().clone();

//     let serde_cron: SerdeCron = serde_json::from_str(&data).unwrap();

//     let result = storage.search_cron(
//         serde_cron.address.clone(),
//         serde_cron.chain.clone(),
//         serde_cron.topic.clone(),
//     );

//     let mut cron = Cron::new(
//         transaction.token_key.clone(),
//         serde_cron.address,
//         serde_cron.topic,
//         serde_cron.token_type,
//         serde_cron.chain,
//         serde_cron.status,
//         serde_cron.meta_contract_id,
//         serde_cron.node_url,
//         transaction.public_key.clone(),
//     );

//     match result {
//         Ok(row) => {
//             if transaction.public_key == row.public_key {
//                 if !serde_cron.hash.is_empty() {
//                     match serde_cron.action.as_str() {
//                         CRON_ACTION_UPDATE => {
//                             let _ = storage.update_cron(serde_cron.hash, cron);
//                         }
//                         CRON_ACTION_UPDATE_STATUS => {
//                             let _ = storage.update_cron_status(serde_cron.hash, serde_cron.status);
//                         }
//                         _ => {
//                             status = STATUS_FAILED;
//                             error_text = "Invalid cron action".to_string();
//                         }
//                     }
//                 } else {
//                     status = STATUS_FAILED;
//                     error_text = format!("Invalid cron hash: {}", serde_cron.hash);
//                 }
//             } else {
//                 status = STATUS_FAILED;
//                 error_text = format!("Invalid owner: {}", transaction.public_key);
//             }
//         }
//         Err(ServiceError::RecordNotFound(_)) => {
//             if serde_cron.action == CRON_ACTION_CREATE {
//                 cron.status = CRON_STATUS_ACTIVE;

//                 let _ = storage.write_cron(cron);
//             } else {
//                 status = STATUS_FAILED;
//                 error_text = "Invalid cron action".to_string();
//             }
//         }
//         Err(_) => {
//             status = STATUS_FAILED;
//             error_text = "Invalid cron".to_string();
//         }
//     }

//     transaction.status = status;
//     transaction.error_text = error_text;

//     let _ = storage.update_transaction_status(
//         transaction.hash.clone(),
//         transaction.status.clone(),
//         transaction.error_text.clone(),
//     );
// }
