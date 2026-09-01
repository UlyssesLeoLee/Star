# tenant.tenant — テーブル詳細設計書

> **テーブル ID**: T01
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.1.1 + ON-001 修正
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 | 備考 |
|---|---|---|
| **テーブル ID** | T01 | per `00-INVENTORY.md` |
| **物理名** | `tenant.tenant` | PostgreSQL 修飾子付き |
| **論理名** | テナント | 業務名 |
| **スキーマ** | `tenant` | 25 Schema の 1 |
| **Module** | `domain-tenant` | 1:1 対応 |
| **種別** | Entity（業務事実の源流） | E |
| **主キー** | `id UUID` | 単一 PK、UUID v7 推奨 |
| **R/W 識別** | R/W（SoR） | §1.3 参照 |
| **RLS 必須** | **No**（Tenant がテナント分離の源流） | §7 / §3.1.2 |
| **パーティション** | None | − |
| **soft delete** | Yes（`deleted_at`） | §3.1.5 |
| **概要** | テナント最高安全境界。テナント分離の源流で、全 SoR テーブルの `tenant_id` FK が本表を参照する。 | §6.1, §R-26 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 | 備考 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | レコード識別子（UUID v7 推奨、App 側生成） | §3.1.2 / §2.3 |
| 2 | `name` | テナント名 | VARCHAR | 200 | NO | − | − | − | − | − | 業務表示名、UI / Email 等で使用 | §3.1.6 |
| 3 | `slug` | スラッグ | VARCHAR | 64 | NO | − | − | − | ✓ | `uq_tenant_slug` | 短标识、URL 友好、テナントサブドメイン / パスで使用 | §4.1.1 / 1 文字以上 64 文字以下、英数 + `-` |
| 4 | `plan` | プラン | VARCHAR | 32 | NO | `'free'` | − | − | − | − | 購読プラン、リソースクォータ影響 | §4.1.1 / `ck_tenant_plan`: `IN ('free','pro','enterprise','trial')` |
| 5 | `status` | 状態 | VARCHAR | 32 | NO | `'ACTIVE'` | − | − | − | `idx_tenant_status` | ライフサイクル状態 | §4.1.1 / `ck_tenant_status`: `IN ('ACTIVE','SUSPENDED','ARCHIVED')` |
| 6 | `contact_email` | 連絡先 email | VARCHAR | 320 | NO | − | − | − | − | − | 主連絡先 email | §3.1.6 / citext 推奨（現状 VARCHAR） |
| 7 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レコード作成日時（UTC） | §3.5 |
| 8 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | レコード更新日時（UTC）、TRIGGER 自動更新 | §3.5 / §4.1.1.4 |
| 9 | `deleted_at` | 削除日時（soft delete） | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除日時、NULL = 存活 | §3.1.5 |
| 10 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック、App 層で UPDATE 毎に +1 + 検証 | §3.1.2 |

> **補足**:
> - `slug` は現在 `citext` ではなく `VARCHAR(64)` で実装。Email ほど大文字小区別重要性は低いが、URL 用途なので厳密一致推奨（小文字のみ許可の CHECK 追加検討 = ON-001 系）
> - `contact_email` は `citext` 列に昇格予定（V1 候補、ON-401 系）

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 / 条件 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `tenant_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `uq_tenant_slug` | UNIQUE | `slug` | − | − | スラッグ一意（部分インデックス `WHERE deleted_at IS NULL`） |
| `ck_tenant_status` | CHECK | `status` | `status IN ('ACTIVE','SUSPENDED','ARCHIVED')` | − | ライフサイクル 3 状態 |
| `ck_tenant_plan` | CHECK | `plan` | `plan IN ('free','pro','enterprise','trial')` | − | 購読プラン 4 種 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 包含列 | 条件 (`WHERE`) | 説明 |
|---|---|---|---|---|---|
| `tenant_pkey` | btree (PK) | `id` | − | − | 主キー |
| `uq_tenant_slug` | btree (UK) | `slug` | − | `deleted_at IS NULL` | スラッグ一意（論理削除考慮） |
| `idx_tenant_status` | btree (PT) | `status` | − | `deleted_at IS NULL` | ステータス別絞り込み |

---

## 5. トリガー一覧

| トリガー名 | 発火 | レベル | 関数 | 説明 |
|---|---|---|---|---|
| `trg_tenant_updated_at` | BEFORE UPDATE | ROW | `public.fn_update_updated_at()` | `updated_at = NOW()` 自動更新 |

> 関数 `fn_update_updated_at()` 本体は Implementation 段階（`migrations/001_fn_update_updated_at.sql`）。本設計書は参照のみ。

---

## 6. 想定レコード件数

| フェーズ | 件数 | 根拠 |
|---|---|---|
| MVP（ローンチ半年） | 100 | 100 テナント × 1 件 = 100 件 |
| 1 年後 | 1,000 | B2B SaaS 想定 1,000 テナント |
| 3 年後（飽和） | 10,000 | エンタープライズ拡大、10,000 テナント想定 |

> MVP 段階は暫定値、SRE Lead 監視で 1 ヶ月以内に実測値に置換。

---

## 7. 想定容量

| 1 行バイト（推定） | 想定件数 | 想定容量 | 備考 |
|---|---|---|---|
| 約 800 B | 10,000 | 約 8 MB | UUID(16) + VARCHAR(200+64+32+32+320) + TIMESTAMPTZ(8×3) + INT(4) + 28B overhead = 約 800 B/行 |

---

## 8. 関連テーブル

### 8.1 依存先（このテーブルが参照）

なし（Tenant が源流）

### 8.2 被参照元（このテーブルを参照）

> 主要な被参照元のみ記載（全 100 テーブルの 80% が `tenant_id` で参照）

| 被参照元 | FK 列 | 関係 | カーディナリティ |
|---|---|---|---|
| `tenant.tenant_policy` | `tenant_id` | 1:N | 1 テナント : N ポリシー |
| `tenant.provider_data_boundary` | `tenant_id` | 1:N | 1 テナント : N プロバイダ境界 |
| `workspace.workspace` | `tenant_id` | 1:N | 1 テナント : N ワークスペース |
| `project.project` | `tenant_id` | 1:N | 1 テナント : N プロジェクト |
| `work_item.work_item` | `tenant_id` | 1:N | 1 テナント : N ワークアイテム |
| `identity.user` | `tenant_id` | 1:N | 1 テナント : N ユーザ |
| `worktree.worktree` | `tenant_id` | 1:N | 1 テナント : N ワークツリー |
| `agent.agent_session` | `tenant_id` | 1:N | 1 テナント : N セッション |
| `feedback.feedback` | `tenant_id` | 1:N | 1 テナント : N フィードバック |
| `audit.audit_event` | `tenant_id` | 1:N | 1 テナント : N 監査イベント |

> 全 80 テーブルの 13 類 RLS 必須对象が `tenant_id` で参照。詳細列挙は §00-CONSTRAINTS.md §3 参照。

### 8.3 兄弟・関連（同 Module / 跨 Module）

- `tables/tenant_tenant_policy.md` — 同一スキーマ内、`tenant_id` FK
- `tables/tenant_provider_data_boundary.md` — 同一スキーマ内、`tenant_id` FK
- `tables/identity_user.md` — `tenant_id` 経由のユーザ管理

---

## 9. RLS Policy

> **本テーブルは RLS を強制しない**（§7 派生 / §3.1.2 派生）。
> 理由: Tenant 自身がテナント分離の源流。Tenant を RLS で絞るとテナント作成自体ができなくなる。

### 9.1 RLS 無効化（明示）

```sql
ALTER TABLE tenant.tenant DISABLE ROW LEVEL SECURITY;
```

> Application 層で `app.is_platform_admin` GUC を見て全テナント参照権限を制御。
> Platform Admin のみが全テナントを一覧・操作可能。一般テナントユーザは自分が所属するテナントのみアクセス。

### 9.2 関連テーブルの RLS 強制

`tenant.tenant` を参照する他テーブルは RLS 必須（`tenant_id` でフィルタ）：

```sql
-- 例: tenant.tenant_policy に対する RLS
ALTER TABLE tenant.tenant_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant.tenant_policy FORCE ROW LEVEL SECURITY;
CREATE POLICY policy_tenant_policy_tenant_isolation ON tenant.tenant_policy
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::uuid);
```

> 詳細は各テーブルの IPA ファイル §9 参照。

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：IPA 標準章立てに整流（10 列 / 4 制約 / 3 INDEX） | per 2026-09-01 15:30 JST Ulysses 拍板（ipa_inline / per_table / scope_everything / opt_dual） |
| v0.1.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | ON-001 関連: 引用 `WAITING_FEEDBACK` 修正済を v0.2.1 で確認 | per ON-001 P0 修正コミット |
