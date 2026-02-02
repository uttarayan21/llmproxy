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
        assert_eq!(key.len(), 37); // "llmp_" (5) + UUID without hyphens (32)
    }

    #[test]
    fn test_generate_api_key_uniqueness() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2, "Generated keys should be unique");
    }

    #[test]
    fn test_hash_api_key() {
        let key = "test_key";
        let hash = hash_api_key(key);
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex chars
    }

    #[test]
    fn test_hash_api_key_consistency() {
        let key = "llmp_test123";
        let hash1 = hash_api_key(key);
        let hash2 = hash_api_key(key);
        assert_eq!(hash1, hash2, "Same key should produce same hash");
    }

    #[test]
    fn test_hash_api_key_different_inputs() {
        let key1 = "llmp_test123";
        let key2 = "llmp_test456";
        let hash1 = hash_api_key(key1);
        let hash2 = hash_api_key(key2);
        assert_ne!(
            hash1, hash2,
            "Different keys should produce different hashes"
        );
    }

    #[test]
    fn test_get_key_prefix() {
        let key = "llmp_1234567890abcdef";
        let prefix = get_key_prefix(key);
        assert_eq!(prefix, "llmp_1234567");
    }

    #[test]
    fn test_get_key_prefix_short_key() {
        let key = "llmp_12";
        let prefix = get_key_prefix(key);
        assert_eq!(prefix, "llmp_12");
    }

    #[test]
    fn test_get_key_prefix_empty_key() {
        let key = "";
        let prefix = get_key_prefix(key);
        assert_eq!(prefix, "");
    }
}
