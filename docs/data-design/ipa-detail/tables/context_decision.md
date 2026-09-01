# context.decision — テーブル詳細設計書

> **テーブル ID**: T88
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.23.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T88 |
| **物理名** | `context.decision` |
| **論理名** | 意思決定（核心） |
| **スキーマ** | `context` |
| **Module** | `domain-context` |
| **種別** | Entity（核心聚合根） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Decision Memory。3 状態（ACTIVE / SUPERSEDED / INVALIDATED、§A.7）。`superseded_by` / `invalidated_by` 自己参照連鎖。`scope` で 作用範囲（例: `'auth-service'`）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `statement` | ステートメント | TEXT | − | NO | − | − | − | − | − | 意思決定文 |
| 5 | `reason` | 理由 | TEXT | − | YES | `NULL` | − | − | − | − | 背景・根拠 |
| 6 | `scope` | スコープ | VARCHAR | 64 | YES | `NULL` | − | − | − | − | 作用範囲 |
| 7 | `source_type` | ソース種別 | VARCHAR | 32 | NO | − | − | − | − | − | 4 値 |
| 8 | `source_id` | ソース ID | UUID | YES | `NULL` | − | − | (App) | − | − | 源 ID |
| 9 | `status` | 状態 | VARCHAR | 16 | NO | `'ACTIVE'` | − | − | − | idx (PT) | 3 値 |
| 10 | `superseded_by` | 置換元 ID | UUID | YES | `NULL` | − | − | `context.decision(id)` ON DELETE SET NULL | − | idx | 自己参照 |
| 11 | `invalidated_by` | 無効化元 ID | UUID | YES | `NULL` | − | − | `context.decision(id)` ON DELETE SET NULL | − | − | 自己参照 |
| 12 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | − |
| 13 | `created_by_user_id` | 作成者 ID | UUID | YES | `NULL` | − | − | `identity.user(id)` (App) | − | − | 意思決定者 |
| 14 | `superseded_at` | 置換日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | − |
| 15 | `invalidated_at` | 無効化日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | − |
| 16 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 17 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `decision_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_decision_*` | FOREIGN KEY (4) | `tenant_id` / `project_id` / `superseded_by` / `invalidated_by` | 各親テーブル | CASCADE / SET NULL | − |
| `ck_decision_status` | CHECK | `status` | `IN ('ACTIVE','SUPERSEDED','INVALIDATED')` | 3 値 |
| `ck_decision_supersede` | CHECK | `status`/`superseded_by` | `(status = 'SUPERSEDED' AND superseded_by IS NOT NULL AND superseded_at IS NOT NULL) OR (status <> 'SUPERSEDED')` | 置換整合 |
| `ck_decision_invalidate` | CHECK | `status`/`invalidated_by` | `(status = 'INVALIDATED' AND invalidated_by IS NOT NULL AND invalidated_at IS NOT NULL) OR (status <> 'INVALIDATED')` | 無効化整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `decision_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_decision_tenant_project_status` | btree (PT) | `(tenant_id, project_id, status)` | `deleted_at IS NULL` | PJ + 状態 |
| `idx_decision_tenant_status_created` | btree (PT) | `(tenant_id, status, created_at DESC)` | `deleted_at IS NULL` | 状態 + 順 |
| `idx_decision_superseded_by` | btree (PT) | `superseded_by` | `superseded_by IS NOT NULL` | 置換連鎖 |
| `idx_decision_active` | btree (PT) | `(tenant_id, project_id)` | `status = 'ACTIVE' AND deleted_at IS NULL` | Active のみ |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_decision_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 10,000 |
| 1 年後 | 100,000 |
| 3 年後 | 1,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 800 B | 1,000,000 | 約 800 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `context.decision` (self) | `superseded_by` / `invalidated_by` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `agent.agent_session.decisions` (配列) | 文字列 ID 参照 |
| `context.context_packet.architecture_constraints` (配列) | 文字列 ID 参照 |
| `context.context_packet.existing_decisions` (配列) | 文字列 ID 参照 |
| `comment.comment` | `parent_type='decision'` |
| `planning.roadmap.business_goal_id` (App) | − |

---

## 9. RLS Policy

```sql
ALTER TABLE context.decision ENABLE ROW LEVEL SECURITY;
ALTER TABLE context.decision FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_decision_tenant_isolation ON context.decision
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
