// SPDX-License-Identifier: MIT OR Apache-2.0
//! `achievement_test` — 6 UT (per DD §10.1.1 UT-19..UT-24 part).
//!
//! Covers 8+7+5 = 20 achievements and the rarity distribution
//! (8 Common + 7 Rare + 3 Epic + 2 Legendary, per DD §11.4).

use star_arg::models::achievement::{
    all_achievements, count_by_category, AchievementCategory, Rarity,
};

#[test]
fn ut19_achievement_topology_count_is_8() {
    assert_eq!(count_by_category(AchievementCategory::Topology), 8);
}

#[test]
fn ut20_achievement_behavior_count_is_7() {
    assert_eq!(count_by_category(AchievementCategory::Behavior), 7);
}

#[test]
fn ut21_achievement_output_count_is_5() {
    assert_eq!(count_by_category(AchievementCategory::Output), 5);
}

#[test]
fn ut22_achievement_rarity_distribution() {
    let all = all_achievements();
    let mut common = 0;
    let mut rare = 0;
    let mut epic = 0;
    let mut legendary = 0;
    for a in &all {
        match a.rarity {
            Rarity::Common => common += 1,
            Rarity::Rare => rare += 1,
            Rarity::Epic => epic += 1,
            Rarity::Legendary => legendary += 1,
        }
    }
    // Per DD §11.4: 8C + 7R + 3E + 2L
    assert_eq!(common, 8);
    assert_eq!(rare, 7);
    assert_eq!(epic, 3);
    assert_eq!(legendary, 2);
    assert_eq!(all.len(), 20);
}

#[test]
fn ut23_achievement_total_20() {
    let all = all_achievements();
    assert_eq!(all.len(), 20);
    // The codes are stable — they are referenced by the topology Cypher
    // queries and the behavior / output metric tables.
    let codes: Vec<&str> = all.iter().map(|a| a.code.as_str()).collect();
    assert!(codes.contains(&"TOP-001-MESH-5DOMAIN"));
    assert!(codes.contains(&"TOP-008-NO-ISLAND"));
    assert!(codes.contains(&"BEH-001-FIRST-EDGE"));
    assert!(codes.contains(&"BEH-007-MENTOR-CHAIN"));
    assert!(codes.contains(&"OUT-001-PEER-REVIEW-80"));
    assert!(codes.contains(&"OUT-005-LEGENDARY-AGENT"));
}

#[tokio::test]
async fn ut24_achievement_unlock_is_idempotent() {
    // We don't need a real Memgraph — the AchievementOps uses an
    // in-process idempotency cache.
    use star_arg::models::achievement_unlock::AchievementUnlock;
    use star_arg::ops::achievement_ops::AchievementOps;
    use star_arg::ops::event_writer::EventWriter;
    use uuid::Uuid;

    let writer = EventWriter::new();
    let ops = AchievementOps::new(writer);
    let tenant = Uuid::new_v4();
    let user = Uuid::new_v4();

    let first = AchievementUnlock::new(
        "TOP-001-MESH-5DOMAIN".into(),
        user,
        vec![Uuid::new_v4()],
        serde_json::json!({}),
        tenant,
    );
    let second = first.clone();

    let r1 = ops.unlock(first).await.unwrap();
    let r2 = ops.unlock(second).await.unwrap();
    assert!(r1);
    assert!(!r2);
    assert_eq!(ops.unique_unlocks(), 1);
}
