use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

pub fn unix_timestamp() -> u64 {
    let now = SystemTime::now();
    let since_epoch = now
        .duration_since(UNIX_EPOCH)
        .expect("Time travel is not allowed");
    since_epoch.as_secs()
}

pub fn hash_string(input: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.into_bytes());
    let hash = hasher.finalize();
    format!("{:x}", hash)
}
