use anyhow::Result;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn generate_api_key() -> String {
    format!("llmp_{}", Uuid::new_v4().to_string().replace("-", ""))
}

pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn get_key_prefix(key: &str) -> String {
    key.chars().take(12).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key();
        assert!(key.starts_with("llmp_"));
        assert!(key.len() > 10);
    }

    #[test]
    fn test_hash_api_key() {
        let key = "test_key";
        let hash = hash_api_key(key);
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex chars
    }

    #[test]
    fn test_get_key_prefix() {
        let key = "llmp_1234567890abcdef";
        let prefix = get_key_prefix(key);
        assert_eq!(prefix, "llmp_1234567");
    }
}
