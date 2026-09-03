# permission.team — テーブル詳細設計書

> **テーブル ID**: T81
> **作成日**: 2026-09-03
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.3 §4.16.7 (本子项 G.3 新增, per 9/3 11:50 JST Ulysses Jira 化指令 + 拍板)
> **P3-G-W1 子项**: G.3 (基础层第三子项, per `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` §1.2)
> **Brief**: `docs/briefs/p3-g-w1.md`

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T81 |
| **物理名** | `permission.team` |
| **論理名** | チーム (Jira 風 Team 抽象, 跟 5 域/22 DDD 解耦 per team_dimension_opt4 拍板) |
| **スキーマ** | `permission` |
| **Module** | `domain-permission` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** (per 守门 #13 派生规 c, Master 100% RLS) |
| **業務分類** | **M (Master)** (per 守门 #13 派生规 b, SCD Type 2 適用) |
| **概要** | Jira 風 Team 抽象。跟 5 域 (player/economy/match/social/admin 历史治理) / 22 DDD bounded context 都解耦 (per Q1-D 拍板 + AGENTS.md §5)。team_member (T82) M:N 桥接 user_account, role_per_team (T83) 跨 team 不同 role。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `name` | チーム名 | VARCHAR | 200 | NO | − | − | − | ✓ | `uq_team_tenant_name` | 业务表示名 (e.g. `platform-ops-team`) |
| 4 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | − |
| 5 | `team_purpose` | チーム目的 | VARCHAR | 64 | NO | − | − | − | − | idx (PT) | 4 値: `Engineering` / `Operations` / `Review` / `CrossFunctional` (跟 5 域解耦) |
| 6 | `lead_user_account_id` | リード | UUID | − | YES | `NULL` | − | `permission.user_account(id)` ON DELETE SET NULL | − | idx (PT) | チームリード (per 守门 #3 5 域独立 Lead 问责结构, 但 team-level) |
| 7 | `lifecycle_status` | ライフサイクル | VARCHAR | 32 | NO | `'active'` | − | − | − | idx (PT) | `active` / `paused` / `archived` / `blocked` (per G.11) |
| 8 | `metadata` | メタデータ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 自由形式 (関連 ADR, 関連 domain, 関連 Saga, ...) |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 (SCD-2 適用) |
| 12 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック (SCD-2 row version) |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `team_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_team_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_team_lead_user_account` | FOREIGN KEY | `lead_user_account_id` | `permission.user_account(id)` | SET NULL | チームリード削除時 SET NULL |
| `uq_team_tenant_name` | UNIQUE (PT) | `(tenant_id, name)` | `WHERE deleted_at IS NULL` | − | チーム名一意 (per tenant) |
| `ck_team_purpose` | CHECK | `team_purpose` | `IN ('Engineering', 'Operations', 'Review', 'CrossFunctional')` | − | 4 値 (跟 5 域 / 22 DDD 解耦) |
| `ck_team_lifecycle` | CHECK | `lifecycle_status` | `IN ('active', 'paused', 'archived', 'blocked')` | − | 4 値 Lifecycle (per G.11) |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `team_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_team_tenant_name` | btree (UK/PT) | `(tenant_id, name)` | `deleted_at IS NULL` | チーム名一意 |
| `idx_team_tenant` | btree (PT) | `tenant_id` | `deleted_at IS NULL` | テナント別 |
| `idx_team_purpose` | btree (PT) | `(tenant_id, team_purpose)` | `deleted_at IS NULL` | 目的別 |
| `idx_team_lead` | btree (PT) | `lead_user_account_id` | `lead_user_account_id IS NOT NULL AND deleted_at IS NULL` | リード別 |
| `idx_team_lifecycle` | btree (PT) | `(tenant_id, lifecycle_status)` | `deleted_at IS NULL` | Lifecycle 状態 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_team_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |
| `trg_team_scd2_version` | BEFORE UPDATE | `public.fn_scd2_version_bump()` | SCD-2 version 自動 bump |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 500 (100 テナント × 5 team: 2 Engineering + 1 Operations + 1 Review + 1 CrossFunctional) |
| 1 年後 | 5,000 |
| 3 年後 | 50,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 800 B (含 JSONB metadata) | 50,000 | 約 40 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `permission.user_account` | `lead_user_account_id` |

### 8.2 被参照元

| 被参照元 | FK 列 | 関係 |
|---|---|---|
| `permission.team_member` (T82) | `team_id` | 1:N (1 team → 1+ user_account M:N 桥接, 多重隶属 per team_dimension_opt4) |
| `permission.role_per_team` (T83) | `team_id` | 1:N (1 team → 1+ team_role 跨 team 不同 role) |
| `worktree.worktree` (TBD, per W3-G.9) | `assigned_team_id` | N:1 (1 worktree → 1 team, per agent.team_id[] 多重隶属) |
| `agent.agent_session` (T77, TBD) | `team_id` | N:1 (session 屬 team) |

---

## 9. RLS Policy

```sql
ALTER TABLE permission.team ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.team FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_team_tenant_isolation ON permission.team
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);

CREATE POLICY policy_team_lifecycle_filter ON permission.team
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
| (a) W | **不適用** | team 是永続参考 / 設定, 不是 session-bound |
| (b) T | **不適用** | team 是参考 / 設定, 不是事件流水 |
| (c) M | **適用 ✓** | 物理删除禁止 (FK 連鎖 violate), SCD Type 2, RLS 13 類必携 |
| (d) M 100% RLS | **✓** | RLS `ENABLE` + `FORCE` + tenant_id 强制 + Lifecycle filter |

**主分類**: M (Master)
**混合**: 无 (per 守门 #13 派生规)

---

## 11. 已知缺口 (per 守门 #11 缺标比错标安全, DDD Review 必查)

| # | 缺口 | 风险 | 缓解 | 评审 Lead |
|---|---|---|---|---|
| 1 | **跟 5 域 (player/economy/match/social/admin) 命名空间解耦** (per Q1-D 拍板 + AGENTS.md §5 v0.6 + team_dimension_opt4 拍板) | 跨文档可能混淆 team 跟 5 域 (e.g. `player-team` vs `player-domain`), 子代理 dispatch 时命名歧义 | W1 落档后 `team_purpose` 4 値 (Engineering/Operations/Review/CrossFunctional) 跟 5 域完全不同, 文档加 disclaimer 显式说明 | 架构师 + 5 域 Lead |
| 2 | **跟 22 DDD bounded context 解耦** (per AGENTS.md §5 + team_dimension_opt4) | `domain-identity` / `domain-work-item` / `domain-permission` 等 22 域跟 team 维度不直接对齐, 跨域编排时容易混 | W3 G.9 agent.team_id[] 多重隶属实现 时显式说明 team 跟 22 DDD 非同一分类 | 架构师 + 5 域 Lead |
| 3 | **lead_user_account_id 指向 user_account (T78) 而非 agent.agent (T77)** | 真人 lead 跟 agent lead 命名空间不同, application 层映射 (human lead → agent lead for AI subagent 代理决策) | W1 落档后 lead 是 user_account (真人), agent lead (per agent.agent) 在 W3 G.10 跨域决策时统一 | 架构师 + 5 域 Lead |
| 4 | **5 域真人 Lead 不到位** (per 守门 #3 + 2026-08-31 22:45 JST 拍板 B 反转) | team lead user_account 字段 ready, 但真人 lead 不到位, Mavis 临时代签决策 | per 拍板 B 反转, Mavis 临时代签, 真人到位后追溯签字 | 5 域 Lead 真人 + 架构师 |

---

## 12. Migration SQL 概略 (per 守门 #13 RLS 100% 强制)

```sql
-- 2026-09-03-create-permission-team.sql

CREATE TABLE permission.team (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID         NOT NULL REFERENCES tenant.tenant(id) ON DELETE CASCADE,
    name            VARCHAR(200) NOT NULL,
    description     TEXT,
    team_purpose    VARCHAR(64)  NOT NULL,
    lead_user_account_id UUID    REFERENCES permission.user_account(id) ON DELETE SET NULL,
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    metadata        JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         INT          NOT NULL DEFAULT 1
);

-- 制約
ALTER TABLE permission.team
    ADD CONSTRAINT ck_team_purpose
        CHECK (team_purpose IN ('Engineering', 'Operations', 'Review', 'CrossFunctional')),
    ADD CONSTRAINT ck_team_lifecycle
        CHECK (lifecycle_status IN ('active', 'paused', 'archived', 'blocked'));

-- 一意制約
CREATE UNIQUE INDEX uq_team_tenant_name
    ON permission.team (tenant_id, name) WHERE deleted_at IS NULL;

-- 検索インデックス
CREATE INDEX idx_team_tenant ON permission.team (tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_team_purpose ON permission.team (tenant_id, team_purpose) WHERE deleted_at IS NULL;
CREATE INDEX idx_team_lead ON permission.team (lead_user_account_id)
    WHERE lead_user_account_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX idx_team_lifecycle ON permission.team (tenant_id, lifecycle_status) WHERE deleted_at IS NULL;

-- Trigger
CREATE TRIGGER trg_team_updated_at BEFORE UPDATE ON permission.team
    FOR EACH ROW EXECUTE FUNCTION public.fn_update_updated_at();

-- RLS
ALTER TABLE permission.team ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.team FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_team_tenant_isolation ON permission.team
    USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);

CREATE POLICY policy_team_lifecycle_filter ON permission.team
    USING (
        lifecycle_status IN ('active', 'paused')
        OR current_setting('app.is_platform_operator', TRUE)::BOOLEAN = TRUE
    )
    WITH CHECK (
        lifecycle_status IN ('active', 'paused')
        OR current_setting('app.is_platform_operator', TRUE)::BOOLEAN = TRUE
    );

COMMENT ON TABLE permission.team IS 'Jira 風 Team 抽象, 跟 5 域/22 DDD 解耦 (T81, P3-G-W1 G.3)';
COMMENT ON COLUMN permission.team.team_purpose IS 'Engineering / Operations / Review / CrossFunctional (跟 5 域/22 DDD 解耦)';
COMMENT ON COLUMN permission.team.lead_user_account_id IS 'team lead (user_account T78, 真人); agent lead 在 W3 跨域决策时统一';
```

---

## 13. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: T81 team 实体 (M 类 SCD-2 + RLS 13 類), 12 カラム, 6 制約, 6 インデックス, 2 トリガー, RLS 2 policy, team_purpose 4 値 (跟 5 域/22 DDD 解耦), 4 已知缺口 | 2026-09-03 P3-G-W1 子项 G.3 落地 |
