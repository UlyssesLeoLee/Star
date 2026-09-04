//! crates/star-credential — Star 仓凭证管理层
//!
//! 真实应用场景: 用户在 Star 设置 UI 自行填入 OpenClaw / Hermes / KMS 凭证
//! 落地路径: UI 填明文 → API 接收 → KMS 加密 → 入库 (Master 类型, 守门 #DB-13)
//! 运行时: 调用方从 manager 取凭证 → KMS 解密 → 用明文调真实 endpoint
//!
//! per 守门 #5 (env 安全) + 守门 #14 (5 域 Lead CONTENT 4 维) + 守门 #DB-13 (W/T/M)
//!
//! 关键不变量:
//! - INV-CR-01: 明文凭证不在 log/stdout/println 出现 (守门 #5 派生)
//! - INV-CR-02: 加密后入库 (DB Master, 永存不删, 物理删除禁止)
//! - INV-CR-03: KMS 解密失败 → 立即返 Err, 不 panic
//! - INV-CR-04: tenant_id 必填 (RLS 13 類, 守门 #DB-13 CW-05)
//! - INV-CR-05: 凭证轮换 (rotate) 生成新 ciphertext, 老 ciphertext 标记 deprecated
//! - INV-CR-06: 凭证撤销 (revoke) 仅标记 revoked_at, 不物理删除 (Master 物理删除禁止)

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

use domain_kms::{KmsClient, LocalMockKms, EncryptedBlob, KeyId};

/// 凭证 Provider (5 类)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Provider {
    /// OpenClaw (LLM agent 编排, B.5)
    OpenClaw,
    /// Hermes (消息总线, B.6)
    Hermes,
    /// KMS: HashiCorp Vault
    KmsVault,
    /// KMS: AWS KMS
    KmsAws,
    /// KMS: 本地 mock (per 5ea9611, F.3 默认)
    KmsLocalMock,
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenClaw => "openclaw",
            Self::Hermes => "hermes",
            Self::KmsVault => "kms_vault",
            Self::KmsAws => "kms_aws",
            Self::KmsLocalMock => "kms_local_mock",
        }
    }
}

/// 凭证元数据 (UI 显示用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    /// 显示名 (e.g. "我的 OpenClaw 账号")
    pub display_name: String,
    /// 描述 (e.g. "用于 LangGraph sub-agent 派发")
    pub description: String,
}

/// 加密后入库的凭证记录 (Master 类型, 守门 #DB-13)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRecord {
    pub id: String,                    // UUID v4
    pub tenant_id: String,             // RLS 必填 (守门 #DB-13 CW-05)
    pub user_id: String,               // 创建者 user_id
    pub provider: Provider,            // 5 类 Provider
    pub metadata: CredentialMetadata,  // UI 显示用
    pub encrypted_blob: EncryptedBlob,// KMS 加密后的密文 (含 dek_id + encrypted_dek + nonce + ciphertext)
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    /// 凭证状态
    pub status: CredentialStatus,
    /// 老 ciphertext 标记 (per INV-CR-05 轮换)
    pub deprecated_at_ms: Option<u64>,
    /// 撤销时间 (per INV-CR-06, 标记不删)
    pub revoked_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialStatus {
    Active,
    Deprecated, // 被新凭证替代, 仍可解密用于回退
    Revoked,    // 撤销, 解密返 Err
}

/// 凭证明文 (在内存中, 永不入 log)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialPlaintext {
    /// API key (OpenClaw / Hermes) 或 token (KMS)
    pub secret: String,
    /// 可选 base_url (per F.1/F.2 默认值)
    pub base_url: Option<String>,
    /// 可选 region (AWS KMS)
    pub region: Option<String>,
}

/// 凭证操作错误
#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("credential not found: {0}")]
    NotFound(String),
    #[error("credential revoked: {0}")]
    Revoked(String),
    #[error("credential deprecated: {0}")]
    Deprecated(String),
    #[error("KMS encrypt error: {0}")]
    KmsEncrypt(String),
    #[error("KMS decrypt error: {0}")]
    KmsDecrypt(String),
    #[error("invalid plaintext: {0}")]
    InvalidPlaintext(String),
    #[error("internal: {0}")]
    Internal(String),
}

/// 凭证管理器 (用户 UI 填 → 加密存储 → 运行时解密)
pub struct CredentialManager {
    kms: Arc<dyn KmsClient>,
    /// 内存索引: id -> CredentialRecord
    records: Arc<RwLock<HashMap<String, CredentialRecord>>>,
    /// 索引: (tenant_id, provider) -> Vec<id> (一个 tenant 可有多个 OpenClaw 凭证)
    by_tenant_provider: Arc<RwLock<HashMap<(String, Provider), Vec<String>>>>,
}

impl CredentialManager {
    /// 新建凭证管理器 (KMS client 由调用方注入, 默认 LocalMockKms)
    pub fn new(kms: Arc<dyn KmsClient>) -> Self {
        Self {
            kms,
            records: Arc::new(RwLock::new(HashMap::new())),
            by_tenant_provider: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 默认 + LocalMockKms (per 守门 #19 [M] 拍板 F.3 mock maturity)
    pub fn with_local_mock_kms() -> Self {
        Self::new(Arc::new(LocalMockKms::new()))
    }

    /// 用户在 UI 填入凭证 → 加密 → 入库
    /// 返回 credential_id (供 UI 显示)
    pub async fn store(
        &self,
        tenant_id: &str,
        user_id: &str,
        provider: Provider,
        metadata: CredentialMetadata,
        plaintext: CredentialPlaintext,
    ) -> Result<String, CredentialError> {
        // 1. 序列化明文
        let plaintext_json = serde_json::to_vec(&plaintext)
            .map_err(|e| CredentialError::InvalidPlaintext(e.to_string()))?;

        // 2. KMS 生成 DEK (per tenant envelope encryption, INV-KMS-02)
        let (dek_id, _encrypted_dek_blob) = self
            .kms
            .generate_dek(&tenant_id.to_string())
            .await
            .map_err(|e| CredentialError::KmsEncrypt(format!("generate_dek: {}", e)))?;

        // 3. KMS 加密明文
        let encrypted_blob = self
            .kms
            .encrypt(&tenant_id.to_string(), &dek_id, &plaintext_json)
            .await
            .map_err(|e| CredentialError::KmsEncrypt(format!("encrypt: {}", e)))?;

        // 4. 创建记录
        let id = Uuid::new_v4().to_string();
        let now = now_ms();
        let record = CredentialRecord {
            id: id.clone(),
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
            provider,
            metadata,
            encrypted_blob,
            created_at_ms: now,
            updated_at_ms: now,
            status: CredentialStatus::Active,
            deprecated_at_ms: None,
            revoked_at_ms: None,
        };

        // 5. 入库 (内存 + 索引)
        self.records.write().await.insert(id.clone(), record);
        self.by_tenant_provider
            .write()
            .await
            .entry((tenant_id.to_string(), provider))
            .or_default()
            .push(id.clone());

        // 守门 #5: 不打印任何凭证内容
        Ok(id)
    }

    /// 运行时调用方取凭证 → 解密 → 用明文调真实 endpoint
    /// 守门 #5: 返回的 plaintext 在调用方负责立即用完丢弃, 不入 log
    pub async fn retrieve(
        &self,
        tenant_id: &str,
        provider: Provider,
    ) -> Result<CredentialPlaintext, CredentialError> {
        // 1. 找凭证 (per (tenant_id, provider) 索引)
        let id = {
            let index = self.by_tenant_provider.read().await;
            let ids = index
                .get(&(tenant_id.to_string(), provider))
                .ok_or_else(|| CredentialError::NotFound(format!("{}/{}", tenant_id, provider.as_str())))?
                .clone();
            drop(index);
            let records = self.records.read().await;
            // 优先: 最新 active; 其次: 最新 deprecated; 末: 最新 revoked
            let mut found_active = None;
            let mut found_deprecated = None;
            let mut found_revoked = None;
            for id in ids.iter().rev() {
                if let Some(r) = records.get(id) {
                    match r.status {
                        CredentialStatus::Active if found_active.is_none() => found_active = Some(id.clone()),
                        CredentialStatus::Deprecated if found_deprecated.is_none() => found_deprecated = Some(id.clone()),
                        CredentialStatus::Revoked if found_revoked.is_none() => found_revoked = Some(id.clone()),
                        _ => {}
                    }
                }
            }
            if let Some(id) = found_active {
                id
            } else if let Some(id) = found_deprecated {
                return Err(CredentialError::Deprecated(id));
            } else if let Some(id) = found_revoked {
                return Err(CredentialError::Revoked(id));
            } else {
                return Err(CredentialError::NotFound(format!("no usable credential for {}/{}", tenant_id, provider.as_str())));
            }
        };

        // 2. KMS 解密
        let record = self.records.read().await.get(&id).cloned()
            .ok_or_else(|| CredentialError::NotFound(id.clone()))?;
        let plaintext_bytes = self
            .kms
            .decrypt(&record.tenant_id, &record.encrypted_blob)
            .await
            .map_err(|e| CredentialError::KmsDecrypt(format!("{}", e)))?;

        // 3. 反序列化
        let plaintext: CredentialPlaintext = serde_json::from_slice(&plaintext_bytes)
            .map_err(|e| CredentialError::InvalidPlaintext(e.to_string()))?;

        Ok(plaintext)
    }

    /// 凭证轮换: 老凭证标 deprecated, 新凭证 active
    /// 守门 #5: 调用方传新明文, 返回新 credential_id
    pub async fn rotate(
        &self,
        tenant_id: &str,
        user_id: &str,
        provider: Provider,
        new_metadata: CredentialMetadata,
        new_plaintext: CredentialPlaintext,
    ) -> Result<String, CredentialError> {
        // 1. 标老凭证 deprecated
        {
            let index = self.by_tenant_provider.read().await;
            let ids = index.get(&(tenant_id.to_string(), provider)).cloned();
            drop(index);
            if let Some(ids) = ids {
                let mut records = self.records.write().await;
                for id in ids {
                    if let Some(r) = records.get_mut(&id) {
                        if r.status == CredentialStatus::Active {
                            r.status = CredentialStatus::Deprecated;
                            r.deprecated_at_ms = Some(now_ms());
                        }
                    }
                }
            }
        }

        // 2. 存新凭证
        self.store(tenant_id, user_id, provider, new_metadata, new_plaintext)
            .await
    }

    /// 凭证撤销: 标 revoked, 不删 (per INV-CR-06)
    pub async fn revoke(&self, credential_id: &str) -> Result<(), CredentialError> {
        let mut records = self.records.write().await;
        let record = records
            .get_mut(credential_id)
            .ok_or_else(|| CredentialError::NotFound(credential_id.to_string()))?;
        record.status = CredentialStatus::Revoked;
        record.revoked_at_ms = Some(now_ms());
        Ok(())
    }

    /// UI 列出 tenant 凭证 (不含密文, per 守门 #5)
    pub async fn list(
        &self,
        tenant_id: &str,
        provider: Option<Provider>,
    ) -> Vec<CredentialRecord> {
        let records = self.records.read().await;
        records
            .values()
            .filter(|r| r.tenant_id == tenant_id)
            .filter(|r| provider.map(|p| r.provider == p).unwrap_or(true))
            .cloned()
            .collect()
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
    .unwrap_or(0)
}

#[cfg(test)]
mod tests;

pub mod api;
pub mod db;
