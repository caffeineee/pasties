use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
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

pub fn pseudoid() -> u32 {
    let id = rand::thread_rng().gen::<u32>();
    id
}

pub fn pseudoid_hexstring() -> String {
    let mut string = format!("{:X}", rand::thread_rng().gen::<u32>());
    let string_pad_len = 8 - string.len();
    if string_pad_len != 0 {
        string = (1..=string_pad_len).map(|_| '0').collect::<String>() + string.as_str();
    }
    string
}
