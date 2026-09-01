# comment.attachment — テーブル詳細設計書

> **テーブル ID**: T27
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.9.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T27 |
| **物理名** | `comment.attachment` |
| **論理名** | 添付ファイル |
| **スキーマ** | `comment` |
| **Module** | `domain-comment` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | 添付ファイル。Object Storage Key `storage_ref` 保持。≤ 100MB。3 Storage Class（HOT / WARM / COLD）。§1.5 / §5.1 派生。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `filename` | ファイル名 | VARCHAR | 512 | NO | − | − | − | − | − | 元ファイル名 |
| 4 | `mime_type` | MIME 種別 | VARCHAR | 128 | NO | − | − | − | − | − | `application/pdf` / `image/png` 等 |
| 5 | `size_bytes` | サイズ（バイト） | BIGINT | 8 | NO | − | − | − | − | − | 0..100MB（CHECK 制約） |
| 6 | `storage_ref` | ストレージ参照 | VARCHAR | 2048 | NO | − | − | − | − | idx | Object Storage Key |
| 7 | `storage_class` | ストレージクラス | VARCHAR | 16 | NO | `'WARM'` | − | − | − | − | `'HOT'` / `'WARM'` / `'COLD'` |
| 8 | `uploader_user_id` | アップローダ ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE RESTRICT | − | idx | アップローダユーザ |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 12 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `attachment_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_attachment_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_attachment_uploader` | FOREIGN KEY | `uploader_user_id` | `identity.user(id)` | RESTRICT | − |
| `ck_attachment_size` | CHECK | `size_bytes` | `size_bytes > 0 AND size_bytes <= 104857600` | − | 0..100MB |
| `ck_storage_class` | CHECK | `storage_class` | `IN ('HOT','WARM','COLD')` | − | 3 Storage Class |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `attachment_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_attachment_tenant_uploader` | btree (PT) | `(tenant_id, uploader_user_id)` | `deleted_at IS NULL` | テナント + アップローダ |
| `idx_attachment_storage_ref` | btree | `storage_ref` | − | Object Storage 検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_attachment_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 100,000 |
| 1 年後 | 1,000,000 |
| 3 年後 | 10,000,000 |

---

## 7. 想定容量

> メタデータのみ（実体は Object Storage）

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 2.5 KB | 10,000,000 | 約 25 MB（メタデータ） |

実体（Object Storage）は別管理：MVP 1 TB / 3 年後 100 TB

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `identity.user` | `uploader_user_id` |

### 8.2 被参照元

なし（末端、Comment / WorkItem からは `comment_id` / `work_item_id` で間接参照、App 層管理）

---

## 9. RLS Policy

```sql
ALTER TABLE comment.attachment ENABLE ROW LEVEL SECURITY;
ALTER TABLE comment.attachment FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_attachment_tenant_isolation ON comment.attachment
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
