# integration.integration_sync_state — テーブル詳細設計書

> **テーブル ID**: T34
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.12.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T34 |
| **物理名** | `integration.integration_sync_state` |
| **論理名** | 統合同期状態 |
| **スキーマ** | `integration` |
| **Module** | `domain-integration` |
| **種別** | Weak Entity（`integration_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Integration 単位の同期状態。`sync_token` で ETag / X-Next-Sync-Token / cursor 保持。`conflict_strategy` 既定 `LatestWins`。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `integration_id` | 統合 ID | UUID | − | NO | − | − | `integration.integration(id)` ON DELETE CASCADE | − | idx | 親 Integration |
| 4 | `resource_type` | リソース種別 | VARCHAR | 32 | NO | − | − | − | ✓ | `uq_integration_sync_state_resource` | 同期対象リソース種別 |
| 5 | `resource_id` | リソース ID | UUID | − | NO | − | − | (App 検証) | ✓ | `uq_integration_sync_state_resource` | 同期対象リソース |
| 6 | `sync_token` | Sync Token | VARCHAR | 1024 | YES | `NULL` | − | − | − | − | ETag / cursor |
| 7 | `last_synced_at` | 最終同期日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 最終同期成功時刻 |
| 8 | `next_sync_at` | 次回同期予定 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | ポーリング次回時刻 |
| 9 | `conflict_strategy` | 衝突戦略 | VARCHAR | 32 | NO | `'LatestWins'` | − | − | − | − | `'LatestWins'` / `'ManualResolve'` |
| 10 | `last_error` | 最終エラー | TEXT | − | YES | `NULL` | − | − | − | − | 同期エラーメッセージ |
| 11 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 12 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 13 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 14 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `integration_sync_state_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_sync_state_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_sync_state_integration` | FOREIGN KEY | `integration_id` | `integration.integration(id)` | CASCADE | 親削除時 State 削除 |
| `uq_integration_sync_state_resource` | UNIQUE | `(integration_id, resource_type, resource_id)` | `WHERE deleted_at IS NULL` | − | リソース 1:1 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `integration_sync_state_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_integration_sync_state_resource` | btree (UK/PT) | `(integration_id, resource_type, resource_id)` | `deleted_at IS NULL` | リソース 1:1 |
| `idx_integration_sync_state_pending` | btree (PT) | `(integration_id, next_sync_at)` | `status = 'PENDING' AND deleted_at IS NULL` | 次回同期ポーリング |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_integration_sync_state_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 50,000 |
| 1 年後 | 500,000 |
| 3 年後 | 5,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.2 KB | 5,000,000 | 約 6 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `integration.integration` | `integration_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE integration.integration_sync_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE integration.integration_sync_state FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_integration_sync_state_tenant_isolation ON integration.integration_sync_state
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
