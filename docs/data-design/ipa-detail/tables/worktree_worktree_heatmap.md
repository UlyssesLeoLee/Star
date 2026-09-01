# worktree.worktree_heatmap — テーブル詳細設計書

> **テーブル ID**: T75
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.20.4

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T75 |
| **物理名** | `worktree.worktree_heatmap` |
| **論理名** | ワークツリーヒートマップ（物化ビュー） |
| **スキーマ** | `worktree` |
| **Module** | `domain-worktree` |
| **種別** | **Materialized View（MV）** |
| **主キー** | `(tenant_id, repository_id, file_path)` |
| **RLS 必須** | **No**（基表 RLS 透過） |
| **概要** | Worktree ヒートマップ物化ビュー。`worktree.changed_files` を展開し `file_path` 別 Worktree 数を集計。REFRESH CONCURRENTLY 対応（UNIQUE INDEX 必須）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | 説明 |
|---|---|---|---|---|---|---|---|
| 1 | `tenant_id` | テナント ID | UUID | − | NO | (派生) | 基表から |
| 2 | `repository_id` | リポジトリ ID | UUID | − | NO | (派生) | 基表から |
| 3 | `file_path` | ファイルパス | VARCHAR | 2048 | NO | (派生) | `unnest(changed_files)` |
| 4 | `worktree_count` | ワークツリー数 | BIGINT | 8 | NO | (派生) | COUNT(DISTINCT wt.id) |
| 5 | `worktree_ids` | ワークツリー ID 配列 | UUID[] | − | NO | (派生) | array_agg(DISTINCT wt.id) |

---

## 3. VIEW 定義

```sql
CREATE MATERIALIZED VIEW worktree.worktree_heatmap AS
SELECT
  wt.tenant_id,
  wt.repository_id,
  file_path,
  COUNT(DISTINCT wt.id) AS worktree_count,
  array_agg(DISTINCT wt.id) AS worktree_ids
FROM worktree.worktree wt,
     unnest(wt.changed_files) AS file_path
WHERE wt.deleted_at IS NULL
  AND wt.status NOT IN ('ABANDONED','ARCHIVED','MERGED')
GROUP BY wt.tenant_id, wt.repository_id, file_path;
```

---

## 4. インデックス

| インデックス名 | 種別 | キー列 | 説明 |
|---|---|---|---|
| `idx_worktree_heatmap_pk` | btree (UNIQUE) | `(tenant_id, repository_id, file_path)` | REFRESH CONCURRENTLY 対応 |
| `idx_worktree_heatmap_repo_count` | btree | `(tenant_id, repository_id, worktree_count DESC)` | ランキング検索 |

---

## 5. Refresh 戦略

| 種別 | 戦略 | 説明 |
|---|---|---|
| Refresh | `REFRESH MATERIALIZED VIEW CONCURRENTLY worktree.worktree_heatmap` | UNIQUE INDEX あるため CONCURRENTLY 可 |
| 頻度 | 5 分間隔 Worker | `proj-runner projection heatmap` |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 1,000,000 |
| 1 年後 | 10,000,000 |
| 3 年後 | 100,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 500 B | 100,000,000 | 約 50 GB |

---

## 8. 関連テーブル

### 8.1 依存先（基表）

| 参照先 | 関係 |
|---|---|
| `worktree.worktree` | 基表（changed_files unnest + GROUP BY） |

### 8.2 被参照元

なし（末端 Read 側のみ）

---

## 9. RLS Policy

```sql
ALTER MATERIALIZED VIEW worktree.worktree_heatmap OWNER TO star_app_role;
-- RLS: Base Table RLS 継承（物化ビュー読基表時受 RLS 制約）
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
