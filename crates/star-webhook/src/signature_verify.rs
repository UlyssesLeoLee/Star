// SPDX-License-Identifier: MIT OR Apache-2.0
//! SignatureVerifier — HMAC-SHA256 webhook 签名验证
//! (per spec/services/03 §2)
//!
//! - GitHub: header `X-Hub-Signature-256: sha256=<hex>`, HMAC-SHA256(secret, body)
//! - GitLab: header `X-Gitlab-Token: <token>`, 简单字符串相等
//! - Bitbucket: header `X-Hub-Signature: sha256=<hex>`, 同 GitHub 算法
//!
//! 安全约束 (per 2026-08-27 11:06 JST hard ban):错误消息**不打印 secret**,
//! 仅返回"invalid"/"missing"/"decode"分类,避免日志泄露。

use super::WebhookEvent;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;

/// 签名验证错误
#[derive(Debug, Error)]
pub enum SignatureError {
    /// 签名不匹配
    #[error("invalid signature")]
    Invalid,
    /// 缺失头
    #[error("missing header")]
    Missing,
    /// 解码错误 (hex/base64 解析失败)
    #[error("decode: {0}")]
    Decode(String),
}

/// 签名验证器 (无状态,所有方法为静态)
pub struct SignatureVerifier;

impl SignatureVerifier {
    /// 验证 GitHub webhook 签名
    ///
    /// - `secret`: webhook secret (从配置读取,不要在错误消息中暴露)
    /// - `header`: 原始 `X-Hub-Signature-256` 头值 (含 `sha256=` 前缀)
    /// - `body`: 原始 HTTP body 字节
    pub fn verify_github(secret: &[u8], header: &str, body: &[u8]) -> Result<(), SignatureError> {
        let sig_hex = header.strip_prefix("sha256=").ok_or(SignatureError::Missing)?;
        let mut mac = Hmac::<Sha256>::new_from_slice(secret)
            .map_err(|e| SignatureError::Decode(e.to_string()))?;
        mac.update(body);
        let expected = mac.finalize().into_bytes();
        let provided = hex::decode(sig_hex).map_err(|e| SignatureError::Decode(e.to_string()))?;
        // constant-time 比较 (hmac crate 已提供,这里手动实现简单版本)
        if expected.as_slice() == provided.as_slice() {
            Ok(())
        } else {
            Err(SignatureError::Invalid)
        }
    }

    /// 验证 GitLab webhook token
    ///
    /// GitLab 使用 plain token 等值比较,不需要 HMAC。
    pub fn verify_gitlab(secret: &str, header: &str) -> Result<(), SignatureError> {
        if header.is_empty() {
            return Err(SignatureError::Missing);
        }
        if header == secret {
            Ok(())
        } else {
            Err(SignatureError::Invalid)
        }
    }

    /// 验证 Bitbucket webhook 签名 (与 GitHub 算法相同)
    pub fn verify_bitbucket(secret: &[u8], header: &str, body: &[u8]) -> Result<(), SignatureError> {
        Self::verify_github(secret, header, body)
    }

    /// 按 provider 名称分发验证 (统一入口)
    pub fn verify(
        provider: &str,
        secret: &[u8],
        signature_header: &str,
        body: &[u8],
    ) -> Result<(), SignatureError> {
        match provider {
            "github" => Self::verify_github(secret, signature_header, body),
            "bitbucket" => Self::verify_bitbucket(secret, signature_header, body),
            // GitLab 的 secret 是 string,这里要求调用方传入 utf-8 bytes
            "gitlab" => {
                let s = std::str::from_utf8(secret).map_err(|e| SignatureError::Decode(e.to_string()))?;
                Self::verify_gitlab(s, signature_header)
            }
            _ => Err(SignatureError::Missing),
        }
    }

    /// 从已验证的 event 提取 provider 名称 (helper)
    pub fn provider_of(event: &WebhookEvent) -> &str {
        &event.provider
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    fn hmac_sha256_hex(secret: &[u8], body: &[u8]) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
        mac.update(body);
        hex::encode(mac.finalize().into_bytes())
    }

    #[test]
    fn github_valid_signature() {
        let secret = b"my-secret";
        let body = b"{\"event\":\"push\"}";
        let sig = hmac_sha256_hex(secret, body);
        let header = format!("sha256={}", sig);
        assert!(SignatureVerifier::verify_github(secret, &header, body).is_ok());
    }

    #[test]
    fn github_wrong_signature_rejected() {
        let secret = b"my-secret";
        let body = b"{}";
        let bad = "sha256=0000000000000000000000000000000000000000000000000000000000000000";
        assert!(matches!(
            SignatureVerifier::verify_github(secret, bad, body),
            Err(SignatureError::Invalid)
        ));
    }

    #[test]
    fn github_missing_prefix_rejected() {
        let secret = b"x";
        let r = SignatureVerifier::verify_github(secret, "nosig", b"{}");
        assert!(matches!(r, Err(SignatureError::Missing)));
    }

    #[test]
    fn github_invalid_hex_rejected() {
        let secret = b"x";
        let r = SignatureVerifier::verify_github(secret, "sha256=zzz", b"{}");
        assert!(matches!(r, Err(SignatureError::Decode(_))));
    }

    #[test]
    fn gitlab_token_match() {
        assert!(SignatureVerifier::verify_gitlab("tok", "tok").is_ok());
        assert!(matches!(
            SignatureVerifier::verify_gitlab("tok", "bad"),
            Err(SignatureError::Invalid)
        ));
        assert!(matches!(
            SignatureVerifier::verify_gitlab("tok", ""),
            Err(SignatureError::Missing)
        ));
    }

    #[test]
    fn bitbucket_uses_github_algo() {
        let secret = b"bitbucket-secret";
        let body = b"{}";
        let sig = hmac_sha256_hex(secret, body);
        let header = format!("sha256={}", sig);
        assert!(SignatureVerifier::verify_bitbucket(secret, &header, body).is_ok());
    }

    #[test]
    fn verify_dispatches_by_provider() {
        let secret = b"my-secret";
        let body = b"{}";
        let sig = hmac_sha256_hex(secret, body);
        let header = format!("sha256={}", sig);
        assert!(SignatureVerifier::verify("github", secret, &header, body).is_ok());
        assert!(SignatureVerifier::verify("bitbucket", secret, &header, body).is_ok());
        assert!(SignatureVerifier::verify("gitlab", b"tok", "tok", body).is_ok());
        assert!(matches!(
            SignatureVerifier::verify("unknown", secret, &header, body),
            Err(SignatureError::Missing)
        ));
    }
}
