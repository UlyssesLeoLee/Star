//! InMemoryValidationService:Phase 2 内存实现
//!
//! 核心不变量:
//! - INV-VL-04:ValidationResult 必带 evidence(VAL-001)
//! - INV-VL-01/VAL-001:AI 自我报告不构成完成
//! - INV-VL-08:Build/Test Log storage_ref 必带 tenant_id 前缀
//! - INV-VL-07:跨 tenant 拒绝
//! - INV-VL-06:Override 必须人类(Protected)
//! - INV-VL-05:AcceptanceCoverage 未达 100% 拒绝 READY_FOR_REVIEW

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

use crate::context::ActorContext;
use crate::entity::{
    AcceptanceCoverage, AcceptanceCoverageReport, EvidenceDownloadURL, ValidationEvidence,
    ValidationOverride, ValidationPolicy, ValidationResult,
};
use crate::error::ValidationError;
use crate::event::{
    AcceptanceCoverageAchieved, AcceptanceCoverageLinked, EventMeta, EvidenceLinked,
    FeedbackRequired, ValidationEvent, ValidationFailed, ValidationOverridden, ValidationPassed,
    ValidationResultSubmitted,
};
use crate::invariants::{
    check_create_invariants, check_invariant_06_override_human_only,
    check_invariant_08_evidence_storage_tenant_prefix, check_status_transition,
};
use crate::port::{
    AddEvidenceCommand, CreateValidationPolicyCommand, LinkAcceptanceEvidenceCommand,
    LinkEvidenceCommand, ListValidationQuery, MarkValidationStatusCommand,
    OverrideValidationCommand, SubmitValidationResultCommand, ValidationCommandPort,
    ValidationQueryPort, ValidationRepository,
};
use crate::value_object::{
    CoverageStatus, EvidenceType, TenantId, UserId, ValidationEvidenceId, ValidationId,
    ValidationKind, ValidationOverrideId, ValidationPolicyId, ValidationStatus, WorkItemId,
    WorktreeId,
};

/// InMemory ValidationService
pub struct InMemoryValidationService {
    pub(crate) results: Arc<RwLock<HashMap<ValidationId, ValidationResult>>>,
    pub(crate) evidences: Arc<RwLock<HashMap<ValidationEvidenceId, ValidationEvidence>>>,
    /// (ValidationId, EvidenceId) 防重复
    pub(crate) evidence_links:
        Arc<RwLock<std::collections::HashSet<(ValidationId, ValidationEvidenceId)>>>,
    pub(crate) coverages: Arc<RwLock<HashMap<uuid::Uuid, AcceptanceCoverage>>>, // key = acceptance_criterion_id
    pub(crate) policies: Arc<RwLock<HashMap<ValidationPolicyId, ValidationPolicy>>>,
    pub(crate) overrides: Arc<RwLock<HashMap<ValidationOverrideId, ValidationOverride>>>,
    pub(crate) event_tx: mpsc::UnboundedSender<ValidationEvent>,
}

impl InMemoryValidationService {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<ValidationEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            results: Arc::new(RwLock::new(HashMap::new())),
            evidences: Arc::new(RwLock::new(HashMap::new())),
            evidence_links: Arc::new(RwLock::new(std::collections::HashSet::new())),
            coverages: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            overrides: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }

    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }

    pub async fn result_count(&self) -> usize {
        self.results.read().expect("lock").len()
    }

    pub async fn evidence_count(&self) -> usize {
        self.evidences.read().expect("lock").len()
    }

    pub async fn coverage_count(&self) -> usize {
        self.coverages.read().expect("lock").len()
    }

    pub async fn policy_count(&self) -> usize {
        self.policies.read().expect("lock").len()
    }

    fn check_tenant(actor: &ActorContext, expected: TenantId) -> Result<(), ValidationError> {
        if actor.tenant_id != expected {
            return Err(ValidationError::PermissionDenied);
        }
        Ok(())
    }
}

impl Default for InMemoryValidationService {
    fn default() -> Self {
        Self::new().0.as_ref().clone()
    }
}

impl Clone for InMemoryValidationService {
    fn clone(&self) -> Self {
        Self {
            results: self.results.clone(),
            evidences: self.evidences.clone(),
            evidence_links: self.evidence_links.clone(),
            coverages: self.coverages.clone(),
            policies: self.policies.clone(),
            overrides: self.overrides.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

// =====================================================================
// ValidationCommandPort 实现
// =====================================================================

#[async_trait]
impl ValidationCommandPort for InMemoryValidationService {
    async fn submit_result(
        &self,
        cmd: SubmitValidationResultCommand,
        actor: ActorContext,
    ) -> Result<ValidationResult, ValidationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;

        // INV-VL-04:log_excerpt_ref 必带(VAL-001)
        if cmd.log_excerpt_ref.trim().is_empty() {
            return Err(ValidationError::InvalidState(
                "INV-VL-04 / VAL-001: log_excerpt_ref 必带,Validation 不能依赖 Agent 自报"
                    .to_string(),
            ));
        }

        let now = chrono::Utc::now();
        let r = ValidationResult {
            id: ValidationId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            work_item_id: cmd.work_item_id,
            worktree_id: cmd.worktree_id,
            agent_session_id: None,
            change_set_id: None,
            commit_id: None,
            triggered_by: if actor.is_service_internal() {
                crate::value_object::TriggeredBy::Agent
            } else {
                crate::value_object::TriggeredBy::User
            },
            triggered_by_id: cmd.triggered_by_id,
            kind: cmd.kind,
            status: ValidationStatus::Pending,
            started_at: None,
            completed_at: None,
            failure_summary: None,
            log_excerpt_ref: Some(cmd.log_excerpt_ref),
            evidence_ids: cmd.evidence_ids.clone(),
            policy_id: cmd.policy_id,
            policy_required: cmd.policy_required,
            is_ai_complete_claim: cmd.is_ai_complete_claim,
            created_at: now,
            updated_at: now,
        };

        check_create_invariants(&r)?;
        // 持久化
        self.results.write().expect("lock").insert(r.id, r.clone());

        // 发布 Submitted 事件
        let evt = ValidationEvent::Submitted(ValidationResultSubmitted {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            validation_id: r.id,
            work_item_id: r.work_item_id,
            worktree_id: r.worktree_id,
            kind: r.kind,
            status: r.status,
        });
        let _ = self.event_tx.send(evt);
        Ok(r)
    }

    async fn mark_status(
        &self,
        cmd: MarkValidationStatusCommand,
        actor: ActorContext,
    ) -> Result<ValidationResult, ValidationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        let mut results = self.results.write().expect("lock");
        let r = results
            .get_mut(&cmd.validation_id)
            .ok_or(ValidationError::NotFound(cmd.validation_id))?;
        if r.tenant_id != cmd.tenant_id {
            return Err(ValidationError::PermissionDenied);
        }
        let from = r.status;
        let to = cmd.new_status;
        // 5 状态机迁移校验
        check_status_transition(from, to, r)?;
        // INV-VL-04 / VAL-001:Passed 必须有 log_excerpt_ref(evidence_ids 可选,可能仅引用 log)
        if matches!(to, ValidationStatus::Passed)
            && r.log_excerpt_ref.as_deref().unwrap_or("").is_empty()
        {
            return Err(ValidationError::InvalidState(
                "Passed status requires non-empty log_excerpt_ref (VAL-001 / INV-VL-04)"
                    .to_string(),
            ));
        }
        let now = chrono::Utc::now();
        r.status = to;
        r.updated_at = now;
        if matches!(to, ValidationStatus::Running) && r.started_at.is_none() {
            r.started_at = Some(now);
        }
        if to.is_terminal() {
            r.completed_at = Some(now);
            if matches!(to, ValidationStatus::Failed) {
                r.failure_summary = cmd.failure_summary.clone();
            }
        }
        let result = r.clone();
        drop(results);

        // 发布事件
        match to {
            ValidationStatus::Passed => {
                let evt = ValidationEvent::Passed(ValidationPassed {
                    meta: EventMeta {
                        actor_user_id: Some(actor.user_id.into_uuid()),
                        ..EventMeta::new(cmd.tenant_id)
                    },
                    validation_id: result.id,
                    kind: result.kind,
                    evidence_ref: result.log_excerpt_ref.clone(),
                });
                let _ = self.event_tx.send(evt);
            }
            ValidationStatus::Failed => {
                let evt = ValidationEvent::Failed(ValidationFailed {
                    meta: EventMeta {
                        actor_user_id: Some(actor.user_id.into_uuid()),
                        ..EventMeta::new(cmd.tenant_id)
                    },
                    validation_id: result.id,
                    kind: result.kind,
                    failure_summary: result.failure_summary.clone(),
                    work_item_id: result.work_item_id,
                });
                let _ = self.event_tx.send(evt);
                // 自动触发 Feedback Required
                if let Some(wi) = result.work_item_id {
                    let evt = ValidationEvent::FeedbackRequired(FeedbackRequired {
                        meta: EventMeta {
                            actor_user_id: Some(actor.user_id.into_uuid()),
                            ..EventMeta::new(cmd.tenant_id)
                        },
                        work_item_id: wi,
                        validation_id: result.id,
                        intervention_queue_priority: 0, // P0
                    });
                    let _ = self.event_tx.send(evt);
                }
            }
            _ => {}
        }
        Ok(result)
    }

    async fn override_result(
        &self,
        cmd: OverrideValidationCommand,
        actor: ActorContext,
    ) -> Result<ValidationOverride, ValidationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // INV-VL-06:必须人类(非 service_internal)
        check_invariant_06_override_human_only(&actor)?;
        // 必带 reason
        if cmd.reason.trim().is_empty() {
            return Err(ValidationError::InvalidState(
                "override 必须填写 reason".to_string(),
            ));
        }
        // Validation 必须存在且属于同 tenant
        {
            let results = self.results.read().expect("lock");
            let r = results
                .get(&cmd.validation_id)
                .ok_or(ValidationError::NotFound(cmd.validation_id))?;
            if r.tenant_id != cmd.tenant_id {
                return Err(ValidationError::PermissionDenied);
            }
        }
        let now = chrono::Utc::now();
        let ovr = ValidationOverride {
            id: ValidationOverrideId::new(),
            tenant_id: cmd.tenant_id,
            validation_id: cmd.validation_id,
            reason: cmd.reason.clone(),
            approver_user_id: cmd.approver_user_id,
            approved_at: now,
        };
        self.overrides
            .write()
            .expect("lock")
            .insert(ovr.id, ovr.clone());

        let evt = ValidationEvent::Overridden(ValidationOverridden {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            override_id: ovr.id,
            validation_id: ovr.validation_id,
            reason: ovr.reason.clone(),
            approver_user_id: ovr.approver_user_id.into_uuid(),
        });
        let _ = self.event_tx.send(evt);
        Ok(ovr)
    }

    async fn link_to_acceptance_criterion(
        &self,
        cmd: LinkAcceptanceEvidenceCommand,
        actor: ActorContext,
    ) -> Result<AcceptanceCoverage, ValidationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // 验证 Validation 存在且属于同 tenant
        {
            let results = self.results.read().expect("lock");
            let r = results
                .get(&cmd.validation_id)
                .ok_or(ValidationError::NotFound(cmd.validation_id))?;
            if r.tenant_id != cmd.tenant_id {
                return Err(ValidationError::PermissionDenied);
            }
            if r.status != ValidationStatus::Passed {
                return Err(ValidationError::InvalidState(format!(
                    "AC 关联要求 ValidationResult.status = Passed,实际 {:?}",
                    r.status
                )));
            }
        }
        let now = chrono::Utc::now();
        let mut coverages = self.coverages.write().expect("lock");
        let c = coverages
            .entry(cmd.acceptance_criterion_id)
            .or_insert_with(|| AcceptanceCoverage {
                id: crate::value_object::AcceptanceCoverageId::new(),
                tenant_id: cmd.tenant_id,
                work_item_id: cmd.work_item_id,
                acceptance_criterion_id: crate::value_object::AcceptanceCriterionId::from_uuid(
                    cmd.acceptance_criterion_id,
                ),
                validation_result_ids: Vec::new(),
                coverage_status: CoverageStatus::Uncovered,
                human_acknowledged_by: None,
                human_acknowledged_at: None,
                created_at: now,
                updated_at: now,
            });
        if !c.validation_result_ids.contains(&cmd.validation_id) {
            c.validation_result_ids.push(cmd.validation_id);
        }
        // 更新 status
        c.coverage_status = if c.validation_result_ids.is_empty() {
            CoverageStatus::Uncovered
        } else {
            CoverageStatus::Covered
        };
        c.updated_at = now;
        let result = c.clone();
        let ac_id = cmd.acceptance_criterion_id;
        drop(coverages);

        let evt = ValidationEvent::AcceptanceCoverageLinked(AcceptanceCoverageLinked {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            coverage_id: result.id,
            acceptance_criterion_id: ac_id,
            validation_id: cmd.validation_id,
            new_status: result.coverage_status,
        });
        let _ = self.event_tx.send(evt);

        // 若 100% 覆盖,发出 CoverageAchieved
        let coverages = self.coverages.read().expect("lock");
        let total: u32 = coverages
            .values()
            .filter(|x| x.work_item_id == cmd.work_item_id)
            .count() as u32;
        let covered: u32 = coverages
            .values()
            .filter(|x| x.work_item_id == cmd.work_item_id && x.is_covered())
            .count() as u32;
        drop(coverages);
        if total > 0 && covered == total {
            let evt = ValidationEvent::CoverageAchieved(AcceptanceCoverageAchieved {
                meta: EventMeta {
                    actor_user_id: Some(actor.user_id.into_uuid()),
                    ..EventMeta::new(cmd.tenant_id)
                },
                work_item_id: cmd.work_item_id,
                total_count: total,
                covered_count: covered,
            });
            let _ = self.event_tx.send(evt);
        }
        Ok(result)
    }

    async fn link_evidence(
        &self,
        cmd: LinkEvidenceCommand,
        actor: ActorContext,
    ) -> Result<ValidationEvidence, ValidationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        {
            let results = self.results.read().expect("lock");
            let r = results
                .get(&cmd.validation_id)
                .ok_or(ValidationError::NotFound(cmd.validation_id))?;
            if r.tenant_id != cmd.tenant_id {
                return Err(ValidationError::PermissionDenied);
            }
        }
        // 防重复
        {
            let links = self.evidence_links.read().expect("lock");
            if links.contains(&(cmd.validation_id, cmd.evidence_id)) {
                return Err(ValidationError::Conflict(format!(
                    "evidence_id={} 已关联到 validation_id={}",
                    cmd.evidence_id, cmd.validation_id
                )));
            }
        }
        // 加 link + 写回 result
        {
            let mut links = self.evidence_links.write().expect("lock");
            links.insert((cmd.validation_id, cmd.evidence_id));
        }
        {
            let mut results = self.results.write().expect("lock");
            if let Some(r) = results.get_mut(&cmd.validation_id) {
                if !r.evidence_ids.contains(&cmd.evidence_id) {
                    r.evidence_ids.push(cmd.evidence_id);
                    r.updated_at = chrono::Utc::now();
                }
            }
        }
        // 取 evidence 详情用于事件
        let evidences = self.evidences.read().expect("lock");
        let e = evidences.get(&cmd.evidence_id).cloned();
        drop(evidences);

        let evt = ValidationEvent::EvidenceLinked(EvidenceLinked {
            meta: EventMeta {
                actor_user_id: Some(actor.user_id.into_uuid()),
                ..EventMeta::new(cmd.tenant_id)
            },
            evidence_id: cmd.evidence_id,
            validation_id: cmd.validation_id,
        });
        let _ = self.event_tx.send(evt);
        e.ok_or(ValidationError::NotFound(
            crate::value_object::ValidationId::default(),
        ))
    }

    async fn add_evidence(
        &self,
        cmd: AddEvidenceCommand,
        actor: ActorContext,
    ) -> Result<ValidationEvidence, ValidationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // 验证 Validation 存在
        {
            let results = self.results.read().expect("lock");
            let r = results
                .get(&cmd.validation_id)
                .ok_or(ValidationError::NotFound(cmd.validation_id))?;
            if r.tenant_id != cmd.tenant_id {
                return Err(ValidationError::PermissionDenied);
            }
        }
        // INV-VL-08:storage_ref 必带 tenant_id 前缀
        let tenant_id_for_check = cmd.tenant_id;
        let dummy = ValidationEvidence {
            id: ValidationEvidenceId::new(),
            tenant_id: tenant_id_for_check,
            validation_result_id: cmd.validation_id,
            evidence_type: cmd.evidence_type,
            storage_ref: cmd.storage_ref.clone(),
            size_bytes: cmd.size_bytes,
            mime_type: cmd.mime_type.clone(),
            url_expires_at: None,
            created_at: chrono::Utc::now(),
        };
        check_invariant_08_evidence_storage_tenant_prefix(&dummy)?;
        self.evidences
            .write()
            .expect("lock")
            .insert(dummy.id, dummy.clone());
        Ok(dummy)
    }

    async fn create_policy(
        &self,
        cmd: CreateValidationPolicyCommand,
        actor: ActorContext,
    ) -> Result<ValidationPolicy, ValidationError> {
        Self::check_tenant(&actor, cmd.tenant_id)?;
        // INV-VL-09/VAL-001:allow_ai_self_claim 必须 false
        if cmd.allow_ai_self_claim {
            return Err(ValidationError::InvariantViolated(
                "VAL-001: ValidationPolicy.allow_ai_self_claim 不可为 true".to_string(),
            ));
        }
        let now = chrono::Utc::now();
        let p = ValidationPolicy {
            id: ValidationPolicyId::new(),
            tenant_id: cmd.tenant_id,
            project_id: cmd.project_id,
            name: cmd.name,
            required_kinds: cmd.required_kinds,
            optional_kinds: cmd.optional_kinds,
            pass_thresholds: cmd.pass_thresholds,
            allow_ai_self_claim: cmd.allow_ai_self_claim,
            override_allow: cmd.override_allow,
            created_at: now,
            updated_at: now,
        };
        self.policies.write().expect("lock").insert(p.id, p.clone());
        Ok(p)
    }
}

// =====================================================================
// ValidationQueryPort 实现
// =====================================================================

#[async_trait]
impl ValidationQueryPort for InMemoryValidationService {
    async fn list_results(
        &self,
        q: ListValidationQuery,
        viewer: ActorContext,
    ) -> Result<Vec<ValidationResult>, ValidationError> {
        if viewer.tenant_id != q.tenant_id {
            return Err(ValidationError::PermissionDenied);
        }
        let results = self.results.read().expect("lock");
        let mut out: Vec<ValidationResult> = results
            .values()
            .filter(|r| r.tenant_id == q.tenant_id)
            .filter(|r| q.work_item_id.is_none_or(|wi| r.work_item_id == Some(wi)))
            .filter(|r| q.worktree_id.is_none_or(|wt| r.worktree_id == Some(wt)))
            .filter(|r| q.kind.is_none_or(|k| r.kind == k))
            .filter(|r| q.status.is_none_or(|s| r.status == s))
            .cloned()
            .collect();
        out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let start = q.offset as usize;
        let end = std::cmp::min(start + q.limit as usize, out.len());
        if start >= out.len() {
            out.clear();
        } else {
            out = out.split_off(start);
            out.truncate(end - start);
        }
        Ok(out)
    }

    async fn get_result(
        &self,
        id: ValidationId,
        viewer: ActorContext,
    ) -> Result<ValidationResult, ValidationError> {
        let results = self.results.read().expect("lock");
        let r = results
            .get(&id)
            .ok_or(ValidationError::NotFound(id))?
            .clone();
        if r.tenant_id != viewer.tenant_id {
            return Err(ValidationError::PermissionDenied);
        }
        Ok(r)
    }

    async fn get_evidence_url(
        &self,
        id: ValidationEvidenceId,
        viewer: ActorContext,
    ) -> Result<EvidenceDownloadURL, ValidationError> {
        let evidences = self.evidences.read().expect("lock");
        let e = evidences
            .get(&id)
            .ok_or(ValidationError::NotFound(ValidationId::default()))?
            .clone();
        if e.tenant_id != viewer.tenant_id {
            return Err(ValidationError::PermissionDenied);
        }
        Ok(EvidenceDownloadURL {
            evidence_id: id,
            url: format!("https://signed.example.com/{}", e.storage_ref),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
        })
    }

    async fn get_acceptance_coverage(
        &self,
        work_item_id: WorkItemId,
        viewer: ActorContext,
    ) -> Result<AcceptanceCoverageReport, ValidationError> {
        // 校验 tenant
        let coverages = self.coverages.read().expect("lock");
        let mut total = 0u32;
        let mut covered = 0u32;
        let mut partial = 0u32;
        let mut uncovered = 0u32;
        let mut disputed = 0u32;
        let mut per: Vec<AcceptanceCoverage> = Vec::new();
        for c in coverages.values() {
            if c.work_item_id != work_item_id {
                continue;
            }
            if c.tenant_id != viewer.tenant_id {
                return Err(ValidationError::PermissionDenied);
            }
            total += 1;
            match c.coverage_status {
                CoverageStatus::Covered => covered += 1,
                CoverageStatus::Partial => partial += 1,
                CoverageStatus::Uncovered => uncovered += 1,
                CoverageStatus::Disputed => disputed += 1,
            }
            per.push(c.clone());
        }
        Ok(AcceptanceCoverageReport {
            work_item_id,
            tenant_id: viewer.tenant_id,
            total_criteria: total,
            covered,
            partial,
            uncovered,
            disputed,
            per_criterion: per,
        })
    }

    async fn list_policies(
        &self,
        viewer: ActorContext,
    ) -> Result<Vec<ValidationPolicy>, ValidationError> {
        let policies = self.policies.read().expect("lock");
        Ok(policies
            .values()
            .filter(|p| p.tenant_id == viewer.tenant_id)
            .cloned()
            .collect())
    }

    async fn list_evidence(
        &self,
        validation_id: ValidationId,
        viewer: ActorContext,
    ) -> Result<Vec<ValidationEvidence>, ValidationError> {
        let evidences = self.evidences.read().expect("lock");
        let out: Vec<ValidationEvidence> = evidences
            .values()
            .filter(|e| e.validation_result_id == validation_id)
            .cloned()
            .collect();
        for e in &out {
            if e.tenant_id != viewer.tenant_id {
                return Err(ValidationError::PermissionDenied);
            }
        }
        Ok(out)
    }
}

// =====================================================================
// ValidationRepository 实现
// =====================================================================

#[async_trait]
impl ValidationRepository for InMemoryValidationService {
    async fn insert_result(&self, r: &ValidationResult) -> Result<(), ValidationError> {
        self.results.write().expect("lock").insert(r.id, r.clone());
        Ok(())
    }
    async fn save_result(&self, r: &ValidationResult) -> Result<(), ValidationError> {
        self.results.write().expect("lock").insert(r.id, r.clone());
        Ok(())
    }
    async fn find_result(
        &self,
        id: ValidationId,
    ) -> Result<Option<ValidationResult>, ValidationError> {
        Ok(self.results.read().expect("lock").get(&id).cloned())
    }
    async fn list_results_raw(
        &self,
        work_item_id: Option<WorkItemId>,
        worktree_id: Option<WorktreeId>,
        kind: Option<ValidationKind>,
        status: Option<ValidationStatus>,
    ) -> Result<Vec<ValidationResult>, ValidationError> {
        let results = self.results.read().expect("lock");
        Ok(results
            .values()
            .filter(|r| work_item_id.is_none_or(|wi| r.work_item_id == Some(wi)))
            .filter(|r| worktree_id.is_none_or(|wt| r.worktree_id == Some(wt)))
            .filter(|r| kind.is_none_or(|k| r.kind == k))
            .filter(|r| status.is_none_or(|s| r.status == s))
            .cloned()
            .collect())
    }

    async fn insert_evidence(&self, e: &ValidationEvidence) -> Result<(), ValidationError> {
        self.evidences
            .write()
            .expect("lock")
            .insert(e.id, e.clone());
        Ok(())
    }
    async fn find_evidence(
        &self,
        id: ValidationEvidenceId,
    ) -> Result<Option<ValidationEvidence>, ValidationError> {
        Ok(self.evidences.read().expect("lock").get(&id).cloned())
    }
    async fn list_evidence_by_validation(
        &self,
        validation_id: ValidationId,
    ) -> Result<Vec<ValidationEvidence>, ValidationError> {
        Ok(self
            .evidences
            .read()
            .expect("lock")
            .values()
            .filter(|e| e.validation_result_id == validation_id)
            .cloned()
            .collect())
    }
    async fn save_evidence(&self, e: &ValidationEvidence) -> Result<(), ValidationError> {
        self.evidences
            .write()
            .expect("lock")
            .insert(e.id, e.clone());
        Ok(())
    }

    async fn insert_coverage(&self, c: &AcceptanceCoverage) -> Result<(), ValidationError> {
        self.coverages
            .write()
            .expect("lock")
            .insert(c.acceptance_criterion_id.into_uuid(), c.clone());
        Ok(())
    }
    async fn save_coverage(&self, c: &AcceptanceCoverage) -> Result<(), ValidationError> {
        self.coverages
            .write()
            .expect("lock")
            .insert(c.acceptance_criterion_id.into_uuid(), c.clone());
        Ok(())
    }
    async fn find_coverage_by_criterion(
        &self,
        ac_id: uuid::Uuid,
    ) -> Result<Option<AcceptanceCoverage>, ValidationError> {
        Ok(self.coverages.read().expect("lock").get(&ac_id).cloned())
    }
    async fn list_coverage_by_work_item(
        &self,
        work_item_id: WorkItemId,
    ) -> Result<Vec<AcceptanceCoverage>, ValidationError> {
        Ok(self
            .coverages
            .read()
            .expect("lock")
            .values()
            .filter(|c| c.work_item_id == work_item_id)
            .cloned()
            .collect())
    }

    async fn insert_policy(&self, p: &ValidationPolicy) -> Result<(), ValidationError> {
        self.policies.write().expect("lock").insert(p.id, p.clone());
        Ok(())
    }
    async fn find_policy(
        &self,
        id: ValidationPolicyId,
    ) -> Result<Option<ValidationPolicy>, ValidationError> {
        Ok(self.policies.read().expect("lock").get(&id).cloned())
    }
    async fn list_policies_raw(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<ValidationPolicy>, ValidationError> {
        Ok(self
            .policies
            .read()
            .expect("lock")
            .values()
            .filter(|p| p.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    async fn insert_override(&self, o: &ValidationOverride) -> Result<(), ValidationError> {
        self.overrides
            .write()
            .expect("lock")
            .insert(o.id, o.clone());
        Ok(())
    }
}

// 静默引用
#[allow(dead_code)]
fn _unused_user(_: UserId) -> UserId {
    uuid::Uuid::new_v4()
}
#[allow(dead_code)]
fn _unused_kind(_: ValidationKind) -> ValidationKind {
    ValidationKind::Build
}
#[allow(dead_code)]
fn _unused_et(_: EvidenceType) -> EvidenceType {
    EvidenceType::TestReport
}
