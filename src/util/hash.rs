use sha2::{Digest, Sha256};

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(bytes);
    let hash = sha256.finalize();

    hex::encode(hash)
}
