# workspace.workspace — テーブル詳細設計書

> **テーブル ID**: T04
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.2.1
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 | 備考 |
|---|---|---|
| **テーブル ID** | T04 | per `00-INVENTORY.md` |
| **物理名** | `workspace.workspace` | − |
| **論理名** | ワークスペース | Tenant 内の中間組織単位 |
| **スキーマ** | `workspace` | − |
| **Module** | `domain-workspace` | − |
| **種別** | Entity | E |
| **主キー** | `id UUID` | − |
| **R/W 識別** | R/W（SoR） | − |
| **RLS 必須** | **Yes** | 13 類对象 |
| **パーティション** | None | − |
| **soft delete** | Yes | `deleted_at` |
| **概要** | テナント内のワークスペース。プロジェクト / ユーザ / ワークツリーを束ねる中間組織単位。3 種別（TEAM / PERSONAL / ENTERPRISE）。 | §1.3 / §4.2.1 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 | 備考 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | レコード識別子 | §3.1.2 |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE RESTRICT | − | idx | テナント分離キー、RLS 必須 | §7 |
| 3 | `name` | ワークスペース名 | VARCHAR | 200 | NO | − | − | − | − | − | 業務表示名 | §3.1.6 |
| 4 | `slug` | スラッグ | VARCHAR | 64 | NO | − | − | − | ✓ | `uq_workspace_tenant_slug` | 短标识、テナント内一意 | §4.2.1 |
| 5 | `type` | 種別 | VARCHAR | 32 | NO | − | − | − | − | idx | 種別（TEAM / PERSONAL / ENTERPRISE） | `ck_workspace_type` |
| 6 | `owner_id` | オーナ ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE RESTRICT | − | idx | オーナーユーザ | §4.2.1 |
| 7 | `settings_json` | 設定 | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | ワークスペース設定（通知設定、テーマ等） | V1 候補 |
| 8 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | 業務説明 | §3.1.6 |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レコード作成日時 | §3.5 |
| 10 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レコード更新日時 | §3.5 |
| 11 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除日時 | §3.1.5 |
| 12 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック | §3.1.2 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 / 条件 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `workspace_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_workspace_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | RESTRICT | テナント削除禁止（参照存在） |
| `fk_workspace_owner` | FOREIGN KEY | `owner_id` | `identity.user(id)` | RESTRICT | オーナー削除禁止 |
| `uq_workspace_tenant_slug` | UNIQUE | `(tenant_id, slug)` | `WHERE deleted_at IS NULL` | − | テナント内 slug 一意 |
| `ck_workspace_type` | CHECK | `type` | `IN ('TEAM','PERSONAL','ENTERPRISE')` | − | 3 種別 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 包含列 | 条件 (`WHERE`) | 説明 |
|---|---|---|---|---|---|
| `workspace_pkey` | btree (PK) | `id` | − | − | 主キー |
| `uq_workspace_tenant_slug` | btree (UK/PT) | `(tenant_id, slug)` | − | `deleted_at IS NULL` | slug 一意 |
| `idx_workspace_tenant_type` | btree (PT) | `(tenant_id, type)` | − | `deleted_at IS NULL` | 種別絞り込み |
| `idx_workspace_tenant_owner` | btree (PT) | `(tenant_id, owner_id)` | − | `deleted_at IS NULL` | オーナー別一覧 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | レベル | 関数 | 説明 |
|---|---|---|---|---|
| `trg_workspace_updated_at` | BEFORE UPDATE | ROW | `public.fn_update_updated_at()` | `updated_at = NOW()` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 | 根拠 |
|---|---|---|
| MVP | 500 | 100 テナント × 5 ワークスペース |
| 1 年後 | 5,000 | 1,000 × 5 |
| 3 年後 | 50,000 | 10,000 × 5 |

---

## 7. 想定容量

| 1 行バイト（推定） | 想定件数 | 想定容量 | 備考 |
|---|---|---|---|
| 約 500 B | 50,000 | 約 25 MB | − |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 | 関係 | カーディナリティ |
|---|---|---|---|
| `tenant.tenant` | `tenant_id` | N:1 | − |
| `identity.user` | `owner_id` | N:1 | − |

### 8.2 被参照元

| 被参照元 | FK 列 | 関係 | カーディナリティ |
|---|---|---|---|
| `project.project` | `workspace_id` | 1:N | 1 ワークスペース : N プロジェクト |
| `collaboration.presence` | `workspace_id` | 1:N | 1 ワークスペース : N 在席 |

### 8.3 兄弟・関連

- `tables/project_project.md` — `workspace_id` 経由
- `tables/identity_user.md` — `owner_id` 経由
- `tables/collaboration_presence.md` — `workspace_id` 経由

---

## 9. RLS Policy

```sql
ALTER TABLE workspace.workspace ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspace.workspace FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_workspace_tenant_isolation ON workspace.workspace
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::uuid);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：IPA 標準章立てに整流（12 列 / 5 制約 / 4 INDEX） | per 2026-09-01 15:30 JST Ulysses 拍板 |
