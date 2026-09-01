# audit.audit_event_outbox — テーブル詳細設計書

> **テーブル ID**: T32
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.11.3 + §3.6

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T32 |
| **物理名** | `audit.audit_event_outbox` |
| **論理名** | 監査イベント Outbox |
| **スキーマ** | `audit` |
| **Module** | `domain-audit` |
| **種別** | **Outbox（O）**（§3.6 派生） |
| **主キー** | `outbox_id UUID` |
| **RLS 必須** | **Yes**（Tenant 隔離） |
| **パーティション** | なし |
| **soft delete** | **No**（物理削除戦略、§ON-304 派生） |
| **概要** | 統一 Outbox。Worker 轮询推送 NATS。指数退避。DLQ は `retry_count >= 5` で trigger。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `outbox_id` | Outbox ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | 主キー |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | − | − | idx | RLS 必須 |
| 3 | `aggregate_type` | 集約種別 | VARCHAR | 64 | NO | − | − | − | − | − | 集約エンティティ種別 |
| 4 | `aggregate_id` | 集約 ID | UUID | − | NO | − | − | (App 検証) | − | − | 集約 ID |
| 5 | `event_type` | イベント種別 | VARCHAR | 64 | NO | − | − | − | − | − | `'work_item.created'` 等 |
| 6 | `subject` | NATS Subject | VARCHAR | 255 | NO | − | − | − | − | − | NATS Subject 名 |
| 7 | `payload_json` | Payload | JSONB | − | NO | − | − | − | − | − | イベント本体 |
| 8 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | キュー追加時刻 |
| 9 | `published_at` | 配信日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | NATS 配信成功時刻 |
| 10 | `retry_count` | リトライ回数 | INT | 4 | NO | `0` | − | − | − | PT | 0..5（CHECK 制約） |
| 11 | `last_error` | 最終エラー | TEXT | − | YES | `NULL` | − | − | − | − | エラーメッセージ |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `audit_event_outbox_pkey` | PRIMARY KEY | `outbox_id` | − | 主キー |
| `ck_outbox_retry` | CHECK | `retry_count` | `retry_count >= 0 AND retry_count <= 5` | リトライ上限 |
| `ck_outbox_published` | CHECK | `published_at`/`retry_count` | `(published_at IS NULL AND retry_count >= 0) OR (published_at IS NOT NULL AND published_at >= created_at)` | 状態整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `audit_event_outbox_pkey` | btree (PK) | `outbox_id` | − | 主キー |
| `idx_outbox_unpublished` | btree (PT) | `(created_at)` | `WHERE published_at IS NULL` | 未配信キュー |
| `idx_outbox_retry_queue` | btree (PT) | `(retry_count, created_at)` | `WHERE published_at IS NULL` | リトライキュー |
| `idx_outbox_tenant` | btree | `(tenant_id, created_at DESC)` | − | テナント別 |

---

## 5. トリガー / Worker 戦略

| 種別 | 名前 / 戦略 | 説明 |
|---|---|---|
| Worker | `outbox-publisher` | 5 秒間隔 poll、`published_at IS NULL` 取得 → NATS publish |
| 戦略 | 指数退避 | `retry_count` 増加毎に 2^N 秒待機（1s → 2s → 4s → 8s → 16s → 32s） |
| DLQ | `retry_count >= 5` | DLQ Topic へ転送、App 側通知 |

---

## 6. 想定レコード件数

| フェーズ | アクティブ件数（未配信） |
|---|---|
| MVP | 10,000 |
| 1 年後 | 100,000 |
| 3 年後 | 1,000,000 |

> 配信済は `published_at` 後 30 日で物理削除（§ON-304 派生）

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1 KB | 1,000,000 | 約 1 GB |

---

## 8. 関連テーブル

### 8.1 依存先

なし（FK なし、独立 Outbox）

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE audit.audit_event_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit.audit_event_outbox FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_audit_event_outbox_tenant_isolation ON audit.audit_event_outbox
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
