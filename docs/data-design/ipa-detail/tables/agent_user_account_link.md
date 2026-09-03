# agent.user_account_link — テーブル詳細設計書

> **テーブル ID**: T84
> **作成日**: 2026-09-03
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.3 §4.21.5 (本子项 G.5 新增, per 9/3 11:50 JST Ulysses Jira 化指令 + 拍板)
> **P3-G-W1 子项**: G.5 (基础层第五子项, per `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` §1.2)
> **Brief**: `docs/briefs/p3-g-w1.md`
> **关键拍板**: 双层 L1 桥接 (user_account + subagent + agent 3 层架构的第一层) per subagent_persist_opt4 拍板

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T84 |
| **物理名** | `agent.user_account_link` |
| **論理名** | エージェント ↔ ユーザーアカウント リンク (双层 L1 桥接) |
| **スキーマ** | `agent` |
| **Module** | `domain-agent` (跟 `agent.agent` T77 平行) |
| **種別** | Entity (Weak) |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** (per 守门 #13 派生规 b, Transaction 100% audit + RLS) |
| **業務分類** | **T (Transaction)** (per 守门 #13 派生规 b, 业务事实 / append-only 適用) |
| **概要** | `agent.agent` (T77) ↔ `permission.user_account` (T78) 1:1 桥接 (双层 L1)。1 agent.agent 可链 1 user_account (e.g. subagent 用 ServiceAccount 账户, 人类用 Human 账户)。W2 G.6 subagent 实体落地后, subagent ↔ user_account N:1 共享 (双层 L3 per subagent_persist_opt4)。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `agent_id` | エージェント ID | UUID | − | NO | − | − | `agent.agent(id)` ON DELETE CASCADE | − | idx (PT) | 所属 agent (1:1, 1 agent → 1 link) |
| 4 | `user_account_id` | ユーザーアカウント ID | UUID | − | NO | − | − | `permission.user_account(id)` ON DELETE CASCADE | − | idx (PT) | 链 user_account (1:1, 1 user_account → 1+ agent link per W2 N:1) |
| 5 | `link_type` | リンク種別 | VARCHAR | 32 | NO | − | − | − | − | idx (PT) | `Primary` / `Secondary` / `Temporary` (跟 agent ↔ user 关系类型) |
| 6 | `linked_at` | リンク日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 链時刻 (audit 用) |
| 7 | `linked_by` | リンク承認者 | UUID | − | YES | `NULL` | − | `permission.user_account(id)` ON DELETE SET NULL | − | − | 承認者 user_account (audit 用) |
| 8 | `valid_from` | 有効開始 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 期間有効 (link 期間管理) |
| 9 | `valid_until` | 有効終了 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | idx (PT) | 期間有効 (NULL = 無期限) |
| 10 | `metadata` | メタデータ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 自由形式 (链理由, 関連 ticket, ...) |
| 11 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 12 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 13 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 (解链履歴保留, audit) |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `user_account_link_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_user_account_link_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_user_account_link_agent` | FOREIGN KEY | `agent_id` | `agent.agent(id)` | CASCADE | − |
| `fk_user_account_link_user_account` | FOREIGN KEY | `user_account_id` | `permission.user_account(id)` | CASCADE | − |
| `fk_user_account_link_linked_by` | FOREIGN KEY | `linked_by` | `permission.user_account(id)` | SET NULL | − |
| `uq_user_account_link_agent` | UNIQUE (PT) | `agent_id` | `WHERE deleted_at IS NULL` | − | 1 agent 最多 1 active link (双层 L1 1:1 强制) |
| `uq_user_account_link_unique` | UNIQUE (PT) | `(agent_id, user_account_id, link_type, valid_from)` | `WHERE deleted_at IS NULL` | − | 同一 (agent, user_account, link_type, valid_from) 唯一 (期间重複不可) |
| `ck_user_account_link_type` | CHECK | `link_type` | `IN ('Primary', 'Secondary', 'Temporary')` | − | 3 値 (链类型) |
| `ck_user_account_link_valid_range` | CHECK | `valid_from` / `valid_until` | `valid_until IS NULL OR valid_until > valid_from` | − | 期間整合 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `user_account_link_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_user_account_link_agent` | btree (UK/PT) | `agent_id` | `deleted_at IS NULL` | 1 agent 1 active link (双层 L1 1:1) |
| `uq_user_account_link_unique` | btree (UK/PT) | `(agent_id, user_account_id, link_type, valid_from)` | `deleted_at IS NULL` | 期間重複不可 |
| `idx_user_account_link_tenant` | btree (PT) | `tenant_id` | `deleted_at IS NULL` | テナント別 |
| `idx_user_account_link_user_account` | btree (PT) | `user_account_id` | `deleted_at IS NULL` | ユーザーアカウント別 (W2 N:1 共享查询) |
| `idx_user_account_link_validity` | btree (PT) | `(agent_id, valid_from, valid_until)` | `deleted_at IS NULL` | 期間検索 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_user_account_link_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |
| `trg_user_account_link_no_overlap` | BEFORE INSERT OR UPDATE | `public.fn_user_account_link_no_overlap()` | 同一 (agent, user_account, link_type) 期间重複不可 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 5,500 (5,000 agent.agent + 500 双层 L3 N:1 共享) |
| 1 年後 | 55,000 |
| 3 年後 | 550,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 400 B (含 JSONB metadata) | 550,000 | 約 220 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `agent.agent` (T77) | `agent_id` |
| `permission.user_account` (T78) | `user_account_id` / `linked_by` |

### 8.2 被参照元

| 被参照元 | FK 列 | 関係 |
|---|---|---|
| (W2 G.6 `agent.subagent` 反向引用) | `user_account_id` (T78 共享) | N:1 (1 user_account → 1+ subagent, 双层 L3) |
| `domain-audit.audit_event` | `target_id` (T84 row id) | 链 / 解链 事件 audit |

---

## 9. RLS Policy

```sql
ALTER TABLE agent.user_account_link ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent.user_account_link FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_user_account_link_tenant_isolation ON agent.user_account_link
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 业务分類根拠 (per 守门 #13 派生规 a/b/c/d 4 段检查清单)

| 派生规 | 判定 | 依据 |
|---|---|---|
| (a) W | **不適用** | user_account_link 是永続业务事实 (解链履歴保留), 不是短 TTL 作业中数据 |
| (b) T | **適用 ✓** | 物理删除禁止 (`deleted_at` 論理削除保留 audit), audit 必須, RLS 13 類必携 |
| (c) M | **不適用** | M 类是参考 / 設定, user_account_link 是业务事实 |
| (d) T 100% audit | **✓** | 链 / 解链 事件记录到 `domain-audit.audit_event` |

**主分類**: T (Transaction)
**混合**: 无 (per 守门 #13 派生规)

---

## 11. 已知缺口 (per 守门 #11 缺标比错标安全, DDD Review 必查)

| # | 缺口 | 风险 | 缓解 | 评审 Lead |
|---|---|---|---|---|
| 1 | **1:1 双层 L1 vs W2 N:1 双层 L3 矛盾** (W1 G.5 落 1:1 强制, W2 G.6 subagent 实体需 N:1 共享 user_account) | 1 agent 1 user_account 强制 → subagent 跟 agent.agent 共享 user_account 时 触发约束冲突 | W1 uq_user_account_link_agent (1 agent 1 link) 暂留, W2 G.6 落地时改 N:1 (多 subagent 共享 1 user_account), 改 `uq_user_account_link_agent` → `uq_user_account_link_agent_type` (1 agent + type 1 link) | 架构师 + 5 域 Lead |
| 2 | **link_type 3 値 (Primary/Secondary/Temporary) 跟 subagent 实体 (W2 G.6) subagent_type 3 値 命名空间不直接对齐** | 跨 stage 类型映射需 application 层映射 | W1 落档后 link_type 3 値 ready, W2 G.6 subagent 实体 subagent_type 3 値独立, 跨 stage 映射在 application 层 | 架构师 + 5 域 Lead |
| 3 | **期间重複不可 trigger** (`fn_user_account_link_no_overlap`) 跨 session 续 W3 跨域决策 | W1 落档后 trigger 是 stub, 实际重複检查 留 W3 实施 | W1 落档 trigger 名 ready, W3 实施时具体 逻辑 (range overlap check via SQL) | 架构师 + 5 域 Lead |
| 4 | **5 域真人 Lead 不到位** (per 守门 #3 + 2026-08-31 22:45 JST 拍板 B 反转) | user_account_link 字段 ready, 但真人 lead 不到位, Mavis 临时代签决策 | per 拍板 B 反转, Mavis 临时代签, 真人到位后追溯签字 | 5 域 Lead 真人 + 架构师 |

---

## 12. Migration SQL 概略 (per 守门 #13 RLS 100% 强制)

```sql
-- 2026-09-03-create-agent-user-account-link.sql

CREATE TABLE agent.user_account_link (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID         NOT NULL REFERENCES tenant.tenant(id) ON DELETE CASCADE,
    agent_id        UUID         NOT NULL REFERENCES agent.agent(id) ON DELETE CASCADE,
    user_account_id UUID         NOT NULL REFERENCES permission.user_account(id) ON DELETE CASCADE,
    link_type       VARCHAR(32)  NOT NULL,
    linked_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    linked_by       UUID         REFERENCES permission.user_account(id) ON DELETE SET NULL,
    valid_from      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    valid_until     TIMESTAMPTZ,
    metadata        JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

-- 制約
ALTER TABLE agent.user_account_link
    ADD CONSTRAINT ck_user_account_link_type
        CHECK (link_type IN ('Primary', 'Secondary', 'Temporary')),
    ADD CONSTRAINT ck_user_account_link_valid_range
        CHECK (valid_until IS NULL OR valid_until > valid_from);

-- 一意制約
CREATE UNIQUE INDEX uq_user_account_link_agent
    ON agent.user_account_link (agent_id) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX uq_user_account_link_unique
    ON agent.user_account_link (agent_id, user_account_id, link_type, valid_from)
    WHERE deleted_at IS NULL;

-- 検索インデックス
CREATE INDEX idx_user_account_link_tenant ON agent.user_account_link (tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_user_account_link_user_account ON agent.user_account_link (user_account_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_user_account_link_validity
    ON agent.user_account_link (agent_id, valid_from, valid_until) WHERE deleted_at IS NULL;

-- Trigger
CREATE TRIGGER trg_user_account_link_updated_at BEFORE UPDATE ON agent.user_account_link
    FOR EACH ROW EXECUTE FUNCTION public.fn_update_updated_at();

-- RLS
ALTER TABLE agent.user_account_link ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent.user_account_link FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_user_account_link_tenant_isolation ON agent.user_account_link
    USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);

COMMENT ON TABLE agent.user_account_link IS 'Agent ↔ UserAccount 1:1 桥接 (双层 L1, T84, P3-G-W1 G.5)';
COMMENT ON COLUMN agent.user_account_link.link_type IS 'Primary / Secondary / Temporary (双层 L1 链类型)';
```

---

## 13. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: T84 user_account_link 实体 (T 类 audit, 双层 L1 桥接 per subagent_persist_opt4 拍板), 13 カラム, 9 制約, 6 インデックス, 2 トリガー, RLS 1 policy, 1:1 强制 (uq_user_account_link_agent), 4 已知缺口 (含 W2 N:1 改 prepare) | 2026-09-03 P3-G-W1 子项 G.5 落地 |
