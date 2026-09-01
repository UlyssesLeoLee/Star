# development.repository_context — テーブル詳細設計書

> **テーブル ID**: T70
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.19.8

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T70 |
| **物理名** | `development.repository_context` |
| **論理名** | リポジトリコンテキスト（Projection） |
| **スキーマ** | `development` |
| **Module** | `domain-development` |
| **種別** | **Projection（P）**（派生） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Repository コンテキスト Projection。`file_count` + `language_breakdown JSONB`（`{'rust': 120, 'typescript': 50}`）。`last_indexed_at` で鮮度管理。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `repository_id` | リポジトリ ID | UUID | − | NO | − | − | `scm.repository(id)` ON DELETE CASCADE | − | idx | 親 Repo |
| 4 | `file_count` | ファイル数 | INT | 4 | NO | `0` | − | − | − | − | 業務統計 |
| 5 | `language_breakdown` | 言語分布 | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 言語別ファイル数 |
| 6 | `last_indexed_at` | 最終索引日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 鮮度管理 |
| 7 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 8 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 10 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `repository_context_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_repository_context_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_repository_context_repository` | FOREIGN KEY | `repository_id` | `scm.repository(id)` | CASCADE | 親 Repo 削除時 削除 |
| `uq_repo_context_per_repo` | UNIQUE | `(repository_id, deleted_at)` | − | − | Repo 1:1 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `repository_context_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_repo_context_per_repo` | btree (UK/PT) | `(repository_id, deleted_at)` | − | Repo 1:1 |
| `idx_repository_context_tenant_repo` | btree (PT) | `(tenant_id, repository_id)` | `deleted_at IS NULL` | テナント + Repo |

---

## 5. トリガー / Worker

| 種別 | 名前 / 戦略 | 説明 |
|---|---|---|
| Worker | `repository-analysis` | ファイル walk → `file_count` / `language_breakdown` 集計 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 5,000 |（1 Repo 1 上下文） |
| 1 年後 | 50,000 |
| 3 年後 | 500,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 800 B | 500,000 | 約 400 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `scm.repository` | `repository_id` |

### 8.2 被参照元

なし（Read-Only Projection）

---

## 9. RLS Policy

```sql
ALTER TABLE development.repository_context ENABLE ROW LEVEL SECURITY;
ALTER TABLE development.repository_context FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_repository_context_tenant_isolation ON development.repository_context
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
-- 注: WITH CHECK なし（Worker INSERT 専用）
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
