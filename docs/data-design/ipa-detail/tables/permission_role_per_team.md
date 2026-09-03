# permission.role_per_team — テーブル詳細設計書

> **テーブル ID**: T83
> **作成日**: 2026-09-03
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.3 §4.16.9 (本子项 G.4 新增, per 9/3 11:50 JST Ulysses Jira 化指令 + 拍板)
> **P3-G-W1 子项**: G.4 (基础层第四子项, per `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` §1.2)
> **Brief**: `docs/briefs/p3-g-w1.md`
> **关键拍板**: 跨 team 不同 role (1 user 在 team A 是 admin, 在 team B 是 viewer) per team_dimension_opt4 拍板

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T83 |
| **物理名** | `permission.role_per_team` |
| **論理名** | チーム内ロール (跨 team 不同 role) |
| **スキーマ** | `permission` |
| **Module** | `domain-permission` |
| **種別** | Entity (Weak) |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** (per 守门 #13 派生规 b, Transaction 100% audit + RLS) |
| **業務分類** | **T (Transaction)** (per 守门 #13 派生规 b, 业务事实 / append-only 適用) |
| **概要** | team_member (T82) → role (T49) 跨 team 独立映射。1 user_account 在 team A 是 admin, 在 team B 是 viewer, 互不影响 (per team_dimension_opt4 拍板)。`role_key` 复用现有 4 値 Role (tenant_admin / project_admin / developer / viewer)。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `team_member_id` | チームメンバー ID | UUID | − | NO | − | − | `permission.team_member(id)` ON DELETE CASCADE | − | idx (PT) | 所属 team_member (跟 T82 1:N) |
| 4 | `team_id` | チーム ID (denormalized) | UUID | − | NO | − | − | `permission.team(id)` ON DELETE CASCADE | − | idx (PT) | 冗長, 查询性能优化 (避免 JOIN T82) |
| 5 | `role_key` | ロールキー | VARCHAR | 64 | NO | − | − | − | − | idx (PT) | 4 値: `tenant_admin` / `project_admin` / `developer` / `viewer` (per T49 builtin_key) |
| 6 | `granted_at` | 付与日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 付与時刻 (audit 用) |
| 7 | `granted_by` | 付与者 | UUID | − | YES | `NULL` | − | `permission.user_account(id)` ON DELETE SET NULL | − | − | 付与者 user_account (audit 用) |
| 8 | `valid_from` | 有効開始 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 期間有効 (Role 期間管理, audit 用) |
| 9 | `valid_until` | 有効終了 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | idx (PT) | 期間有効 (NULL = 無期限) |
| 10 | `metadata` | メタデータ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 自由形式 (付与理由, 関連 ticket, ...) |
| 11 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 12 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 13 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 (剥奪履歴保持, audit) |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `role_per_team_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_role_per_team_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_role_per_team_team_member` | FOREIGN KEY | `team_member_id` | `permission.team_member(id)` | CASCADE | − |
| `fk_role_per_team_team` | FOREIGN KEY | `team_id` | `permission.team(id)` | CASCADE | − |
| `fk_role_per_team_granted_by` | FOREIGN KEY | `granted_by` | `permission.user_account(id)` | SET NULL | − |
| `uq_role_per_team_unique` | UNIQUE (PT) | `(team_member_id, role_key, valid_from)` | `WHERE deleted_at IS NULL` | − | 同一 (team_member, role, valid_from) 唯一 (期間重複不可) |
| `ck_role_per_team_role_key` | CHECK | `role_key` | `IN ('tenant_admin', 'project_admin', 'developer', 'viewer')` | − | 4 値 (跟 T49 builtin_key 一致) |
| `ck_role_per_team_valid_range` | CHECK | `valid_from` / `valid_until` | `valid_until IS NULL OR valid_until > valid_from` | − | 期間整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `role_per_team_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_role_per_team_unique` | btree (UK/PT) | `(team_member_id, role_key, valid_from)` | `deleted_at IS NULL` | 一意 (期間重複不可) |
| `idx_role_per_team_tenant` | btree (PT) | `tenant_id` | `deleted_at IS NULL` | テナント別 |
| `idx_role_per_team_team` | btree (PT) | `team_id` | `deleted_at IS NULL` | チーム別 |
| `idx_role_per_team_role_key` | btree (PT) | `(team_id, role_key)` | `deleted_at IS NULL AND valid_until IS NULL` | ロール検索 (現職) |
| `idx_role_per_team_validity` | btree (PT) | `(team_member_id, valid_from, valid_until)` | `deleted_at IS NULL` | 期間検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_role_per_team_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |
| `trg_role_per_team_no_overlap` | BEFORE INSERT OR UPDATE | `public.fn_role_per_team_no_overlap()` | 同一 (team_member, role_key) 期間重複不可 (跨 session 续 W3 跨域决策) |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 50,000 (25,000 team_member × 2 role 平均) |
| 1 年後 | 500,000 |
| 3 年後 | 5,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 400 B (含 JSONB metadata) | 5,000,000 | 約 2 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `permission.team_member` (T82) | `team_member_id` |
| `permission.team` (T81) | `team_id` (denormalized) |
| `permission.user_account` (T78) | `granted_by` |

### 8.2 被参照元

| 被参照元 | FK 列 | 関係 |
|---|---|---|
| `permission.permission_scheme` (T51, TBD per W3-G.10) | `team_id` + `role_key` (JSONB 引用) | 跨域 Permission Scheme 跨 team 扩展使用 |
| `domain-audit.audit_event` | `target_id` (T83 row id) | ロール 付与 / 剥奪 事件 audit |

---

## 9. RLS Policy

```sql
ALTER TABLE permission.role_per_team ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.role_per_team FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_role_per_team_tenant_isolation ON permission.role_per_team
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 业务分類根拠 (per 守门 #13 派生规 a/b/c/d 4 段检查清单)

| 派生规 | 判定 | 依据 |
|---|---|---|
| (a) W | **不適用** | role_per_team 是永続业务事实 (剥奪履歴保留), 不是短 TTL 作业中数据 |
| (b) T | **適用 ✓** | 物理删除禁止 (`deleted_at` 論理削除保留 audit), audit 必須, RLS 13 類必携 |
| (c) M | **不適用** | M 类是参考 / 設定, role_per_team 是业务事实 |
| (d) T 100% audit | **✓** | ロール 付与 / 剥奪 事件记录到 `domain-audit.audit_event` |

**主分類**: T (Transaction)
**混合**: 无 (per 守门 #13 派生规)

---

## 11. 已知缺口 (per 守门 #11 缺标比错标安全, DDD Review 必查)

| # | 缺口 | 风险 | 缓解 | 评审 Lead |
|---|---|---|---|---|
| 1 | **期间重複不可 trigger** (`fn_role_per_team_no_overlap`) 跨 session 续 W3 跨域决策 | W1 落档后 trigger 是 stub, 实际重複检查 留 W3 G.10 实施 | W1 落档 trigger 名 ready, W3 实施时具体 逻辑 (range overlap check via SQL) | 架构师 + 5 域 Lead |
| 2 | **role_key 4 値 跟 builtin 4 値 Role (T49) 命名空间一致** | 跨 team role 跟 4 builtin role 复用, 跟 5 域/22 DDD 解耦 | W1 落档后 role_key 4 値 跟 T49 builtin_key 一致, 跨 team role 跟 tenant role 共享 enum | 架构师 + 5 域 Lead |
| 3 | **5 域真人 Lead 不到位** (per 守门 #3 + 2026-08-31 22:45 JST 拍板 B 反转) | role_per_team 字段 ready, 但真人 lead 不到位, Mavis 临时代签决策 | per 拍板 B 反转, Mavis 临时代签, 真人到位后追溯签字 | 5 域 Lead 真人 + 架构师 |
| 4 | **W3 G.10 Permission Scheme 跨 team 扩展 跨 session 续** | role_per_team 跨 team role ready, 但 Permission Scheme 跨 team 实际应用 (e.g. project + team 双重 scope) 留 W3 G.10 | W1 落档后 role_per_team schema 完整, W3 实施 Permission Scheme 跨 team JSONB 扩展 | 架构师 + 5 域 Lead |

---

## 12. Migration SQL 概略 (per 守门 #13 RLS 100% 强制)

```sql
-- 2026-09-03-create-permission-role-per-team.sql

CREATE TABLE permission.role_per_team (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID         NOT NULL REFERENCES tenant.tenant(id) ON DELETE CASCADE,
    team_member_id  UUID         NOT NULL REFERENCES permission.team_member(id) ON DELETE CASCADE,
    team_id         UUID         NOT NULL REFERENCES permission.team(id) ON DELETE CASCADE,
    role_key        VARCHAR(64)  NOT NULL,
    granted_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    granted_by      UUID         REFERENCES permission.user_account(id) ON DELETE SET NULL,
    valid_from      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    valid_until     TIMESTAMPTZ,
    metadata        JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

-- 制約
ALTER TABLE permission.role_per_team
    ADD CONSTRAINT ck_role_per_team_role_key
        CHECK (role_key IN ('tenant_admin', 'project_admin', 'developer', 'viewer')),
    ADD CONSTRAINT ck_role_per_team_valid_range
        CHECK (valid_until IS NULL OR valid_until > valid_from);

-- 一意制約
CREATE UNIQUE INDEX uq_role_per_team_unique
    ON permission.role_per_team (team_member_id, role_key, valid_from)
    WHERE deleted_at IS NULL;

-- 検索インデックス
CREATE INDEX idx_role_per_team_tenant ON permission.role_per_team (tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_role_per_team_team ON permission.role_per_team (team_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_role_per_team_role_key
    ON permission.role_per_team (team_id, role_key)
    WHERE deleted_at IS NULL AND valid_until IS NULL;
CREATE INDEX idx_role_per_team_validity
    ON permission.role_per_team (team_member_id, valid_from, valid_until)
    WHERE deleted_at IS NULL;

-- Trigger
CREATE TRIGGER trg_role_per_team_updated_at BEFORE UPDATE ON permission.role_per_team
    FOR EACH ROW EXECUTE FUNCTION public.fn_update_updated_at();

-- RLS
ALTER TABLE permission.role_per_team ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.role_per_team FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_role_per_team_tenant_isolation ON permission.role_per_team
    USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);

COMMENT ON TABLE permission.role_per_team IS 'Team 内ロール, 跨 team 不同 role (T83, P3-G-W1 G.4)';
COMMENT ON COLUMN permission.role_per_team.role_key IS 'tenant_admin / project_admin / developer / viewer (跟 T49 builtin_key 一致)';
```

---

## 13. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: T83 role_per_team 实体 (T 类 audit, 跨 team 不同 role per team_dimension_opt4 拍板), 13 カラム, 8 制約, 6 インデックス, 2 トリガー, RLS 1 policy, 期间重複不可 trigger, 4 已知缺口 | 2026-09-03 P3-G-W1 子项 G.4 落地 |
