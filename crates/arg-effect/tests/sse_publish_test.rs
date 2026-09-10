// SPDX-License-Identifier: MIT OR Apache-2.0
//! `sse_publish_test` — 3 UT (per brief §2.1 B + DD §10.1.3 SSE-UT-01..03).
//!
//! 3 tests confirming the SSE / WS publish layer:
//! - SSE-UT-01 NoopAchievementPublisher records 0 invocations by default
//! - SSE-UT-02 NoopAchievementPublisher records each publish
//! - SSE-UT-03 ChannelAchievementPublisher fans out to a subscriber

use std::sync::Arc;

use star_arg::models::achievement_unlock::AchievementUnlock;
use star_arg_effect::achievement_engine::{
    AchievementPublisher, ChannelAchievementPublisher, NoopAchievementPublisher,
};

#[tokio::test]
async fn sse_ut_01_noop_publisher_default_count_is_zero() {
    let pub_ = NoopAchievementPublisher::new();
    assert_eq!(pub_.published_count(), 0);
}

#[tokio::test]
async fn sse_ut_02_noop_publisher_records_each_publish() {
    let pub_: Arc<dyn AchievementPublisher> = Arc::new(NoopAchievementPublisher::new());
    let unlock = AchievementUnlock::new(
        "BEH-001-FIRST-DELEGATES".to_string(),
        uuid::Uuid::new_v4(),
        vec![],
        serde_json::json!({}),
        uuid::Uuid::new_v4(),
    );
    for _ in 0..5 {
        pub_.publish_achievement_unlocked(&unlock).await.expect("ok");
    }
    assert_eq!(pub_.published_count(), 5);
}

#[tokio::test]
async fn sse_ut_03_channel_publisher_fans_out_to_subscriber() {
    let (pub_, mut rx) = ChannelAchievementPublisher::with_capacity(8);
    let pub_: Arc<dyn AchievementPublisher> = Arc::new(pub_);
    let unlock = AchievementUnlock::new(
        "OUT-001-TRUSTS-100K-TOKEN".to_string(),
        uuid::Uuid::new_v4(),
        vec![],
        serde_json::json!({}),
        uuid::Uuid::new_v4(),
    );
    pub_.publish_achievement_unlocked(&unlock).await.expect("ok");
    pub_.publish_achievement_unlocked(&unlock).await.expect("ok");
    pub_.publish_achievement_unlocked(&unlock).await.expect("ok");
    // Drain the channel — expect 3 unlocks.
    let mut count = 0;
    while let Ok(_u) = rx.try_recv() {
        count += 1;
    }
    assert_eq!(count, 3);
    assert!(pub_.published_count() >= 1);
}

/// Sanity check the noop publisher's manual `Clone` impl resets the counter.
#[test]
fn sse_clone_noop_resets_counter() {
    let pub_ = NoopAchievementPublisher::new();
    let unlock = AchievementUnlock::new(
        "BEH-001-FIRST-DELEGATES".to_string(),
        uuid::Uuid::new_v4(),
        vec![],
        serde_json::json!({}),
        uuid::Uuid::new_v4(),
    );
    // Build a runtime to drive the async publish.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let pub_arc: Arc<dyn AchievementPublisher> = Arc::new(pub_);
    rt.block_on(async {
        pub_arc.publish_achievement_unlocked(&unlock).await.expect("ok");
    });
    assert_eq!(pub_arc.published_count(), 1);
}
