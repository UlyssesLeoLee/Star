# feedback.feedback_consumed_event — テーブル詳細設計書

> **テーブル ID**: T83
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.22.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T83 |
| **物理名** | `feedback.feedback_consumed_event` |
| **論理名** | フィードバック消費イベント（Append-only） |
| **スキーマ** | `feedback` |
| **Module** | `domain-feedback` |
| **種別** | **Append-only（A）**（§R-FBK-002） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Feedback 消費追跡。3 消費元（`agent_session` / `context_packet` / `change_set`）× 3 行動（`acknowledged` / `applied` / `verified`）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | idx | RLS 必須 |
| 3 | `feedback_id` | フィードバック ID | UUID | − | NO | − | − | `feedback.feedback(id)` ON DELETE CASCADE | idx | 親 Feedback |
| 4 | `consumed_by_type` | 消費元種別 | VARCHAR | 16 | NO | − | − | − | idx | 3 値 |
| 5 | `consumed_by_id` | 消費元 ID | UUID | − | NO | − | − | (App 検証) | idx | 消費元 ID |
| 6 | `consumption_type` | 消費種別 | VARCHAR | 32 | NO | − | − | − | − | 3 値 |
| 7 | `consumed_at` | 消費日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | 消費時刻 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `feedback_consumed_event_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_feedback_consumed_event_*` | FOREIGN KEY (2) | `tenant_id` / `feedback_id` | 各親テーブル | CASCADE | − |
| `ck_consumed_by_type` | CHECK | `consumed_by_type` | `IN ('agent_session','context_packet','change_set')` | 3 値 |
| `ck_consumption_type` | CHECK | `consumption_type` | `IN ('acknowledged','applied','verified')` | 3 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 説明 |
|---|---|---|---|
| `feedback_consumed_event_pkey` | btree (PK) | `id` | 主キー |
| `idx_feedback_consumed_tenant_feedback` | btree | `(tenant_id, feedback_id)` | テナント + Feedback |
| `idx_feedback_consumed_tenant_by` | btree | `(tenant_id, consumed_by_type, consumed_by_id)` | テナント + 消費元 |

---

## 5. トリガー一覧

なし

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
| 約 250 B | 100,000,000 | 約 25 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `feedback.feedback` | `feedback_id` |
| `agent.agent_session` (App) | `consumed_by_id` (`consumed_by_type='agent_session'`) |
| `context.context_packet` (App) | `consumed_by_id` (`consumed_by_type='context_packet'`) |
| `development.change_set` (App) | `consumed_by_id` (`consumed_by_type='change_set'`) |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE feedback.feedback_consumed_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE feedback.feedback_consumed_event FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_feedback_consumed_event_tenant_isolation ON feedback.feedback_consumed_event
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
