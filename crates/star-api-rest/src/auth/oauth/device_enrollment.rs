// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=e7b7dd5dc2e292c95338c0caf76946447764b4a1360a7f56a1c19257ef4ac1fe
MERGE (self:File {path:"crates/star-api-rest/src/auth/oauth/device_enrollment.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/auth/oauth/device_enrollment.rs::module",kind:"module"})
MERGE (router:Symbol {id:"crates/star-api-rest/src/auth/oauth/device_enrollment.rs::router",kind:"function"})
MERGE (challenge:Symbol {id:"crates/star-api-rest/src/auth/oauth/device_enrollment.rs::create_challenge",kind:"function"})
MERGE (enroll:Symbol {id:"crates/star-api-rest/src/auth/oauth/device_enrollment.rs::enroll_device",kind:"function"})
MERGE (revoke:Symbol {id:"crates/star-api-rest/src/auth/oauth/device_enrollment.rs::revoke_device",kind:"function"})
MERGE (message:Symbol {id:"crates/star-api-rest/src/auth/oauth/device_enrollment.rs::proof_message",kind:"function"})
MERGE (fingerprint:Symbol {id:"crates/star-api-rest/src/auth/oauth/device_enrollment.rs::fingerprint",kind:"function"})
MERGE (state:Type {id:"crates/star-api-rest/src/auth/oauth/handlers.rs::OAuth2State",kind:"struct"})
MERGE (deviceContext:Type {id:"crates/star-api-rest/src/auth/mod.rs::DeviceTokenContext",kind:"struct"})
MERGE (deviceRepo:Type {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::DeviceBindingRepository",kind:"trait"})
MERGE (self)-[:DEFINES]->(module),(module)-[:DEFINES]->(router),(module)-[:DEFINES]->(challenge),(module)-[:DEFINES]->(enroll),(module)-[:DEFINES]->(revoke),(module)-[:DEFINES]->(message),(module)-[:DEFINES]->(fingerprint)
MERGE (router)-[:USES_TYPE]->(state),(challenge)-[:USES_TYPE]->(state),(enroll)-[:USES_TYPE]->(state),(revoke)-[:USES_TYPE]->(state)
MERGE (challenge)-[:CALLS]->(deviceRepo),(enroll)-[:CALLS]->(deviceRepo),(revoke)-[:CALLS]->(deviceRepo),(enroll)-[:CALLS]->(message),(enroll)-[:CALLS]->(fingerprint),(enroll)-[:CALLS]->(deviceContext)
MERGE (router)-[:EXPOSES]->(challengeEndpoint:Endpoint {id:"POST /oauth/device/challenges",kind:"http_endpoint"}),(router)-[:EXPOSES]->(enrollEndpoint:Endpoint {id:"POST /oauth/device/enrollments",kind:"http_endpoint"}),(router)-[:EXPOSES]->(revokeEndpoint:Endpoint {id:"POST /oauth/device/{device_id}/revoke",kind:"http_endpoint"})
MERGE (self)-[:HANDLES]->(challengeEndpoint),(self)-[:HANDLES]->(enrollEndpoint),(self)-[:HANDLES]->(revokeEndpoint)
@endcypher
*/
//! Proof-of-possession device enrollment and owner-scoped revocation endpoints.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use ring::{
    digest,
    rand::{SecureRandom, SystemRandom},
    signature,
};
use serde::{Deserialize, Serialize};
use star_pg_adapter::repository::{DeviceChallengeCreateResult, DeviceEnrollmentChallenge};
use uuid::Uuid;

use super::{
    handlers::{issue_tokens, OAuthHandlerError},
    middleware::AuthenticatedUser,
    OAuth2State,
};
use crate::auth::DeviceTokenContext;

const ENROLLMENT_CHALLENGE_TTL_SECONDS: i64 = 300;
const DEVICE_CLIENT_ID: &str = "star-device";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateChallengeRequest {
    public_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompleteEnrollmentRequest {
    challenge_id: Uuid,
    challenge: String,
    proof_signature: String,
}

#[derive(Debug, Serialize)]
struct CreateChallengeResponse {
    challenge_id: Uuid,
    challenge: String,
    expires_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct CompleteEnrollmentResponse {
    device_id: Uuid,
    generation: i64,
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: Option<String>,
    scope: Option<String>,
}

pub(crate) fn router() -> Router<OAuth2State> {
    Router::new()
        .route("/oauth/device/challenges", post(create_challenge))
        .route("/oauth/device/enrollments", post(enroll_device))
        .route("/oauth/device/{device_id}/revoke", post(revoke_device))
}

async fn create_challenge(
    State(state): State<OAuth2State>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Json(request): Json<CreateChallengeRequest>,
) -> Result<Json<CreateChallengeResponse>, OAuthHandlerError> {
    if actor.user_id.is_nil() || actor.tenant_id.is_nil() {
        return Err(OAuthHandlerError::InvalidGrant(
            "authenticated identity required".into(),
        ));
    }
    let public_key = URL_SAFE_NO_PAD
        .decode(request.public_key.as_bytes())
        .map_err(|_| OAuthHandlerError::InvalidRequest("invalid device public key".into()))?;
    if public_key.len() != 32 {
        return Err(OAuthHandlerError::InvalidRequest(
            "invalid device public key".into(),
        ));
    }

    let mut nonce = [0_u8; 32];
    SystemRandom::new()
        .fill(&mut nonce)
        .map_err(|_| OAuthHandlerError::Internal("device challenge RNG unavailable".into()))?;
    let now = Utc::now();
    let expires_at = now + Duration::seconds(ENROLLMENT_CHALLENGE_TTL_SECONDS);
    let challenge = DeviceEnrollmentChallenge {
        challenge_id: Uuid::new_v4(),
        tenant_id: actor.tenant_id,
        actor_id: actor.user_id,
        challenge_hash: digest::digest(&digest::SHA256, &nonce).as_ref().to_vec(),
        public_key_fingerprint: fingerprint(&public_key),
        public_key,
        expires_at,
    };
    let result = state
        .device_binding_repo
        .create_challenge(&challenge)
        .await
        .map_err(|_| OAuthHandlerError::Internal("create device challenge failed".into()))?;
    if result == DeviceChallengeCreateResult::RateLimited {
        return Err(OAuthHandlerError::RateLimited(
            "too many active device challenges".into(),
        ));
    }

    Ok(Json(CreateChallengeResponse {
        challenge_id: challenge.challenge_id,
        challenge: URL_SAFE_NO_PAD.encode(nonce),
        expires_at,
    }))
}

async fn enroll_device(
    State(state): State<OAuth2State>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Json(request): Json<CompleteEnrollmentRequest>,
) -> Result<Json<CompleteEnrollmentResponse>, OAuthHandlerError> {
    let challenge_bytes = URL_SAFE_NO_PAD
        .decode(request.challenge.as_bytes())
        .map_err(|_| OAuthHandlerError::InvalidGrant("device proof invalid".into()))?;
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(request.proof_signature.as_bytes())
        .map_err(|_| OAuthHandlerError::InvalidGrant("device proof invalid".into()))?;
    if challenge_bytes.len() != 32 || signature_bytes.len() != 64 {
        return Err(OAuthHandlerError::InvalidGrant(
            "device proof invalid".into(),
        ));
    }

    let challenge = state
        .device_binding_repo
        .find_challenge(actor.tenant_id, actor.user_id, request.challenge_id)
        .await
        .map_err(|_| OAuthHandlerError::Internal("device challenge lookup failed".into()))?
        .ok_or_else(|| {
            OAuthHandlerError::InvalidGrant("device challenge invalid or expired".into())
        })?;
    let supplied_hash = digest::digest(&digest::SHA256, &challenge_bytes);
    if supplied_hash.as_ref() != challenge.challenge_hash.as_slice() {
        return Err(OAuthHandlerError::InvalidGrant(
            "device proof invalid".into(),
        ));
    }
    let signed_message = proof_message(
        actor.tenant_id,
        actor.user_id,
        request.challenge_id,
        &challenge_bytes,
    );
    signature::UnparsedPublicKey::new(&signature::ED25519, &challenge.public_key)
        .verify(&signed_message, &signature_bytes)
        .map_err(|_| OAuthHandlerError::InvalidGrant("device proof invalid".into()))?;

    let device_id = Uuid::new_v4();
    let now = Utc::now();
    let binding = state
        .device_binding_repo
        .complete_challenge(
            actor.tenant_id,
            actor.user_id,
            request.challenge_id,
            supplied_hash.as_ref(),
            device_id,
            now,
        )
        .await
        .map_err(|_| OAuthHandlerError::Internal("complete device enrollment failed".into()))?
        .ok_or_else(|| OAuthHandlerError::InvalidGrant("device challenge already used".into()))?;

    let device_context = DeviceTokenContext {
        device_id: binding.device_id,
        generation: binding.generation,
    };
    let tokens = match issue_tokens(
        &state,
        DEVICE_CLIENT_ID,
        Some(actor.user_id),
        actor.tenant_id,
        &actor.scope,
        "refresh_token",
        Some(device_context),
    )
    .await
    {
        Ok(tokens) => tokens.0,
        Err(error) => {
            let _ = state
                .device_binding_repo
                .revoke(
                    actor.tenant_id,
                    actor.user_id,
                    binding.device_id,
                    Utc::now(),
                )
                .await;
            return Err(error);
        }
    };

    Ok(Json(CompleteEnrollmentResponse {
        device_id: binding.device_id,
        generation: binding.generation,
        access_token: tokens.access_token,
        token_type: tokens.token_type,
        expires_in: tokens.expires_in,
        refresh_token: tokens.refresh_token,
        scope: tokens.scope,
    }))
}

async fn revoke_device(
    State(state): State<OAuth2State>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(device_id): Path<String>,
) -> Result<StatusCode, OAuthHandlerError> {
    let device_id = Uuid::parse_str(&device_id)
        .map_err(|_| OAuthHandlerError::InvalidRequest("invalid device id".into()))?;
    let revoked = state
        .device_binding_repo
        .revoke(actor.tenant_id, actor.user_id, device_id, Utc::now())
        .await
        .map_err(|_| OAuthHandlerError::Internal("revoke device failed".into()))?;
    if revoked.is_none() {
        return Err(OAuthHandlerError::InvalidGrant(
            "device not found or already revoked".into(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn proof_message(tenant_id: Uuid, actor_id: Uuid, challenge_id: Uuid, nonce: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(96);
    message.extend_from_slice(b"star-device-enrollment-v1\0");
    message.extend_from_slice(tenant_id.as_bytes());
    message.extend_from_slice(actor_id.as_bytes());
    message.extend_from_slice(challenge_id.as_bytes());
    message.extend_from_slice(nonce);
    message
}

fn fingerprint(public_key: &[u8]) -> String {
    hex::encode(digest::digest(&digest::SHA256, public_key).as_ref())
}
