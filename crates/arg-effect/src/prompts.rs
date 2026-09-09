// SPDX-License-Identifier: MIT OR Apache-2.0
//! 10 套 challenges 双向论证 prompt 模板 (per DD-AGENT-RELATIONSHIP-001 §7).
//!
//! 5 decision_type × 2 trust_tier = 10 套 (per brief §2.1 C + DD §7).
//! Each entry is a [`ChallengePrompt`] (already in `star_arg::models::peer_review`)
//! containing a `self_justify_prompt` (B 自证) and an `evaluate_prompt`
//! (A 接受 / reject / escalate).
//!
//! The matrix is indexed by `(DecisionType, TrustTier)` and is exposed via
//! [`CHALLENGE_PROMPTS`] for the [`crate::output_evaluator::ARGOutputEvaluator`].

use std::collections::HashMap;

use star_arg::models::decision::DecisionType;
use star_arg::models::peer_review::ChallengePrompt;

/// 2 trust tiers (per brief + DD §7). Per the brief we use 2 tiers, not
/// the 3 in the WBS draft — the WBS draft treats HIGH+VERY_HIGH as one
/// tier to land at 10 (= 5 × 2) prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustTier {
    /// `[0.0, 0.7)` — Low / Medium. Stricter justification.
    Low,
    /// `[0.7, 1.0]` — High / VeryHigh. Lighter justification.
    High,
}

impl TrustTier {
    /// Bucket a weight (the challenges-edge `weight`, in `[0.0, 1.0]`)
    /// into a 2-tier enum.
    pub fn from_weight(weight: f32) -> Self {
        if weight < 0.7 {
            Self::Low
        } else {
            Self::High
        }
    }

    /// Stable string label used in log / metric labels.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::High => "high",
        }
    }
}

/// Build the 10 套 challenges prompt templates (per DD §7).
///
/// Returns a `HashMap` keyed by `(DecisionType, TrustTier)`. The brief
/// §2.1 A and DD §7 mandate exactly 10 entries (5 × 2).
pub fn challenge_prompts() -> HashMap<(DecisionType, TrustTier), ChallengePrompt> {
    let mut m: HashMap<(DecisionType, TrustTier), ChallengePrompt> = HashMap::with_capacity(10);

    // ====== Architectural (2 套) ======
    m.insert(
        (DecisionType::Architectural, TrustTier::Low),
        ChallengePrompt::new(
            "You are agent {from_agent}, and you are being challenged by agent {to_agent} on your architectural decision.\nDecision: {decision}\nContext: {context}\n\nProvide a detailed justification covering:\n1. Why this architectural choice over alternatives\n2. Trade-offs considered\n3. Risk mitigation strategy\n4. Reversibility plan\n\nKeep response under 500 words. Be concise and direct."
                .into(),
            "You are agent {to_agent}, evaluating {from_agent}'s architectural justification.\nDecision: {decision}\nJustification: {justification}\n\nEvaluate based on:\n1. Soundness of reasoning\n2. Completeness of trade-off analysis\n3. Risk awareness\n4. Reversibility\n\nReturn JSON: {\"verdict\": \"accept\"|\"reject\"|\"escalate\", \"reason\": \"...\"}"
                .into(),
        ),
    );
    m.insert(
        (DecisionType::Architectural, TrustTier::High),
        ChallengePrompt::new(
            "You are agent {from_agent}, and you are being challenged by agent {to_agent} on your architectural decision.\nDecision: {decision}\n\nProvide a brief justification (under 200 words) covering: alternatives considered + reversibility plan."
                .into(),
            "You are agent {to_agent}, evaluating {from_agent}'s brief architectural justification.\nDecision: {decision}\nJustification: {justification}\n\nReturn JSON: {\"verdict\": \"accept\"|\"reject\"|\"escalate\", \"reason\": \"...\"}"
                .into(),
        ),
    );

    // ====== Business (2 套) ======
    m.insert(
        (DecisionType::Business, TrustTier::Low),
        ChallengePrompt::new(
            "You are agent {from_agent}, and you are being challenged by agent {to_agent} on your business decision.\nDecision: {decision}\nContext: {context}\n\nProvide a detailed justification covering:\n1. Expected business impact (revenue, engagement, retention)\n2. User segments affected\n3. Rollback plan if metrics drop\n4. A/B test plan\n\nKeep response under 500 words."
                .into(),
            "You are agent {to_agent}, evaluating {from_agent}'s business justification.\nDecision: {decision}\nJustification: {justification}\n\nEvaluate based on:\n1. Business impact clarity\n2. User-segment awareness\n3. Rollback plan\n4. Measurement strategy\n\nReturn JSON: {\"verdict\": \"accept\"|\"reject\"|\"escalate\", \"reason\": \"...\"}"
                .into(),
        ),
    );
    m.insert(
        (DecisionType::Business, TrustTier::High),
        ChallengePrompt::new(
            "You are agent {from_agent}, challenged on a business decision.\nDecision: {decision}\n\nProvide a brief justification (under 200 words) covering: business impact + rollback plan."
                .into(),
            "You are agent {to_agent}, evaluating {from_agent}'s brief business justification.\nJustification: {justification}\n\nReturn JSON: {\"verdict\": \"accept\"|\"reject\"|\"escalate\", \"reason\": \"...\"}"
                .into(),
        ),
    );

    // ====== Security (2 套) ======
    m.insert(
        (DecisionType::Security, TrustTier::Low),
        ChallengePrompt::new(
            "You are agent {from_agent}, and you are being challenged by agent {to_agent} on your security decision (HIGHEST PRIORITY).\nDecision: {decision}\nContext: {context}\n\nProvide a detailed justification covering:\n1. Threat model addressed\n2. Compliance impact (GDPR / SOC2 / internal)\n3. Audit trail strategy\n4. Incident response plan if breach occurs\n\nKeep response under 500 words. Be exhaustive — security decisions are non-reversible."
                .into(),
            "You are agent {to_agent}, evaluating {from_agent}'s security justification (HIGHEST PRIORITY).\nDecision: {decision}\nJustification: {justification}\n\nEvaluate based on:\n1. Threat model completeness\n2. Compliance coverage\n3. Audit trail adequacy\n4. Incident response readiness\n\nIf ANY of the above is unclear, return \"escalate\". Return JSON: {\"verdict\": \"accept\"|\"reject\"|\"escalate\", \"reason\": \"...\"}"
                .into(),
        ),
    );
    m.insert(
        (DecisionType::Security, TrustTier::High),
        ChallengePrompt::new(
            "You are agent {from_agent}, challenged on a security decision.\nDecision: {decision}\n\nProvide a justification covering: threat model + audit trail + incident response (under 300 words)."
                .into(),
            "You are agent {to_agent}, evaluating {from_agent}'s security justification.\nJustification: {justification}\n\nIf threat model is missing, return \"escalate\". Return JSON: {\"verdict\": \"accept\"|\"reject\"|\"escalate\", \"reason\": \"...\"}"
                .into(),
        ),
    );

    // ====== Performance (2 套) ======
    m.insert(
        (DecisionType::Performance, TrustTier::Low),
        ChallengePrompt::new(
            "You are agent {from_agent}, challenged on a performance decision.\nDecision: {decision}\nContext: {context}\n\nProvide a detailed justification covering:\n1. Baseline metrics (P50 / P95 / P99)\n2. Expected improvement (with confidence interval)\n3. Resource cost (CPU / memory / IO)\n4. Rollback plan if regression\n\nKeep response under 500 words."
                .into(),
            "You are agent {to_agent}, evaluating {from_agent}'s performance justification.\nJustification: {justification}\n\nEvaluate based on:\n1. Baseline + expected improvement\n2. Confidence interval realism\n3. Resource cost vs benefit\n4. Rollback feasibility\n\nReturn JSON: {\"verdict\": \"accept\"|\"reject\"|\"escalate\", \"reason\": \"...\"}"
                .into(),
        ),
    );
    m.insert(
        (DecisionType::Performance, TrustTier::High),
        ChallengePrompt::new(
            "You are agent {from_agent}, challenged on a performance decision.\nDecision: {decision}\n\nProvide a brief justification (under 200 words) covering: baseline metric + expected improvement + rollback plan."
                .into(),
            "You are agent {to_agent}, evaluating {from_agent}'s brief performance justification.\nJustification: {justification}\n\nReturn JSON: {\"verdict\": \"accept\"|\"reject\"|\"escalate\", \"reason\": \"...\"}"
                .into(),
        ),
    );

    // ====== Ux (2 套) ======
    m.insert(
        (DecisionType::Ux, TrustTier::Low),
        ChallengePrompt::new(
            "You are agent {from_agent}, challenged on a UX decision.\nDecision: {decision}\nContext: {context}\n\nProvide a detailed justification covering:\n1. User research / data backing the decision\n2. Affected user segments + personas\n3. Accessibility (a11y) impact\n4. Measurement plan (analytics / qualitative)\n\nKeep response under 500 words."
                .into(),
            "You are agent {to_agent}, evaluating {from_agent}'s UX justification.\nJustification: {justification}\n\nEvaluate based on:\n1. Research backing\n2. Persona coverage\n3. Accessibility impact\n4. Measurement plan\n\nReturn JSON: {\"verdict\": \"accept\"|\"reject\"|\"escalate\", \"reason\": \"...\"}"
                .into(),
        ),
    );
    m.insert(
        (DecisionType::Ux, TrustTier::High),
        ChallengePrompt::new(
            "You are agent {from_agent}, challenged on a UX decision.\nDecision: {decision}\n\nProvide a brief justification (under 200 words) covering: research backing + a11y impact + measurement plan."
                .into(),
            "You are agent {to_agent}, evaluating {from_agent}'s brief UX justification.\nJustification: {justification}\n\nReturn JSON: {\"verdict\": \"accept\"|\"reject\"|\"escalate\", \"reason\": \"...\"}"
                .into(),
        ),
    );

    m
}

/// A read-only view of the 10 套 prompts for tests / inspection.
pub fn prompt_count() -> usize {
    challenge_prompts().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_prompts_has_10_entries() {
        assert_eq!(challenge_prompts().len(), 10);
    }

    #[test]
    fn trust_tier_from_weight_boundary() {
        assert_eq!(TrustTier::from_weight(0.0), TrustTier::Low);
        assert_eq!(TrustTier::from_weight(0.69), TrustTier::Low);
        assert_eq!(TrustTier::from_weight(0.7), TrustTier::High);
        assert_eq!(TrustTier::from_weight(1.0), TrustTier::High);
    }

    #[test]
    fn all_decision_types_have_both_tiers() {
        let m = challenge_prompts();
        for dt in DecisionType::all().iter() {
            assert!(
                m.contains_key(&(*dt, TrustTier::Low)),
                "missing (DecisionType::{:?}, Low)",
                dt
            );
            assert!(
                m.contains_key(&(*dt, TrustTier::High)),
                "missing (DecisionType::{:?}, High)",
                dt
            );
        }
    }
}
