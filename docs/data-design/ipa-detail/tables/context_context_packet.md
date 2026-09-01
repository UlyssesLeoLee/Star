# context.context_packet — テーブル詳細設計書

> **テーブル ID**: T86
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.23.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T86 |
| **物理名** | `context.context_packet` |
| **論理名** | コンテキストパケット（核心） |
| **スキーマ** | `context` |
| **Module** | `domain-context` |
| **種別** | Entity（核心聚合根） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | ContextPacket 核心。Provenance 強制（§4.4.3 / §R-26.2）。Priority Layers 5 段階（P0 不可裁剪 / P5 = Untrusted）。`token_budget` + `priority_layers` JSONB。`full_content_ref` で > 100KB 時 Object Storage。 |

---

## 2. カラム一覧（主要）

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `work_item_id` | WorkItem ID | UUID | − | NO | − | − | `work_item.work_item(id)` ON DELETE CASCADE | − | idx | 親 WorkItem |
| 5 | `worktree_id` | ワークツリー ID | UUID | − | NO | − | − | `worktree.worktree(id)` ON DELETE CASCADE | − | idx | 親 Worktree |
| 6 | `agent_session_id` | エージェントセッション ID | UUID | YES | `NULL` | − | − | `agent.agent_session(id)` (App) | − | idx | 消費 Session |
| 7 | `intent` | 意図 | TEXT | − | NO | − | − | − | − | − | 業務意図 |
| 8 | `objective` | 目標 | TEXT | − | NO | − | − | − | − | − | 達成目標 |
| 9 | `scope` | スコープ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | WorktreeScope |
| 10 | `relevant_requirements` | 関連 Requirement ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | 関連要件 |
| 11 | `acceptance_criteria` | 関連 AC ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | 関連受入基準 |
| 12 | `relevant_files` | 関連ファイル配列 | VARCHAR(2048)[] | − | NO | `'{}'` | − | − | − | GIN | ファイル |
| 13 | `relevant_symbols` | 関連シンボル配列 | VARCHAR(512)[] | − | NO | `'{}'` | − | − | − | GIN | シンボル |
| 14 | `architecture_constraints` | アーキ制約 ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | Decision 連動 |
| 15 | `existing_decisions` | 既存 Decision ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | Decision 連動 |
| 16 | `current_change_set_id` | 現 ChangeSet ID | UUID | YES | `NULL` | − | − | `development.change_set(id)` (App) | − | − | 進捗連動 |
| 17 | `open_feedback` | Open FB ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | FB 配列 |
| 18 | `failed_validation` | 失敗検証 ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | Validation 連動 |
| 19 | `preserve_rules` | 保持ルール | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 保持事項 |
| 20 | `prohibited_changes` | 禁止変更 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 禁止事項 |
| 21 | `expected_output` | 期待出力 | TEXT | − | YES | `NULL` | − | − | − | − | 期待結果 |
| 22 | `verification_instructions` | 検証手順 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 検証手順 |
| 23 | `token_budget` | トークン予算 | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | `{total, by_layer: {P0:..., P1:...}}` |
| 24 | `actual_tokens` | 実使用トークン | INT | 4 | YES | `NULL` | − | − | − | − | 実測値 |
| 25 | `priority_layers` | 優先度レイヤ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | `{P0:[], P1:[], P2:[], P3:[], P4:[]}` |
| 26 | `full_content_ref` | 全文参照 | VARCHAR | 2048 | YES | `NULL` | − | − | − | − | Object Storage Key (>100KB) |
| 27 | `created_by_type` | 作成者種別 | VARCHAR | 16 | NO | − | − | − | − | − | `'user'` / `'system:context-compiler'` |
| 28 | `created_by_id` | 作成者 ID | UUID | YES | `NULL` | − | − | (App) | − | − | 作成者 ID |
| 29 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 30 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 31 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `context_packet_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_context_packet_*` | FOREIGN KEY (4) | `tenant_id` / `project_id` / `work_item_id` / `worktree_id` | 各親テーブル | CASCADE | − |
| `ck_context_packet_created_by` | CHECK | `created_by_type` | `IN ('user','system:context-compiler')` | 2 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `context_packet_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_context_packet_tenant_workitem` | btree (PT) | `(tenant_id, work_item_id)` | `deleted_at IS NULL` | テナント + WorkItem |
| `idx_context_packet_tenant_agent_session` | btree (PT) | `(tenant_id, agent_session_id)` | `agent_session_id IS NOT NULL` | テナント + Agent Session |
| `idx_context_packet_tenant_worktree` | btree (PT) | `(tenant_id, worktree_id, created_at DESC)` | `deleted_at IS NULL` | テナント + Worktree + 順 |
| `idx_context_packet_relevant_files_gin` | GIN | `relevant_files` | `deleted_at IS NULL` | ファイル配列 |
| `idx_context_packet_relevant_symbols_gin` | GIN | `relevant_symbols` | `deleted_at IS NULL` | シンボル配列 |
| `idx_context_packet_open_feedback_gin` | GIN | `open_feedback` | `deleted_at IS NULL` | FB 配列 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_context_packet_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 100,000 |
| 1 年後 | 1,000,000 |
| 3 年後 | 10,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 2.5 KB (配列×8 + JSONB×3 + TEXT) | 10,000,000 | 約 25 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `work_item.work_item` | `work_item_id` |
| `worktree.worktree` | `worktree_id` |
| `agent.agent_session` (App) | `agent_session_id` |
| `development.change_set` (App) | `current_change_set_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `context.provenance_entry` | `context_packet_id` |
| `audit.ai_audit_metadata` (App) | `context_packet_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE context.context_packet ENABLE ROW LEVEL SECURITY;
ALTER TABLE context.context_packet FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_context_packet_tenant_isolation ON context.context_packet
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
