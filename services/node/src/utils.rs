use sha2::{Digest, Sha256};

pub fn hasher(content: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}", content).as_bytes());

    bs58::encode(hasher.finalize()).into_string()
}
