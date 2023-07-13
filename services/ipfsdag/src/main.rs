#![allow(improper_ctypes)]

mod block;

use marine_rs_sdk::module_manifest;
use marine_rs_sdk::MountedBinaryResult;
use marine_rs_sdk::WasmLoggerBuilder;
use marine_rs_sdk::{get_call_parameters, marine};
use types::{IpfsDagGetResult, IpfsDagPutResult, IpfsPutResult};

use block::{deserialize, serialize};
use eyre::Result;

use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_TIMEOUT_SEC: u64 = 1u64;
const DEFAULT_IPFS_MULTIADDR: &str = "/ip4/127.0.0.1/tcp/5001";

module_manifest!();

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Info)
        .build()
        .unwrap();
}

fn make_cmd_args(args: Vec<String>, api_multiaddr: String, timeout_sec: u64) -> Vec<String> {
    args.into_iter()
        .chain(vec![
            String::from("--timeout"),
            get_timeout_string(timeout_sec),
            String::from("--api"),
            api_multiaddr,
        ])
        .collect()
}

fn unwrap_mounted_binary_result(result: MountedBinaryResult) -> Result<String> {
    result
        .into_std()
        .ok_or(eyre::eyre!(
            "stdout or stderr contains non valid UTF8 string"
        ))?
        .map_err(|e| eyre::eyre!("ipfs cli call failed: {}", e))
}

#[inline]
fn get_timeout_string(timeout: u64) -> String {
    format!("{}s", timeout)
}

/**
 * Store content string as IPLD format
 */
#[marine]
pub fn put_ipld(
    content: String,
    previous_cid: String,
    transaction: String,
    api_multiaddr: String,
    timeout_sec: u64,
) -> IpfsDagPutResult {
    let address: String;
    let t;

    if api_multiaddr.is_empty() {
        address = DEFAULT_IPFS_MULTIADDR.to_string();
    } else {
        address = api_multiaddr;
    }

    if timeout_sec == 0 {
        t = DEFAULT_TIMEOUT_SEC;
    } else {
        t = timeout_sec;
    }

    let block = serialize(content.clone(), previous_cid.clone(), transaction.clone());

    let input;

    if previous_cid.is_empty() {
        input = format!(
            r#"echo '{{"timestamp": {}, "content": {}, "previous": "{{}}", "transaction": {} }}' | ipfs dag put"#,
            block.timestamp, block.content, block.transaction
        );
    } else {
        input = format!(
            r#"echo '{{"timestamp": {}, "content": {}, "previous": {{"/": "{}" }}, "transaction": {} }}' | ipfs dag put"#,
            block.timestamp, block.content, previous_cid, block.transaction
        );
    }

    let args = make_cmd_args(vec![input], address, t);

    let cmd = vec![String::from("-c"), args.join(" ")];

    log::info!("ipfs put args : {:?}", cmd);

    unwrap_mounted_binary_result(bash(cmd))
        .map(|res| res.trim().to_string())
        .into()
}

/**
 * Retrieve IPFS-DAG data using cid
 */
#[marine]
pub fn get_ipld(hash: String, api_multiaddr: String, timeout_sec: u64) -> IpfsDagGetResult {
    let address: String;
    let t;

    if api_multiaddr.is_empty() {
        address = DEFAULT_IPFS_MULTIADDR.to_string();
    } else {
        address = api_multiaddr;
    }

    if timeout_sec == 0 {
        t = DEFAULT_TIMEOUT_SEC;
    } else {
        t = timeout_sec;
    }

    log::info!("get called with hash {}", hash);

    let args = vec![String::from("dag"), String::from("get"), hash];

    let cmd = make_cmd_args(args, address, t);

    log::info!("ipfs dag get args {:?}", cmd);

    unwrap_mounted_binary_result(ipfs(cmd))
        .map(|res| res.trim().to_string())
        .into()
}

/**
 * Put bytecode to IPFS
 * to make it work in ipfs-cli, convert data to base64 and pipe it to `ipfs add`
 */
#[marine]
pub fn put(content: String, api_multiaddr: String, timeout_sec: u64) -> IpfsPutResult {
    let address;

    let t;

    if api_multiaddr.is_empty() {
        address = DEFAULT_IPFS_MULTIADDR.to_string();
    } else {
        address = api_multiaddr;
    }

    if timeout_sec == 0 {
        t = DEFAULT_TIMEOUT_SEC;
    } else {
        t = timeout_sec;
    }

    let file = vault_dir().join("raw".to_string());
    let file_str = file.to_string_lossy().to_string();

    let result: Result<_, _>;

    if is_base64(&content) {
        let decode_content = base64::decode(content.clone()).unwrap();
        result = fs::write(PathBuf::from(&file), decode_content);
    } else {
        result = fs::write(PathBuf::from(&file), content.clone());
    }

    if let Err(e) = result {
        log::info!("error: {:?}", e);
        return IpfsPutResult {
            success: false,
            error: format!("file can't be written: {}", e),
            cid: "".to_string(),
        };
    };
    
    let input = format!("ipfs add {}", "tmp/vault/raw");

    let args = make_cmd_args(vec![input], address, t);

    let cmd = vec![String::from("-c"), args.join(" ")];

    log::info!("ipfs put args : {:?}", cmd);

    unwrap_mounted_binary_result(bash(cmd))
        .map(|res| res.trim().to_string())
        .into()
}

fn vault_dir() -> PathBuf {
    let particle_id = get_call_parameters().particle_id;
    let vault = Path::new("/tmp").join("vault");

    vault
}

fn inject_vault_host_path(path: String) -> String {
    let vault = "/tmp/vault";
    if let Some(stripped) = path.strip_prefix(&vault) {
        let host_vault_path = std::env::var(vault).expect("vault must be mapped to /tmp/vault");
        log::info!("host vault: {} {}", host_vault_path, stripped);
        format!("/{}{}", host_vault_path, stripped)
    } else {
        path
    }
}

fn is_base64(input: &str) -> bool {
    // Attempt to decode the input string
    match base64::decode(input) {
        Ok(_) => true,   // The string is Base64 encoded
        Err(_) => false, // The string is not Base64 encoded
    }
}

#[marine]
#[link(wasm_import_module = "host")]
extern "C" {
    /// Execute provided cmd as a parameters of ipfs cli, return result.
    pub fn ipfs(cmd: Vec<String>) -> MountedBinaryResult;

    /// Execute command, return result
    pub fn bash(cmd: Vec<String>) -> MountedBinaryResult;

    fn curl(cmd: Vec<String>) -> MountedBinaryResult;
}
