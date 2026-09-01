# permission.permission — テーブル詳細設計書

> **テーブル ID**: T50
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.16.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T50 |
| **物理名** | `permission.permission` |
| **論理名** | パーミッション（全局 enum） |
| **スキーマ** | `permission` |
| **Module** | `domain-permission` |
| **種別** | **Lookup Table（L）**（全局 enum、無 tenant_id） |
| **主キー** | `permission_key VARCHAR(128)` |
| **RLS 必須** | **No**（DISABLE RLS、平台級共有） |
| **概要** | 全局パーミッション enum。`permission_key` 形式 `{resource}:{action}`。`category` で 8 リソース分類。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `permission_key` | パーミッションキー | VARCHAR | 128 | NO | − | ✓ | − | − | PK | `'work_item:read'` 等 |
| 2 | `description` | 説明 | TEXT | − | NO | − | − | − | − | − | 動作説明 |
| 3 | `category` | カテゴリ | VARCHAR | 64 | NO | − | − | − | − | − | 8 リソース分類 |
| 4 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 5 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `permission_pkey` | PRIMARY KEY | `permission_key` | − | 主キー |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `permission_pkey` | btree (PK) | `permission_key` | − | 主キー |

---

## 5. トリガー一覧

なし

---

## 6. Seed データ（主要キー）

| permission_key | category | 説明 |
|---|---|---|
| `work_item:read` | work_item | WorkItem 閲覧 |
| `work_item:create` | work_item | WorkItem 作成 |
| `work_item:update` | work_item | WorkItem 更新 |
| `work_item:transition` | work_item | WorkItem 状態遷移 |
| `work_item:assign` | work_item | WorkItem 担当割当 |
| `worktree:read` | worktree | WorkTree 閲覧 |
| `worktree:create` | worktree | WorkTree 作成 |
| `worktree:assign` | worktree | WorkTree Agent 割当 |
| `agent_session:start` | agent | Agent Session 開始 |
| `agent_session:stop` | agent | Agent Session 停止 |
| `feedback:read` | feedback | Feedback 閲覧 |
| `feedback:resolve` | feedback | Feedback 解決 |
| `validation:run` | validation | 検証実行 |
| `validation:approve` | validation | 検証承認 |
| `scm:push` | scm | SCM push |
| `scm:merge` | scm | SCM merge |
| `runtime:command` | runtime | Local Runtime コマンド実行 |

> 8 categories: work_item / worktree / agent / feedback / context / validation / scm / runtime

---

## 7. 想定レコード件数

固定 50-200 行（静的 enum）

---

## 8. 関連テーブル

### 8.1 依存先

なし（全局 enum）

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `permission.role.permission_keys` (VARCHAR[]) | 文字列参照 |

---

## 9. RLS Policy

```sql
ALTER TABLE permission.permission DISABLE ROW LEVEL SECURITY;
```

> 理由: 平台級共有 enum、App 全体から参照可

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
