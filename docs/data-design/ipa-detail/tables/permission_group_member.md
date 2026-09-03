# permission.group_member — テーブル詳細設計書

> **テーブル ID**: T80
> **作成日**: 2026-09-03
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.3 §4.16.6 (本子项 G.2 新增, per 9/3 11:50 JST Ulysses Jira 化指令 + 拍板)
> **P3-G-W1 子项**: G.2 (基础层第二子项, per `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` §1.2)
> **Brief**: `docs/briefs/p3-g-w1.md`

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T80 |
| **物理名** | `permission.group_member` |
| **論理名** | グループメンバー (M:N 桥接 user_account ↔ group) |
| **スキーマ** | `permission` |
| **Module** | `domain-permission` |
| **種別** | Entity (Weak) |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** (per 守门 #13 派生规 b, Transaction 100% audit + RLS) |
| **業務分類** | **T (Transaction)** (per 守门 #13 派生规 b, 业务事实 / append-only 適用) |
| **概要** | user_account (T78) ↔ group (T79) M:N 桥接。1 user_account 可属 N group, 1 group 可含 N user_account。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `group_id` | グループ ID | UUID | − | NO | − | − | `permission.group(id)` ON DELETE CASCADE | − | idx (PT) | 所属 group |
| 4 | `user_account_id` | ユーザーアカウント ID | UUID | − | NO | − | − | `permission.user_account(id)` ON DELETE CASCADE | − | idx (PT) | 所属 user_account |
| 5 | `joined_at` | 参加日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 参加時刻 (audit 用) |
| 6 | `joined_by` | 参加承認者 | UUID | − | YES | `NULL` | − | `permission.user_account(id)` ON DELETE SET NULL | − | − | 承認者 user_account (audit 用) |
| 7 | `is_primary` | 主所属 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 主 group 標識 (1 user_account 最多 1 primary group, 業務主担当) |
| 8 | `metadata` | メタデータ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | 自由形式 (加入理由, 関連 ticket, ...) |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 11 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 (退出履歴保持, audit) |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `group_member_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_group_member_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_group_member_group` | FOREIGN KEY | `group_id` | `permission.group(id)` | CASCADE | − |
| `fk_group_member_user_account` | FOREIGN KEY | `user_account_id` | `permission.user_account(id)` | CASCADE | − |
| `fk_group_member_joined_by` | FOREIGN KEY | `joined_by` | `permission.user_account(id)` | SET NULL | − |
| `uq_group_member_unique` | UNIQUE (PT) | `(group_id, user_account_id)` | `WHERE deleted_at IS NULL` | − | 同一 (group, user_account) 唯一 |
| `ck_group_member_primary_unique` | UNIQUE (PT, partial) | `(user_account_id)` | `WHERE is_primary = TRUE AND deleted_at IS NULL` | − | 1 user_account 最多 1 primary group |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `group_member_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_group_member_unique` | btree (UK/PT) | `(group_id, user_account_id)` | `deleted_at IS NULL` | 一意 |
| `uq_group_member_primary` | btree (UK/PT) | `(user_account_id)` | `is_primary = TRUE AND deleted_at IS NULL` | 主 group 一意 |
| `idx_group_member_tenant` | btree (PT) | `tenant_id` | `deleted_at IS NULL` | テナント別 |
| `idx_group_member_group` | btree (PT) | `group_id` | `deleted_at IS NULL` | グループ別 |
| `idx_group_member_user_account` | btree (PT) | `user_account_id` | `deleted_at IS NULL` | ユーザーアカウント別 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | 関数 | 説明 |
|---|---|---|---|
| `trg_group_member_updated_at` | BEFORE UPDATE | `public.fn_update_updated_at()` | `updated_at` 自動更新 |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 25,000 (5,000 user_account × 5 group 平均) |
| 1 年後 | 250,000 |
| 3 年後 | 2,500,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 300 B (含 JSONB metadata) | 2,500,000 | 約 750 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `permission.group` | `group_id` |
| `permission.user_account` | `user_account_id` / `joined_by` |

### 8.2 被参照元

| 被参照元 | FK 列 | 関係 |
|---|---|---|
| (无直接被参照) | − | group_member 是終端实体, audit 透过 `domain-audit` 独立表记录 |

---

## 9. RLS Policy

```sql
ALTER TABLE permission.group_member ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.group_member FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_group_member_tenant_isolation ON permission.group_member
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 业务分類根拠 (per 守门 #13 派生规 a/b/c/d 4 段检查清单)

| 派生规 | 判定 | 依据 |
|---|---|---|
| (a) W = 物理删除 / タイマー失効 / 短 TTL 明示 retention | **不適用** | group_member 是永続业务事实 (退出履歴保留), 不是短 TTL 作业中数据 |
| (b) T = 物理删除禁止 + 監査必須 + RLS 13 類必携 | **適用 ✓** | 物理删除禁止 (`deleted_at` 論理削除保留 audit), audit 必須 (透过 `domain-audit` 追加退出事件), RLS 13 類必携 |
| (c) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携 | **不適用** | M 类是参考 / 設定, group_member 是业务事实, 不是 SCD 慢变 |
| (d) Transaction 100% audit | **T 100% audit ✓** | 退出事件记录到 `domain-audit.audit_event` (per `docs/architecture/2026-08-26-upgrade/spec/flows/07-audit-model.md`) |

**主分類**: T (Transaction)
**混合**: 无 (不混合 M/T, per 守门 #13 派生规)

---

## 11. 已知缺口 (per 守门 #11 缺标比错标安全, DDD Review 必查)

| # | 缺口 | 风险 | 缓解 | 评审 Lead |
|---|---|---|---|---|
| 1 | `is_primary` 主 group 唯一性 跟 `permission.user_account` lifecycle 状态联动 (e.g. user_account archived → primary group 自动转移) | 主 group 业务主担当 角色 转移逻辑, W1 不实现, 留 W3 G.10 Permission Scheme 跨 team 落地时统一 | W1 落档后 is_primary 字段 ready, 主 group 转移逻辑 跨 session 续 | 架构师 + 5 域 Lead |
| 2 | Group 退出 (group_member 論理削除) 跟 user_account 退出 (user_account 論理削除) 联动 (CASCADE ON DELETE 已设) | user_account 物理删除会 cascade 删 group_member, 但 論理削除 (deleted_at) 不会, 可能出现"用户已 archived 但 group_member 仍 active" 不一致 | 应用层在 user_account lifecycle → archived 时显式标记 group_member.deleted_at, 留 W3 跨域决策 | 架构师 + 5 域 Lead |

---

## 12. Migration SQL 概略 (per 守门 #13 RLS 100% 强制)

```sql
-- 2026-09-03-create-permission-group-member.sql

CREATE TABLE permission.group_member (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID         NOT NULL REFERENCES tenant.tenant(id) ON DELETE CASCADE,
    group_id        UUID         NOT NULL REFERENCES permission.group(id) ON DELETE CASCADE,
    user_account_id UUID         NOT NULL REFERENCES permission.user_account(id) ON DELETE CASCADE,
    joined_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    joined_by       UUID         REFERENCES permission.user_account(id) ON DELETE SET NULL,
    is_primary      BOOLEAN      NOT NULL DEFAULT FALSE,
    metadata        JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

-- 一意制約
CREATE UNIQUE INDEX uq_group_member_unique
    ON permission.group_member (group_id, user_account_id) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX uq_group_member_primary
    ON permission.group_member (user_account_id) WHERE is_primary = TRUE AND deleted_at IS NULL;

-- 検索インデックス
CREATE INDEX idx_group_member_tenant ON permission.group_member (tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_group_member_group ON permission.group_member (group_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_group_member_user_account ON permission.group_member (user_account_id) WHERE deleted_at IS NULL;

-- Trigger
CREATE TRIGGER trg_group_member_updated_at BEFORE UPDATE ON permission.group_member
    FOR EACH ROW EXECUTE FUNCTION public.fn_update_updated_at();

-- RLS
ALTER TABLE permission.group_member ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.group_member FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_group_member_tenant_isolation ON permission.group_member
    USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);

COMMENT ON TABLE permission.group_member IS 'Group ↔ UserAccount M:N 桥接 (T80, P3-G-W1 G.2)';
```

---

## 13. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: T80 group_member 实体 (T 类, audit + RLS 13 類), 11 カラム, 6 制約, 6 インデックス, 1 トリガー, RLS 1 policy, is_primary 主 group 唯一, 2 已知缺口 | 2026-09-03 P3-G-W1 子项 G.2 落地 |
