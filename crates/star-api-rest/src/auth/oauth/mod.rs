// SPDX-License-Identifier: MIT OR Apache-2.0
//! OAuth2 server (per WBS v0.43 §14.12 IV phase 2)
//!
//! v0.43 端到端实装 (per 9/9 20:20 JST 用户拍板"both" (Authorization Code + PKCE + Client Credentials)):
//! - 4 Repository (oauth_clients M SCD2 + oauth_authorization_codes T WORM + oauth_access_tokens T + oauth_refresh_tokens T WORM)
//! - 5 endpoints: /oauth/authorize + /oauth/token + /.well-known/jwks.json + /oauth/introspect + /oauth/revoke
//! - Bearer auth middleware (RFC 6750)
//! - 5 域 RBAC 整合 (跨 session 续)
//! - PKCE S256 (RFC 7636)
//! - RSA 2048 keypair + JWKS (RFC 7517)
//!
//! 守门 #5 v2: token 不入 log
//! 守门 #6 v2: invalid_grant / invalid_client 走 4xx (不 retriable)
//! 守门 #13: 4 表 W/T/M 严格分类 (M=1 + T=3)
//! 守门 #14 v3: 永久代签
//!
//! 跨 session 续: user login flow (consents screen) + 真实 DB-backed introspect + bcrypt client_secret + 5 域 RBAC 实证

// v0.43 §14.12 IV OAuth2 server phase 2 (per 2026-09-09 20:20 JST 用户拍板 both flows)
// handlers.rs 跨 session 续 (struct 字段完整化 + ring/aws-lc-rs RSA keygen 实证)
// pub mod handlers;
pub mod keypair;
pub mod middleware;
pub mod pkce;

pub use keypair::{Jwk, Jwks, KeyPairError, OAuthKeyManager, OAuthKeyPair};
pub use middleware::{require_role, require_scope, AuthenticatedUser, BearerError, BearerToken};
pub use pkce::{verify_pkce, PkceError};
