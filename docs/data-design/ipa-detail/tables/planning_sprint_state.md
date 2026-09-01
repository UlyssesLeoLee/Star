# planning.sprint_state — テーブル詳細設計書

> **テーブル ID**: T22
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.7.1 + §3.3.2 派生

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T22 |
| **物理名** | `planning.sprint_state` |
| **論理名** | スプリント状態 Lookup Table |
| **スキーマ** | `planning` |
| **Module** | `domain-planning` |
| **種別** | Lookup Table（L） |
| **主キー** | `state_code VARCHAR(16)` |
| **RLS 必須** | **No**（DISABLE RLS） |
| **概要** | Sprint 状態 enum Lookup。3 状態（PLANNING / ACTIVE / CLOSED）。`is_terminal` フラグで終端状態管理。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `state_code` | 状態コード | VARCHAR | 16 | NO | − | ✓ | − | − | PK | 一意識別子 |
| 2 | `display_name` | 表示名 | VARCHAR | 64 | NO | − | − | − | − | − | UI 表示用 |
| 3 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | 状態意味 |
| 4 | `is_terminal` | 終端状態 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 終端か |
| 5 | `sort_order` | 並び順 | INT | 4 | NO | − | − | − | − | − | 表示順 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `sprint_state_pkey` | PRIMARY KEY | `state_code` | − | 主キー |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `sprint_state_pkey` | btree (PK) | `state_code` | − | 主キー |

---

## 5. トリガー一覧

なし

---

## 6. Seed データ

| state_code | display_name | is_terminal | sort_order |
|---|---|---|---|
| `PLANNING` | Planning | FALSE | 10 |
| `ACTIVE` | Active | FALSE | 20 |
| `CLOSED` | Closed | TRUE | 30 |

---

## 7. 想定レコード件数

固定 3 行（静的 Lookup）

---

## 8. 関連テーブル

### 8.1 依存先

なし

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `planning.sprint` | `state` (CHECK 制約 + 業務参照) |

---

## 9. RLS Policy

```sql
ALTER TABLE planning.sprint_state DISABLE ROW LEVEL SECURITY;
```

> 理由: 全局 enum、App 全体から参照可。

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
