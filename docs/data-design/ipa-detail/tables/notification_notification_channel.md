# notification.notification_channel — テーブル詳細設計書

> **テーブル ID**: T45
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.15.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T45 |
| **物理名** | `notification.notification_channel` |
| **論理名** | 通知チャネル |
| **スキーマ** | `notification` |
| **Module** | `domain-notification` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | ユーザ通知チャネル。5 種別（email / in_app / slack / dingtalk / webhook）。MVP メール + 站内。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `user_id` | ユーザ ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE CASCADE | − | idx | 所有者 |
| 4 | `channel_type` | チャネル種別 | VARCHAR | 32 | NO | − | − | − | ✓ | `uq_notification_channel_tenant_key` | 5 値 |
| 5 | `config` | 設定 | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 種別依存設定 |
| 6 | `is_enabled` | 有効フラグ | BOOLEAN | 1 | NO | `TRUE` | − | − | − | − | − |
| 7 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 8 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 10 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `notification_channel_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_notification_channel_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `fk_notification_channel_user` | FOREIGN KEY | `user_id` | `identity.user(id)` ON DELETE CASCADE | ユーザ削除時 チャネル削除 |
| `uq_notification_channel_tenant_key` | UNIQUE | `(tenant_id, channel_key)` | `WHERE deleted_at IS NULL` | チャネルキー一意 |
| `ck_channel_type` | CHECK | `channel_type` | `IN ('email','in_app','slack','dingtalk','webhook')` | 5 種別 |

> **注**: data-design §4.15.1 には `channel_key` 列が未実装、UK 制約は §00-INVENTORY.md 派生

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `notification_channel_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_notification_channel_tenant_key` | btree (UK/PT) | `(tenant_id, channel_key)` | `deleted_at IS NULL` | チャネルキー一意 |
| `idx_notification_channel_tenant_user` | btree (PT) | `(tenant_id, user_id)` | `deleted_at IS NULL` | テナント + ユーザ |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_notification_channel_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 20,000 |（10,000 ユーザ × 2 チャネル平均） |
| 1 年後 | 200,000 |
| 3 年後 | 2,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 500 B | 2,000,000 | 約 1 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `identity.user` | `user_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `notification.notification` | `channel_id` (ON DELETE SET NULL) |

---

## 9. RLS Policy

```sql
ALTER TABLE notification.notification_channel ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.notification_channel FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_notification_channel_tenant_isolation ON notification.notification_channel
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
