# permission.user_account — テーブル詳細設計書

> **テーブル ID**: T78
> **作成日**: 2026-09-03
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.3 §4.16.4 (本子项 G.1 新增, per 9/3 11:50 JST Ulysses Jira 化指令 + 拍板)
> **P3-G-W1 子项**: G.1 (基础层第一子项, per `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` §1.2)
> **Brief**: `docs/briefs/p3-g-w1.md`

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T78 |
| **物理名** | `permission.user_account` |
| **論理名** | ユーザーアカウント（Identity / 認証基底） |
| **スキーマ** | `permission` |
| **Module** | `domain-permission` (跟 `permission.role` 平行) |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** (per 守门 #13 派生规 c, Master 100% RLS) |
| **業務分類** | **M (Master)** (per 守门 #13 派生规 b, SCD Type 2 適用) |
| **概要** | Jira 風 User Account (login/email/avatar) 实体。`agent.agent` (T77) 跟 `subagent` (W2 G.6) 共享 user_account 1:N (双层 L1 桥接 per subagent_persist_opt4 拍板) |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `login` | ログイン ID | VARCHAR | 200 | NO | − | − | − | ✓ | `uq_user_account_tenant_login` | 业务一意登录 (e.g. `ulysses@acme.com` 或 `agent-worker-001`) |
| 4 | `display_name` | 表示名 | VARCHAR | 200 | NO | − | − | − | − | idx | 业务表示名 (e.g. `Ulysses` / `Worker Subagent 001`) |
| 5 | `email` | メール | VARCHAR | 320 | YES | `NULL` | − | − | − | idx (PT) | 任意, subagent 不一定需要 |
| 6 | `avatar_url` | アバター URL | VARCHAR | 1024 | YES | `NULL` | − | − | − | − | 任意, subagent 用 icon URL |
| 7 | `account_type` | アカウント種別 | VARCHAR | 32 | NO | `'Human'` | − | − | − | idx (PT) | `Human` / `Subagent` / `ServiceAccount` (跟 agent.agent.agent_type 区分) |
| 8 | `is_enabled` | 有効 | BOOLEAN | 1 | NO | `TRUE` | − | − | − | idx (PT) | アカウント有効 / 無効 |
| 9 | `last_login_at` | 最終ログイン | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 最終セッション開始時刻 |
| 10 | `lifecycle_status` | ライフサイクル | VARCHAR | 32 | NO | `'active'` | − | − | − | idx (PT) | `active` / `paused` / `archived` / `blocked` (per G.11 Lifecycle 状态机) |
| 11 | `metadata` | メタデータ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 自由形式 (timezone, locale, preferences, ...) |
| 12 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 13 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 14 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 (SCD-2 適用, per 守门 #13 派生规 b) |
| 15 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック (SCD-2 row version) |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `user_account_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_user_account_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `uq_user_account_tenant_login` | UNIQUE (PT) | `(tenant_id, login)` | `WHERE deleted_at IS NULL` | − | 登录 ID 一意 (per tenant) |
| `ck_user_account_type` | CHECK | `account_type` | `IN ('Human', 'Subagent', 'ServiceAccount')` | − | 3 値 (跟 agent.agent.agent_type 区分) |
| `ck_user_account_lifecycle` | CHECK | `lifecycle_status` | `IN ('active', 'paused', 'archived', 'blocked')` | − | 4 値 Lifecycle (per G.11) |
| `ck_user_account_email_format` | CHECK | `email` | `email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'` | − | メール書式 (任意, 設定時のみ) |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `user_account_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_user_account_tenant_login` | btree (UK/PT) | `(tenant_id, login)` | `deleted_at IS NULL` | ログイン ID 一意 |
| `idx_user_account_tenant_type` | btree (PT) | `(tenant_id, account_type)` | `deleted_at IS NULL` | 種別別 |
| `idx_user_account_enabled` | btree (PT) | `(tenant_id, is_enabled)` | `is_enabled = TRUE AND deleted_at IS NULL` | 有効アカウント |
| `idx_user_account_lifecycle` | btree (PT) | `(tenant_id, lifecycle_status)` | `deleted_at IS NULL` | Lifecycle 状態 |
| `idx_user_account_email` | btree (PT) | `(tenant_id, email)` | `email IS NOT NULL AND deleted_at IS NULL` | メール検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_user_account_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |
| `trg_user_account_scd2_version` | BEFORE UPDATE | `public.fn_scd2_version_bump()` | SCD-2 version 自動 bump (per 守门 #13 派生规 b) |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 5,000 (100 テナント × 50 アカウント: 40 Human + 9 Subagent + 1 ServiceAccount) |
| 1 年後 | 50,000 |
| 3 年後 | 500,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.2 KB (含 JSONB metadata) | 500,000 | 約 600 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |

### 8.2 被参照元

| 被参照元 | FK 列 | 関係 |
|---|---|---|
| `permission.user_account_link` (G.5 双层 L1) | `user_account_id` | 1:N (1 user_account → 1+ agent.agent) |
| `agent.subagent` (W2 G.6 双层 L2) | `user_account_id` | 1:N (1 user_account → 1+ subagent) |
| `permission.group_member` (G.2) | `user_account_id` | M:N (user_account ↔ group) |
| `permission.team_member` (G.4) | `user_account_id` | M:N (user_account ↔ team, 多重隶属 per team_dimension_opt4) |
| `feedback.feedback` | `author_user_account_id` (TBD, per W2-G.8) | 1:N (1 user_account → 1+ feedback) |
| `agent.agent_session` (T77) | `actor_user_account_id` (TBD, per W2-G.8) | 1:N (1 user_account → 1+ session) |

---

## 9. RLS Policy

```sql
ALTER TABLE permission.user_account ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.user_account FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_user_account_tenant_isolation ON permission.user_account
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);

-- Lifecycle 状態 filter: blocked / archived 是 platform operator only
CREATE POLICY policy_user_account_lifecycle_filter ON permission.user_account
  USING (
    lifecycle_status IN ('active', 'paused')
    OR current_setting('app.is_platform_operator', TRUE)::BOOLEAN = TRUE
  )
  WITH CHECK (
    lifecycle_status IN ('active', 'paused')
    OR current_setting('app.is_platform_operator', TRUE)::BOOLEAN = TRUE
  );
```

---

## 10. 业务分類根拠 (per 守门 #13 派生规 a/b/c/d 4 段检查清单)

| 派生规 | 判定 | 依据 |
|---|---|---|
| (a) W = 物理删除 / タイマー失効 / 短 TTL 明示 retention | **不適用** | user_account 是永続身份, 不是 session-bound 作业中数据 |
| (b) T = 物理删除禁止 + 監査必須 + RLS 13 類必携 | **不適用** | T 类是业务事实 / append-only, user_account 是参考 / 設定, 不是事件流水 |
| (c) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携 | **適用 ✓** | 物理删除禁止 (FK 連鎖 violate), SCD Type 2 (`version` 字段 + `deleted_at` PT 索引), RLS 13 類必携 (`tenant_id` + Lifecycle 状态 filter) |
| (d) Master 100% RLS / Transaction 100% audit / Work 100% retention_period | **M 100% RLS ✓** | RLS `ENABLE` + `FORCE` + tenant_id 强制 + Lifecycle filter |

**主分類**: M (Master)
**混合**: 无 (不混合 M/T 或 T/W, per 守门 #13 派生规"混合分類主分類单计 + §已知缺口 显式列出待 DDD Review Lead 确认")

---

## 11. 已知缺口 (per 守门 #11 缺标比错标安全, DDD Review 必查)

| # | 缺口 | 风险 | 缓解 | 评审 Lead |
|---|---|---|---|---|
| 1 | `account_type` 3 值 (Human/Subagent/ServiceAccount) 跟 `agent.agent.agent_type` 6 值 (Codex/ClaudeCode/...) 命名空间不直接对齐 | 子代理类型映射 (WorkerSubagent → Subagent 账户) 需要 entity-level bridge, 在 W2 G.6 `agent.subagent` 落地时显式 | W1 G.5 `agent.user_account_link` 提供 1:1 桥接, W2 G.6 subagent 实体 account_type='Subagent' + subagent_type='WorkerSubagent' 二级枚举 | 架构师 + 5 域 Lead |
| 2 | Lifecycle 状态 4 值 (active/paused/archived/blocked) 是 G.11 W3 落地范围, W1 提前占位 | W1 落档后, W3 之前 lifecycle 状态机不完整, blocked / archived 切换无 API | migration SQL 加 CHECK 约束, 落档后 status 默认 'active', blocked/archived 切换 API 在 W3 落地 | 架构师 + 5 域 Lead |
| 3 | 双层 L1 (W1 G.5) + L2 (W2 G.6) + L3 (W2 G.8) 跨 session 续, W1 落地后 user_account 1:1 跟 agent.agent 桥接, subagent 端未落地 | W1 期间, subagent dispatch (本地 mavis worker/explore/verifier) 仍是 session-bound, 不持久化到 user_account | W1 临时方案: dispatcher.py (W2 G.13 落地) 自动注册; W1 不派子代理 (per 守门 #9 #3 实证) | 架构师 + 5 域 Lead |

---

## 12. Migration SQL 概略 (per 守门 #13 RLS 100% 强制)

```sql
-- 2026-09-03-create-permission-user-account.sql

CREATE TABLE permission.user_account (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID         NOT NULL REFERENCES tenant.tenant(id) ON DELETE CASCADE,
    login           VARCHAR(200) NOT NULL,
    display_name    VARCHAR(200) NOT NULL,
    email           VARCHAR(320),
    avatar_url      VARCHAR(1024),
    account_type    VARCHAR(32)  NOT NULL DEFAULT 'Human',
    is_enabled      BOOLEAN      NOT NULL DEFAULT TRUE,
    last_login_at   TIMESTAMPTZ,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    metadata        JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         INT          NOT NULL DEFAULT 1
);

-- 制約
ALTER TABLE permission.user_account
    ADD CONSTRAINT ck_user_account_type
        CHECK (account_type IN ('Human', 'Subagent', 'ServiceAccount')),
    ADD CONSTRAINT ck_user_account_lifecycle
        CHECK (lifecycle_status IN ('active', 'paused', 'archived', 'blocked')),
    ADD CONSTRAINT ck_user_account_email_format
        CHECK (email IS NULL OR email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$');

-- 一意制約 (PT: 部分索引)
CREATE UNIQUE INDEX uq_user_account_tenant_login
    ON permission.user_account (tenant_id, login)
    WHERE deleted_at IS NULL;

-- 検索インデックス
CREATE INDEX idx_user_account_tenant_type
    ON permission.user_account (tenant_id, account_type)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_user_account_enabled
    ON permission.user_account (tenant_id, is_enabled)
    WHERE is_enabled = TRUE AND deleted_at IS NULL;
CREATE INDEX idx_user_account_lifecycle
    ON permission.user_account (tenant_id, lifecycle_status)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_user_account_email
    ON permission.user_account (tenant_id, email)
    WHERE email IS NOT NULL AND deleted_at IS NULL;

-- Trigger
CREATE TRIGGER trg_user_account_updated_at
    BEFORE UPDATE ON permission.user_account
    FOR EACH ROW EXECUTE FUNCTION public.fn_update_updated_at();

-- RLS
ALTER TABLE permission.user_account ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.user_account FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_user_account_tenant_isolation ON permission.user_account
    USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);

CREATE POLICY policy_user_account_lifecycle_filter ON permission.user_account
    USING (
        lifecycle_status IN ('active', 'paused')
        OR current_setting('app.is_platform_operator', TRUE)::BOOLEAN = TRUE
    )
    WITH CHECK (
        lifecycle_status IN ('active', 'paused')
        OR current_setting('app.is_platform_operator', TRUE)::BOOLEAN = TRUE
    );

COMMENT ON TABLE permission.user_account IS 'Jira 風 User Account (login/email/avatar) 实体 (T78, P3-G-W1 G.1)';
COMMENT ON COLUMN permission.user_account.account_type IS 'Human / Subagent / ServiceAccount';
COMMENT ON COLUMN permission.user_account.lifecycle_status IS 'active / paused / archived / blocked (per G.11)';
```

---

## 13. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: T78 user_account 实体 (M 类 SCD-2 + RLS 13 類), 15 カラム, 6 制約, 6 インデックス, 2 トリガー, RLS 2 policy (tenant isolation + lifecycle filter), 業務分類 M (per 守门 #13), 3 已知缺口 | 2026-09-03 11:50 JST Ulysses Jira 化指令 + 3 步 ask_user 拍板 (direction_opt4 + team_dimension_opt4 + subagent_persist_opt4) + P3-G-W1 子项 G.1 落地 |
