# local_runtime.runtime_observation — テーブル詳細設計書

> **テーブル ID**: T98
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.25.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T98 |
| **物理名** | `local_runtime.runtime_observation` |
| **論理名** | ランタイム観測（Append-only） |
| **スキーマ** | `local_runtime` |
| **Module** | `domain-local-runtime` |
| **種別** | **Append-only（A）**（§4.6.5、30 日冷归档） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **パーティション** | **RANGE (observed_at) 月次** |
| **概要** | Local Daemon 上報イベント。7 種別（WorktreeStatusObserved / AgentSessionStateObserved / BuildCompleted / TestCompleted / DiffAvailable / Heartbeat / Disconnected）。`sequence_number` Monotonic 厳密増加。`idempotency_key` で幂等。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | (FK なし、高頻) | idx | RLS 必須 |
| 3 | `runtime_id` | ランタイム ID | UUID | − | NO | − | − | (App 検証) | idx | 親 Runtime |
| 4 | `observation_type` | 観測種別 | VARCHAR | 32 | NO | − | − | − | idx | 7 値 |
| 5 | `payload` | ペイロード | JSONB | − | NO | − | − | − | − | 観測データ |
| 6 | `sequence_number` | シーケンス番号 | BIGINT | 8 | NO | − | − | − | idx | Monotonic 厳密増加 |
| 7 | `observed_at` | 観測日時 | TIMESTAMPTZ | 8 | NO | − | − | − | BRIN | パーティションキー |
| 8 | `received_at` | 受信日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | サーバー受信時刻 |
| 9 | `idempotency_key` | 幂等キー | VARCHAR | 256 | YES | `NULL` | − | − | ✓ | `uq_runtime_observation_idempotency` | 幂等制御 |

---

## 3. 制約一覧

| 制約名 | 種類 | 説明 |
|---|---|---|
| `runtime_observation_pkey` | PRIMARY KEY | 主キー |
| `uq_runtime_observation_idempotency` | UNIQUE (PT) `(runtime_id, idempotency_key)` WHERE `idempotency_key IS NOT NULL` | 幂等 |

> **FK なし**: 高頻書込（§4.25.3 注釈）App 層検証、RLS 強制。

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `runtime_observation_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_runtime_observation_idempotency` | btree (UK/PT) | `(runtime_id, idempotency_key)` | `idempotency_key IS NOT NULL` | 幂等 |
| `idx_runtime_observation_tenant_runtime` | btree | `(tenant_id, runtime_id, sequence_number DESC)` | − | テナント + Runtime + 順 |
| `idx_runtime_observation_type` | btree | `(tenant_id, observation_type, observed_at DESC)` | − | 種別別 |
| `idx_runtime_observation_observed_brin` | BRIN | `observed_at` | − | 大規模時系列 |

---

## 5. パーティション

| パーティション | 範囲 |
|---|---|
| `runtime_observation_2026_09` | 2026-09-01 〜 2026-10-01 |
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
| 約 1 KB | 1,000,000 | 約 1 GB |

---

## 8. 関連テーブル

### 8.1 依存先

なし（FK なし、孤立許容、App 層で runtime_id 整合性検証）

### 8.2 被参照元

なし（末端 Append-only）

---

## 9. RLS Policy

```sql
ALTER TABLE local_runtime.runtime_observation ENABLE ROW LEVEL SECURITY;
ALTER TABLE local_runtime.runtime_observation FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_runtime_observation_tenant_isolation ON local_runtime.runtime_observation
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
-- 注: WITH CHECK なし（INSERT 専用）
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
