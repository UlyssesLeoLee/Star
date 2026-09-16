# graph.graph_fingerprint — テーブル詳細設計書

> **テーブル ID**: T-NEW-003
> **作成日**: 2026-09-02
> **改訂人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **一次出典**: `docs/architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md` §3.4

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T-NEW-003 |
| **物理名** | `graph.graph_fingerprint` |
| **論理名** | 指紋監査ログ (Transaction, append-only) |
| **スキーマ** | `graph` |
| **Module** | `domain-graph-agent` (Phase 2 新設) |
| **種別** | **Transaction (T)** — append-only + 物理削除禁止 + 監査必須 + RLS 13 類必携 |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | work_item 毎の冪等キー履歴, agent 実行全記録, append-only。同一 work_item で重複 fingerprint も実行時刻別行で保持。90 日 TTL (per AI Content Retention Policy §6.8)。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | 内部 ID |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx (PT) | 13 類 RLS 必須 |
| 3 | `work_item_id` | work item | UUID | − | NO | − | − | `work.work_item(id)` ON DELETE CASCADE | − | idx (PT) | 業務 work_item FK |
| 4 | `fingerprint` | 指紋 | VARCHAR | 64 | NO | − | − | − | ✓ | `idx_graph_fp_work_fp` (PT) | sha256 ハッシュ |
| 5 | `worktree_branch` | worktree branch | VARCHAR | 200 | YES | `NULL` | − | − | − | − | git branch (worktree なし時 NULL) |
| 6 | `worktree_sha` | worktree commit | VARCHAR | 40 | YES | `NULL` | − | − | − | − | git commit SHA |
| 7 | `source` | データ源 | VARCHAR | 8 | NO | − | − | − | − | − | `"local"` \| `"git"` |
| 8 | `project_id` | プロジェクト | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx (PT) | PJ FK |
| 9 | `agent_session_id` | agent session | UUID | YES | `NULL` | − | − | `agent.agent_session(id)` ON DELETE SET NULL | − | idx (PT) | 生成 agent (Phase 2) |
| 10 | `phase` | フェーズ | VARCHAR | 20 | NO | − | − | − | − | − | 7 値 enum |
| 11 | `started_at` | 開始日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | ✓ | `idx_graph_fp_uniq` (PT) | agent 開始 |
| 12 | `ended_at` | 終了日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | `NULL` = 実行中 |
| 13 | `error_message` | エラー | TEXT | − | YES | `NULL` | − | − | − | − | 失敗時のみ |
| 14 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 監査 (append-only) |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `graph_fingerprint_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_graph_fp_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | RLS |
| `fk_graph_fp_work_item` | FOREIGN KEY | `work_item_id` | `work.work_item(id)` ON DELETE CASCADE | 業務連動 |
| `fk_graph_fp_project` | FOREIGN KEY | `project_id` | `project.project(id)` ON DELETE CASCADE | PJ |
| `fk_graph_fp_agent_session` | FOREIGN KEY | `agent_session_id` | `agent.agent_session(id)` ON DELETE SET NULL | 監査連動 |
| `ck_graph_fp_phase` | CHECK | `phase` | `IN ('scanning', 'ast_extract', 'llm_infer', 'upsert', 'verify', 'success', 'failed')` | 7 値 |
| `ck_graph_fp_source` | CHECK | `source` | `IN ('local', 'git')` | データ源 2 値 |
| `ck_graph_fp_timing` | CHECK | `ended_at` | `ended_at IS NULL OR ended_at >= started_at` | 時刻整合 |
| `idx_graph_fp_uniq` | UNIQUE | `(work_item_id, fingerprint, started_at)` | − | 同 work_item + 同 fingerprint でも実行時刻で別行 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `graph_fingerprint_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_graph_fp_tenant` | btree (PT) | `tenant_id` | − | RLS 補助 |
| `idx_graph_fp_work_item` | btree (PT) | `(tenant_id, work_item_id, started_at DESC)` | − | work_item 履歴 |
| `idx_graph_fp_work_fp` | btree (PT) | `(tenant_id, work_item_id, fingerprint)` | − | fingerprint 一致検索 |
| `idx_graph_fp_project` | btree (PT) | `(tenant_id, project_id, started_at DESC)` | − | PJ 全体監査 |
| `idx_graph_fp_agent_session` | btree (PT) | `agent_session_id` | `WHERE agent_session_id IS NOT NULL` | agent session 連動 |
| `idx_graph_fp_phase` | btree (PT) | `(tenant_id, phase, started_at DESC)` | − | フェーズ別監視 |
| `idx_graph_fp_uniq` | btree (UK/PT) | `(work_item_id, fingerprint, started_at)` | − | 一意制約 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_graph_fp_no_update` | BEFORE UPDATE | (inline) | UPDATE / DELETE を RAISE EXCEPTION で拒否 (append-only 強制) |
| `trg_graph_fp_ttl_job` | (cron) | `public.fn_archive_old_fingerprints()` | 90 日超の行を cold storage に移動 (per AI Content Retention §6.8) |

---

## 6. 想定レコード件数

| フェーズ | 件数 | 根拠 |
|---|---|---|
| MVP (Phase 1) | 0 (実体なし) | MSW mock のみ |
| V1 (Phase 2) | < 10K/日 (1 tenant) | work_item 100 × 10 build/日 |
| V2 (Phase 3) | < 100K/日 (1 tenant) | 実 memgraph 展開, retention 90 日 = < 9M |

---

## 7. RLS ポリシー (per 13 類, per REQ-SEC-001)

```sql
ALTER TABLE graph.graph_fingerprint ENABLE ROW LEVEL SECURITY;
ALTER TABLE graph.graph_fingerprint FORCE ROW LEVEL SECURITY;

CREATE POLICY rls_graph_fp_tenant ON graph.graph_fingerprint
  USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

CREATE POLICY rls_graph_fp_tenant_write ON graph.graph_fingerprint
  FOR INSERT
  WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- UPDATE / DELETE は許可しない (append-only)
```

---

## 8. データ保持 (per AI Content Retention Policy §6.8)

| 期間 | 保管場所 | アクセス |
|---|---|---|
| 0-90 日 | `graph.graph_fingerprint` (PostgreSQL 本体) | 高頻度, RLS 経由 |
| 90-365 日 | `graph.graph_fingerprint_cold` (S3 / Object Storage) | 低頻度, 緊急時のみ |
| 365+ 日 | 削除 (per AI Content Retention Policy §6.8 "Full Prompt/Response default 90 day") | − |

> **注**: fingerprint 自体は SHA-256 ハッシュなので PII なし, 90 日超でも参照用に保持する選択肢あり (Phase 3 設計で確定)

---

## 9. 関連

- 上位 spec: `docs/architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md` §3.4
- ノード表: `graph.graph_node` (T-NEW-001)
- エッジ表: `graph.graph_edge` (T-NEW-002)
- 関連: ADR-0041 §2.2.3 冪等性, §2.2.4 排他性, AGENTS.md §4 守門 #13 DB 三類

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 初版: Transaction (T) 類 / append-only / 14 列 / 8 制約 / 8 索引 / 13 類 RLS / 90 日 TTL | 2026-09-02 02:10 JST Ulysses "需求和基本设计, 詳細设计 補完" |
