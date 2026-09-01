# local_runtime.runtime — テーブル詳細設計書

> **テーブル ID**: T96
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.25.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T96 |
| **物理名** | `local_runtime.runtime` |
| **論理名** | ランタイム（登録） |
| **スキーマ** | `local_runtime` |
| **Module** | `domain-local-runtime` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | 集群外 Local Runtime の サーバー側 Registry。**重要**: これは work-core プロセス内 registry で、Local Daemon バイナリ自体ではない（2 つの 制品、§4.6.1）。4 種別（LocalMachine / SelfHostedRunner / CloudWorkspace / FutureRuntime）+ 3 状態。`device_identity = mTLS Cert CN`。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ |
| 4 | `kind` | 種別 | VARCHAR | 32 | NO | − | − | − | − | idx (PT) | 4 値 |
| 5 | `device_id` | デバイス ID | UUID | − | NO | − | − | `identity.device(id)` (App) | ✓ | `uq_runtime_tenant_device` | 紐付 Device |
| 6 | `device_identity` | デバイス識別子 | VARCHAR | 2048 | NO | − | − | − | − | − | mTLS Cert CN (`runtime:{runtime_id}`) |
| 7 | `capabilities` | 能力 | JSONB | − | NO | `'["git","build","test"]'::jsonb` | − | − | − | − | 能力配列 |
| 8 | `status` | 状態 | VARCHAR | 16 | NO | `'OFFLINE'` | − | − | − | idx (PT) | 3 値 |
| 9 | `runtime_version` | ランタイムバージョン | VARCHAR | 32 | NO | − | − | − | − | − | Daemon バージョン |
| 10 | `last_heartbeat_at` | 最終ハートビート | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | idx (PT) | ハートビート |
| 11 | `hostname` | ホスト名 | VARCHAR | 255 | YES | `NULL` | − | − | − | − | OS ホスト名 |
| 12 | `os_info` | OS 情報 | VARCHAR | 64 | YES | `NULL` | − | − | − | − | `'linux 5.15'` / `'macos 14.0'` |
| 13 | `ip_address` | IP アドレス | INET | − | YES | `NULL` | − | − | − | − | 接続 IP |
| 14 | `is_revoked` | 失効フラグ | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 即時失効 |
| 15 | `revoked_at` | 失効日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | − |
| 16 | `revoked_reason` | 失効理由 | TEXT | − | YES | `NULL` | − | − | − | − | − |
| 17 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 18 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 19 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 20 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `runtime_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_runtime_*` | FOREIGN KEY (3) | `tenant_id` / `project_id` / `device_id` (App) | 各親テーブル | CASCADE | − |
| `uq_runtime_tenant_device` | UNIQUE | `(tenant_id, device_id, deleted_at)` | − | Device 1:1 |
| `ck_runtime_kind` | CHECK | `kind` | `IN ('LocalMachine','SelfHostedRunner','CloudWorkspace','FutureRuntime')` | 4 値 |
| `ck_runtime_status` | CHECK | `status` | `IN ('ONLINE','OFFLINE','STALE')` | 3 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `runtime_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_runtime_tenant_device` | btree (UK/PT) | `(tenant_id, device_id, deleted_at)` | − | Device 1:1 |
| `idx_runtime_tenant_status` | btree (PT) | `(tenant_id, status)` | `deleted_at IS NULL` | ステータス別 |
| `idx_runtime_tenant_project_kind` | btree (PT) | `(tenant_id, project_id, kind)` | `deleted_at IS NULL` | PJ + 種別 |
| `idx_runtime_tenant_last_heartbeat` | btree (PT) | `(tenant_id, last_heartbeat_at DESC)` | `status = 'ONLINE'` | ハートビート監視 |
| `idx_runtime_stale` | btree (PT) | `(tenant_id, last_heartbeat_at)` | `status = 'STALE' AND deleted_at IS NULL` | Stale 監視 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_runtime_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 1,000 |（1,000 テナント × 1 Runtime 平均） |
| 1 年後 | 10,000 |
| 3 年後 | 100,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.5 KB (JSONB + INET) | 100,000 | 約 150 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `identity.device` (App) | `device_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `local_runtime.runtime_command` | `runtime_id` |
| `local_runtime.runtime_observation` | `runtime_id` (App) |
| `local_runtime.reconciliation_report` | `runtime_id` |
| `worktree.worktree.runtime_id` (App) | − |

---

## 9. RLS Policy

```sql
ALTER TABLE local_runtime.runtime ENABLE ROW LEVEL SECURITY;
ALTER TABLE local_runtime.runtime FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_runtime_tenant_isolation ON local_runtime.runtime
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
