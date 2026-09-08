// SPDX-License-Identifier: MIT OR Apache-2.0
//! `trust_score_test` — 4 UT (5 档 boundary, per DD §3.3.3 + §10.1.1).

use star_arg::models::trust_score::{update_trust_score, TrustScoreTier};

#[test]
fn ut25_trust_score_5_tier_boundaries() {
    // Bucket boundaries: 0.0 / 0.2 / 0.4 / 0.7 / 0.9 / 1.0
    assert_eq!(TrustScoreTier::from_score(0.0), TrustScoreTier::Untrusted);
    assert_eq!(TrustScoreTier::from_score(0.19), TrustScoreTier::Untrusted);
    assert_eq!(TrustScoreTier::from_score(0.2), TrustScoreTier::Low);
    assert_eq!(TrustScoreTier::from_score(0.39), TrustScoreTier::Low);
    assert_eq!(TrustScoreTier::from_score(0.4), TrustScoreTier::Medium);
    assert_eq!(TrustScoreTier::from_score(0.69), TrustScoreTier::Medium);
    assert_eq!(TrustScoreTier::from_score(0.7), TrustScoreTier::High);
    assert_eq!(TrustScoreTier::from_score(0.89), TrustScoreTier::High);
    assert_eq!(TrustScoreTier::from_score(0.9), TrustScoreTier::VeryHigh);
    assert_eq!(TrustScoreTier::from_score(1.0), TrustScoreTier::VeryHigh);
}

#[test]
fn ut26_trust_score_tier_ranges() {
    assert_eq!(TrustScoreTier::Untrusted.to_score_range(), (0.0, 0.2));
    assert_eq!(TrustScoreTier::Low.to_score_range(), (0.2, 0.4));
    assert_eq!(TrustScoreTier::Medium.to_score_range(), (0.4, 0.7));
    assert_eq!(TrustScoreTier::High.to_score_range(), (0.7, 0.9));
    assert_eq!(TrustScoreTier::VeryHigh.to_score_range(), (0.9, 1.0));
}

#[test]
fn ut27_trust_score_update_clamps_and_delta() {
    // Success path: +0.01, clamped at 1.0
    assert!((update_trust_score(0.99, true) - 1.0).abs() < 1e-6);
    assert!((update_trust_score(0.5, true) - 0.51).abs() < 1e-6);
    // Failure path: -0.05, clamped at 0.0
    assert!((update_trust_score(0.04, false) - 0.0).abs() < 1e-6);
    assert!((update_trust_score(0.5, false) - 0.45).abs() < 1e-6);
}

#[test]
fn ut28_trust_score_5_tier_ordering() {
    // Tiers are PartialOrd: Untrusted < Low < Medium < High < VeryHigh
    assert!(TrustScoreTier::Untrusted < TrustScoreTier::Low);
    assert!(TrustScoreTier::Low < TrustScoreTier::Medium);
    assert!(TrustScoreTier::Medium < TrustScoreTier::High);
    assert!(TrustScoreTier::High < TrustScoreTier::VeryHigh);
}
