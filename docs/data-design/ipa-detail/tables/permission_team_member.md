# permission.team_member — テーブル詳細設計書

> **テーブル ID**: T82
> **作成日**: 2026-09-03
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.3 §4.16.8 (本子项 G.4 新增, per 9/3 11:50 JST Ulysses Jira 化指令 + 拍板)
> **P3-G-W1 子项**: G.4 (基础层第四子项, per `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` §1.2)
> **Brief**: `docs/briefs/p3-g-w1.md`
> **关键拍板**: 多重隶属 (1 agent → N team, 跨 team 不同 role) per team_dimension_opt4 拍板

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T82 |
| **物理名** | `permission.team_member` |
| **論理名** | チームメンバー (M:N 桥接 user_account ↔ team, 多重隶属) |
| **スキーマ** | `permission` |
| **Module** | `domain-permission` |
| **種別** | Entity (Weak) |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** (per 守门 #13 派生规 b, Transaction 100% audit + RLS) |
| **業務分類** | **T (Transaction)** (per 守门 #13 派生规 b, 业务事实 / append-only 適用) |
| **概要** | user_account (T78) ↔ team (T81) M:N 桥接, **多重隶属** (1 user_account 可属 N team, per team_dimension_opt4 拍板)。`role_per_team` (T83) 跨 team 不同 role (per 同拍板)。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `team_id` | チーム ID | UUID | − | NO | − | − | `permission.team(id)` ON DELETE CASCADE | − | idx (PT) | 所属 team |
| 4 | `user_account_id` | ユーザーアカウント ID | UUID | − | NO | − | − | `permission.user_account(id)` ON DELETE CASCADE | − | idx (PT) | 所属 user_account (1 user_account 可属 N team, 多重隶属) |
| 5 | `joined_at` | 参加日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 参加時刻 (audit 用) |
| 6 | `joined_by` | 参加承認者 | UUID | − | YES | `NULL` | − | `permission.user_account(id)` ON DELETE SET NULL | − | − | 承認者 user_account (audit 用) |
| 7 | `is_primary` | 主所属 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 主 team 標識 (1 user_account 最多 1 primary team) |
| 8 | `is_lead` | リードフラグ | BOOLEAN | 1 | NO | `FALSE` | − | − | − | idx (PT) | team リード (跟 `team.lead_user_account_id` 一致性验证) |
| 9 | `metadata` | メタデータ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 自由形式 (加入理由, 関連 ticket, ...) |
| 10 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 12 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 (退出履歴保持, audit) |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `team_member_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_team_member_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_team_member_team` | FOREIGN KEY | `team_id` | `permission.team(id)` | CASCADE | − |
| `fk_team_member_user_account` | FOREIGN KEY | `user_account_id` | `permission.user_account(id)` | CASCADE | − |
| `fk_team_member_joined_by` | FOREIGN KEY | `joined_by` | `permission.user_account(id)` | SET NULL | − |
| `uq_team_member_unique` | UNIQUE (PT) | `(team_id, user_account_id)` | `WHERE deleted_at IS NULL` | − | 同一 (team, user_account) 唯一 (但 1 user_account 仍可属 N team) |
| `ck_team_member_primary_unique` | UNIQUE (PT, partial) | `(user_account_id)` | `WHERE is_primary = TRUE AND deleted_at IS NULL` | − | 1 user_account 最多 1 primary team |
| `ck_team_member_lead_consistency` | CHECK (App) | `is_lead` | − | − | App 层验证: `is_lead=TRUE` 时, 该 user_account_id 必须跟 `team.lead_user_account_id` 一致 (DB 层 trigger 实现) |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `team_member_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_team_member_unique` | btree (UK/PT) | `(team_id, user_account_id)` | `deleted_at IS NULL` | 一意 (但跨 team 多重隶属允许) |
| `uq_team_member_primary` | btree (UK/PT) | `(user_account_id)` | `is_primary = TRUE AND deleted_at IS NULL` | 主 team 一意 |
| `idx_team_member_tenant` | btree (PT) | `tenant_id` | `deleted_at IS NULL` | テナント別 |
| `idx_team_member_team` | btree (PT) | `team_id` | `deleted_at IS NULL` | チーム別 |
| `idx_team_member_user_account` | btree (PT) | `user_account_id` | `deleted_at IS NULL` | ユーザーアカウント別 (多重隶属 查询) |
| `idx_team_member_lead` | btree (PT) | `(team_id, is_lead)` | `is_lead = TRUE AND deleted_at IS NULL` | リード検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_team_member_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |
| `trg_team_member_lead_consistency` | BEFORE INSERT OR UPDATE | `public.fn_team_member_lead_consistency()` | `is_lead=TRUE` 时验证跟 `team.lead_user_account_id` 一致 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 25,000 (5,000 user_account × 5 team 平均) |
| 1 年後 | 250,000 |
| 3 年後 | 2,500,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 350 B (含 JSONB metadata) | 2,500,000 | 約 875 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `permission.team` | `team_id` |
| `permission.user_account` | `user_account_id` / `joined_by` |

### 8.2 被参照元

| 被参照元 | FK 列 | 関係 |
|---|---|---|
| `permission.role_per_team` (T83) | `team_member_id` | 1:N (1 team_member → 1+ role_per_team, 跨 team 不同 role) |
| `agent.agent` (T77, TBD per W3-G.9) | `team_member_id` (反范式 or 关联) | N:1 (1 agent 跨 N team, 多重隶属 per team_dimension_opt4) |

---

## 9. RLS Policy

```sql
ALTER TABLE permission.team_member ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.team_member FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_team_member_tenant_isolation ON permission.team_member
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 业务分類根拠 (per 守门 #13 派生规 a/b/c/d 4 段检查清单)

| 派生规 | 判定 | 依据 |
|---|---|---|
| (a) W | **不適用** | team_member 是永続业务事实 (退出履歴保留), 不是短 TTL 作业中数据 |
| (b) T | **適用 ✓** | 物理删除禁止 (`deleted_at` 論理削除保留 audit), audit 必須, RLS 13 類必携 |
| (c) M | **不適用** | M 类是参考 / 設定, team_member 是业务事实, 不是 SCD 慢变 |
| (d) T 100% audit | **✓** | 退出 / 加入事件记录到 `domain-audit.audit_event` |

**主分類**: T (Transaction)
**混合**: 无 (per 守门 #13 派生规)

---

## 11. 已知缺口 (per 守门 #11 缺标比错标安全, DDD Review 必查)

| # | 缺口 | 风险 | 缓解 | 评审 Lead |
|---|---|---|---|---|
| 1 | **多重隶属 (1 user_account → N team) 跨 session 续 W3 G.9 实施** | W1 落档后 schema 允许多重隶属, 但 application 层 (e.g. worktree 分配, agent 调度) 跨 team 选择逻辑在 W3 落地 | W1 落档后 is_primary 字段 ready, W3 G.9 实施 application 层 primary team 选 逻辑 | 架构师 + 5 域 Lead |
| 2 | **跨 team 不同 role (per team_dimension_opt4)** | 1 user_account 在 team A 是 admin, 在 team B 是 viewer, role_per_team (T83) 跨 team 独立, W1 落档 | role_per_team (T83) 跟 team_member (T82) 1:N, 跨 team 不同 role ready | 架构师 + 5 域 Lead |
| 3 | **lead 一致性 trigger** (team.lead_user_account_id 跟 team_member.is_lead=TRUE 一致) | 跨 team lead 跟 team_member 标记不一致, application 层需强制 | W1 trigger `trg_team_member_lead_consistency` 实现 DB 层强制, W3 跨域决策时统一 | 架构师 + 5 域 Lead |
| 4 | **5 域真人 Lead 不到位** (per 守门 #3 + 2026-08-31 22:45 JST 拍板 B 反转) | team lead user_account 字段 ready, 但真人 lead 不到位, Mavis 临时代签决策 | per 拍板 B 反转, Mavis 临时代签, 真人到位后追溯签字 | 5 域 Lead 真人 + 架构师 |

---

## 12. Migration SQL 概略 (per 守门 #13 RLS 100% 强制)

```sql
-- 2026-09-03-create-permission-team-member.sql

CREATE TABLE permission.team_member (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID         NOT NULL REFERENCES tenant.tenant(id) ON DELETE CASCADE,
    team_id         UUID         NOT NULL REFERENCES permission.team(id) ON DELETE CASCADE,
    user_account_id UUID         NOT NULL REFERENCES permission.user_account(id) ON DELETE CASCADE,
    joined_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    joined_by       UUID         REFERENCES permission.user_account(id) ON DELETE SET NULL,
    is_primary      BOOLEAN      NOT NULL DEFAULT FALSE,
    is_lead         BOOLEAN      NOT NULL DEFAULT FALSE,
    metadata        JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

-- 一意制約
CREATE UNIQUE INDEX uq_team_member_unique
    ON permission.team_member (team_id, user_account_id) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX uq_team_member_primary
    ON permission.team_member (user_account_id) WHERE is_primary = TRUE AND deleted_at IS NULL;

-- 検索インデックス
CREATE INDEX idx_team_member_tenant ON permission.team_member (tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_team_member_team ON permission.team_member (team_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_team_member_user_account ON permission.team_member (user_account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_team_member_lead ON permission.team_member (team_id, is_lead)
    WHERE is_lead = TRUE AND deleted_at IS NULL;

-- Trigger
CREATE TRIGGER trg_team_member_updated_at BEFORE UPDATE ON permission.team_member
    FOR EACH ROW EXECUTE FUNCTION public.fn_update_updated_at();

-- RLS
ALTER TABLE permission.team_member ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.team_member FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_team_member_tenant_isolation ON permission.team_member
    USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);

COMMENT ON TABLE permission.team_member IS 'Team ↔ UserAccount M:N 桥接, 多重隶属 (T82, P3-G-W1 G.4)';
COMMENT ON COLUMN permission.team_member.is_lead IS 'team リード (跟 team.lead_user_account_id 一致性 trigger 强制)';
```

---

## 13. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: T82 team_member 实体 (T 类 audit, 多重隶属 1 user → N team per team_dimension_opt4 拍板), 12 カラム, 7 制約, 7 インデックス, 2 トリガー, RLS 1 policy, is_lead 一致性 trigger, 4 已知缺口 | 2026-09-03 P3-G-W1 子项 G.4 落地 |
