// SPDX-License-Identifier: MIT OR Apache-2.0
//! OAuth2 RSA 2048 keypair 管理 (per WBS v0.47 §14.12 IV OAuth2 server phase 2)
//!
//! v0.47 真实 RSA 2048 keygen (per brief v0.47 §3.1):
//! - 用 `rsa = "0.9"` (pure Rust) + `pkcs8 = "0.10"` 真正生成 RSA 2048
//! - 用 env var (JWT_PRIVATE_KEY_PEM / JWT_PUBLIC_KEY_PEM) 加载外部 PEM
//! - 守门 #5 v2: private key 走 env var (JWT_PRIVATE_KEY_PEM) 或 star-credential 存储, 不入 log
//! - 守门 #14 v3: Mavis 永久代签

use base64::Engine;
use rand::rngs::OsRng;
use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    traits::PublicKeyParts,
    RsaPrivateKey, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// RSA 2048 keypair 错误 (跟 JwtError 模式同源)
#[derive(Debug, Error)]
pub enum KeyPairError {
    /// key generation failed
    #[error("RSA keypair generation failed: {0}")]
    Generation(String),
    /// PEM parse failed
    #[error("PEM parse failed: {0}")]
    PemParse(String),
    /// JWK encode failed
    #[error("JWK encode failed: {0}")]
    JwkEncode(String),
    /// private key 不可导出 (e.g. attempt to get public from private-only)
    #[error("private key extraction failed: {0}")]
    PrivateExtract(String),
    /// not implemented (跨 session 续)
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

impl KeyPairError {
    /// 6-field code (跟 v0.30 6-field 错误模型一致)
    pub fn code(&self) -> &'static str {
        match self {
            KeyPairError::Generation(_) => "OAUTH_KEYGEN_FAILED",
            KeyPairError::PemParse(_) => "OAUTH_PEM_INVALID",
            KeyPairError::JwkEncode(_) => "OAUTH_JWK_ENCODE_FAILED",
            KeyPairError::PrivateExtract(_) => "OAUTH_PRIVATE_EXTRACT_FAILED",
            KeyPairError::NotImplemented(_) => "OAUTH_NOT_IMPLEMENTED",
        }
    }

    /// retriable (per 守门 #6 v2)
    pub fn is_retriable(&self) -> bool {
        matches!(self, KeyPairError::Generation(_))
    }
}

/// RSA 2048 keypair (持有 private + public key)
pub struct OAuthKeyPair {
    /// private key (PEM 格式, 守门 #5 v2 不入 log)
    private_pem: String,
    /// public key (PEM 格式, 可通过 JWKS endpoint 暴露)
    public_pem: String,
    /// key id (kid) — SHA-256(public_pem)[0..8] bytes hex = 16 chars
    kid: String,
    /// RSA public key modulus (base64url), 给 JWK 用
    jwk_n: String,
    /// RSA public key exponent (base64url), 给 JWK 用
    jwk_e: String,
}

impl OAuthKeyPair {
    /// 全新生成 RSA 2048 keypair (per brief v0.47 §3.1: replace placeholder with real keygen)
    ///
    /// 优先从 env var `JWT_PRIVATE_KEY_PEM` / `JWT_PUBLIC_KEY_PEM` 加载 (守门 #5 v2),
    /// 缺失则用 `rsa` crate 实时生成 2048-bit RSA keypair.
    pub fn generate() -> Result<Self, KeyPairError> {
        // 优先从 env var 加载 (守门 #5 v2)
        if let (Ok(priv_pem), Ok(pub_pem)) = (
            std::env::var("JWT_PRIVATE_KEY_PEM"),
            std::env::var("JWT_PUBLIC_KEY_PEM"),
        ) {
            return Self::from_pem(priv_pem, pub_pem);
        }

        // 真实 RSA 2048 keygen (per `rsa` crate pure Rust)
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048)
            .map_err(|e| KeyPairError::Generation(format!("rsa 2048: {}", e)))?;
        let public_key = RsaPublicKey::from(&private_key);

        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| KeyPairError::Generation(format!("private PEM: {}", e)))?
            .to_string();
        let public_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| KeyPairError::Generation(format!("public PEM: {}", e)))?;

        let kid = compute_kid(public_pem.as_bytes());
        let (jwk_n, jwk_e) = extract_jwk_components(&public_key)?;

        Ok(Self {
            private_pem,
            public_pem,
            kid,
            jwk_n,
            jwk_e,
        })
    }

    /// 从 PEM 字符串加载 (守门 #5 v2: private_pem 来自 env / star-credential)
    pub fn from_pem(private_pem: String, public_pem: String) -> Result<Self, KeyPairError> {
        let kid = compute_kid(public_pem.as_bytes());

        // 解析 public_pem 提取 jwk n/e 组件
        let public_key = RsaPublicKey::from_public_key_pem(&public_pem)
            .map_err(|e| KeyPairError::PemParse(format!("public PEM parse: {}", e)))?;
        let (jwk_n, jwk_e) = extract_jwk_components(&public_key)?;

        Ok(Self {
            private_pem,
            public_pem,
            kid,
            jwk_n,
            jwk_e,
        })
    }

    /// private PEM (守门 #5 v2: 仅签名用, 绝不暴露)
    pub fn private_pem(&self) -> &str {
        &self.private_pem
    }

    /// public PEM (JWKS 用)
    pub fn public_pem(&self) -> &str {
        &self.public_pem
    }

    /// key id (kid)
    pub fn kid(&self) -> &str {
        &self.kid
    }

    /// RSA private key (给 jsonwebtoken 签 token 用)
    /// 跨 session 续: 真实签名跨 session 续 (per 守门 #25 v25 owner 拍板后切 no_network_mode=false)
    pub fn private_key(&self) -> Result<RsaPrivateKey, KeyPairError> {
        RsaPrivateKey::from_pkcs8_pem(&self.private_pem)
            .map_err(|e| KeyPairError::PrivateExtract(format!("pkcs8 pem: {}", e)))
    }
}

/// Compute key id (kid) = SHA-256(public_pem)[0..8] bytes hex (16 chars)
fn compute_kid(public_pem: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_pem);
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

/// 从 RSA public key 提取 JWK n/e 组件 (RFC 7518 §6.1)
fn extract_jwk_components(public_key: &RsaPublicKey) -> Result<(String, String), KeyPairError> {
    let n = public_key.n().to_bytes_be();
    let e = public_key.e().to_bytes_be();
    let n_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&n);
    let e_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&e);
    Ok((n_b64, e_b64))
}

/// JWK (JSON Web Key) public representation (RFC 7517 §4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    /// 密钥类型 ("RSA")
    pub kty: String,
    /// 公开用途 ("sig" for signature)
    #[serde(rename = "use")]
    pub use_: String,
    /// 密钥 ID
    pub kid: String,
    /// 算法 ("RS256")
    pub alg: String,
    /// RSA modulus (base64url)
    pub n: String,
    /// RSA public exponent (base64url)
    pub e: String,
}

/// JWKS (JSON Web Key Set) - RFC 7517 §5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

impl OAuthKeyPair {
    /// 转换为 JWK (public only, 给 /.well-known/jwks.json 用)
    /// v0.47 真实 RSA 公钥组件 (n/e 从 RsaPublicKey 解析)
    pub fn to_jwk(&self) -> Result<Jwk, KeyPairError> {
        Ok(Jwk {
            kty: "RSA".to_string(),
            use_: "sig".to_string(),
            kid: self.kid.clone(),
            alg: "RS256".to_string(),
            n: self.jwk_n.clone(),
            e: self.jwk_e.clone(),
        })
    }

    /// 导出 JWKS (单 key 集合)
    pub fn to_jwks(&self) -> Result<Jwks, KeyPairError> {
        Ok(Jwks {
            keys: vec![self.to_jwk()?],
        })
    }
}

/// 全局 keypair 管理 (在 axum state 中共享)
#[derive(Clone)]
pub struct OAuthKeyManager {
    keypair: Arc<RwLock<OAuthKeyPair>>,
}

impl OAuthKeyManager {
    /// 全新生成
    pub fn new() -> Result<Self, KeyPairError> {
        let kp = OAuthKeyPair::generate()?;
        Ok(Self {
            keypair: Arc::new(RwLock::new(kp)),
        })
    }

    /// 从 PEM 加载 (守门 #5 v2: 来自 env / star-credential)
    pub fn from_pem(private_pem: String, public_pem: String) -> Result<Self, KeyPairError> {
        let kp = OAuthKeyPair::from_pem(private_pem, public_pem)?;
        Ok(Self {
            keypair: Arc::new(RwLock::new(kp)),
        })
    }

    /// 获取 keypair (read guard)
    pub async fn get(&self) -> tokio::sync::RwLockReadGuard<'_, OAuthKeyPair> {
        self.keypair.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_generation_produces_2048_bit_rsa() {
        let kp = OAuthKeyPair::generate().expect("generate");
        assert!(kp.private_pem().contains("BEGIN PRIVATE KEY"));
        assert!(kp.public_pem().contains("BEGIN PUBLIC KEY"));
        // kid 是 16 hex chars (8 bytes)
        assert_eq!(kp.kid().len(), 16);
    }

    #[test]
    fn keypair_from_pem_round_trip() {
        let kp1 = OAuthKeyPair::generate().expect("generate");
        let kp2 =
            OAuthKeyPair::from_pem(kp1.private_pem().to_string(), kp1.public_pem().to_string())
                .expect("from_pem");
        assert_eq!(kp1.kid(), kp2.kid());
    }

    #[test]
    fn jwk_has_rsa_components() {
        let kp = OAuthKeyPair::generate().expect("generate");
        let jwk = kp.to_jwk().expect("to_jwk");
        assert_eq!(jwk.kty, "RSA");
        assert_eq!(jwk.alg, "RS256");
        assert_eq!(jwk.use_, "sig");
        assert!(!jwk.n.is_empty());
        assert!(!jwk.e.is_empty());
        // RSA 2048 modulus 应该是 256 字节 base64url = ~342 chars
        assert!(
            jwk.n.len() > 300,
            "RSA 2048 modulus too short: {}",
            jwk.n.len()
        );
        // Standard public exponent 65537 = AQAB
        assert_eq!(jwk.e, "AQAB");
    }

    #[test]
    fn jwks_contains_one_key() {
        let kp = OAuthKeyPair::generate().expect("generate");
        let jwks = kp.to_jwks().expect("to_jwks");
        assert_eq!(jwks.keys.len(), 1);
        assert_eq!(jwks.keys[0].kid, kp.kid());
    }

    #[test]
    fn keypair_error_codes_are_distinct() {
        assert_eq!(
            KeyPairError::Generation("x".to_string()).code(),
            "OAUTH_KEYGEN_FAILED"
        );
        assert_eq!(
            KeyPairError::PemParse("x".to_string()).code(),
            "OAUTH_PEM_INVALID"
        );
        assert_eq!(
            KeyPairError::JwkEncode("x".to_string()).code(),
            "OAUTH_JWK_ENCODE_FAILED"
        );
        assert_eq!(
            KeyPairError::PrivateExtract("x".to_string()).code(),
            "OAUTH_PRIVATE_EXTRACT_FAILED"
        );
    }

    #[test]
    fn keypair_error_retriable() {
        assert!(KeyPairError::Generation("x".to_string()).is_retriable());
        assert!(!KeyPairError::PemParse("x".to_string()).is_retriable());
        assert!(!KeyPairError::JwkEncode("x".to_string()).is_retriable());
        assert!(!KeyPairError::PrivateExtract("x".to_string()).is_retriable());
    }

    #[test]
    fn real_keygen_produces_distinct_kids() {
        // 真实 RSA keygen 每次产生不同的 kid
        let kp1 = OAuthKeyPair::generate().expect("gen1");
        let kp2 = OAuthKeyPair::generate().expect("gen2");
        assert_ne!(
            kp1.kid(),
            kp2.kid(),
            "real keygen should produce unique kids"
        );
        assert_ne!(kp1.jwk_n, kp2.jwk_n, "real keygen should produce unique n");
    }

    #[test]
    fn private_key_can_be_parsed_back() {
        let kp = OAuthKeyPair::generate().expect("generate");
        let priv_key = kp.private_key().expect("parse back");
        assert_eq!(priv_key.size(), 256); // RSA 2048 = 256 bytes
    }
}
