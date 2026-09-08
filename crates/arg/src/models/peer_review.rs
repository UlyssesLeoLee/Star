// SPDX-License-Identifier: MIT OR Apache-2.0
//! `PeerReviewVerdict` / `ChallengeVerdict` / `ChallengePrompt`
//! (per DD-AGENT-RELATIONSHIP-001 §3.2.5).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::decision::{EscalationInfo, Output, Verdict};

/// Verdict of a peer-review round (per DD §3.2.5 + §4.3.4).
///
/// A round is **accepted** when both agents' scores are `>= 0.8` (per
/// SRS §4.3.4). The struct carries both `agent_a` and `agent_b` so
/// downstream consumers can audit the round.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerReviewVerdict {
    /// First reviewer.
    agent_a: Uuid,
    /// Second reviewer.
    agent_b: Uuid,
    /// The output that was reviewed.
    output: Output,
    /// Combined score (min of the two reviewer scores, in `[0.0, 1.0]`).
    score: f32,
    /// Free-form feedback from both reviewers.
    feedback: String,
    /// True iff `score >= 0.8` (per SRS §4.3.4 threshold).
    accepted: bool,
}

impl PeerReviewVerdict {
    /// Build a new verdict. `accepted` is computed from the threshold.
    pub fn new(agent_a: Uuid, agent_b: Uuid, output: Output, score: f32, feedback: String) -> Self {
        let accepted = score >= 0.8;
        Self {
            agent_a,
            agent_b,
            output,
            score,
            feedback,
            accepted,
        }
    }

    /// First reviewer.
    pub fn agent_a(&self) -> Uuid {
        self.agent_a
    }
    /// Second reviewer.
    pub fn agent_b(&self) -> Uuid {
        self.agent_b
    }
    /// Reviewed output.
    pub fn output(&self) -> &Output {
        &self.output
    }
    /// Combined score.
    pub fn score(&self) -> f32 {
        self.score
    }
    /// Feedback string.
    pub fn feedback(&self) -> &str {
        &self.feedback
    }
    /// True iff the round was accepted.
    pub fn accepted(&self) -> bool {
        self.accepted
    }
}

/// Verdict of a challenge round (per DD §4.3.4 + §3.2.5).
///
/// Carries the `from` and `to` agents, the `justification` returned by
/// the challenged agent, the LLM `verdict` and any escalation metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChallengeVerdict {
    /// Challenger.
    pub from_agent: Uuid,
    /// Challenged agent.
    pub to_agent: Uuid,
    /// The challenged agent's justification.
    pub justification: String,
    /// LLM verdict (accept / reject / escalate).
    pub verdict: Verdict,
    /// Escalation metadata, set when `verdict == Verdict::Escalate`.
    pub escalation: Option<EscalationInfo>,
}

impl ChallengeVerdict {
    /// True iff the round was accepted.
    pub fn is_accepted(&self) -> bool {
        self.verdict == Verdict::Accept
    }
}

/// A two-prompt template (per DD §7).
///
/// The challenge round runs `self_justify_prompt` on the challenged agent
/// first, then `evaluate_prompt` on the challenger to decide the verdict.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChallengePrompt {
    /// Prompt asking the challenged agent to justify the decision.
    pub self_justify_prompt: String,
    /// Prompt asking the challenger to evaluate the justification.
    pub evaluate_prompt: String,
}

impl ChallengePrompt {
    /// Build a new two-prompt template.
    pub fn new(self_justify_prompt: String, evaluate_prompt: String) -> Self {
        Self {
            self_justify_prompt,
            evaluate_prompt,
        }
    }
}
