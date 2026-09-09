// SPDX-License-Identifier: MIT OR Apache-2.0
//! PKCE (Proof Key for Code Exchange) 工具 (per RFC 7636)
//!
//! 守门 #5 v2: code_verifier 不入 log (一次性 token)
//! 守门 #14 v3: Mavis 永久代签

use base64::Engine;
use ring::digest::{digest, SHA256};
use thiserror::Error;

/// PKCE error
#[derive(Debug, Error)]
pub enum PkceError {
    /// invalid code_verifier format
    #[error("PKCE invalid verifier format: {0}")]
    InvalidVerifier(String),
    /// code_challenge 不匹配
    #[error("PKCE challenge mismatch: {0}")]
    ChallengeMismatch(String),
    /// unsupported method
    #[error("PKCE unsupported method: {0}")]
    UnsupportedMethod(String),
}

impl PkceError {
    /// 6-field code
    pub fn code(&self) -> &'static str {
        match self {
            PkceError::InvalidVerifier(_) => "OAUTH_PKCE_VERIFIER_INVALID",
            PkceError::ChallengeMismatch(_) => "OAUTH_PKCE_CHALLENGE_MISMATCH",
            PkceError::UnsupportedMethod(_) => "OAUTH_PKCE_METHOD_UNSUPPORTED",
        }
    }

    /// retriable (per 守门 #6 v2)
    pub fn is_retriable(&self) -> bool {
        false // PKCE 错误都是用户/客户端错误, 不重试
    }
}

/// 验证 PKCE code_verifier 匹配 code_challenge (RFC 7636 §4.6)
///
/// method: "S256" (生产) 或 "plain" (测试, 不推荐)
pub fn verify_pkce(
    code_verifier: &str,
    code_challenge: &str,
    code_challenge_method: &str,
) -> Result<(), PkceError> {
    // 验证 verifier 格式: 43-128 chars, [A-Z][a-z][0-9]-._~
    if code_verifier.len() < 43 || code_verifier.len() > 128 {
        return Err(PkceError::InvalidVerifier(format!(
            "len {} not in [43, 128]",
            code_verifier.len()
        )));
    }
    for c in code_verifier.chars() {
        if !matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '.' | '_' | '~') {
            return Err(PkceError::InvalidVerifier(format!("invalid char: {}", c)));
        }
    }

    match code_challenge_method {
        "S256" => {
            // challenge = base64url(sha256(verifier))
            let hashed = digest(&SHA256, code_verifier.as_bytes());
            let computed = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hashed.as_ref());
            if computed != code_challenge {
                return Err(PkceError::ChallengeMismatch(format!(
                    "S256 expected {}, got {}",
                    code_challenge, computed
                )));
            }
        }
        "plain" => {
            // plain = verifier 直接比较 (仅测试, 不推荐)
            if code_verifier != code_challenge {
                return Err(PkceError::ChallengeMismatch(
                    "plain: verifier != challenge".to_string(),
                ));
            }
        }
        other => {
            return Err(PkceError::UnsupportedMethod(other.to_string()));
        }
    }

    Ok(())
}

/// 生成 code_verifier (测试用, RFC 7636 §4.1)
#[cfg(test)]
pub fn generate_code_verifier() -> String {
    use ring::rand::{SecureRandom, SystemRandom};
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes).expect("rng");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 7636 §4.6 官方测试向量
    /// code_verifier: "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
    /// code_challenge (S256): "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    #[test]
    fn pkce_s256_matches_rfc7636_test_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        verify_pkce(verifier, challenge, "S256").expect("RFC 7636 test vector must pass");
    }

    #[test]
    fn pkce_s256_mismatch_returns_error() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let wrong_challenge = "wrong_challenge_value_aaaaaaaaaaaaaaaaaaaaaaa";
        let result = verify_pkce(verifier, wrong_challenge, "S256");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PkceError::ChallengeMismatch(_)
        ));
    }

    #[test]
    fn pkce_plain_method_supported() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        verify_pkce(verifier, verifier, "plain").expect("plain should match");
    }

    #[test]
    fn pkce_too_short_verifier_rejected() {
        let result = verify_pkce("short", "short", "plain");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PkceError::InvalidVerifier(_)));
    }

    #[test]
    fn pkce_invalid_char_in_verifier_rejected() {
        let bad = "a".repeat(43) + "!invalid";
        let result = verify_pkce(&bad, &bad, "plain");
        assert!(result.is_err());
    }

    #[test]
    fn pkce_unsupported_method_rejected() {
        let verifier = "a".repeat(43);
        let result = verify_pkce(&verifier, &verifier, "SHA1");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PkceError::UnsupportedMethod(_)
        ));
    }

    #[test]
    fn pkce_error_codes_are_distinct() {
        assert_eq!(
            PkceError::InvalidVerifier("x".to_string()).code(),
            "OAUTH_PKCE_VERIFIER_INVALID"
        );
        assert_eq!(
            PkceError::ChallengeMismatch("x".to_string()).code(),
            "OAUTH_PKCE_CHALLENGE_MISMATCH"
        );
        assert_eq!(
            PkceError::UnsupportedMethod("x".to_string()).code(),
            "OAUTH_PKCE_METHOD_UNSUPPORTED"
        );
    }
}
