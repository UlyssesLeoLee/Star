// SPDX-License-Identifier: MIT OR Apache-2.0
//! OAuth2 5 endpoints end-to-end integration tests
//! (per brief v0.51 §3 + §14.12 IV phase 2 收尾)
//!
//! 用 `common::MockOAuth2Bundles` 构造 in-memory 仓库, 走 `axum::Router::oneshot`
//! 触发 5 endpoints:
//! - GET  /oauth/authorize       → 302 + `?code=...&state=...`
//! - POST /oauth/token grant_type=authorization_code → 200 + `access_token` + `refresh_token`
//! - GET  /.well-known/jwks.json → 200 + `{"keys":[{kty,kid,alg,n,e}]}`
//! - POST /oauth/introspect      → 501 not_implemented (跨 session 续, 真实 DB lookup 待 P3+)
//! - POST /oauth/revoke          → 200 OK
//!
//! 守门 #25 v25: 集成测试不发真 HTTP, 走 `Router::oneshot`; 也不发真 PG, 走 mock.
//! 守门 #5 v2: token / code 不入 log / println.
//! 守门 #19 v19: mock helper Python 化 (本测试不用子代理, fixture 全 Rust).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine;
use serde_json::Value;
use sha2::{Digest, Sha256};
use star_api_rest::build_oauth_router;
use tower::ServiceExt;

mod common;

use common::MockOAuth2Bundles;

/// Helper: compute `sha256_hex(input)` to match the handler-side logic.
fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a PKCE code_verifier (43-128 chars, base64url-no-pad, per RFC 7636 §4.1).
/// Local copy because `pkce::generate_code_verifier` is `#[cfg(test)]` gated.
fn generate_code_verifier() -> String {
    use ring::rand::{SecureRandom, SystemRandom};
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes).expect("rng");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Build a PKCE pair (verifier + S256 challenge) for tests.
fn pkce_pair() -> (String, String) {
    let verifier = generate_code_verifier();
    let challenge = {
        use ring::digest::{digest, SHA256};
        let hashed = digest(&SHA256, verifier.as_bytes());
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hashed.as_ref())
    };
    (verifier, challenge)
}

// ============================================
// Case 1: GET /oauth/authorize → 302 + code
// ============================================

#[tokio::test]
async fn integration_authorize_returns_302_with_code_after_v0_51() {
    let bundles = MockOAuth2Bundles::new();
    let state = bundles.build_state().await;

    let client_id = "star-frontend-spa";
    let redirect_uri = "https://app.star.local/callback";
    bundles.seed_public_client(client_id, redirect_uri);

    let (_, code_challenge) = pkce_pair();

    let app = build_oauth_router().with_state(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256&state=xyz",
                    urlencoding(client_id),
                    urlencoding(redirect_uri),
                    urlencoding(&code_challenge),
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 302 or 303 redirect with `?code=...&state=...` (axum's `Redirect::to()`
    // emits 303 See Other by default; both are valid for OAuth2 auth code flow).
    let status = resp.status();
    assert!(
        status == StatusCode::FOUND || status == StatusCode::SEE_OTHER,
        "authorize must redirect (302/303), got {status}"
    );
    let location = resp
        .headers()
        .get("location")
        .expect("location header missing")
        .to_str()
        .expect("location header is ascii")
        .to_string();
    assert!(
        location.starts_with(redirect_uri),
        "redirect target wrong: {location}"
    );
    assert!(location.contains("code="), "missing code in location: {location}");
    assert!(
        location.contains("state=xyz"),
        "missing state echo in location: {location}"
    );
}

// ============================================
// Case 2: POST /oauth/token authorization_code → 200 + access_token
// ============================================

#[tokio::test]
async fn integration_token_authorization_code_returns_200_with_access_token_after_v0_51() {
    let bundles = MockOAuth2Bundles::new();
    let state = bundles.build_state().await;

    let client_id = "star-frontend-spa";
    let redirect_uri = "https://app.star.local/callback";
    bundles.seed_public_client(client_id, redirect_uri);

    // Pre-seed an authorization code the client can exchange.
    let (verifier, challenge) = pkce_pair();
    let plaintext_code = "test-code-very-secret-and-long-enough-1234567890";
    let code_hash = sha256_hex(plaintext_code);
    bundles.seed_authorization_code(
        client_id,
        &code_hash,
        &challenge,
        "S256",
        redirect_uri,
        "read",
    );

    let body = format!(
        "grant_type=authorization_code&code={}&code_verifier={}&client_id={}&redirect_uri={}",
        urlencoding(plaintext_code),
        urlencoding(&verifier),
        urlencoding(client_id),
        urlencoding(redirect_uri),
    );

    let app = build_oauth_router().with_state(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/oauth/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK, "token must 200");
    let body_bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["token_type"], "Bearer", "token_type must be Bearer");
    assert!(
        body["access_token"].is_string(),
        "access_token missing: {body}"
    );
    assert!(
        body["refresh_token"].is_string(),
        "refresh_token missing: {body}"
    );
    assert_eq!(body["expires_in"], 3600);
    assert_eq!(body["scope"], "read");

    // Verify the access_token is a real JWT (3 dot-separated base64url segments).
    let access_token = body["access_token"].as_str().unwrap();
    let parts: Vec<&str> = access_token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT must have 3 segments: {access_token}");

    // Verify the auth code was marked consumed.
    let snap = bundles.auth_code_repo.snapshot();
    assert_eq!(snap.len(), 1, "one code expected in repo");
    assert!(
        snap[0].consumed_at.is_some(),
        "auth code should be marked consumed"
    );
}

// ============================================
// Case 3: GET /.well-known/jwks.json → 200 + {keys:[{kty,kid,alg}]}
// ============================================

#[tokio::test]
async fn integration_jwks_returns_200_with_rsa_public_key_after_v0_51() {
    let bundles = MockOAuth2Bundles::new();
    let state = bundles.build_state().await;

    let app = build_oauth_router().with_state(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/.well-known/jwks.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK, "jwks must 200");
    let body_bytes = axum::body::to_bytes(resp.into_body(), 16 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // Per RFC 7517 §5: { "keys": [ {kty, kid, alg, n, e, use?} ] }
    let keys = body["keys"].as_array().expect("keys must be array");
    assert_eq!(keys.len(), 1, "exactly one public key expected");
    let k = &keys[0];
    assert_eq!(k["kty"], "RSA");
    assert_eq!(k["alg"], "RS256");
    assert_eq!(k["use"], "sig");
    assert!(k["kid"].is_string(), "kid must be string");
    assert!(k["n"].is_string(), "modulus n must be present");
    assert!(k["e"].is_string(), "exponent e must be present");
    // RSA 2048 modulus = 256 bytes base64url ≈ 342 chars
    assert!(
        k["n"].as_str().unwrap().len() > 300,
        "RSA 2048 modulus too short"
    );
    // Standard public exponent 65537 → base64url "AQAB"
    assert_eq!(k["e"], "AQAB");
}

// ============================================
// Case 4: POST /oauth/introspect → 501 not_implemented (跨 session 续)
// ============================================

#[tokio::test]
async fn integration_introspect_returns_501_not_implemented_after_v0_51() {
    let bundles = MockOAuth2Bundles::new();
    let state = bundles.build_state().await;

    let app = build_oauth_router().with_state(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/oauth/introspect")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("token=any-fake-jwt-token"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Per brief v0.51 §3 deliverable #4: introspect 跨 session 续 (DB lookup), 现状 501.
    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "introspect must 501 not_implemented (跨 session 续)"
    );
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["error"], "not_implemented");
}

// ============================================
// Case 5: POST /oauth/revoke → 200 OK
// ============================================

#[tokio::test]
async fn integration_revoke_returns_200_after_v0_51() {
    let bundles = MockOAuth2Bundles::new();
    let state = bundles.build_state().await;

    // Per RFC 7009 §2.1, revoke is idempotent: unknown token returns 200 too.
    let app = build_oauth_router().with_state(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/oauth/revoke")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "token=any-token-string-revocation-is-idempotent",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK, "revoke must 200");

    // No body bytes for 200 (per handler signature `Result<StatusCode, _>`).
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1)
        .await
        .unwrap();
    assert!(
        body_bytes.is_empty(),
        "revoke 200 should have empty body"
    );
}

/// End-to-end authorize → token roundtrip, exercising the full PKCE flow.
#[tokio::test]
async fn integration_authorize_then_token_roundtrip_with_pkce_after_v0_51() {
    let bundles = MockOAuth2Bundles::new();
    let state = bundles.build_state().await;

    let client_id = "star-frontend-spa";
    let redirect_uri = "https://app.star.local/callback";
    bundles.seed_public_client(client_id, redirect_uri);

    let (verifier, challenge) = pkce_pair();

    // Step 1: hit /oauth/authorize
    let app = build_oauth_router().with_state(state.clone());
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256",
                    urlencoding(client_id),
                    urlencoding(redirect_uri),
                    urlencoding(&challenge),
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    assert!(
        status == StatusCode::FOUND || status == StatusCode::SEE_OTHER,
        "authorize must redirect, got {status}"
    );
    let location = resp
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let code = location
        .split("code=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .expect("code in location")
        .to_string();

    // Step 2: exchange the code at /oauth/token
    let body = format!(
        "grant_type=authorization_code&code={}&code_verifier={}&client_id={}&redirect_uri={}",
        urlencoding(&code),
        urlencoding(&verifier),
        urlencoding(client_id),
        urlencoding(redirect_uri),
    );
    let app = build_oauth_router().with_state(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/oauth/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
}

/// Minimal URL-encoder for query string values (no external dep).
fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
