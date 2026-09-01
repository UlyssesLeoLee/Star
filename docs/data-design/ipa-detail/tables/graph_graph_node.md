# graph.graph_node — テーブル詳細設計書

> **テーブル ID**: T-NEW-001
> **作成日**: 2026-09-02
> **改訂人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **一次出典**: `docs/architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md` §3.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T-NEW-001 |
| **物理名** | `graph.graph_node` |
| **論理名** | グラフノード (cypher 投影, Master) |
| **スキーマ** | `graph` |
| **Module** | `domain-graph-agent` (Phase 2 新設) |
| **種別** | **Master (M)** — SCD Type 2 + 物理削除禁止 + RLS 13 類必携 |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | 25 domain ノード (work_item / worktree / agent_session / ...) の cypher 投影。SCD Type 2 で履歴保持。同一 work_item に対し fingerprint 単位で upsert (MERGE)。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | 内部 ID (例 `WI:wi-7c9e2f8a-...`) |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx (PT) | 13 類 RLS 必須 |
| 3 | `kind` | 種別 | VARCHAR | 32 | NO | − | − | − | ✓ | `idx_graph_node_kind` (PT) | 25 domain kind union |
| 4 | `label` | 表示ラベル | VARCHAR | 200 | NO | − | − | − | − | − | UI 表示用 |
| 5 | `external_ref` | 業務 ID | VARCHAR | 200 | YES | `NULL` | − | − | ✓ | `uq_graph_node_extref` (PT) | 例 `work_item.key = "PHYSIS-123"` |
| 6 | `properties` | プロパティ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | MRU フィールド透過 |
| 7 | `is_current` | 現 hop 中心 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | (Phase 3) Modal で中心ノードか |
| 8 | `fingerprint` | 指紋 | VARCHAR | 64 | NO | − | − | − | − | `idx_graph_node_fp` (PT) | 当該 work_item の sha256 |
| 9 | `source` | データ源 | VARCHAR | 8 | NO | − | − | − | − | − | `"local"` \| `"git"` |
| 10 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 自動更新 |
| 12 | `valid_from` | SCD 開始 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | ✓ | `uq_graph_node_scd` (PT) | SCD Type 2 開始 |
| 13 | `valid_to` | SCD 終了 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | `idx_graph_node_valid_to` | `NULL` = 現有効 |
| 14 | `version` | 楽観ロック | INT | 4 | NO | `1` | − | − | − | − | − |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `graph_node_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_graph_node_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | RLS 必須 |
| `ck_graph_node_kind` | CHECK | `kind` | `IN ('work_item','worktree','agent_session','change_set','scm_repository','pull_request','feedback','validation_case','comment','design_artifact','identity','cratemodule','symbol','tenant','project','workspace','permission_scheme','workflow','local_runtime','context_packet','audit_event','automation_rule','notification','incident_record','integration_webhook')` | 25 値 |
| `ck_graph_node_source` | CHECK | `source` | `IN ('local', 'git')` | データ源 2 値 |
| `uq_graph_node_extref` | UNIQUE | `(tenant_id, kind, external_ref, valid_from)` | `WHERE external_ref IS NOT NULL AND valid_to IS NULL` | SCD UK |
| `uq_graph_node_scd` | UNIQUE | `(tenant_id, kind, external_ref, valid_from)` | `WHERE valid_to IS NULL` | 現有効 1 行 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `graph_node_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_graph_node_tenant` | btree (PT) | `tenant_id` | − | RLS 補助 |
| `idx_graph_node_kind` | btree (PT) | `(tenant_id, kind)` | `valid_to IS NULL` | 種別検索 |
| `idx_graph_node_fp` | btree (PT) | `(tenant_id, fingerprint)` | `valid_to IS NULL` | fingerprint 一致検索 |
| `idx_graph_node_valid_to` | btree (PT) | `valid_to` | − | SCD 履歴検索 |
| `uq_graph_node_extref` | btree (UK/PT) | `(tenant_id, kind, external_ref, valid_from)` | `valid_to IS NULL` | 業務 ID UK |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_graph_node_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 | 根拠 |
|---|---|---|
| MVP (Phase 1) | 0 (実体なし, MSW mock のみ) | 13 ノード fixture は frontend mock |
| V1 (Phase 2) | < 10K (1 tenant) | 25 domain × 100 work_item + 履歴 10 倍 |
| V2 (Phase 3) | < 100K (1 tenant) | 実 memgraph 展開, SCD 累積 |

---

## 7. RLS ポリシー (per 13 類, per REQ-SEC-001)

```sql
ALTER TABLE graph.graph_node ENABLE ROW LEVEL SECURITY;
ALTER TABLE graph.graph_node FORCE ROW LEVEL SECURITY;

CREATE POLICY rls_graph_node_tenant ON graph.graph_node
  USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- 書込み
CREATE POLICY rls_graph_node_tenant_write ON graph.graph_node
  FOR INSERT
  WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);
```

---

## 8. 関連

- 上位 spec: `docs/architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md` §3.2
- エッジ表: `graph.graph_edge` (T-NEW-002)
- 監査表: `graph.graph_fingerprint` (T-NEW-003)
- 関連: ADR-0041 §2.1 ノード種類, AGENTS.md §4 守門 #13 DB 三類

---

## 9. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 初版: Master (M) 類 / SCD Type 2 / 14 列 / 6 制約 / 6 索引 / 13 類 RLS | 2026-09-02 02:10 JST Ulysses "需求和基本设计, 詳細设计 補完" |
