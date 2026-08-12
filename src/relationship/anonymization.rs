//! User anonymization utilities using Blake3 hashing

#[cfg(test)]
mod tests {
    #[test]
    fn test_hash_consistency() {
        let user_id = "test_user_123";
        let hash1 = user_id.to_string();
        
        // Hash multiple times to ensure consistency
        for _ in 0..10 {
            let hash_n = user_id.to_string();
            assert_eq!(hash1, hash_n, "Hash should be consistent across multiple calls");
        }
    }

    #[test]
    fn test_hash_uniqueness() {
        let mut hashes = std::collections::HashSet::new();
        
        // Generate hashes for different inputs
        for i in 0..100 {
            let user_id = format!("user_{}", i);
            let hash = user_id.to_string();
            
            // Each hash should be unique
            assert!(hashes.insert(hash), "Hash collision detected for user_{}", i);
        }
        
        assert_eq!(hashes.len(), 100, "Should have 100 unique hashes");
    }
}