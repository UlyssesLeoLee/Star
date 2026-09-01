# work_item.work_item_status — テーブル詳細設計書

> **テーブル ID**: T12
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.4.5
> **本ファイル役割**: IPA データモデル詳細設計書 — テーブル詳細

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T12 |
| **物理名** | `work_item.work_item_status` |
| **論理名** | WorkItem 状態 Lookup Table |
| **スキーマ** | `work_item` |
| **Module** | `domain-work-item` |
| **種別** | Lookup Table（L） |
| **主キー** | `status_code VARCHAR(32)` |
| **RLS 必須** | **No**（全局 enum、DISABLE RLS） |
| **概要** | WorkItem 状態枚举 Lookup。MVP 既定 3 状態（`is_default = TRUE`）+ 7 拡張状態（`is_default = FALSE`）。`is_default` フラグで §R-WF-001 強约束（既定 3 状態）遵守を担保。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `status_code` | 状態コード | VARCHAR | 32 | NO | − | ✓ | − | − | PK | 一意識別子（`'TODO'` 等） |
| 2 | `display_name` | 表示名 | VARCHAR | 64 | NO | − | − | − | − | − | UI 表示用 |
| 3 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | 状態意味の説明 |
| 4 | `is_terminal` | 終端状態 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 終端状態（遷移不可）か |
| 5 | `is_default` | 既定フラグ | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | TRUE = MVP 既定（§R-WF-001 強约束） |
| 6 | `sort_order` | 並び順 | INT | 4 | NO | − | − | − | − | − | 表示順 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `work_item_status_pkey` | PRIMARY KEY | `status_code` | − | 主キー |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `work_item_status_pkey` | btree (PK) | `status_code` | − | 主キー |

---

## 5. トリガー一覧

なし（静的 Lookup Table）

---

## 6. Seed データ

| status_code | display_name | description | is_terminal | is_default | sort_order |
|---|---|---|---|---|---|
| `TODO` | To Do | 待办 | FALSE | **TRUE** | 10 |
| `IN_PROGRESS` | In Progress | 进行中 | FALSE | **TRUE** | 20 |
| `DONE` | Done | 完成 | TRUE | **TRUE** | 30 |
| `IN_REVIEW` | In Review | 审查中 | FALSE | FALSE | 40 |
| `BLOCKED` | Blocked | 阻塞 | FALSE | FALSE | 50 |
| `CANCELLED` | Cancelled | 已取消 | TRUE | FALSE | 60 |
| `IN_TESTING` | In Testing | 测试中 | FALSE | FALSE | 70 |
| `READY_FOR_DEPLOY` | Ready For Deploy | 待部署 | FALSE | FALSE | 80 |
| `NEEDS_INFO` | Needs Info | 需补充 | FALSE | FALSE | 90 |

> **§R-WF-001 强约束**: MVP 必須支持 `TODO` / `IN_PROGRESS` / `DONE`（`is_default = TRUE`）。他の 6 状態は Project Policy で任意拡張。

---

## 7. 想定レコード件数

固定 9 行（静的 Lookup）

---

## 8. 関連テーブル

### 8.1 依存先

なし

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `work_item.work_item` | `status` (CHECK 制約 + 業務参照) |

---

## 9. RLS Policy

```sql
ALTER TABLE work_item.work_item_status DISABLE ROW LEVEL SECURITY;
```

> 理由: 全局 enum、App 全体から参照可、Project Policy 拡張時の Seed データ更新も Platform Admin のみが行う。

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
