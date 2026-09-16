# graph.graph_edge — テーブル詳細設計書

> **テーブル ID**: T-NEW-002
> **作成日**: 2026-09-02
> **改訂人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **一次出典**: `docs/architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md` §3.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T-NEW-002 |
| **物理名** | `graph.graph_edge` |
| **論理名** | グラフエッジ (cypher 投影, Master) |
| **スキーマ** | `graph` |
| **Module** | `domain-graph-agent` (Phase 2 新設) |
| **種別** | **Master (M)** — SCD Type 2 + 物理削除禁止 + RLS 13 類必携 |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | 24 edge kind (ASSIGNED_TO / IN_PROJECT / ON_WORKTREE / ...) の cypher 投影。`graph_node` の SCD と同調, source/target 両 FK 必須。`hop_level` 1 or 2 (2 は code-side のみ)。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | 内部 ID |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx (PT) | 13 類 RLS 必須 |
| 3 | `kind` | 種別 | VARCHAR | 32 | NO | − | − | − | ✓ | `idx_graph_edge_kind` (PT) | 24 edge kind |
| 4 | `source_id` | 始点 | UUID | − | NO | − | − | `graph.graph_node(id)` ON DELETE RESTRICT | − | idx (PT) | 始点ノード FK |
| 5 | `target_id` | 終点 | UUID | − | NO | − | − | `graph.graph_node(id)` ON DELETE RESTRICT | − | idx (PT) | 終点ノード FK |
| 6 | `hop_level` | 跳躍階層 | SMALLINT | 2 | NO | `1` | − | − | − | − | `1` = 1-hop, `2` = 2-hop (code-side のみ) |
| 7 | `properties` | プロパティ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 透過プロパティ |
| 8 | `fingerprint` | 指紋 | VARCHAR | 64 | NO | − | − | − | − | `idx_graph_edge_fp` (PT) | 当該 work_item の sha256 |
| 9 | `valid_from` | SCD 開始 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | ✓ | `uq_graph_edge_scd` | SCD Type 2 開始 |
| 10 | `valid_to` | SCD 終了 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | `idx_graph_edge_valid_to` | `NULL` = 現有効 |
| 11 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 監査 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `graph_edge_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_graph_edge_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | RLS |
| `fk_graph_edge_source` | FOREIGN KEY | `source_id` | `graph.graph_node(id)` ON DELETE RESTRICT | ノード連動 |
| `fk_graph_edge_target` | FOREIGN KEY | `target_id` | `graph.graph_node(id)` ON DELETE RESTRICT | ノード連動 |
| `ck_graph_edge_kind` | CHECK | `kind` | `IN ('ASSIGNED_TO','REPORTED_BY','IN_PROJECT','IN_WORKSPACE','ON_WORKTREE','PRODUCED','HAS_FEEDBACK','VALIDATED_BY','COMMENTED_ON','DESIGNED_BY','RUNS_ON','POWERS','INTEGRATES','REFERENCES','LIVES_IN','DEPENDS_ON','INHERITS_FROM','TRIGGERS','RAISED_INCIDENT','WEBHOOK_FOR','HAS_PR','TARGETS_BRANCH','WITH_PERMISSION','FOLLOWING_WORKFLOW')` | 24 値 |
| `ck_graph_edge_hop` | CHECK | `hop_level` | `IN (1, 2)` | 跳躍階層 2 値 |
| `ck_graph_edge_noself` | CHECK | `source_id, target_id` | `source_id <> target_id` | 自己ループ禁止 |
| `uq_graph_edge_scd` | UNIQUE | `(tenant_id, kind, source_id, target_id, valid_from)` | `WHERE valid_to IS NULL` | SCD UK |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `graph_edge_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_graph_edge_tenant` | btree (PT) | `tenant_id` | − | RLS 補助 |
| `idx_graph_edge_kind` | btree (PT) | `(tenant_id, kind)` | `valid_to IS NULL` | 種別検索 |
| `idx_graph_edge_source` | btree (PT) | `(tenant_id, source_id)` | `valid_to IS NULL` | 始点から辺検索 |
| `idx_graph_edge_target` | btree (PT) | `(tenant_id, target_id)` | `valid_to IS NULL` | 終点から辺検索 |
| `idx_graph_edge_fp` | btree (PT) | `(tenant_id, fingerprint)` | `valid_to IS NULL` | fingerprint 一致 |
| `idx_graph_edge_valid_to` | btree (PT) | `valid_to` | − | SCD 履歴検索 |
| `uq_graph_edge_scd` | btree (UK/PT) | `(tenant_id, kind, source_id, target_id, valid_from)` | `valid_to IS NULL` | SCD UK |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_graph_edge_validate_hop` | BEFORE INSERT | (inline) | hop_level=2 は kind が REFERENCES / LIVES_IN / DEPENDS_ON / INHERITS_FROM の 4 種のみ |

---

## 6. 想定レコード件数

| フェーズ | 件数 | 根拠 |
|---|---|---|
| MVP (Phase 1) | 0 (実体なし, MSW mock のみ) | 13 エッジ fixture は frontend mock |
| V1 (Phase 2) | < 100K (1 tenant) | 13 エッジ/ノード × 100 work_item + 履歴 10 倍 |
| V2 (Phase 3) | < 1M (1 tenant) | 実 memgraph 展開, SCD 累積 |

---

## 7. RLS ポリシー (per 13 類, per REQ-SEC-001)

```sql
ALTER TABLE graph.graph_edge ENABLE ROW LEVEL SECURITY;
ALTER TABLE graph.graph_edge FORCE ROW LEVEL SECURITY;

CREATE POLICY rls_graph_edge_tenant ON graph.graph_edge
  USING (tenant_id = current_setting('app.tenant_id', true)::UUID);

CREATE POLICY rls_graph_edge_tenant_write ON graph.graph_edge
  FOR INSERT
  WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);
```

---

## 8. 関連

- 上位 spec: `docs/architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md` §3.3
- ノード表: `graph.graph_node` (T-NEW-001)
- 監査表: `graph.graph_fingerprint` (T-NEW-003)
- 関連: ADR-0041 §2.1 エッジ種類, AGENTS.md §4 守門 #13 DB 三類

---

## 9. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 初版: Master (M) 類 / SCD Type 2 / 11 列 / 8 制約 / 8 索引 / 13 類 RLS / hop_level=2 4 種限定 | 2026-09-02 02:10 JST Ulysses "需求和基本设计, 詳細设计 補完" |
