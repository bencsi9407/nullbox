//! Content-addressed entry type for ctxgraph.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A single entry in the ctxgraph store.
/// Content-addressed: the hash is derived from (agent_id, key, value).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub hash: String,
    pub agent_id: String,
    pub key: String,
    pub value: serde_json::Value,
    pub timestamp: i64,
}

/// Compute the content-address hash for an entry.
/// Hash = SHA-256(agent_id || "\0" || key || "\0" || canonical_json(value))
pub fn compute_hash(
    agent_id: &str,
    key: &str,
    value: &serde_json::Value,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(agent_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(key.as_bytes());
    hasher.update(b"\0");
    // Canonical JSON: sorted keys, no whitespace
    let json_bytes = canonical_json(value);
    hasher.update(&json_bytes);
    format!("{:x}", hasher.finalize())
}

/// Produce canonical JSON bytes with sorted object keys for deterministic hashing.
fn canonical_json(value: &serde_json::Value) -> Vec<u8> {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted: Vec<_> = map.iter().collect();
            sorted.sort_by_key(|(k, _)| *k);
            let sorted_map: serde_json::Map<String, serde_json::Value> =
                sorted.into_iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            serde_json::to_vec(&serde_json::Value::Object(sorted_map)).unwrap_or_default()
        }
        _ => serde_json::to_vec(value).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hash_is_deterministic() {
        let h1 = compute_hash("agent-1", "key", &json!("value"));
        let h2 = compute_hash("agent-1", "key", &json!("value"));
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_agents_produce_different_hashes() {
        let h1 = compute_hash("agent-1", "key", &json!("value"));
        let h2 = compute_hash("agent-2", "key", &json!("value"));
        assert_ne!(h1, h2);
    }

    #[test]
    fn different_keys_produce_different_hashes() {
        let h1 = compute_hash("agent-1", "key-a", &json!("value"));
        let h2 = compute_hash("agent-1", "key-b", &json!("value"));
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_is_64_hex_chars() {
        let hash = compute_hash("a", "b", &json!(null));
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
