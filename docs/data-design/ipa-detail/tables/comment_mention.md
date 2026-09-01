# comment.mention — テーブル詳細設計書

> **テーブル ID**: T26
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.9.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T26 |
| **物理名** | `comment.mention` |
| **論理名** | メンション |
| **スキーマ** | `comment` |
| **Module** | `domain-comment` |
| **種別** | Weak Entity（`comment_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | @mention。`offset` で文字位置保持、UI レンダリング時のハイライトレンダリング用。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `comment_id` | コメント ID | UUID | − | NO | − | − | `comment.comment(id)` ON DELETE CASCADE | − | idx | 親コメント |
| 4 | `mentioned_user_id` | メンション対象ユーザ ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE CASCADE | − | idx | メンション受信者 |
| 5 | `offset` | 文字オフセット | INT | 4 | NO | − | − | − | ✓ | `uq_mention_comment_user` | 文字位置 |
| 6 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 7 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `mention_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_mention_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_mention_comment` | FOREIGN KEY | `comment_id` | `comment.comment(id)` | CASCADE | 親コメント削除時 メンション削除 |
| `fk_mention_user` | FOREIGN KEY | `mentioned_user_id` | `identity.user(id)` | CASCADE | ユーザ削除時 メンション削除 |
| `uq_mention_comment_user` | UNIQUE | `(comment_id, mentioned_user_id, offset)` | − | − | 同一位置重複禁止 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `mention_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_mention_comment_user` | btree (UK) | `(comment_id, mentioned_user_id, offset)` | − | 重複禁止 |
| `idx_mention_tenant_user` | btree (PT) | `(tenant_id, mentioned_user_id)` | `deleted_at IS NULL` | 受信者別 |
| `idx_mention_comment` | btree | `comment_id` | − | コメント別 |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 500,000 |（1 コメント平均 0.5 メンション） |
| 1 年後 | 5,000,000 |
| 3 年後 | 50,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 200 B | 50,000,000 | 約 10 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `comment.comment` | `comment_id` |
| `identity.user` | `mentioned_user_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `notification.notification` | 派生トリガーで自動生成 |

---

## 9. RLS Policy

```sql
ALTER TABLE comment.mention ENABLE ROW LEVEL SECURITY;
ALTER TABLE comment.mention FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_mention_tenant_isolation ON comment.mention
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
