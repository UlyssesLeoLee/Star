# integration.integration — テーブル詳細設計書

> **テーブル ID**: T33
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.12.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T33 |
| **物理名** | `integration.integration` |
| **論理名** | 外部統合 |
| **スキーマ** | `integration` |
| **Module** | `domain-integration` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | サードパーティ統合（GitHub / GitLab / Jira / Slack / Linear / PagerDuty / Email）。Credential は `credential_ref` 経由で Credential Broker 抽象（§4.10.8 / §28.4 派生）。4 状態。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `provider` | Provider | VARCHAR | 32 | NO | − | − | − | ✓ | `uq_integration_tenant_provider` | 7 プロバイダ |
| 5 | `integration_type` | 統合種別 | VARCHAR | 32 | NO | − | − | − | − | − | `'scm'` / `'notification'` / `'project_sync'` |
| 6 | `config` | 設定 | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | Provider 特定設定 |
| 7 | `credential_ref` | 資格情報参照 | VARCHAR | 255 | NO | − | − | (Credential Broker) | − | − | Credential Broker キー |
| 8 | `status` | 状態 | VARCHAR | 32 | NO | `'ACTIVE'` | − | − | − | PT | 4 状態 |
| 9 | `display_name` | 表示名 | VARCHAR | 200 | NO | − | − | − | − | − | 業務表示名 |
| 10 | `external_id` | 外部 ID | VARCHAR | 256 | YES | `NULL` | − | − | ✓ | `uq_integration_tenant_provider` | Provider 内外部 ID |
| 11 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 12 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 13 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 14 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `integration_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_integration_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_integration_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `uq_integration_tenant_provider` | UNIQUE | `(tenant_id, provider, external_id)` | `WHERE deleted_at IS NULL` | − | Provider 内一意 |
| `ck_integration_status` | CHECK | `status` | `IN ('ACTIVE','PAUSED','ERROR','DISABLED')` | − | 4 状態 |
| `ck_integration_provider` | CHECK | `provider` | `IN ('github','gitlab','jira','slack','linear','pagerduty','email')` | − | 7 プロバイダ |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `integration_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_integration_tenant_provider` | btree (UK/PT) | `(tenant_id, provider, external_id)` | `deleted_at IS NULL` | Provider 内一意 |
| `idx_integration_tenant_project_provider` | btree (PT) | `(tenant_id, project_id, provider)` | `deleted_at IS NULL` | PJ + Provider |
| `idx_integration_status` | btree (PT) | `(status)` | `status IN ('ERROR','PAUSED') AND deleted_at IS NULL` | 異常状態監視 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_integration_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

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
| 約 1.5 KB (config + credential_ref) | 500,000 | 約 750 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `identity.credential` (Credential Broker) | `credential_ref` 経由 |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `integration.integration_sync_state` | `integration_id` |
| `scm.webhook_event` | `integration_id` |

---

## 9. RLS Policy

```sql
ALTER TABLE integration.integration ENABLE ROW LEVEL SECURITY;
ALTER TABLE integration.integration FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_integration_tenant_isolation ON integration.integration
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
