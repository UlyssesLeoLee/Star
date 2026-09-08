// SPDX-License-Identifier: MIT OR Apache-2.0
//! `cypher_cache_test` — 2 UT (per DD §10.1.1 UT-24 + LRU eviction).

use star_arg::client::CypherCache;

#[test]
fn ut29_cypher_cache_hit_and_miss() {
    let mut cache = CypherCache::new(2);
    assert!(cache.is_empty());
    let params = serde_json::json!({"tenant_id": "t1"});
    let rows = vec![serde_json::json!({"id": 1})];
    assert!(cache.get("MATCH (a) RETURN a", &params).is_none());
    cache.put("MATCH (a) RETURN a", &params, &rows);
    let got = cache.get("MATCH (a) RETURN a", &params).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0]["id"], 1);
}

#[test]
fn ut30_cypher_cache_lru_eviction() {
    let mut cache = CypherCache::new(2);
    let p = serde_json::json!({});
    cache.put("Q1", &p, &[serde_json::json!({"k": 1})]);
    cache.put("Q2", &p, &[serde_json::json!({"k": 2})]);
    assert_eq!(cache.len(), 2);
    // Third insert evicts the oldest (Q1).
    cache.put("Q3", &p, &[serde_json::json!({"k": 3})]);
    assert_eq!(cache.len(), 2);
    assert!(cache.get("Q1", &p).is_none());
    assert!(cache.get("Q2", &p).is_some());
    assert!(cache.get("Q3", &p).is_some());
    assert_eq!(cache.capacity(), 2);
}
