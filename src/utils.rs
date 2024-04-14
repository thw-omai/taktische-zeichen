use std::fs::File;
use std::io;
use sha2::{Digest, Sha256};

pub(crate) fn calc_hash(file_name: &str) -> String {
    let mut file = File::open(file_name).unwrap();
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).expect("TODO: panic message");
    let hash_bytes = hasher.finalize();
    let final_hash = format!("{:X}", hash_bytes);
    final_hash
}