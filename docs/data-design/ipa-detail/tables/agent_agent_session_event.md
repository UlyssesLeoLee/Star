# agent.agent_session_event — テーブル詳細設計書

> **テーブル ID**: T79
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.21.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T79 |
| **物理名** | `agent.agent_session_event` |
| **論理名** | セッションイベント（Append-only） |
| **スキーマ** | `agent` |
| **Module** | `domain-agent` |
| **種別** | **Append-only（A）**（§API-3.22.3） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **パーティション** | **RANGE (occurred_at) 月次** |
| **概要** | AgentSession 状態遷移イベント。月次パーティション。`triggered_by` 5 値（application / local_runtime / agent / worker / system）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | (FK なし、高頻) | idx | RLS 必須 |
| 3 | `agent_session_id` | エージェントセッション ID | UUID | − | NO | − | − | (App 検証) | idx | 親 Session |
| 4 | `from_status` | 遷移元状態 | VARCHAR | 32 | YES | `NULL` | − | − | − | 14 値 |
| 5 | `to_status` | 遷移先状態 | VARCHAR | 32 | NO | − | − | − | idx | 14 値 |
| 6 | `triggered_by` | トリガ元 | VARCHAR | 16 | NO | − | − | − | − | 5 値 |
| 7 | `triggered_by_id` | トリガ元 ID | UUID | YES | `NULL` | − | − | (App) | − | トリガ元 ID |
| 8 | `reason` | 理由 | TEXT | − | YES | `NULL` | − | − | − | 遷移理由 |
| 9 | `occurred_at` | 発生日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | BRIN | パーティションキー |

---

## 3. 制約一覧

| 制約名 | 種類 | 説明 |
|---|---|---|
| `agent_session_event_pkey` | PRIMARY KEY | 主キー |

> **FK なし**: 高頻書込（§4.21.3 注釈）App 層検証、RLS 強制。

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `agent_session_event_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_agent_session_event_tenant_session` | btree | `(tenant_id, agent_session_id, occurred_at DESC)` | − | テナント + Session + 順 |
| `idx_agent_session_event_to_status` | btree | `(tenant_id, to_status, occurred_at DESC)` | − | 遷移先別 |
| `idx_agent_session_event_occurred_brin` | BRIN | `occurred_at` | − | 大規模時系列 |

---

## 5. パーティション

| パーティション | 範囲 |
|---|---|
| `agent_session_event_2026_09` | 2026-09-01 〜 2026-10-01 |
| ... | 月次継続追加 |

---

## 6. 想定レコード件数

| フェーズ | 件数 / 月 |
|---|---|
| MVP | 1,000,000 |
| 1 年後 | 10,000,000 |
| 3 年後 | 100,000,000 |

---

## 7. 想定容量

| 1 行バイト | 月次件数 | 月次容量 |
|---|---|---|
| 約 500 B | 1,000,000 | 約 500 MB |

---

## 8. 関連テーブル

### 8.1 依存先

なし（FK なし、孤立許容、App 層で session_id 整合性検証）

### 8.2 被参照元

なし（末端 Append-only）

---

## 9. RLS Policy

```sql
ALTER TABLE agent.agent_session_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent.agent_session_event FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_agent_session_event_tenant_isolation ON agent.agent_session_event
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
-- 注: WITH CHECK なし（INSERT 専用）
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
