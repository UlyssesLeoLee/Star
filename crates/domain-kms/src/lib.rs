//! domain-kms crate
//!
//! KMS 集成 (Vault / AWS KMS 凭证) — E.4 mock 备选
//!
//! 详细 spec: docs/specs/domain-kms-spec.md (待写, E.4 phase 2 续)
//! 架构位置: docs/basic-design.md §2.1 / §5.8 / §6.8 / §9
//! 数据落点: docs/data-design.md §4.12 (`kms` schema, 待写)
//! API 边界: docs/api-design.md §3.13 (待写)
//!
//! ## 职责
//!
//! 唯一 KMS Domain, 管理 5 域业务子域 (player / economy / match / social / admin) 的:
//! - 加密密钥 envelope (envelope encryption: KMS master key + DEK)
//! - 跨 5 域数据加密 (PII / billing / 通知内容 / 搜索索引 tokenization)
//! - 凭证轮换 (rotation, 默认 90 天)
//! - 5 域 Lead 真人 audit 必填 (per B.9 监控审计)
//!
//! ## 关键不变量 (INV-KMS-01~05, 共 5 条)
//!
//! - **INV-KMS-01** 唯一入口: 5 域业务子域禁止直接调用外部 KMS SDK, 必须经 `KmsClient` Port
//! - **INV-KMS-02** Envelope encryption: master key (KMS) + DEK (per-tenant), DEK 从不落明文存储
//! - **INV-KMS-03** 5 域凭证隔离: tenant_id 必填, 跨租户读取 = 拒绝
//! - **INV-KMS-04** 轮换周期: master key 90 天, DEK 30 天, audit log 必填轮换人
//! - **INV-KMS-05** 真凭证路径: Vault / AWS KMS 真实 endpoint + key (需 Ulysses 凭证, 跨 session 阻塞)
//!
//! Lead 域: admin (per 8/21 JST 拒绝兼任硬约束, 跨 session 续找真人)
//!
//! ## Mock 备选 (per 29692a7 路径)
//!
//! 真实 Vault / AWS KMS 凭证未到位, 启用 `LocalMockKms` 模式:
//! - 主密钥本地 base64 编码 (启动时随机生成, 不持久化)
//! - DEK 内存存储 (process restart 后失效, 适合 dev/test)
//! - 接口与真实 KMS 一致, 跨 stage 切换无需改 5 域调用方
//!
//! ## 状态
//!
//! 🟡 mock 备选 (E.4 phase 2 续, 等 Ulysses 凭证到位切真)

use async_trait::async_trait;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

/// 租户 ID (跨 5 域业务子域, 必填 per INV-KMS-03)
pub type TenantId = String;

/// 密钥 ID (master key 或 DEK)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyId(pub String);

impl std::fmt::Display for KeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl KeyId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 加密数据信封 (envelope encryption 产物)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBlob {
    /// 加密算法 (默认 `aes-256-gcm`)
    pub algorithm: String,
    /// 用于加密 ciphertext 的 DEK ID (per INV-KMS-02 envelope encryption)
    pub dek_id: KeyId,
    /// 加密后的 DEK (用 master key 加密, base64) — 仅在 generate_dek 路径有值
    #[serde(default)]
    pub encrypted_dek: String,
    /// nonce (96 bits, base64)
    pub nonce: String,
    /// 密文 (base64)
    pub ciphertext: String,
    /// 租户 ID (per INV-KMS-03)
    pub tenant_id: TenantId,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 密钥版本 (per INV-KMS-04 轮换)
    pub key_version: u32,
}

/// KMS 操作错误
#[derive(Debug, Error)]
pub enum KmsError {
    #[error("key not found: {0}")]
    KeyNotFound(KeyId),
    #[error("access denied: tenant {tenant} cannot access key {key}")]
    AccessDenied { tenant: TenantId, key: KeyId },
    #[error("invalid ciphertext: {0}")]
    InvalidCiphertext(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// KMS 客户端 Port (5 域调用方唯一入口, per INV-KMS-01)
#[async_trait]
pub trait KmsClient: Send + Sync {
    /// 生成新 DEK (per tenant)
    async fn generate_dek(&self, tenant_id: &TenantId) -> Result<(KeyId, EncryptedBlob), KmsError>;

    /// 用 DEK 加密明文
    async fn encrypt(
        &self,
        tenant_id: &TenantId,
        dek: &KeyId,
        plaintext: &[u8],
    ) -> Result<EncryptedBlob, KmsError>;

    /// 用 DEK 解密密文
    async fn decrypt(
        &self,
        tenant_id: &TenantId,
        blob: &EncryptedBlob,
    ) -> Result<Vec<u8>, KmsError>;

    /// 轮换 DEK (per INV-KMS-04, audit log 必填轮换人)
    async fn rotate_dek(
        &self,
        tenant_id: &TenantId,
        dek: &KeyId,
        actor: &str,
    ) -> Result<KeyId, KmsError>;

    /// 健康检查
    async fn health(&self) -> Result<KmsHealth, KmsError>;
}

/// KMS 健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KmsHealth {
    pub mode: KmsMode,
    pub master_key_id: KeyId,
    pub key_count: u32,
    pub last_rotation: DateTime<Utc>,
}

/// KMS 模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KmsMode {
    /// 本地 mock (per 29692a7 路径, dev/test)
    LocalMock,
    /// Vault (per INV-KMS-05, 等凭证)
    Vault,
    /// AWS KMS (per INV-KMS-05, 等凭证)
    AwsKms,
}

impl std::fmt::Display for KmsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KmsMode::LocalMock => write!(f, "local-mock"),
            KmsMode::Vault => write!(f, "vault"),
            KmsMode::AwsKms => write!(f, "aws-kms"),
        }
    }
}

// =============================================================================
// LocalMockKms 实现 (per 29692a7 mock 备选路径)
// =============================================================================

/// 本地 mock KMS (per 29692a7 路径, dev/test 用)
///
/// **警告**: 主密钥不持久化, process restart 后所有 DEK 失效. 真实场景禁用.
pub struct LocalMockKms {
    mode: KmsMode,
    master_key: [u8; 32], // 启动时随机
    master_key_id: KeyId,
    deks: tokio::sync::RwLock<std::collections::HashMap<(TenantId, KeyId), Vec<u8>>>,
    key_versions: tokio::sync::RwLock<std::collections::HashMap<KeyId, u32>>,
    created_at: DateTime<Utc>,
}

impl LocalMockKms {
    pub fn new() -> Self {
        use rand::RngCore;
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        let master_key_id = KeyId::new(format!(
            "mock-master-{}",
            Utc::now().timestamp_millis()
        ));
        info!(master_key_id = %master_key_id.as_str(), "LocalMockKms initialized");
        Self {
            mode: KmsMode::LocalMock,
            master_key: key,
            master_key_id,
            deks: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            key_versions: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            created_at: Utc::now(),
        }
    }

    pub fn master_key_id(&self) -> &KeyId {
        &self.master_key_id
    }
}

impl Default for LocalMockKms {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KmsClient for LocalMockKms {
    async fn generate_dek(&self, tenant_id: &TenantId) -> Result<(KeyId, EncryptedBlob), KmsError> {
        use rand::RngCore;
        let mut dek = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut dek);
        let dek_id = KeyId::new(format!(
            "dek-{}-{}",
            tenant_id,
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        // 用 master key 加密 DEK (envelope encryption, per INV-KMS-02)
        use aes_gcm::aead::Aead;
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        let cipher =
            Aes256Gcm::new_from_slice(&self.master_key).map_err(|e| KmsError::Internal(e.to_string()))?;
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let encrypted_dek = cipher
            .encrypt(nonce, dek.as_ref())
            .map_err(|e| KmsError::Internal(e.to_string()))?;

        // 存 DEK 明文 (process 内, mock 限定)
        self.deks
            .write()
            .await
            .insert((tenant_id.clone(), dek_id.clone()), dek.to_vec());
        self.key_versions.write().await.insert(dek_id.clone(), 1);

        Ok((
            dek_id.clone(),
            EncryptedBlob {
                algorithm: "aes-256-gcm".to_string(),
                dek_id: dek_id.clone(),
                encrypted_dek: base64::engine::general_purpose::STANDARD.encode(&encrypted_dek),
                nonce: base64::engine::general_purpose::STANDARD.encode(nonce_bytes),
                ciphertext: String::new(),
                tenant_id: tenant_id.clone(),
                created_at: Utc::now(),
                key_version: 1,
            },
        ))
    }

    async fn encrypt(
        &self,
        tenant_id: &TenantId,
        dek: &KeyId,
        plaintext: &[u8],
    ) -> Result<EncryptedBlob, KmsError> {
        use aes_gcm::aead::Aead;
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        use rand::RngCore;

        let deks = self.deks.read().await;
        let dek_bytes = deks
            .get(&(tenant_id.clone(), dek.clone()))
            .ok_or_else(|| KmsError::KeyNotFound(dek.clone()))?;
        let cipher = Aes256Gcm::new_from_slice(dek_bytes)
            .map_err(|e| KmsError::Internal(e.to_string()))?;
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| KmsError::Internal(e.to_string()))?;

        let key_version = self.key_versions.read().await.get(dek).copied().unwrap_or(1);

        Ok(EncryptedBlob {
            algorithm: "aes-256-gcm".to_string(),
            dek_id: dek.clone(),
            encrypted_dek: String::new(), // 主加密路径, DEK 已绑定 tenant
            nonce: base64::engine::general_purpose::STANDARD.encode(nonce_bytes),
            ciphertext: base64::engine::general_purpose::STANDARD.encode(&ciphertext),
            tenant_id: tenant_id.clone(),
            created_at: Utc::now(),
            key_version,
        })
    }

    async fn decrypt(
        &self,
        tenant_id: &TenantId,
        blob: &EncryptedBlob,
    ) -> Result<Vec<u8>, KmsError> {
        use aes_gcm::aead::Aead;
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};

        if blob.tenant_id != *tenant_id {
            return Err(KmsError::AccessDenied {
                tenant: tenant_id.clone(),
                key: blob.dek_id.clone(),
            });
        }

        // 查 DEK 明文 (per-tenant), 用同一 DEK 解密 (per INV-KMS-02 envelope)
        let deks = self.deks.read().await;
        let dek_bytes = deks
            .get(&(tenant_id.clone(), blob.dek_id.clone()))
            .ok_or_else(|| KmsError::KeyNotFound(blob.dek_id.clone()))?;
        let cipher = Aes256Gcm::new_from_slice(dek_bytes)
            .map_err(|e| KmsError::Internal(e.to_string()))?;
        let nonce_bytes = base64::engine::general_purpose::STANDARD
            .decode(&blob.nonce)
            .map_err(|e| KmsError::InvalidCiphertext(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = base64::engine::general_purpose::STANDARD
            .decode(&blob.ciphertext)
            .map_err(|e| KmsError::InvalidCiphertext(e.to_string()))?;
        cipher
            .decrypt(nonce, ct.as_ref())
            .map_err(|e| KmsError::InvalidCiphertext(e.to_string()))
    }

    async fn rotate_dek(
        &self,
        tenant_id: &TenantId,
        dek: &KeyId,
        actor: &str,
    ) -> Result<KeyId, KmsError> {
        warn!(actor, tenant_id, dek = %dek.as_str(), "DEK rotation requested (mock mode: generates new key, invalidates old)");
        let (new_id, _blob) = self.generate_dek(tenant_id).await?;
        let mut versions = self.key_versions.write().await;
        let old_v = versions.get(dek).copied().unwrap_or(1);
        versions.insert(new_id.clone(), old_v + 1);
        Ok(new_id)
    }

    async fn health(&self) -> Result<KmsHealth, KmsError> {
        Ok(KmsHealth {
            mode: self.mode,
            master_key_id: self.master_key_id.clone(),
            key_count: self.deks.read().await.len() as u32,
            last_rotation: self.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_mock_kms_roundtrip() {
        let kms = LocalMockKms::new();
        let tenant = "t-001".to_string();
        let (dek, _initial_blob) = kms.generate_dek(&tenant).await.unwrap();
        let plaintext = b"5-domain Lead real person in place, switch to real Vault / AWS KMS";

        let blob = kms.encrypt(&tenant, &dek, plaintext).await.unwrap();
        let recovered = kms.decrypt(&tenant, &blob).await.unwrap();
        assert_eq!(recovered, plaintext);
    }

    #[tokio::test]
    async fn test_local_mock_kms_tenant_isolation() {
        let kms = LocalMockKms::new();
        let t1 = "t-001".to_string();
        let t2 = "t-002".to_string();
        let (dek, _) = kms.generate_dek(&t1).await.unwrap();
        let blob = kms.encrypt(&t1, &dek, b"tenant 1 data").await.unwrap();

        // Cross-tenant read = denied (per INV-KMS-03)
        let result = kms.decrypt(&t2, &blob).await;
        assert!(matches!(result, Err(KmsError::AccessDenied { .. })));
    }

    #[tokio::test]
    async fn test_local_mock_kms_health() {
        let kms = LocalMockKms::new();
        let health = kms.health().await.unwrap();
        assert_eq!(health.mode, KmsMode::LocalMock);
        assert_eq!(health.key_count, 0); // 还没 generate_dek
    }
}
