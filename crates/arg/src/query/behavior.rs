// SPDX-License-Identifier: MIT OR Apache-2.0
//! 7 协作行为 event patterns (per DD-AGENT-RELATIONSHIP-001 §3.1 + §6).
//!
//! These are **placeholders**: the real `BehaviorEvaluator` lands in
//! P3-E ARG.8 (per WBS §14.3). Each entry is the `(code, pattern)`
//! pair consumed by the evaluator.

/// A `(code, event_pattern)` pair for one 行为 achievement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorPattern {
    /// Achievement code (e.g. `"BEH-001-FIRST-EDGE"`).
    pub code: String,
    /// Event pattern, e.g. `"edge.created"`.
    pub pattern: String,
}

/// Returns the 7 行为 event patterns in canonical order.
pub fn all_behavior_patterns() -> Vec<BehaviorPattern> {
    vec![
        BehaviorPattern {
            code: "BEH-001-FIRST-EDGE".into(),
            pattern: "edge.created".into(),
        },
        BehaviorPattern {
            code: "BEH-002-10-EDGES".into(),
            pattern: "edge.created.count>=10".into(),
        },
        BehaviorPattern {
            code: "BEH-003-100-EDGES".into(),
            pattern: "edge.created.count>=100".into(),
        },
        BehaviorPattern {
            code: "BEH-004-CONSULT-ROUND".into(),
            pattern: "consult.round.completed".into(),
        },
        BehaviorPattern {
            code: "BEH-005-CHALLENGE-ROUND".into(),
            pattern: "challenge.round.completed".into(),
        },
        BehaviorPattern {
            code: "BEH-006-STAND-IN-RESCUE".into(),
            pattern: "stand_in.activated".into(),
        },
        BehaviorPattern {
            code: "BEH-007-MENTOR-CHAIN".into(),
            pattern: "mentor.context_injected".into(),
        },
    ]
}
