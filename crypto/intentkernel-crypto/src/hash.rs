use sha2::{Digest, Sha384};

pub fn sha384(data: &[u8]) -> [u8; 48] {
    let mut hasher = Sha384::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn context_hash(action: &[u8]) -> [u8; 48] {
    sha384(action)
}