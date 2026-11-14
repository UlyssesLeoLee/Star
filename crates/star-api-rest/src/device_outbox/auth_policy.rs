// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"crates/star-api-rest/src/device_outbox/auth_policy.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/device_outbox/auth_policy.rs::module",kind:"module"})
MERGE (decision:Type {id:"crates/star-api-rest/src/device_outbox/auth_policy.rs::DeviceAuthorizationDecision",kind:"enum"})
MERGE (projection:Type {id:"crates/star-api-rest/src/device_outbox/auth_policy.rs::ProjectionRecord",kind:"struct"})
MERGE (recheck:Type {id:"crates/star-api-rest/src/device_outbox/auth_policy.rs::IdentityRecheckOutcome",kind:"enum"})
MERGE (decide:Symbol {id:"crates/star-api-rest/src/device_outbox/auth_policy.rs::CollaborationDeviceAuthPolicy.decide",kind:"function"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(decision),(module)-[:DEFINES]->(projection),(module)-[:DEFINES]->(recheck),(module)-[:DEFINES]->(decide)
MERGE (decide)-[:USES_TYPE]->(projection),(decide)-[:USES_TYPE]->(recheck),(decide)-[:RETURNS_TYPE]->(decision)
@endcypher
*/
//! Collaboration authorization policy over the local invalidation projection
//! (review 合约 #5/#6).
//!
//! The received events and the local projection only *accelerate* revocation
//! propagation. They never grant anything. In particular: **no event in the
//! projection is never a valid authorization** — the only positive decision
//! comes from a fresh authoritative recheck against the Identity owner's
//! current generation/status, and any unavailability fails closed.

use uuid::Uuid;

/// Local projection record owned by the Collaboration consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectionRecord {
    pub actor_id: Uuid,
    pub device_id: Uuid,
    /// Revocation generation observed in the projection.
    pub revoked_generation: i64,
}

/// Fresh recheck result from the Identity owner's authoritative binding state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityRecheckOutcome {
    /// Binding is active at exactly the presented generation.
    Active { generation: i64 },
    /// Binding is revoked at (or before) the presented generation.
    Revoked { generation: i64 },
    /// The identity owner could not be consulted (unavailable, unknown
    /// device, unknown generation): the caller must fail closed.
    Unavailable,
}

/// Decision for one shared-operation authorization request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceAuthorizationDecision {
    /// Denied because the local projection already carries a revocation at
    /// this or a newer generation (acceleration path; deny-safe).
    DeniedProjectionRevoked { revoked_generation: i64 },
    /// Allowed only by the fresh authoritative Identity recheck; the
    /// projection alone can never produce this.
    AuthorizedViaOwner { generation: i64 },
    /// The Identity owner reports the device revoked.
    DeniedOwnerRevoked { generation: i64 },
    /// Fail closed: the projection carries no revocation fact and the
    /// authoritative recheck is unavailable/unknown — the request is denied
    /// and the session must re-verify before any shared operation.
    DeniedFailClosed { reason: FailClosedReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailClosedReason {
    /// No revocation event in the projection AND the identity recheck could
    /// not be completed. Absence of events is never authorization.
    NoProjectionAndUnavailableAuthority,
    /// The projection carries a revocation for an older generation than the
    /// token presents; newer state is unknown and the recheck is unavailable.
    StaleProjectionAndUnavailableAuthority,
}

/// Pure policy. `token_generation` is the device generation the presented
/// credential claims; anything at or below a known revocation generation is
/// denied, and everything else must be proven by the authoritative owner.
pub fn decide_device_authorization(
    token_device_id: Uuid,
    token_generation: i64,
    projection: Option<ProjectionRecord>,
    authoritative: Option<IdentityRecheckOutcome>,
) -> DeviceAuthorizationDecision {
    if let Some(record) = projection {
        if record.device_id == token_device_id && record.revoked_generation >= token_generation {
            // Deny-safe acceleration: a revocation at this or a newer
            // generation invalidates the presented credential immediately.
            return DeviceAuthorizationDecision::DeniedProjectionRevoked {
                revoked_generation: record.revoked_generation,
            };
        }
    }
    match authoritative {
        Some(IdentityRecheckOutcome::Active { generation }) => {
            if generation == token_generation {
                DeviceAuthorizationDecision::AuthorizedViaOwner { generation }
            } else {
                // Generation drift: the presented generation is not current;
                // treat as failed closed via the stale-projection path.
                DeviceAuthorizationDecision::DeniedFailClosed {
                    reason: FailClosedReason::StaleProjectionAndUnavailableAuthority,
                }
            }
        }
        Some(IdentityRecheckOutcome::Revoked { generation }) => {
            DeviceAuthorizationDecision::DeniedOwnerRevoked { generation }
        }
        None | Some(IdentityRecheckOutcome::Unavailable) => {
            DeviceAuthorizationDecision::DeniedFailClosed {
                reason: if projection.is_some() {
                    FailClosedReason::StaleProjectionAndUnavailableAuthority
                } else {
                    FailClosedReason::NoProjectionAndUnavailableAuthority
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device() -> Uuid {
        Uuid::from_u128(0xD1)
    }

    #[test]
    fn no_event_is_never_valid_authorization() {
        // No projection record + no authoritative outcome (identity owner
        // unavailable) must fail closed — never "no event → still valid".
        let decision = decide_device_authorization(device(), 3, None, None);
        assert_eq!(
            decision,
            DeviceAuthorizationDecision::DeniedFailClosed {
                reason: FailClosedReason::NoProjectionAndUnavailableAuthority,
            }
        );
        // Same when the recheck explicitly reports unavailability.
        let decision = decide_device_authorization(
            device(),
            3,
            None,
            Some(IdentityRecheckOutcome::Unavailable),
        );
        assert_eq!(
            decision,
            DeviceAuthorizationDecision::DeniedFailClosed {
                reason: FailClosedReason::NoProjectionAndUnavailableAuthority,
            }
        );
    }

    #[test]
    fn no_event_plus_authoritative_active_check_is_allowed_via_owner_only() {
        let decision = decide_device_authorization(
            device(),
            3,
            None,
            Some(IdentityRecheckOutcome::Active { generation: 3 }),
        );
        assert_eq!(
            decision,
            DeviceAuthorizationDecision::AuthorizedViaOwner { generation: 3 }
        );
    }

    #[test]
    fn projection_revocation_denies_even_without_authority() {
        let decision = decide_device_authorization(
            device(),
            3,
            Some(ProjectionRecord {
                actor_id: Uuid::from_u128(0xE1),
                device_id: device(),
                revoked_generation: 3,
            }),
            None,
        );
        assert_eq!(
            decision,
            DeviceAuthorizationDecision::DeniedProjectionRevoked {
                revoked_generation: 3
            }
        );
        // A newer revocation generation also denies an older credential.
        let decision = decide_device_authorization(
            device(),
            2,
            Some(ProjectionRecord {
                actor_id: Uuid::from_u128(0xE1),
                device_id: device(),
                revoked_generation: 3,
            }),
            Some(IdentityRecheckOutcome::Active { generation: 2 }),
        );
        assert_eq!(
            decision,
            DeviceAuthorizationDecision::DeniedProjectionRevoked {
                revoked_generation: 3
            }
        );
    }

    #[test]
    fn stale_projection_without_authority_fails_closed() {
        // Projection knows generation 2 was revoked, credential claims
        // generation 3; without the authoritative owner the newer state is
        // unknown → fail closed, not allowed.
        let decision = decide_device_authorization(
            device(),
            3,
            Some(ProjectionRecord {
                actor_id: Uuid::from_u128(0xE1),
                device_id: device(),
                revoked_generation: 2,
            }),
            None,
        );
        assert_eq!(
            decision,
            DeviceAuthorizationDecision::DeniedFailClosed {
                reason: FailClosedReason::StaleProjectionAndUnavailableAuthority,
            }
        );
    }

    #[test]
    fn owner_revoked_denies_and_generation_drift_fails_closed() {
        let decision = decide_device_authorization(
            device(),
            3,
            None,
            Some(IdentityRecheckOutcome::Revoked { generation: 3 }),
        );
        assert_eq!(
            decision,
            DeviceAuthorizationDecision::DeniedOwnerRevoked { generation: 3 }
        );
        let decision = decide_device_authorization(
            device(),
            3,
            None,
            Some(IdentityRecheckOutcome::Active { generation: 4 }),
        );
        assert_eq!(
            decision,
            DeviceAuthorizationDecision::DeniedFailClosed {
                reason: FailClosedReason::StaleProjectionAndUnavailableAuthority,
            }
        );
    }
}
