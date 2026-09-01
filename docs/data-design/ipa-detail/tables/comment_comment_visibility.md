# comment.comment_visibility — テーブル詳細設計書

> **テーブル ID**: T28
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.9.1 + §3.3.2 派生

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T28 |
| **物理名** | `comment.comment_visibility` |
| **論理名** | コメント可視性 Lookup |
| **スキーマ** | `comment` |
| **Module** | `domain-comment` |
| **種別** | Lookup Table（L） |
| **主キー** | `visibility_code VARCHAR(16)` |
| **RLS 必須** | **No**（DISABLE RLS） |
| **概要** | コメント可視性 enum Lookup。3 段階（PUBLIC / INTERNAL / PRIVATE）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `visibility_code` | 可視性コード | VARCHAR | 16 | NO | − | ✓ | − | − | PK | 一意識別子 |
| 2 | `display_name` | 表示名 | VARCHAR | 64 | NO | − | − | − | − | − | UI 表示用 |
| 3 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | 状態意味 |
| 4 | `sort_order` | 並び順 | INT | 4 | NO | − | − | − | − | − | 表示順 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `comment_visibility_pkey` | PRIMARY KEY | `visibility_code` | − | 主キー |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `comment_visibility_pkey` | btree (PK) | `visibility_code` | − | 主キー |

---

## 5. Seed データ

| visibility_code | display_name | sort_order |
|---|---|---|
| `PUBLIC` | Public | 10 |
| `INTERNAL` | Internal | 20 |
| `PRIVATE` | Private | 30 |

---

## 6. 想定レコード件数

固定 3 行

---

## 7. 関連テーブル

### 8.1 依存先

なし

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `comment.comment` | `visibility` (CHECK 制約 + 業務参照) |

---

## 8. RLS Policy

```sql
ALTER TABLE comment.comment_visibility DISABLE ROW LEVEL SECURITY;
```

---

## 9. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
