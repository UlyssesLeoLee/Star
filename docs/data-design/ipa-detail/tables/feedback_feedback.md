# feedback.feedback — テーブル詳細設計書

> **テーブル ID**: T82
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.22.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T82 |
| **物理名** | `feedback.feedback` |
| **論理名** | フィードバック（核心） |
| **スキーマ** | `feedback` |
| **Module** | `domain-feedback` |
| **種別** | Entity（核心聚合根） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | 構造化 Feedback。**14 種 target**（WorkItem / Requirement / AcceptanceCriterion / Worktree / AgentSession / File / Symbol / DiffHunk / Test / Build / RuntimeLog / ArchitectureDecision / PullRequest / ReviewFinding）+ **11 種 type**（Fix / Preserve / Refactor / Reject / Question / Constraint / Architecture / Security / Performance / Testing / Scope）+ **6 状態**。Author XOR（user / agent 排他）。`predecessor_id` で Supersede 連鎖。 |

---

## 2. カラム一覧（主要）

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4-17 | `target_*` | Target 14 種 | UUID / VARCHAR / INT4RANGE | 混合 | YES | `NULL` | − | (App 検証) | − | idx (PT) | target_type で 14 列分散（WorkItem / Requirement / AcceptanceCriterion / Worktree / AgentSession / File / Symbol / DiffHunk / Test / Build / RuntimeLog / ArchitectureDecision / PullRequest / ReviewFinding） |
| 18 | `type` | 種別 | VARCHAR | 32 | NO | − | − | − | − | − | 11 値 |
| 19 | `severity` | 重大度 | VARCHAR | 8 | NO | − | − | − | − | idx (PT) | 4 値 |
| 20 | `intent` | 意図 | TEXT | − | NO | − | − | − | − | − | 短文要求 |
| 21 | `expected_behavior` | 期待動作 | TEXT | − | YES | `NULL` | − | − | − | − | − |
| 22 | `preserve` | 保持事項配列 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 文字列配列 |
| 23 | `prohibit` | 禁止事項配列 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 文字列配列 |
| 24 | `acceptance_criterion_id` | AC ID | UUID | YES | `NULL` | − | − | `work_item.acceptance_criterion(id)` (App) | − | idx | 紐付 AC |
| 25 | `author_user_id` | 著者ユーザ ID | UUID | YES | `NULL` | − | − | `identity.user(id)` (App) | − | idx (PT) | 人間著者 |
| 26 | `author_agent_id` | 著者エージェント ID | UUID | YES | `NULL` | − | − | `agent.agent(id)` (App) | − | idx (PT) | AI 著者 |
| 27 | `status` | 状態 | VARCHAR | 32 | NO | `'OPEN'` | − | − | − | idx (PT) | 6 値 |
| 28 | `predecessor_id` | 先行 Feedback ID | UUID | YES | `NULL` | − | − | `feedback.feedback(id)` ON DELETE SET NULL | − | idx | Supersede 連鎖 |
| 29 | `resolved_at` | 解決日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | − |
| 30 | `resolution_evidence` | 解決証拠 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 証拠配列 |
| 31 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 32 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 33 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 34 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `feedback_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_feedback_*` | FOREIGN KEY (4) | `tenant_id` / `project_id` / `predecessor_id` | 各親テーブル | CASCADE / SET NULL | − |
| `ck_feedback_type` | CHECK | `type` | `IN (11 値, §4.22.1)` | 11 値 |
| `ck_feedback_severity` | CHECK | `severity` | `IN ('P0','P1','P2','P3')` | 4 値 |
| `ck_feedback_status` | CHECK | `status` | `IN ('OPEN','ACKNOWLEDGED','APPLIED','VERIFIED','REJECTED','SUPERSEDED')` | 6 値 |
| `ck_feedback_target_type` | CHECK | `target_type` | `IN (14 値, §4.22.1)` | 14 値 |
| `ck_feedback_author_xor` | CHECK | `author_user_id`/`author_agent_id` | `(author_user_id IS NOT NULL)::int + (author_agent_id IS NOT NULL)::int = 1` | Author XOR |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `feedback_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_feedback_tenant_project_status` | btree (PT) | `(tenant_id, project_id, status)` | `deleted_at IS NULL` | PJ + 状態 |
| `idx_feedback_tenant_target` | btree (PT) | `(tenant_id, target_type, target_id)` | `deleted_at IS NULL` | Target 別 |
| `idx_feedback_tenant_severity_status` | btree (PT) | `(tenant_id, severity, status)` | `deleted_at IS NULL` | 重大度 + 状態 |
| `idx_feedback_tenant_author_user` | btree (PT) | `(tenant_id, author_user_id)` | `author_user_id IS NOT NULL AND deleted_at IS NULL` | 人間著者別 |
| `idx_feedback_tenant_author_agent` | btree (PT) | `(tenant_id, author_agent_id)` | `author_agent_id IS NOT NULL AND deleted_at IS NULL` | AI 著者別 |
| `idx_feedback_predecessor` | btree (PT) | `predecessor_id` | `predecessor_id IS NOT NULL` | Supersede 連鎖 |
| `idx_feedback_ac` | btree (PT) | `acceptance_criterion_id` | `acceptance_criterion_id IS NOT NULL` | AC 別 |
| `idx_feedback_open_critical` | btree (PT) | `(tenant_id, severity)` | `status = 'OPEN' AND severity IN ('P0','P1')` | Open 重大度監視 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_feedback_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 500,000 |
| 1 年後 | 5,000,000 |
| 3 年後 | 50,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 2.5 KB (TEXT + JSONB ×2) | 50,000,000 | 約 125 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `feedback.feedback` (self) | `predecessor_id` |
| `work_item.acceptance_criterion` (App) | `acceptance_criterion_id` |
| `identity.user` (App) | `author_user_id` |
| `agent.agent` (App) | `author_agent_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `feedback.feedback_consumed_event` | `feedback_id` |
| `feedback.feedback_inbox_item` (MV) | `feedback_id` 経由 |
| `agent.agent_session.feedback_consumed_ids` (配列) | 文字列 ID 参照 |

---

## 9. RLS Policy

```sql
ALTER TABLE feedback.feedback ENABLE ROW LEVEL SECURITY;
ALTER TABLE feedback.feedback FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_feedback_tenant_isolation ON feedback.feedback
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
