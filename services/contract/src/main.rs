use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::WasmLoggerBuilder;
use types::{IpfsGetResult, MetaContractResult};

module_manifest!();

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Info)
        .build()
        .unwrap();
}

/**
 * upon starting up, read WASM bytecode form
 */
#[marine]
pub fn execute_contract(cid: String) -> MetaContractResult {
    let mut error: String = "".to_string();
    let result = get_contract(cid, "".to_string(), 0);
    if !result.success {
        error = "Internal Error".to_string();
    }

    MetaContractResult {
        result: true,
        metadatas: Vec::new(),
        error_string: error,
    }
}

#[marine]
#[link(wasm_import_module = "ipfsdag")]
extern "C" {
    #[link_name = "get_contract"]
    pub fn get_contract(cid: String, api_multiaddr: String, timeout_sec: u64) -> IpfsGetResult;
}
