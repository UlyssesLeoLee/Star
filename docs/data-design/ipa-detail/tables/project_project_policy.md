# project.project_policy — テーブル詳細設計書

> **テーブル ID**: T06
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.3.2
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 | 備考 |
|---|---|---|
| **テーブル ID** | T06 | per `00-INVENTORY.md` |
| **物理名** | `project.project_policy` | − |
| **論理名** | プロジェクトポリシー | プロジェクト単位の AI / 自動化 / 統合制御 |
| **スキーマ** | `project` | − |
| **Module** | `domain-project` | − |
| **種別** | Weak Entity（`project_id` 必須） | W |
| **主キー** | `id UUID` | − |
| **RLS 必須** | **Yes** | 13 類对象 |
| **概要** | プロジェクト単位の AI / 自動化 / 統合ポリシー。`project_id` 1:1 関係。Tenant Policy を上書き（部分）。 | §4.3.2 / §4.10.5 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 | 備考 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | レコード識別子 | §3.1.2 |
| 2 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | ✓ | `uq_project_policy_project` | 弱実体 1:1 関係、UK 強制 | §4.3.2 |
| 3 | `cloud_ai_allowed` | クラウド AI 許可 | BOOLEAN | 1 | YES | `NULL` | − | − | − | − | Tenant Policy 上書き（NULL = 継承） | §4.10.5 |
| 4 | `local_ai_only` | ローカル AI のみ | BOOLEAN | 1 | YES | `NULL` | − | − | − | − | 同上 | §4.10.5 |
| 5 | `specific_provider_allowed` | 特定 Provider 許可 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 許可 Provider 配列 | §4.10.5 |
| 6 | `no_code_upload` | コードアップロード禁止 | BOOLEAN | 1 | YES | `NULL` | − | − | − | − | Tenant Policy 上書き | §4.10.5 |
| 7 | `metadata_only` | メタデータのみ | BOOLEAN | 1 | YES | `NULL` | − | − | − | − | 同上 | §4.10.5 |
| 8 | `automation_enabled` | 自動化有効 | BOOLEAN | 1 | NO | `TRUE` | − | − | − | − | プロジェクト内の自動化ルール実行可否 | §4.13 |
| 9 | `integration_enabled` | 統合有効 | BOOLEAN | 1 | NO | `TRUE` | − | − | − | − | 外部統合（GitHub / Jira 等）有効 | §4.12 |
| 10 | `feedback_audience_scope` | Feedback 対象範囲 | VARCHAR | 32 | NO | `'human'` | − | − | − | − | `'human'` / `'agent'` / `'system'` | REQ-NOTIF-002 連動 |
| 11 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − | §3.5 |
| 12 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − | §3.5 |
| 13 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | − | §3.1.5 |
| 14 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | − | §3.1.2 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 / 条件 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `project_policy_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_project_policy_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | プロジェクト削除時 Policy 削除 |
| `uq_project_policy_project` | UNIQUE | `project_id` (partial) | `WHERE deleted_at IS NULL` | − | 1 プロジェクト 1 Policy |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `project_policy_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_project_policy_project` | btree (UK/PT) | `project_id` | `deleted_at IS NULL` | 1:1 強制 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_project_policy_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 1,000 |
| 1 年後 | 10,000 |
| 3 年後 | 100,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 400 B | 100,000 | 約 40 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `project.project` | `project_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `tenant.provider_data_boundary` | `project_policy_id` (App 側検証) |

---

## 9. RLS Policy

```sql
ALTER TABLE project.project_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE project.project_policy FORCE ROW LEVEL SECURITY;

-- Project 経由でテナント分離（project_id → project.tenant_id 参照は App 層で解決）
-- 簡略化のため tenant_id を Session GUC で直接参照する方針:
CREATE POLICY policy_project_policy_tenant_isolation ON project.project_policy
  USING (
    project_id IN (
      SELECT id FROM project.project
      WHERE tenant_id = current_setting('app.current_tenant_id', TRUE)::uuid
    )
  );
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
