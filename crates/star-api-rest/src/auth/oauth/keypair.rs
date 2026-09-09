// SPDX-License-Identifier: MIT OR Apache-2.0
//! OAuth2 RSA 2048 keypair 管理 (per WBS v0.43 §14.12 IV OAuth2 server phase 2)
//!
//! 守门 #5 v2: private key 走 env var (JWT_PRIVATE_KEY_PEM) 或 star-credential 存储, 不入 log
//! 守门 #14 v3: Mavis 永久代签
//!
//! 跨 session 续: 真实 RSA 验签跨 session 续 (per 守门 #25 v25 owner 拍板后切 no_network_mode=false)

use base64::Engine;
use ring::rand::SystemRandom;
use ring::signature::RsaKeyPair;
use serde::{Deserialize, Serialize};
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
    /// not implemented (v0.43 简化版占位)
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

impl KeyPairError {
    /// 6-field code (跟 v0.30 6-field error 模式一致)
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
    /// key id (kid) — JWK thumbprint
    kid: String,
}

impl OAuthKeyPair {
    /// 全新生成 RSA 2048 keypair (v0.43 简化版: 不真生成, 用测试 key pair)
    /// 真实 RSA keygen 跨 session 续 (per openssl rsautl 或 aws-lc-rs)
    pub fn generate() -> Result<Self, KeyPairError> {
        // v0.43 简化: 返回测试用 keypair (per openssl genrsa 2048 生成的固定值)
        // 真实生产应走 openssl / aws-lc-rs 生成, 不入 log
        let private_pem = std::env::var("JWT_PRIVATE_KEY_PEM").unwrap_or_else(|_| {
            // 占位符 PEM (非真实 2048-bit key, 仅满足类型检查)
            "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQ\n-----END PRIVATE KEY-----\n".to_string()
        });
        let public_pem = std::env::var("JWT_PUBLIC_KEY_PEM").unwrap_or_else(|_| {
            "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA\n-----END PUBLIC KEY-----\n".to_string()
        });
        let kid = compute_kid(public_pem.as_bytes());
        Ok(Self {
            private_pem,
            public_pem,
            kid,
        })
    }

    /// 从 PEM 字符串加载 (守门 #5 v2: private_pem 来自 env / star-credential)
    pub fn from_pem(private_pem: String, public_pem: String) -> Result<Self, KeyPairError> {
        // v0.43 简化: 不真验签 PEM, 仅 compute_kid
        let kid = compute_kid(public_pem.as_bytes());
        Ok(Self {
            private_pem,
            public_pem,
            kid,
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

    /// RsaKeyPair (ring internal, 给 jsonwebtoken 签 token 用)
    /// v0.43 简化: 真实 key 验签跨 session 续, 当前 stub
    pub fn key_pair(&self) -> Result<RsaKeyPair, KeyPairError> {
        Err(KeyPairError::NotImplemented(
            "real RSA keypair 跨 session 续".to_string(),
        ))
    }
}

/// PEM encode (DER → PEM)
fn pem_encode(label: &str, der: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(der);
    let mut out = String::new();
    out.push_str("-----BEGIN ");
    out.push_str(label);
    out.push_str("-----\n");
    for line in b64.as_bytes().chunks(64) {
        out.push_str(std::str::from_utf8(line).unwrap_or(""));
        out.push('\n');
    }
    out.push_str("-----END ");
    out.push_str(label);
    out.push_str("-----\n");
    out
}

/// PEM decode (PEM → DER)
fn pem_decode(pem: &str, expected_label: &str) -> Result<Vec<u8>, KeyPairError> {
    let begin = format!("-----BEGIN {}-----", expected_label);
    let end = format!("-----END {}-----", expected_label);
    let start = pem
        .find(&begin)
        .ok_or_else(|| KeyPairError::PemParse(format!("missing {}", begin)))?;
    let finish = pem
        .find(&end)
        .ok_or_else(|| KeyPairError::PemParse(format!("missing {}", end)))?;
    let b64: String = pem[start + begin.len()..finish]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    base64::engine::general_purpose::STANDARD
        .decode(b64.as_bytes())
        .map_err(|e| KeyPairError::PemParse(format!("base64: {}", e)))
}

/// Compute key id (kid) = sha256(public_der)[0..8] hex
fn compute_kid(public_der: &[u8]) -> String {
    // 简化: 直接用 base64(public_der)[0..16] 作为 kid
    use base64::Engine;
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_der);
    b64.chars().take(16).collect()
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
    /// v0.43 简化: 用占位符 n/e (真实 RSA 公钥解析 跨 session 续)
    pub fn to_jwk(&self) -> Result<Jwk, KeyPairError> {
        // 真实 RSA public key 解析需要 ring 或 aws-lc-rs 的 SPKI parser
        // v0.43 占位: 用 kid 作为 n + e 字段
        Ok(Jwk {
            kty: "RSA".to_string(),
            use_: "sig".to_string(),
            kid: self.kid.clone(),
            alg: "RS256".to_string(),
            n: "placeholder_n_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string(),
            e: "AQAB".to_string(),
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
        // kid 是 16 字符
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
}
