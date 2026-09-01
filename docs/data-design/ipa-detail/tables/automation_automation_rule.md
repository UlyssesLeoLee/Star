# automation.automation_rule — テーブル詳細設計書

> **テーブル ID**: T36
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.13.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T36 |
| **物理名** | `automation.automation_rule` |
| **論理名** | 自動化ルール |
| **スキーマ** | `automation` |
| **Module** | `domain-automation` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Trigger-Conditions-Actions ルール。`trigger_config` JSONB で Event / Schedule / Cron 3 種別対応（S1 落点 / REQ-AUTO-002 V1 候補）。MVP 単表、UI Builder 必要時に `automation_trigger` / `automation_action` 分割（V1 候補）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `name` | ルール名 | VARCHAR | 200 | NO | − | − | − | ✓ | `uq_automation_rule_tenant_key` | 業務表示名 |
| 5 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | − |
| 6 | `trigger_config` | トリガ設定 | JSONB | − | NO | − | − | − | − | GIN | Event / Schedule / Cron 3 種別（`kind` フィールド） |
| 7 | `conditions` | 条件 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 条件配列 |
| 8 | `actions` | アクション | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | アクション配列 |
| 9 | `is_enabled` | 有効フラグ | BOOLEAN | 1 | NO | `TRUE` | − | − | − | PT | ルール有効 / 無効 |
| 10 | `last_executed_at` | 最終実行日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 実行統計 |
| 11 | `execution_count` | 実行回数 | BIGINT | 8 | NO | `0` | − | − | − | − | 累積実行回数 |
| 12 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 13 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 14 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 15 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `automation_rule_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_automation_rule_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_automation_rule_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `uq_automation_rule_tenant_key` | UNIQUE | `(tenant_id, project_id, rule_key)` | `WHERE deleted_at IS NULL` | − | PJ 内ルール名一意（実装時、rule_key 列追加） |

> **注**: `rule_key` 列は data-design §4.13.1 に未実装、UK 制約は §00-INVENTORY.md 派生。実 SQL 適用時に列追加必要。

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `automation_rule_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_automation_tenant_project_enabled` | btree (PT) | `(tenant_id, project_id, is_enabled)` | `deleted_at IS NULL` | PJ + 有効 |
| `idx_automation_trigger_gin` | GIN | `trigger_config` | − | トリガ JSONB 検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_automation_rule_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 5,000 |
| 1 年後 | 50,000 |
| 3 年後 | 500,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.5 KB (JSONB ×3) | 500,000 | 約 750 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE automation.automation_rule ENABLE ROW LEVEL SECURITY;
ALTER TABLE automation.automation_rule FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_automation_rule_tenant_isolation ON automation.automation_rule
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
