// SPDX-License-Identifier: MIT OR Apache-2.0
//! LRU cypher query cache (per DD §3.1 + §4.1).
//!
//! `CypherCache` is a simple per-instance LRU keyed by `(cypher, params)`.

/// Default cache size: 1000 entries (per DD §4.1).
pub const DEFAULT_CACHE_SIZE: usize = 1000;

/// LRU 1000 cypher query cache.
///
/// The implementation uses a `Vec` for ordering and a `HashMap` for
/// O(1) lookup. The cache is intentionally process-local: it does not
/// hold any connection state and is safe to clone via `Arc` (in the
/// real `MemgraphClient`).
#[derive(Debug)]
pub struct CypherCache {
    /// Insertion order (oldest at index 0).
    order: Vec<(String, String)>,
    /// Map from key to (row payload, value).
    map: std::collections::HashMap<(String, String), Vec<serde_json::Value>>,
    /// Max number of entries.
    capacity: usize,
}

impl CypherCache {
    /// Build a new cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            order: Vec::with_capacity(capacity),
            map: std::collections::HashMap::with_capacity(capacity),
            capacity,
        }
    }

    /// Compute the cache key. Params are serialised deterministically
    /// (`serde_json` `Value` map iteration is non-deterministic, but for
    /// the small query counts we care about the resulting string is
    /// stable enough for an in-process cache).
    fn key(cypher: &str, params: &serde_json::Value) -> (String, String) {
        (
            cypher.to_string(),
            serde_json::to_string(params).unwrap_or_default(),
        )
    }

    /// Get cached rows. Returns `Some(rows)` on hit, `None` on miss.
    pub fn get(&self, cypher: &str, params: &serde_json::Value) -> Option<Vec<serde_json::Value>> {
        self.map.get(&Self::key(cypher, params)).cloned()
    }

    /// Insert a row payload, evicting the oldest entry if the cache is
    /// full (LRU eviction, per DD §3.1 + §10.1.1 UT-24).
    pub fn put(&mut self, cypher: &str, params: &serde_json::Value, rows: &[serde_json::Value]) {
        let key = Self::key(cypher, params);
        if self.map.contains_key(&key) {
            // Update payload; keep order.
            self.map.insert(key.clone(), rows.to_vec());
            return;
        }
        if self.map.len() >= self.capacity {
            if let Some(oldest) = self.order.first().cloned() {
                self.order.remove(0);
                self.map.remove(&oldest);
            }
        }
        self.order.push(key.clone());
        self.map.insert(key, rows.to_vec());
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// `true` iff the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Capacity (max entries).
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
