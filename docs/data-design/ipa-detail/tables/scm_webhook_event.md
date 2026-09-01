# scm.webhook_event — テーブル詳細設計書

> **テーブル ID**: T61
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.18.7

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T61 |
| **物理名** | `scm.webhook_event` |
| **論理名** | Webhook イベント（入站） |
| **スキーマ** | `scm` |
| **Module** | `domain-scm` |
| **種別** | **Append-only（A）**（短 TTL 物理削除、30 日） |
| **主キー** | `id UUID` |
| **RLS 必須** | **No**（**DISABLE RLS**、WebHook 入站時 Tenant 未知） |
| **soft delete** | **No**（物理削除、30 日後） |
| **概要** | SCM Webhook 入站。`tenant_id` 解析前は NULL 許容。`signature` 検証、`idempotency_key` 幂等制御、`is_processed` 処理状態。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | YES | `NULL` | − | (解析後設定) | − | idx | 解析前は NULL |
| 3 | `provider` | Provider | VARCHAR | 32 | NO | − | − | − | − | idx | `'github'` 等 |
| 4 | `event_type` | イベント種別 | VARCHAR | 64 | NO | − | − | − | − | idx | `'push'` / `'pull_request'` 等 |
| 5 | `payload` | ペイロード | JSONB | − | NO | − | − | − | − | − | 原始 payload |
| 6 | `signature` | 署名 | VARCHAR | 512 | YES | `NULL` | − | − | − | − | Webhook 署名 |
| 7 | `signature_verified` | 署名検証済 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 検証結果 |
| 8 | `received_at` | 受信日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | 入站時刻 |
| 9 | `processed_at` | 処理日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 処理完了 |
| 10 | `processing_error` | 処理エラー | TEXT | − | YES | `NULL` | − | − | − | − | エラーメッセージ |
| 11 | `retry_count` | リトライ回数 | INT | 4 | NO | `0` | − | − | − | − | − |
| 12 | `idempotency_key` | 幂等キー | VARCHAR | 256 | YES | `NULL` | − | − | ✓ | `uq_webhook_event_idempotency` | 幂等制御 |
| 13 | `is_processed` | 処理済 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | PT | 処理状態 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `webhook_event_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `uq_webhook_event_idempotency` | UNIQUE (PT) | `(provider, idempotency_key)` | `WHERE idempotency_key IS NOT NULL` | 幂等制御 |

> **FK なし**: `tenant_id` は WebHook 解析後に App 層で設定。`integration_id` も同様（解析後）。

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `webhook_event_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_webhook_event_idempotency` | btree (UK/PT) | `(provider, idempotency_key)` | `idempotency_key IS NOT NULL` | 幂等制御 |
| `idx_webhook_event_provider_received` | btree | `(provider, received_at DESC)` | − | Provider + 受信順 |
| `idx_webhook_event_unprocessed` | btree (PT) | `(received_at)` | `is_processed = FALSE` | 未処理キュー |
| `idx_webhook_event_tenant` | btree | `(tenant_id, received_at DESC)` | − | 解析後 テナント別 |

---

## 5. トリガー / 物理削除戦略

| 種別 | 戦略 | 説明 |
|---|---|---|
| TRIGGER | (なし) | App 側で処理 |
| 物理削除 | 30 日後 | `is_processed = TRUE` のレコード対象、App cron job で削除 |

---

## 6. 想定レコード件数

| フェーズ | 件数 / 日 |
|---|---|
| MVP | 100,000 |
| 1 年後 | 1,000,000 |
| 3 年後 | 10,000,000 |

---

## 7. 想定容量

| 1 行バイト | 日次件数 | 30 日保持 |
|---|---|---|
| 約 2 KB (JSONB) | 1,000,000 | 約 60 GB |

---

## 8. 関連テーブル

### 8.1 依存先

なし（FK なし、孤立許容、解析後 App 層で tenant_id 設定）

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE scm.webhook_event DISABLE ROW LEVEL SECURITY;
COMMENT ON TABLE scm.webhook_event IS 'RLS 無効化: Webhook 入站時 Tenant 未知; 解析後 App 層 + credential_id で Tenant 検証';
```

> **注**: §4.18.7 で `DISABLE ROW LEVEL SECURITY` が選択された理由：WebHook は Tenant 未知で受信、解析後 App 層で Tenant 検証。

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
