# audit.audit_event — テーブル詳細設計書

> **テーブル ID**: T30
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.11.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T30 |
| **物理名** | `audit.audit_event` |
| **論理名** | 監査イベント（Append-only / WORM） |
| **スキーマ** | `audit` |
| **Module** | `domain-audit` |
| **種別** | **Append-only（A）**（物理削除禁止、UPDATE 禁止） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes**（Tenant 削除後記録は保持、`tenant_id` FK なし） |
| **パーティション** | **RANGE (occurred_at) 月次**（§9 / ON-501 派生） |
| **soft delete** | **No**（WORM、削除禁止） |
| **概要** | 監査ログ。月次パーティション、7 年保持（企業級）、WORM（Write-Once-Read-Many）。`actor_type` 3 値（user / agent / system）。`context_refs` で Provenance 連動。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | (FK なし) | − | idx | Tenant 削除後も保持 |
| 3 | `actor_type` | 行為者種別 | VARCHAR | 16 | NO | − | − | − | − | idx | `'user'` / `'agent'` / `'system'` |
| 4 | `actor_id` | 行為者 ID | UUID | − | YES | `NULL` | − | (App 検証) | − | idx | user_id / agent_session_id |
| 5 | `action` | アクション | VARCHAR | 64 | NO | − | − | − | − | idx | `'work_item:create'` 等 |
| 6 | `resource_type` | リソース種別 | VARCHAR | 64 | NO | − | − | − | − | idx | `'work_item'` 等 |
| 7 | `resource_id` | リソース ID | UUID | − | NO | − | − | (App 検証) | − | idx | − |
| 8 | `before_state` | 変更前状態 | JSONB | − | YES | `NULL` | − | − | − | − | 変更前スナップショット |
| 9 | `after_state` | 変更後状態 | JSONB | − | YES | `NULL` | − | − | − | − | 変更後スナップショット |
| 10 | `context_refs` | コンテキスト参照 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | GIN | Provenance 連動 |
| 11 | `request_id` | リクエスト ID | UUID | − | YES | `NULL` | − | − | − | − | X-Request-Id 連動 |
| 12 | `trace_id` | トレース ID | UUID | − | YES | `NULL` | − | − | − | − | W3C Trace ID |
| 13 | `client_ip` | クライアント IP | INET | − | YES | `NULL` | − | − | − | − | クライアント IP アドレス |
| 14 | `user_agent` | User-Agent | TEXT | − | YES | `NULL` | − | − | − | − | HTTP UA |
| 15 | `occurred_at` | 発生日時（パーティションキー） | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | BRIN | パーティションキー |
| 16 | `is_archived` | アーカイブ済 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | WORM 後アーカイブ |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `audit_event_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `ck_audit_actor_type` | CHECK | `actor_type` | `IN ('user','agent','system')` | 3 種別 |
| `audit_event_2026_09` 等 | PARTITION | `occurred_at` | `FOR VALUES FROM ('2026-09-01') TO ('2026-10-01')` | 月次パーティション |
| `ck_audit_action_v0_2` (per 2026-09-02 追加) | DOMAIN (App) | `action` | `action ~ '^onboarding\.test_key\.(failed\|started\|succeeded)$'` | onboarding event 3 値許容, actor_type='system' 必須 (per ADR-0043 §2.2) |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `audit_event_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_audit_tenant_occurred` | btree | `(tenant_id, occurred_at DESC)` | − | テナント + 時系列 |
| `idx_audit_tenant_actor_action` | btree | `(tenant_id, actor_type, actor_id, action, occurred_at DESC)` | − | 行為者 + アクション |
| `idx_audit_tenant_resource` | btree | `(tenant_id, resource_type, resource_id, occurred_at DESC)` | − | リソース別 |
| `idx_audit_occurred_brin` | BRIN | `occurred_at` | − | 大規模時系列 |
| `idx_audit_context_refs_gin` | GIN | `context_refs` | − | Provenance JSONB |

---

## 5. トリガー / 権限

| 種別 | 名前 / 戦略 | 説明 |
|---|---|---|
| REVOKE | `UPDATE, DELETE ON audit.audit_event FROM PUBLIC` | WORM 強制 |
| REVOKE | `UPDATE, DELETE ON audit.audit_event FROM star_app_role` | App 層も禁止 |

> ON-303 派生: TRIGGER `trg_audit_event_immutable` で UPDATE / DELETE 完全拒否も将来検討

---

## 6. パーティション

| パーティション | 範囲 |
|---|---|
| `audit_event_2026_09` | 2026-09-01 〜 2026-10-01 |
| `audit_event_2026_10` | 2026-10-01 〜 2026-11-01 |
| `audit_event_2026_11` | 2026-11-01 〜 2026-12-01 |
| ... | 継続追加（pg_partman 推奨） |

---

## 7. 想定レコード件数

| フェーズ | 件数 / 月 | 累積 1 年 |
|---|---|---|
| MVP | 1,000,000 | 12,000,000 |
| 1 年後 | 10,000,000 | 120,000,000 |
| 3 年後 | 100,000,000 | 1,200,000,000 |

---

## 8. 想定容量

| 1 行バイト | 月次件数 | 月次容量 |
|---|---|---|
| 約 1.5 KB | 1,000,000 | 約 1.5 GB |

---

## 9. 関連テーブル

### 9.1 依存先

なし（FK なし、Append-only、孤立許容）

### 9.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `audit.ai_audit_metadata` | `audit_event_id` ON DELETE RESTRICT |
| `audit.audit_event_outbox` | `aggregate_id` (動的) |

---

## 10. RLS Policy

```sql
ALTER TABLE audit.audit_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit.audit_event FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_audit_event_tenant_isolation ON audit.audit_event
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
-- 注: WITH CHECK なし（INSERT 専用、業務で tenant_id 設定）
```

> 特殊ケース: Tenant 削除後記録保持のため `tenant_id` FK なし、ただし RLS で他テナント参照不可

---

## 11. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
| v0.2 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | onboarding event_type 追加 (per ADR-0043-audit-onboarding-failed v0.1 §2.2):<br>- `ck_audit_action_v0_2` DOMAIN 制約追加: `action ~ '^onboarding\.test_key\.(failed\|started\|succeeded)$'` (Phase 2 用, onboarding 3 event 許容)<br>- onboarding.test_key.failed: actor_type=`system`, resource_type=`api_key`, after_state={provider, label, attempts, status_code, error_message, detected_key_id}<br>- §10 RLS 不变 (per 13 類), §6 パーティション不变 (月次 BRIN), §5 REVOKE 不变 (WORM)<br>- 既存 16 字段不拡張, 新表 0 (per ADR-0043 §2.1 既存活用方針) | 2026-09-02 08:39 JST Ulysses 4 拍板 (仅 audit / .env 凭证 / 单 session / 增量 commit), 既存 audit_audit_event 活用戦略 |
