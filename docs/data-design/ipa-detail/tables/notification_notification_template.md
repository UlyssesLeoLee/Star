# notification.notification_template — テーブル詳細設計書

> **テーブル ID**: T46
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.15.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T46 |
| **物理名** | `notification.notification_template` |
| **論理名** | 通知テンプレート |
| **スキーマ** | `notification` |
| **Module** | `domain-notification` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | 通知テンプレート。Handlebars 変数 `{{...}}` 対応。`event_type` × `locale` で UK 制御。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `event_type` | イベント種別 | VARCHAR | 64 | NO | − | − | − | ✓ | `uq_template_event_locale` | `'work_item.assigned'` 等 |
| 4 | `subject_template` | 件名テンプレート | TEXT | − | NO | − | − | − | − | − | Handlebars |
| 5 | `body_template` | 本文テンプレート | TEXT | − | NO | − | − | − | − | − | Handlebars |
| 6 | `locale` | ロケール | VARCHAR | 16 | NO | `'en'` | − | − | ✓ | `uq_template_event_locale` | `'en'` / `'zh-CN'` / `'ja'` |
| 7 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 8 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 10 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `notification_template_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_notification_template_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `uq_template_event_locale` | UNIQUE | `(tenant_id, event_type, locale, deleted_at)` | − | − | イベント × locale 一意 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `notification_template_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_template_event_locale` | btree (UK/PT) | `(tenant_id, event_type, locale, deleted_at)` | − | イベント × locale |
| `idx_notification_template_event` | btree (PT) | `(tenant_id, event_type)` | `deleted_at IS NULL` | イベント別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_notification_template_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 5,000 |（20 イベント × 3 locale × 80 テナント 平均） |
| 1 年後 | 50,000 |
| 3 年後 | 500,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 2 KB (TEXT テンプレート) | 500,000 | 約 1 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `notification.notification` | `template_id` (ON DELETE RESTRICT) |

---

## 9. RLS Policy

```sql
ALTER TABLE notification.notification_template ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.notification_template FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_notification_template_tenant_isolation ON notification.notification_template
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
