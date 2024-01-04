use sha1::{Sha1, Digest};

pub fn hash<T: AsRef<[u8]>>(data: T) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(&data);
    hasher.finalize().to_vec()
}
