# worktree.worktree_status_observed — テーブル詳細設計書

> **テーブル ID**: T73
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.20.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T73 |
| **物理名** | `worktree.worktree_status_observed` |
| **論理名** | ワークツリー観測状態（Projection） |
| **スキーマ** | `worktree` |
| **Module** | `domain-worktree` |
| **種別** | **Projection（P）**（高頻 Observed State、§R-DATA-003） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **パーティション** | **RANGE (last_observed_at) 週次**（§4.20.2 派生） |
| **概要** | Worktree 観測状態。Local Runtime 心跳で 高頻度書込。`sequence_number BIGINT` で Monotonic 厳密増加。`display_state` 4 値（CURRENT / POSSIBLY_STALE / OFFLINE / UNKNOWN）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | (FK なし、高頻) | idx | RLS 必須 |
| 3 | `worktree_id` | ワークツリー ID | UUID | − | NO | − | − | (App 検証) | idx | 親 Worktree |
| 4 | `dirty` | Dirty | BOOLEAN | 1 | NO | `FALSE` | − | − | − | ダーティ状態 |
| 5 | `dirty_files` | ダーティファイル配列 | VARCHAR(2048)[] | − | NO | `'{}'` | − | − | GIN | ダーティファイル |
| 6 | `ahead` | ahead | INT | 4 | NO | `0` | − | − | − | ahead commits |
| 7 | `behind` | behind | INT | 4 | NO | `0` | − | − | − | behind commits |
| 8 | `current_agent_session_id` | 現セッション ID | UUID | YES | `NULL` | − | − | (App) | − | Agent Session 連動 |
| 9 | `current_pid` | 現 PID | INT | 4 | YES | `NULL` | − | − | − | Local Daemon PID |
| 10 | `last_heartbeat_at` | 最終ハートビート | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | idx | 心拍時刻 |
| 11 | `last_observed_at` | 最終観測 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | idx / BRIN | パーティションキー |
| 12 | `sequence_number` | シーケンス番号 | BIGINT | 8 | NO | − | − | − | − | Monotonic 厳密増加 |
| 13 | `runtime_id` | ランタイム ID | UUID | YES | `NULL` | − | − | (App) | idx | Runtime 連動 |
| 14 | `display_state` | 表示状態 | VARCHAR | 16 | NO | `'UNKNOWN'` | − | − | − | 4 値（§4.1.5 / §23.4） |

---

## 3. 制約一覧

| 制約名 | 種類 | 説明 |
|---|---|---|
| `worktree_status_observed_pkey` | PRIMARY KEY | 主キー |

> **FK なし**: 高頻書込（§4.20.2 注釈）App 層検証、RLS 強制。

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `worktree_status_observed_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_worktree_observed_tenant_worktree` | btree | `(tenant_id, worktree_id, last_observed_at DESC)` | − | テナント + Worktree + 観測順 |
| `idx_worktree_observed_runtime` | btree | `(runtime_id, last_heartbeat_at DESC)` | − | Runtime + ハートビート |
| `idx_worktree_observed_brin` | BRIN | `last_observed_at` | − | 大規模時系列 |
| `idx_worktree_observed_dirty_files_gin` | GIN | `dirty_files` | − | ダーティファイル検索 |

---

## 5. パーティション

| パーティション | 範囲 |
|---|---|
| `worktree_status_observed_2026_w36` | 2026-08-31 〜 2026-09-07 |
| `worktree_status_observed_2026_w37` | 2026-09-07 〜 2026-09-14 |
| ... | 週次継続追加 |

---

## 6. 想定レコード件数

| フェーズ | 件数 / 週 |
|---|---|
| MVP | 1,000,000 |
| 1 年後 | 10,000,000 |
| 3 年後 | 100,000,000 |

---

## 7. 想定容量

| 1 行バイト | 週次件数 | 週次容量 |
|---|---|---|
| 約 800 B | 1,000,000 | 約 800 MB |

---

## 8. 関連テーブル

### 8.1 依存先

なし（FK なし、孤立許容、App 層で worktree_id / runtime_id 整合性検証）

### 8.2 被参照元

なし（末端、Read 側のみ）

---

## 9. RLS Policy

```sql
ALTER TABLE worktree.worktree_status_observed ENABLE ROW LEVEL SECURITY;
ALTER TABLE worktree.worktree_status_observed FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_worktree_status_observed_tenant_isolation ON worktree.worktree_status_observed
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
-- 注: WITH CHECK なし（INSERT 専用）
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
