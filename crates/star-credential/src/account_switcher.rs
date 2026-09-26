//! crates/star-credential/src/account_switcher.rs
//!
//! ULYS-203 FR-ORCA-042 §14: Account Switcher + Usage Tracking
//!
//! 不变量:
//! - INV-AS-01: switch 不丢其它 provider 的 active 选择 (per (tenant, provider) 独立)
//! - INV-AS-02: 4 类 LLM (OpenAI / Anthropic / GitHub Copilot / Google) 才允许 switch
//! - INV-AS-03: 用 token 持久化通过 star-quota::QuotaStore 复用 (LlmToken ResourceKind)
//! - INV-AS-04: Usage 按天 bucket 聚合 (day_bucket = YYYY-MM-DD)
//! - INV-AS-05: credential 撤销后 active 选择自动清 (retrieve 时检查 status)

use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use star_quota::{QuotaError, QuotaStore, ResourceKind};

use crate::db::{AccountSelection, CredentialDb};
use crate::{CredentialError, CredentialManager, CredentialPlaintext, CredentialStatus, Provider};

/// Account Switcher 错误
#[derive(Debug, Error)]
pub enum SwitcherError {
    /// 底层 CredentialError 透传
    #[error("credential: {0}")]
    Credential(#[from] CredentialError),
    /// DB 错误
    #[error("db: {0}")]
    Db(#[from] crate::db::DbError),
    /// Quota 错误
    #[error("quota: {0}")]
    Quota(#[from] QuotaError),
    /// Provider 不是 4 类 LLM 之一
    #[error("not an LLM provider: {0}")]
    NotLlm(Provider),
    /// active 选择的 credential 已 revoked, 不能继续使用
    #[error("active selection revoked: credential_id={0}")]
    ActiveRevoked(String),
}

/// Account Switcher (per tenant + LLM provider → active credential_id)
///
/// 持有:
/// - CredentialManager (凭证存储 + KMS 解密)
/// - CredentialDb (active selection 持久化 + usage bucket)
/// - QuotaStore (LlmToken 配额检查, via star-quota)
pub struct AccountSwitcher {
    manager: Arc<CredentialManager>,
    db: Arc<CredentialDb>,
    quota: Arc<dyn QuotaStore>,
    /// 内存缓存: (tenant, provider) → active credential_id
    /// 与 db.account_selection 同步; 启动时从 db 加载
    cache: RwLock<std::collections::HashMap<(String, Provider), String>>,
}

/// Usage 聚合视图 (跨多个 day_bucket 求和)
#[derive(Debug, Clone)]
pub struct UsageSummary {
    /// credential_id
    pub credential_id: String,
    /// LLM provider
    pub provider: Provider,
    /// 输入 token 累计
    pub total_tokens_in: u64,
    /// 输出 token 累计
    pub total_tokens_out: u64,
    /// 输入+输出 token 总和
    pub total_tokens: u64,
    /// 调用次数累计
    pub total_call_count: u64,
    /// day bucket 数
    pub day_count: u32,
    /// 最早一次使用时间(毫秒)
    pub first_used_at_ms: Option<u64>,
    /// 最近一次使用时间(毫秒)
    pub last_used_at_ms: Option<u64>,
}

impl AccountSwitcher {
    /// 构造 AccountSwitcher
    pub fn new(
        manager: Arc<CredentialManager>,
        db: Arc<CredentialDb>,
        quota: Arc<dyn QuotaStore>,
    ) -> Self {
        Self {
            manager,
            db,
            quota,
            cache: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// 毫秒时间戳 → YYYY-MM-DD (UTC)
    fn day_bucket_from_ms(ms: u64) -> String {
        let secs = (ms / 1000) as i64;
        chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "1970-01-01".to_string())
    }

    /// 当前毫秒时间戳 (helper for tests)
    pub fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// 切换 tenant/provider 的 active credential_id
    ///
    /// 步骤:
    /// 1. 校验 provider 是 4 类 LLM 之一
    /// 2. 校验目标 credential 存在且属于该 tenant + provider, 且 status = Active
    /// 3. upsert account_selection 表 (per (tenant, provider) 唯一)
    /// 4. 更新内存缓存
    pub async fn switch(
        &self,
        tenant_id: &str,
        provider: Provider,
        new_credential_id: &str,
    ) -> Result<(), SwitcherError> {
        if !provider.is_llm() {
            return Err(SwitcherError::NotLlm(provider));
        }

        // 校验 credential 存在且 active
        let records = self.manager.list(tenant_id, Some(provider)).await;
        let target = records
            .iter()
            .find(|r| r.id == new_credential_id)
            .ok_or_else(|| {
                SwitcherError::Credential(CredentialError::NotFound(new_credential_id.to_string()))
            })?;
        if target.status != CredentialStatus::Active {
            return Err(SwitcherError::Credential(CredentialError::Deprecated(
                new_credential_id.to_string(),
            )));
        }

        let sel = AccountSelection {
            tenant_id: tenant_id.to_string(),
            provider,
            active_credential_id: new_credential_id.to_string(),
            switched_at_ms: Self::now_ms(),
        };
        self.db.upsert_selection(&sel)?;
        self.cache.write().await.insert(
            (tenant_id.to_string(), provider),
            new_credential_id.to_string(),
        );
        Ok(())
    }

    /// 取 (tenant, provider) 当前 selection 的 raw credential_id (不校验状态)
    ///
    /// 与 `active()` 区别: 即使 credential 已被 revoke/deprecate, 这里仍返 Some(id)
    /// `retrieve_active` 用此方法来区分 "从未 switch" (NotFound) vs "已 revoked" (ActiveRevoked)
    async fn raw_active_id(
        &self,
        tenant_id: &str,
        provider: Provider,
    ) -> Result<Option<String>, SwitcherError> {
        let cache = self.cache.read().await;
        if let Some(id) = cache.get(&(tenant_id.to_string(), provider)) {
            return Ok(Some(id.clone()));
        }
        drop(cache);
        let sel = self.db.get_selection(tenant_id, provider)?;
        if let Some(s) = sel {
            self.cache.write().await.insert(
                (tenant_id.to_string(), provider),
                s.active_credential_id.clone(),
            );
            Ok(Some(s.active_credential_id))
        } else {
            Ok(None)
        }
    }

    /// 查 (tenant, provider) 当前 active credential_id (None = 未选 / 已被撤销)
    ///
    /// 校验: 即使 cache/db 里有 selection, 若对应 credential 已 revoked 则返 None
    /// 并清缓存 (per INV-AS-05, 保证 `active()` 永远不会返一个 revoked 的 credential_id)
    pub async fn active(
        &self,
        tenant_id: &str,
        provider: Provider,
    ) -> Result<Option<String>, SwitcherError> {
        if !provider.is_llm() {
            return Err(SwitcherError::NotLlm(provider));
        }
        // 1. 取 raw selection id (cache 优先, fallback db)
        let candidate_id = match self.raw_active_id(tenant_id, provider).await? {
            Some(id) => id,
            None => return Ok(None),
        };

        // 2. 校验 credential 仍 Active (per INV-AS-05)
        let records = self.manager.list(tenant_id, Some(provider)).await;
        let is_active = records
            .iter()
            .any(|r| r.id == candidate_id && r.status == CredentialStatus::Active);

        if is_active {
            Ok(Some(candidate_id))
        } else {
            // 已 revoke/deprecate — 清缓存, 让后续 switch 重选
            self.cache
                .write()
                .await
                .remove(&(tenant_id.to_string(), provider));
            // Note: account_selection.db 行保留供审计, 不物理删除 (per 守门 #DB-13 Master)
            Ok(None)
        }
    }

    /// 列出 tenant 所有 active selections (per provider)
    pub async fn list_active(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<AccountSelection>, SwitcherError> {
        let sels = self.db.list_selections(tenant_id)?;
        // 同步缓存
        {
            let mut cache = self.cache.write().await;
            for s in &sels {
                cache.insert(
                    (tenant_id.to_string(), s.provider),
                    s.active_credential_id.clone(),
                );
            }
        }
        Ok(sels)
    }

    /// 运行时调用方取 active credential 的明文
    ///
    /// 步骤:
    /// 1. 取 active credential_id (缓存 → db, 用 raw_active_id 不做状态过滤)
    /// 2. 若无 selection → 返 NotFound ("no active for ...")
    /// 3. 若 credential 已 revoked → 清缓存 + 返 ActiveRevoked (per INV-AS-05)
    /// 4. CredentialManager.retrieve 解密 → 明文
    pub async fn retrieve_active(
        &self,
        tenant_id: &str,
        provider: Provider,
    ) -> Result<CredentialPlaintext, SwitcherError> {
        if !provider.is_llm() {
            return Err(SwitcherError::NotLlm(provider));
        }
        let active_id = self
            .raw_active_id(tenant_id, provider)
            .await?
            .ok_or_else(|| {
                SwitcherError::Credential(CredentialError::NotFound(format!(
                    "no active for {}/{}",
                    tenant_id,
                    provider.as_str()
                )))
            })?;

        // 检查 credential 是否仍 Active
        let records = self.manager.list(tenant_id, Some(provider)).await;
        let target = records
            .iter()
            .find(|r| r.id == active_id)
            .ok_or_else(|| SwitcherError::ActiveRevoked(active_id.clone()))?;
        if target.status == CredentialStatus::Revoked {
            // 清缓存, 让后续调用重新 switch
            self.cache
                .write()
                .await
                .remove(&(tenant_id.to_string(), provider));
            return Err(SwitcherError::ActiveRevoked(active_id));
        }

        // 解密
        let pt = self.manager.retrieve(tenant_id, provider).await?;
        Ok(pt)
    }

    /// 记录 usage (一次 LLM 调用后调用)
    ///
    /// 步骤:
    /// 1. 校验 provider 是 LLM
    /// 2. 校验 active credential 存在
    /// 3. db.add_usage (per day_bucket)
    /// 4. quota.increment_used (LlmToken resource, per star-quota)
    pub async fn record_usage(
        &self,
        tenant_id: &str,
        provider: Provider,
        tokens_in: u64,
        tokens_out: u64,
        tenant_uuid: uuid::Uuid,
    ) -> Result<(), SwitcherError> {
        if !provider.is_llm() {
            return Err(SwitcherError::NotLlm(provider));
        }
        let active_id = self.active(tenant_id, provider).await?.ok_or_else(|| {
            SwitcherError::Credential(CredentialError::NotFound(format!(
                "no active for {}/{}",
                tenant_id,
                provider.as_str()
            )))
        })?;

        let now = Self::now_ms();
        let day_bucket = Self::day_bucket_from_ms(now);

        self.db.add_usage(
            tenant_id,
            &active_id,
            provider,
            &day_bucket,
            tokens_in,
            tokens_out,
            now,
        )?;

        // 配额: LlmToken resource (per star-quota)
        let total_tokens = tokens_in.saturating_add(tokens_out);
        self.quota
            .increment_used(tenant_uuid, ResourceKind::LlmToken, total_tokens)
            .await?;

        Ok(())
    }

    /// 查 credential 聚合 usage summary (跨所有 day_bucket)
    pub async fn usage_summary(
        &self,
        tenant_id: &str,
        credential_id: &str,
    ) -> Result<UsageSummary, SwitcherError> {
        let records = self.db.list_usage(tenant_id, credential_id)?;
        let mut summary = UsageSummary {
            credential_id: credential_id.to_string(),
            provider: records
                .first()
                .map(|r| r.provider)
                .unwrap_or(Provider::LlmOpenAi),
            total_tokens_in: 0,
            total_tokens_out: 0,
            total_tokens: 0,
            total_call_count: 0,
            day_count: 0,
            first_used_at_ms: None,
            last_used_at_ms: None,
        };
        for r in &records {
            summary.total_tokens_in = summary.total_tokens_in.saturating_add(r.tokens_in);
            summary.total_tokens_out = summary.total_tokens_out.saturating_add(r.tokens_out);
            summary.total_call_count = summary.total_call_count.saturating_add(r.call_count);
            summary.day_count = summary.day_count.saturating_add(1);
            summary.first_used_at_ms = match summary.first_used_at_ms {
                None => Some(r.last_used_at_ms),
                Some(prev) if r.last_used_at_ms < prev => Some(r.last_used_at_ms),
                Some(prev) => Some(prev),
            };
            summary.last_used_at_ms = match summary.last_used_at_ms {
                None => Some(r.last_used_at_ms),
                Some(prev) if r.last_used_at_ms > prev => Some(r.last_used_at_ms),
                Some(prev) => Some(prev),
            };
        }
        summary.total_tokens = summary
            .total_tokens_in
            .saturating_add(summary.total_tokens_out);
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CredentialMetadata;
    use star_quota::{InMemoryQuotaStore, Quota, ResourceKind};

    fn make_meta(name: &str) -> CredentialMetadata {
        CredentialMetadata {
            display_name: name.into(),
            description: "test".into(),
        }
    }

    fn make_pt(secret: &str) -> CredentialPlaintext {
        CredentialPlaintext {
            secret: secret.to_string(),
            base_url: Some("https://api.example.com/v1".into()),
            region: None,
        }
    }

    /// 预置 quota (LlmToken 100_000) — 用于 record_usage 测试
    async fn make_quota_store(tenant: uuid::Uuid) -> Arc<InMemoryQuotaStore> {
        let store = Arc::new(InMemoryQuotaStore::new());
        let q = Quota {
            quota_id: uuid::Uuid::new_v4(),
            tenant_id: tenant,
            resource: ResourceKind::LlmToken,
            limit: 100_000,
            used: 0,
            created_at_ms: now_ms_fn(),
            valid_from_ms: now_ms_fn(),
            valid_to_ms: None,
            revision: 1,
        };
        store.insert_new_revision(q).await.unwrap();
        store
    }

    /// 测试用 now_ms helper (避免 Self::now_ms() 调用语法)
    fn now_ms_fn() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    async fn make_switcher(tenant_uuid: uuid::Uuid) -> AccountSwitcher {
        let manager = Arc::new(CredentialManager::with_local_mock_kms());
        let db = Arc::new(CredentialDb::in_memory().unwrap());
        let quota = make_quota_store(tenant_uuid).await;
        AccountSwitcher::new(manager, db, quota)
    }

    async fn store_llm_cred(
        manager: &CredentialManager,
        tenant: &str,
        provider: Provider,
        name: &str,
        secret: &str,
    ) -> String {
        manager
            .store(tenant, "user-1", provider, make_meta(name), make_pt(secret))
            .await
            .unwrap()
    }

    /// AC-1: switch 4 类 LLM 之一的 active credential
    #[tokio::test]
    async fn ac1_switch_llm_active() {
        let tenant_uuid = uuid::Uuid::new_v4();
        let switcher = make_switcher(tenant_uuid).await;
        let id_a = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::LlmOpenAi,
            "OpenAI-A",
            "oa_a",
        )
        .await;
        let id_b = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::LlmOpenAi,
            "OpenAI-B",
            "oa_b",
        )
        .await;

        switcher
            .switch("t1", Provider::LlmOpenAi, &id_a)
            .await
            .unwrap();
        assert_eq!(
            switcher.active("t1", Provider::LlmOpenAi).await.unwrap(),
            Some(id_a.clone())
        );

        // 切换到 B
        switcher
            .switch("t1", Provider::LlmOpenAi, &id_b)
            .await
            .unwrap();
        assert_eq!(
            switcher.active("t1", Provider::LlmOpenAi).await.unwrap(),
            Some(id_b.clone())
        );

        // retrieve_active 应取到最新 (B)
        let pt = switcher
            .retrieve_active("t1", Provider::LlmOpenAi)
            .await
            .unwrap();
        assert_eq!(pt.secret, "oa_b");
    }

    /// AC-2: switch 非 LLM provider (OpenClaw / Hermes / KMS) 应被拒绝 (INV-AS-02)
    #[tokio::test]
    async fn ac2_switch_non_llm_rejected() {
        let tenant_uuid = uuid::Uuid::new_v4();
        let switcher = make_switcher(tenant_uuid).await;
        let id = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::OpenClaw,
            "OpenClaw-A",
            "oc_a",
        )
        .await;

        // OpenClaw 非 LLM, 应返 NotLlm
        let err = switcher
            .switch("t1", Provider::OpenClaw, &id)
            .await
            .unwrap_err();
        assert!(
            matches!(err, SwitcherError::NotLlm(Provider::OpenClaw)),
            "expected NotLlm(OpenClaw), got {:?}",
            err
        );

        // KMS / Hermes 同理
        let kms_id = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::KmsVault,
            "Vault-A",
            "v_a",
        )
        .await;
        let err = switcher
            .switch("t1", Provider::KmsVault, &kms_id)
            .await
            .unwrap_err();
        assert!(matches!(err, SwitcherError::NotLlm(Provider::KmsVault)));

        // retrieve_active 对非 LLM 也拒绝
        let err = switcher
            .retrieve_active("t1", Provider::OpenClaw)
            .await
            .unwrap_err();
        assert!(matches!(err, SwitcherError::NotLlm(Provider::OpenClaw)));
    }

    /// AC-3: switch 不存在 / 跨 tenant 的 credential_id 应被拒绝 (CredentialError::NotFound)
    #[tokio::test]
    async fn ac3_switch_nonexistent_or_wrong_tenant_rejected() {
        let tenant_uuid = uuid::Uuid::new_v4();
        let switcher = make_switcher(tenant_uuid).await;
        let _id_a = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::LlmAnthropic,
            "Ant-A",
            "ant_a",
        )
        .await;

        // 假 UUID 不存在
        let fake_id = "00000000-0000-0000-0000-000000000000";
        let err = switcher
            .switch("t1", Provider::LlmAnthropic, fake_id)
            .await
            .unwrap_err();
        assert!(
            matches!(err, SwitcherError::Credential(CredentialError::NotFound(_))),
            "expected Credential::NotFound, got {:?}",
            err
        );

        // 不同 tenant 的凭证: 即使 id 真实, 不属于该 tenant
        let _id_other = store_llm_cred(
            &switcher.manager,
            "t-other",
            Provider::LlmAnthropic,
            "Ant-Other",
            "ant_other",
        )
        .await;
        let err = switcher
            .switch("t1", Provider::LlmAnthropic, &_id_other)
            .await
            .unwrap_err();
        assert!(
            matches!(err, SwitcherError::Credential(CredentialError::NotFound(_))),
            "expected Credential::NotFound (cross-tenant), got {:?}",
            err
        );
    }

    /// AC-4: switch 已 deprecated 的 credential 应被拒绝 (仅 Active 可切)
    #[tokio::test]
    async fn ac4_switch_deprecated_credential_rejected() {
        let tenant_uuid = uuid::Uuid::new_v4();
        let switcher = make_switcher(tenant_uuid).await;
        // store 后立即 rotate → 老的变 Deprecated
        let id_a = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::LlmGitHubCopilot,
            "Copilot-A",
            "cop_a",
        )
        .await;
        let _id_b = switcher
            .manager
            .rotate(
                "t1",
                "user-1",
                Provider::LlmGitHubCopilot,
                make_meta("Copilot-B"),
                make_pt("cop_b"),
            )
            .await
            .unwrap();

        // id_a 现在是 Deprecated, switch 应返 Deprecated
        let err = switcher
            .switch("t1", Provider::LlmGitHubCopilot, &id_a)
            .await
            .unwrap_err();
        assert!(
            matches!(
                err,
                SwitcherError::Credential(CredentialError::Deprecated(_))
            ),
            "expected Credential::Deprecated, got {:?}",
            err
        );
    }

    /// AC-5: retrieve_active 对已 revoked 的 active selection 自动清缓存 + 返 ActiveRevoked (INV-AS-05)
    #[tokio::test]
    async fn ac5_retrieve_active_clears_on_revoked() {
        let tenant_uuid = uuid::Uuid::new_v4();
        let switcher = make_switcher(tenant_uuid).await;
        let id = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::LlmGoogle,
            "Google-A",
            "goog_a",
        )
        .await;
        switcher
            .switch("t1", Provider::LlmGoogle, &id)
            .await
            .unwrap();
        // 切前 retrieve_active OK
        let pt = switcher
            .retrieve_active("t1", Provider::LlmGoogle)
            .await
            .unwrap();
        assert_eq!(pt.secret, "goog_a");

        // 撤销该 credential (manager 内存)
        switcher.manager.revoke(&id).await.unwrap();

        // 再 retrieve_active 应返 ActiveRevoked (per INV-AS-05)
        let err = switcher
            .retrieve_active("t1", Provider::LlmGoogle)
            .await
            .unwrap_err();
        assert!(
            matches!(err, SwitcherError::ActiveRevoked(_)),
            "expected ActiveRevoked, got {:?}",
            err
        );

        // active() 也应清缓存 (per INV-AS-05)
        assert_eq!(
            switcher.active("t1", Provider::LlmGoogle).await.unwrap(),
            None
        );
    }

    /// AC-6: record_usage 写入 db (account_usage) + increment quota (star-quota::LlmToken)
    #[tokio::test]
    async fn ac6_usage_tracking_records_tokens_and_quota() {
        let tenant_uuid = uuid::Uuid::new_v4();
        let switcher = make_switcher(tenant_uuid).await;
        let id = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::LlmOpenAi,
            "OpenAI-A",
            "oa_a",
        )
        .await;
        switcher
            .switch("t1", Provider::LlmOpenAi, &id)
            .await
            .unwrap();

        // 第 1 次调用: in=100, out=50 → total=150
        switcher
            .record_usage("t1", Provider::LlmOpenAi, 100, 50, tenant_uuid)
            .await
            .unwrap();
        // 第 2 次调用: in=200, out=100 → total=300
        switcher
            .record_usage("t1", Provider::LlmOpenAi, 200, 100, tenant_uuid)
            .await
            .unwrap();

        // 验证 db 里有 1 个 day_bucket, call_count=2, tokens_in=300, tokens_out=150
        let usage = switcher.db.list_usage("t1", &id).unwrap();
        assert_eq!(usage.len(), 1, "expected 1 day_bucket");
        assert_eq!(usage[0].call_count, 2);
        assert_eq!(usage[0].tokens_in, 300);
        assert_eq!(usage[0].tokens_out, 150);
        assert_eq!(usage[0].provider, Provider::LlmOpenAi);

        // 验证 quota LlmToken used = 150 + 300 = 450
        let quota = switcher
            .quota
            .get(tenant_uuid, ResourceKind::LlmToken)
            .await
            .unwrap();
        assert_eq!(quota.used, 450);
    }

    /// AC-7: usage_summary 跨 day_bucket 聚合 (total / call_count / first / last)
    #[tokio::test]
    async fn ac7_usage_summary_aggregates_across_buckets() {
        let tenant_uuid = uuid::Uuid::new_v4();
        let switcher = make_switcher(tenant_uuid).await;
        let id = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::LlmAnthropic,
            "Ant-A",
            "ant_a",
        )
        .await;
        // 直接写 db 模拟跨天聚合 (避开时间依赖)
        switcher
            .db
            .add_usage(
                "t1",
                &id,
                Provider::LlmAnthropic,
                "2026-09-01",
                100,
                50,
                1_700_000_000_000,
            )
            .unwrap();
        switcher
            .db
            .add_usage(
                "t1",
                &id,
                Provider::LlmAnthropic,
                "2026-09-02",
                200,
                80,
                1_700_010_000_000,
            )
            .unwrap();
        switcher
            .db
            .add_usage(
                "t1",
                &id,
                Provider::LlmAnthropic,
                "2026-09-02",
                50,
                20,
                1_700_010_500_000,
            ) // 同 bucket 累加
            .unwrap();

        let s = switcher.usage_summary("t1", &id).await.unwrap();
        assert_eq!(s.total_tokens_in, 350);
        assert_eq!(s.total_tokens_out, 150);
        assert_eq!(s.total_tokens, 500);
        assert_eq!(s.total_call_count, 3);
        assert_eq!(s.day_count, 2);
        assert_eq!(s.first_used_at_ms, Some(1_700_000_000_000));
        assert_eq!(s.last_used_at_ms, Some(1_700_010_500_000));
        assert_eq!(s.provider, Provider::LlmAnthropic);
    }

    /// AC-8: (tenant, provider) 独立 — OpenAI 切换不影响 Anthropic / GitHub Copilot / Google
    #[tokio::test]
    async fn ac8_independent_per_provider_selection() {
        let tenant_uuid = uuid::Uuid::new_v4();
        let switcher = make_switcher(tenant_uuid).await;

        // 4 个 provider 各 1 个凭证
        let id_oa = store_llm_cred(&switcher.manager, "t1", Provider::LlmOpenAi, "OA", "oa").await;
        let id_ant = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::LlmAnthropic,
            "Ant",
            "ant",
        )
        .await;
        let id_cop = store_llm_cred(
            &switcher.manager,
            "t1",
            Provider::LlmGitHubCopilot,
            "Cop",
            "cop",
        )
        .await;
        let id_goog =
            store_llm_cred(&switcher.manager, "t1", Provider::LlmGoogle, "Goog", "goog").await;

        switcher
            .switch("t1", Provider::LlmOpenAi, &id_oa)
            .await
            .unwrap();
        switcher
            .switch("t1", Provider::LlmAnthropic, &id_ant)
            .await
            .unwrap();
        switcher
            .switch("t1", Provider::LlmGitHubCopilot, &id_cop)
            .await
            .unwrap();
        switcher
            .switch("t1", Provider::LlmGoogle, &id_goog)
            .await
            .unwrap();

        // 4 个 active 互不影响
        assert_eq!(
            switcher.active("t1", Provider::LlmOpenAi).await.unwrap(),
            Some(id_oa.clone())
        );
        assert_eq!(
            switcher.active("t1", Provider::LlmAnthropic).await.unwrap(),
            Some(id_ant.clone())
        );
        assert_eq!(
            switcher
                .active("t1", Provider::LlmGitHubCopilot)
                .await
                .unwrap(),
            Some(id_cop.clone())
        );
        assert_eq!(
            switcher.active("t1", Provider::LlmGoogle).await.unwrap(),
            Some(id_goog.clone())
        );

        // 切 OpenAI 不影响 Anthropic
        let id_oa2 =
            store_llm_cred(&switcher.manager, "t1", Provider::LlmOpenAi, "OA-2", "oa2").await;
        switcher
            .switch("t1", Provider::LlmOpenAi, &id_oa2)
            .await
            .unwrap();
        assert_eq!(
            switcher.active("t1", Provider::LlmOpenAi).await.unwrap(),
            Some(id_oa2)
        );
        assert_eq!(
            switcher.active("t1", Provider::LlmAnthropic).await.unwrap(),
            Some(id_ant)
        );
        assert_eq!(
            switcher
                .active("t1", Provider::LlmGitHubCopilot)
                .await
                .unwrap(),
            Some(id_cop)
        );
        assert_eq!(
            switcher.active("t1", Provider::LlmGoogle).await.unwrap(),
            Some(id_goog)
        );

        // list_active 应返 4 行
        let sels = switcher.list_active("t1").await.unwrap();
        assert_eq!(sels.len(), 4);
    }
}
