# local_runtime.reconciliation_report — テーブル詳細設計書

> **テーブル ID**: T99
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.25.4

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T99 |
| **物理名** | `local_runtime.reconciliation_report` |
| **論理名** | 調整レポート |
| **スキーマ** | `local_runtime` |
| **Module** | `domain-local-runtime` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Desired vs Observed 比対レポート。`desired_state_hash` / `observed_state_hash` で SHA-256 ハッシュ。`diff_items JSONB` で 差分詳細。4 状態（IN_SYNC / DRIFT_DETECTED / RECONCILED / RECONCILIATION_FAILED）。`DRIFT_DETECTED` は 人間介入必須。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `runtime_id` | ランタイム ID | UUID | − | NO | − | − | `local_runtime.runtime(id)` ON DELETE CASCADE | − | idx (PT) | 親 Runtime |
| 4 | `desired_state_hash` | 期待状態ハッシュ | VARCHAR | 64 | NO | − | − | − | − | − | SHA-256 |
| 5 | `observed_state_hash` | 観測状態ハッシュ | VARCHAR | 64 | NO | − | − | − | − | − | SHA-256 |
| 6 | `diff_items` | 差分アイテム | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | `[{type, path, expected, actual}]` |
| 7 | `status` | 状態 | VARCHAR | 16 | NO | − | − | − | − | idx (PT) | 4 値 |
| 8 | `reconciled_at` | 調整日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | 調整実行時刻 |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `reconciliation_report_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_reconciliation_report_*` | FOREIGN KEY (2) | `tenant_id` / `runtime_id` | 各親テーブル | CASCADE | − |
| `ck_reconciliation_status` | CHECK | `status` | `IN ('IN_SYNC','DRIFT_DETECTED','RECONCILED','RECONCILIATION_FAILED')` | 4 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `reconciliation_report_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_reconciliation_tenant_runtime` | btree | `(tenant_id, runtime_id, reconciled_at DESC)` | − | テナント + Runtime + 順 |
| `idx_reconciliation_drift` | btree (PT) | `(tenant_id, status)` | `status = 'DRIFT_DETECTED'` | Drift 検出監視 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_reconciliation_report_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 10,000 |（1 Runtime 平均 10 report） |
| 1 年後 | 100,000 |
| 3 年後 | 1,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.2 KB | 1,000,000 | 約 1.2 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `local_runtime.runtime` | `runtime_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE local_runtime.reconciliation_report ENABLE ROW LEVEL SECURITY;
ALTER TABLE local_runtime.reconciliation_report FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_reconciliation_report_tenant_isolation ON local_runtime.reconciliation_report
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
