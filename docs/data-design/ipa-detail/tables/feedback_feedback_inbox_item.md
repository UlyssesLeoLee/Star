# feedback.feedback_inbox_item — テーブル詳細設計書

> **テーブル ID**: T84
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.22.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T84 |
| **物理名** | `feedback.feedback_inbox_item` |
| **論理名** | フィードバック Inbox（物化ビュー） |
| **スキーマ** | `feedback` |
| **Module** | `domain-feedback` |
| **種別** | **Materialized View（MV）** |
| **主キー** | `feedback_id` |
| **RLS 必須** | **No**（基表 RLS 透過） |
| **概要** | Feedback Inbox Projection。P0/P1 優先ソート。`priority_sort` 列で severity → P0=0/P1=1/P2=2/P3=3。Worktree 自動紐付け。5 分間隔 refresh。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | 説明 |
|---|---|---|---|---|---|---|---|
| 1 | `feedback_id` | フィードバック ID | UUID | − | NO | (派生) | 主キー（基表から） |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | (派生) | 基表から |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | (派生) | 基表から |
| 4 | `target_type` | Target 種別 | VARCHAR | 32 | NO | (派生) | 14 値 |
| 5 | `target_id` | Target ID | UUID | − | YES | (派生) | 主リソース ID |
| 6 | `type` | 種別 | VARCHAR | 32 | NO | (派生) | 11 値 |
| 7 | `severity` | 重大度 | VARCHAR | 8 | NO | (派生) | 4 値 |
| 8 | `status` | 状態 | VARCHAR | 32 | NO | (派生) | OPEN のみ |
| 9 | `intent` | 意図 | TEXT | − | NO | (派生) | 短文要求 |
| 10 | `author_user_id` | 著者ユーザ ID | UUID | − | YES | (派生) | 人間著者 |
| 11 | `priority_sort` | 優先度ソート | INT | 4 | NO | (派生) | P0=0, P1=1, P2=2, P3=3 |
| 12 | `worktree_id` | ワークツリー ID | UUID | − | YES | (派生) | 自動紐付け |
| 13 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | (派生) | Inbox ソート |
| 14 | `sla_due_at` | SLA 期限 | TIMESTAMPTZ | 8 | YES | (派生) | Project Policy 計算 |

---

## 3. VIEW 定義

```sql
CREATE MATERIALIZED VIEW feedback.feedback_inbox_item AS
SELECT
  f.id AS feedback_id,
  f.tenant_id,
  f.project_id,
  f.target_type,
  f.target_id,
  f.type,
  f.severity,
  f.status,
  f.intent,
  f.author_user_id,
  CASE f.severity
    WHEN 'P0' THEN 0
    WHEN 'P1' THEN 1
    WHEN 'P2' THEN 2
    WHEN 'P3' THEN 3
  END AS priority_sort,
  (SELECT id FROM worktree.worktree w
   WHERE w.tenant_id = f.tenant_id
     AND (
       (f.target_type = 'Worktree' AND w.id = f.target_id) OR
       (f.target_type = 'WorkItem' AND w.work_item_id = f.target_id) OR
       (f.target_type = 'AgentSession' AND w.current_agent_session_id = f.target_id) OR
       (f.target_type = 'File' AND f.target_id IS NULL AND w.id = ANY(
         SELECT worktree_id FROM development.file_change fc
         WHERE fc.tenant_id = f.tenant_id AND fc.path = f.target_file_path
       ))
     )
   LIMIT 1
  ) AS worktree_id,
  f.created_at,
  NULL::TIMESTAMPTZ AS sla_due_at
FROM feedback.feedback f
WHERE f.deleted_at IS NULL
  AND f.status = 'OPEN';
```

---

## 4. インデックス

| インデックス名 | 種別 | キー列 | 説明 |
|---|---|---|---|
| `idx_feedback_inbox_pk` | btree (UNIQUE) | `feedback_id` | REFRESH CONCURRENTLY 対応 |
| `idx_feedback_inbox_priority` | btree | `(tenant_id, project_id, priority_sort, created_at)` | 優先度ソート |
| `idx_feedback_inbox_worktree` | btree | `(tenant_id, worktree_id)` | Worktree 別 |

---

## 5. Refresh 戦略

| 種別 | 戦略 | 説明 |
|---|---|---|
| Refresh | `REFRESH MATERIALIZED VIEW CONCURRENTLY feedback.feedback_inbox_item` | 5 分間隔 Worker |
| 頻度 | 5 分 | Inbox 鮮度 |

---

## 6. 想定レコード件数

| フェーズ | OPEN 件数 |
|---|---|
| MVP | 50,000 |
| 1 年後 | 500,000 |
| 3 年後 | 5,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 500 B | 5,000,000 | 約 2.5 GB |

---

## 8. 関連テーブル

### 8.1 依存先（基表）

| 参照先 | 関係 |
|---|---|
| `feedback.feedback` | 基表 |

### 8.2 被参照元

なし（末端、UI 読取のみ）

---

## 9. RLS Policy

```sql
ALTER MATERIALIZED VIEW feedback.feedback_inbox_item OWNER TO star_app_role;
-- RLS: Base Table RLS 継承
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
