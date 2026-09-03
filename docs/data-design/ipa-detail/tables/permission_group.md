# permission.group — テーブル詳細設計書

> **テーブル ID**: T79
> **作成日**: 2026-09-03
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.3 §4.16.5 (本子项 G.2 新增, per 9/3 11:50 JST Ulysses Jira 化指令 + 拍板)
> **P3-G-W1 子项**: G.2 (基础层第二子项, per `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` §1.2)
> **Brief**: `docs/briefs/p3-g-w1.md`

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T79 |
| **物理名** | `permission.group` |
| **論理名** | グループ（クロスチーム集合） |
| **スキーマ** | `permission` |
| **Module** | `domain-permission` (跟 `permission.role` 平行) |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** (per 守门 #13 派生规 c, Master 100% RLS) |
| **業務分類** | **M (Master)** (per 守门 #13 派生规 b, SCD Type 2 適用) |
| **概要** | Jira 風 Group 抽象。`user_account` (T78) 跟 `team` (T81) 跨 team 集合 (e.g. `jira-administrators`, `domain-experts`)。`group_member` (T80) 是 M:N 桥接。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `name` | グループ名 | VARCHAR | 200 | NO | − | − | − | ✓ | `uq_group_tenant_name` | 业务表示名 (e.g. `jira-administrators`) |
| 4 | `description` | 説明 | TEXT | − | YES | `NULL` | − | − | − | − | − |
| 5 | `is_builtin` | Built-in フラグ | BOOLEAN | 1 | NO | `FALSE` | − | − | ✓ | `uq_group_builtin_key` | システム定義 (e.g. `jira-administrators`) |
| 6 | `builtin_key` | Built-in キー | VARCHAR | 64 | YES | `NULL` | − | − | ✓ | `uq_group_builtin_key` | 内蔵 group 4 値: `jira-administrators` / `jira-users` / `platform-operators` / `all-users` |
| 7 | `lifecycle_status` | ライフサイクル | VARCHAR | 32 | NO | `'active'` | − | − | − | idx (PT) | `active` / `paused` / `archived` / `blocked` (per G.11) |
| 8 | `metadata` | メタデータ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 自由形式 (用途説明, 関連 ADR, ...) |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 (SCD-2 適用, per 守门 #13 派生规 b) |
| 12 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック (SCD-2 row version) |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `group_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_group_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `uq_group_tenant_name` | UNIQUE (PT) | `(tenant_id, name)` | `WHERE deleted_at IS NULL` | − | グループ名一意 (per tenant) |
| `uq_group_builtin_key` | UNIQUE (PT) | `(tenant_id, builtin_key)` | `WHERE is_builtin = TRUE AND deleted_at IS NULL` | − | Built-in 一意 (4 値) |
| `ck_group_builtin_xor` | CHECK | `is_builtin` / `builtin_key` | `(is_builtin = FALSE) OR (is_builtin = TRUE AND builtin_key IS NOT NULL)` | − | Built-in 整合 |
| `ck_group_lifecycle` | CHECK | `lifecycle_status` | `IN ('active', 'paused', 'archived', 'blocked')` | − | 4 値 Lifecycle (per G.11) |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `group_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_group_tenant_name` | btree (UK/PT) | `(tenant_id, name)` | `deleted_at IS NULL` | グループ名一意 |
| `uq_group_builtin_key` | btree (UK/PT) | `(tenant_id, builtin_key)` | `is_builtin = TRUE AND deleted_at IS NULL` | Built-in 一意 |
| `idx_group_tenant` | btree (PT) | `tenant_id` | `deleted_at IS NULL` | テナント別 |
| `idx_group_lifecycle` | btree (PT) | `(tenant_id, lifecycle_status)` | `deleted_at IS NULL` | Lifecycle 状態 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_group_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |
| `trg_group_scd2_version` | BEFORE UPDATE | `public.fn_scd2_version_bump()` | SCD-2 version 自動 bump (per 守门 #13 派生规 b) |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 500 (100 テナント × 5 group: 4 Built-in + 1 custom) |
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

### 8.2 被参照元

| 被参照元 | FK 列 | 関係 |
|---|---|---|
| `permission.group_member` (T80) | `group_id` | 1:N (1 group → 1+ user_account M:N 桥接) |
| `permission.role_assignments` (T51 JSONB) | `group_id` (JSONB 引用) | N:M (group → role, per permission_scheme) |

---

## 9. RLS Policy

```sql
ALTER TABLE permission.group ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.group FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_group_tenant_isolation ON permission.group
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);

CREATE POLICY policy_group_lifecycle_filter ON permission.group
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
| (a) W = 物理删除 / タイマー失効 / 短 TTL 明示 retention | **不適用** | group 是永続参考 / 設定, 不是 session-bound 作业中数据 |
| (b) T = 物理删除禁止 + 監査必須 + RLS 13 類必携 | **不適用** | T 类是业务事实 / append-only, group 是参考 / 設定, 不是事件流水 |
| (c) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携 | **適用 ✓** | 物理删除禁止 (FK 連鎖 violate), SCD Type 2 (`version` 字段 + `deleted_at` PT 索引), RLS 13 類必携 |
| (d) Master 100% RLS | **M 100% RLS ✓** | RLS `ENABLE` + `FORCE` + tenant_id 强制 + Lifecycle filter |

**主分類**: M (Master)
**混合**: 无 (不混合 M/T, per 守门 #13 派生规"混合分類主分類单计 + §已知缺口 显式列出待 DDD Review Lead 确认")

---

## 11. 已知缺口 (per 守门 #11 缺标比错标安全, DDD Review 必查)

| # | 缺口 | 风险 | 缓解 | 评审 Lead |
|---|---|---|---|---|
| 1 | Built-in 4 値 (jira-administrators / jira-users / platform-operators / all-users) 跟现有 4 値 Role (tenant_admin / project_admin / developer / viewer) 命名空间不直接对齐 | 既存 role → group 映射 (e.g. `tenant_admin` ↔ `jira-administrators`) 需要 application 层映射, 在 W3 G.10 Permission Scheme 跨 team 落地时显式 | W1 落档后 builtin_key 4 値 ready, W3 G.10 跨域决策时统一映射 | 架构师 + 5 域 Lead |
| 2 | Cross-team Group 抽象 (W3 G.10 Permission Scheme 跨 team 落地) 跨 session 续, W1 只落 group + group_member 基础 | W1 期间, 跨 team group (e.g. `domain-experts` 跨 5 个 team) 不可用, 仅 per-team group 显式 | W1 落档后 per-team group 显式, W3 跨 team group 抽象跨 session 续 | 架构师 + 5 域 Lead |

---

## 12. Migration SQL 概略 (per 守门 #13 RLS 100% 强制)

```sql
-- 2026-09-03-create-permission-group.sql

CREATE TABLE permission.group (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID         NOT NULL REFERENCES tenant.tenant(id) ON DELETE CASCADE,
    name            VARCHAR(200) NOT NULL,
    description     TEXT,
    is_builtin      BOOLEAN      NOT NULL DEFAULT FALSE,
    builtin_key     VARCHAR(64),
    lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'active',
    metadata        JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    version         INT          NOT NULL DEFAULT 1
);

-- 制約
ALTER TABLE permission.group
    ADD CONSTRAINT ck_group_builtin_xor
        CHECK ((is_builtin = FALSE) OR (is_builtin = TRUE AND builtin_key IS NOT NULL)),
    ADD CONSTRAINT ck_group_lifecycle
        CHECK (lifecycle_status IN ('active', 'paused', 'archived', 'blocked')),
    ADD CONSTRAINT ck_group_builtin_key CHECK (builtin_key IS NULL OR builtin_key IN
        ('jira-administrators', 'jira-users', 'platform-operators', 'all-users'));

-- 一意制約
CREATE UNIQUE INDEX uq_group_tenant_name
    ON permission.group (tenant_id, name) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX uq_group_builtin_key
    ON permission.group (tenant_id, builtin_key)
    WHERE is_builtin = TRUE AND deleted_at IS NULL;

-- 検索インデックス
CREATE INDEX idx_group_tenant ON permission.group (tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_group_lifecycle
    ON permission.group (tenant_id, lifecycle_status) WHERE deleted_at IS NULL;

-- Trigger
CREATE TRIGGER trg_group_updated_at BEFORE UPDATE ON permission.group
    FOR EACH ROW EXECUTE FUNCTION public.fn_update_updated_at();

-- RLS
ALTER TABLE permission.group ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.group FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_group_tenant_isolation ON permission.group
    USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);

CREATE POLICY policy_group_lifecycle_filter ON permission.group
    USING (
        lifecycle_status IN ('active', 'paused')
        OR current_setting('app.is_platform_operator', TRUE)::BOOLEAN = TRUE
    )
    WITH CHECK (
        lifecycle_status IN ('active', 'paused')
        OR current_setting('app.is_platform_operator', TRUE)::BOOLEAN = TRUE
    );

COMMENT ON TABLE permission.group IS 'Jira 風 Group 抽象 (T79, P3-G-W1 G.2)';
COMMENT ON COLUMN permission.group.builtin_key IS 'jira-administrators / jira-users / platform-operators / all-users';
```

---

## 13. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: T79 group 实体 (M 类 SCD-2 + RLS 13 類), 12 カラム, 6 制約, 5 インデックス, 2 トリガー, RLS 2 policy, builtin 4 値, 2 已知缺口 | 2026-09-03 P3-G-W1 子项 G.2 落地 |
