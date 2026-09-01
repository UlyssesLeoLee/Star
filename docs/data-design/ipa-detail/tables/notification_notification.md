# notification.notification — テーブル詳細設計書

> **テーブル ID**: T47
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.15.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T47 |
| **物理名** | `notification.notification` |
| **論理名** | 通知（送信済） |
| **スキーマ** | `notification` |
| **Module** | `domain-notification` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **パーティション** | **RANGE (created_at) 月次**（§4.15.3 派生） |
| **概要** | 送信済通知。状態 PENDING / SENT / FAILED / READ。**S2 落点**: `requires_human_decision` + `audience_scope` で人間决策节点のみ触达（REQ-NOTIF-002）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `recipient_user_id` | 受信者ユーザ ID | UUID | − | NO | − | − | `identity.user(id)` (App) | − | idx | 通知受信者 |
| 4 | `event_type` | イベント種別 | VARCHAR | 64 | NO | − | − | − | − | − | − |
| 5 | `channel_id` | チャネル ID | UUID | − | YES | `NULL` | − | `notification.notification_channel(id)` ON DELETE SET NULL | − | − | 配信チャネル |
| 6 | `subject` | 件名 | TEXT | − | NO | − | − | − | − | − | 配信済件名 |
| 7 | `body` | 本文 | TEXT | − | NO | − | − | − | − | − | 配信済本文 |
| 8 | `payload` | ペイロード | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | メタデータ |
| 9 | `status` | 状態 | VARCHAR | 16 | NO | `'PENDING'` | − | − | − | idx | 4 状態 |
| 10 | `sent_at` | 送信日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 配信成功時刻 |
| 11 | `read_at` | 既読日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | ユーザ既読 |
| 12 | `retry_count` | リトライ回数 | INT | 4 | NO | `0` | − | − | − | − | 配信リトライ |
| 13 | `last_error` | 最終エラー | TEXT | − | YES | `NULL` | − | − | − | − | 配信失敗エラー |
| 14 | `requires_human_decision` | 人間决策必須 | BOOLEAN | 1 | NO | `TRUE` | − | − | − | idx (PT) | REQ-NOTIF-002 S2 落点 |
| 15 | `audience_scope` | 対象範囲 | VARCHAR | 16 | NO | `'human'` | − | − | − | idx (PT) | `'human'` / `'agent'` / `'system'` |
| 16 | `suppression_reason` | 抑制理由 | TEXT | − | YES | `NULL` | − | − | − | − | `'agent_mid_step'` / `'rate_limited'` 等 |
| 17 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | パーティションキー |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `notification_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_notification_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `fk_notification_channel` | FOREIGN KEY | `channel_id` | `notification.notification_channel(id)` ON DELETE SET NULL | チャネル削除時 NULL |
| `ck_notification_status` | CHECK | `status` | `IN ('PENDING','SENT','FAILED','READ')` | 4 状態 |
| `ck_notification_audience_scope` | CHECK | `audience_scope` | `IN ('human','agent','system')` | 3 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `notification_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_notification_tenant_user_status` | btree | `(tenant_id, recipient_user_id, status)` | − | 受信者 + 状態 |
| `idx_notification_tenant_created` | btree | `(tenant_id, created_at DESC)` | − | 作成順 |
| `idx_notification_tenant_user_human` | btree (PT) | `(tenant_id, recipient_user_id, created_at DESC)` | `requires_human_decision = TRUE AND audience_scope = 'human' AND status = 'PENDING'` | 人間决策 Inbox 最適化 |

---

## 5. トリガー / パーティション

| 種別 | 名前 / 戦略 | 説明 |
|---|---|---|
| PARTITION | `notification_2026_09` 等月次 | `FOR VALUES FROM ('YYYY-MM-01') TO ('YYYY-MM+1-01')` |
| Worker | `notification-dispatcher` | PENDING → SENT / FAILED リトライ（指数退避） |

---

## 6. 想定レコード件数

| フェーズ | 件数 / 月 |
|---|---|
| MVP | 100,000 |
| 1 年後 | 1,000,000 |
| 3 年後 | 10,000,000 |

---

## 7. 想定容量

| 1 行バイト | 月次件数 | 月次容量 |
|---|---|---|
| 約 1.5 KB | 1,000,000 | 約 1.5 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `notification.notification_channel` | `channel_id` |
| `identity.user` | `recipient_user_id` (App 検証) |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE notification.notification ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.notification FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_notification_tenant_isolation ON notification.notification
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
-- 注: WITH CHECK なし（INSERT 専用）
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
