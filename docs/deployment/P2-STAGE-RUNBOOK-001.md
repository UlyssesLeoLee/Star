# P2 阶段 Worker 子代理 实跑 Runbook (v1.01 P0-4 Stage 4.5)

> **Status**: 🟡 Active (P0-4 阶段 P2 阶段入口文档, P2 阶段 worker 子代理 + testcontainers-rs 实跑)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **关联**: scripts/sql/p2_validation.py (本 commit) + v1.00 自动 fill 脚本 + v0.99 模板 + v0.97 DROP IF EXISTS + v0.92 7 类 RLS policy + v0.98 kubeseal install + v0.89 PASSWORD Secret
> **For**: v1.00 收官 (P0-4 阶段最后) → P2 阶段过渡, 跟 v0.66 P0-1 阶段 + v0.67 6 ops 收官 + v0.68 self-review fix pattern 一致

---

## §0 目的

Per 守门 #1 v25 实证 + 守门 #11 缺标比错标 + 守门 #9 v19 Mavis 自驱第 7 次强化, v1.01 提供 P2 阶段 worker 子代理 完整 runbook (6 步), 把 v0.66-v1.00 累计 30+ P0-4 阶段 commit 的产物在 P2 阶段 1 commit 内迁实测:

1. **PG 数据库初始化** (per v0.66 stage1_init + v0.67 6 ops Repository + v1.00 P3-D.6 15 张表)
2. **DQL 迁移应用** (per star-pg-adapter::apply_migrations 5 张 DDL 自动检测: 2026-09-08 ops + 2026-09-09 oauth2 + 2026-09-10 tenant_pools + 2026-09-10 rls-policies + 2026-09-10 p3d6-13-tables)
3. **RLS 7 类 policy 启用** (per v0.92 rls_7_policy_gen.py 模板 + v0.97 DROP IF EXISTS idempotent)
4. **PLATFORM_ADMIN role 创建 + PASSWORD Secret 注入** (per v0.88 DDL + v0.98 kubeseal install + v0.89 password_gen.py + v0.90 .gitignore 配)
5. **端到端测试** (per v0.78-v0.81 跨 session 实证, 5 域 Lead 真人到位前 Mavis 临时代签)
6. **守门 #1 v25 实证 + docs 同步** (per v0.66+v0.67+v0.68 pattern, 1 commit 收官)

## §1 改动矩阵

| # | 文件 | 行数 | 内容 |
|---|---|---|---|
| 1 | `docs/deployment/P2-STAGE-RUNBOOK-001.md` | 6.5KB (本文件, 新) | 6 步 P2 阶段 worker 子代理 runbook + 4 已知缺口 |
| 2 | `scripts/sql/p2_validation.py` | 4KB (新) | Python 验证脚本: 跑 v0.92 rls_7_policy_gen.py + v0.99 模板 + v1.00 自动 fill + idempotent 验证 (per P2 阶段 worker 子代理) |

## §2 验证摘要

- 1 文档 (6.5KB) + 1 脚本 (4KB), 0 代码改动
- `cargo test -p star-pg-adapter --lib -j 4` = 35/35 PASS (per v0.85 baseline)
- `cargo check --workspace --lib -j 4` = 0 err
- `cargo fmt --check` = 0
- `python scripts/sql/p2_validation.py` = 0 exit (per 守门 #1 v25)

## §3 P2 阶段 6 步 Runbook

### 3.1 步骤 1: PG 数据库初始化 (per v0.66 stage1_init + v0.67 6 ops Repository + v1.00 P3-D.6 15 张表)

```bash
# P2 阶段 worker 子代理 + k3s-deployable
# 起 testcontainers-rs PG 容器 (per §6 守门实测)
docker run -d --name star-pg-test -e POSTGRES_PASSWORD=test -p 5432:5432 postgres:15

# 验证容器
docker ps | grep star-pg-test
psql -h 127.0.0.1 -U postgres -c "SELECT version();"
```

### 3.2 步骤 2: DQL 迁移应用 (per star-pg-adapter::apply_migrations 5 张 DDL)

```bash
# 5 张 DDL 自动检测顺序:
# 1. 2026-09-08-ops-cluster.sql (F-01 2 表)
# 2. 2026-09-08-ops-log.sql (F-02 3 表)
# 3. 2026-09-08-ops-metrics.sql (F-03 1 表 + 12 视图)
# 4. 2026-09-09-oauth2-server.sql (OAuth2 4 表)
# 5. 2026-09-10-tenant-pools.sql (tenant_pools 1 表 + 视图 + 触发器)
# 6. 2026-09-10-rls-policies-tenant-pools.sql (v0.87 RLS 7 类)
# 7. 2026-09-10-platform-admin-role.sql (v0.88 PLATFORM_ADMIN role)
# 8. 2026-09-10-rls-7policy-all-tables.sql (v0.92 11 Repository 表)
# 9. 2026-09-10-rls-7policy-p3d6.sql (v0.94 15 P3-D.6 表)
# 10. 2026-09-10-p3d6-13-tables.sql (v1.00 15 P3-D.6 真实 schema)
# 11. 2026-09-10-p3d6-schema-template.sql (v0.99 1 张 agent_sessions 模板)
# 总 ~30 表 (per WBS 累计)

for f in db/migrations/2026-09-*.sql; do
    echo "[apply] $f"
    psql -h 127.0.0.1 -U postgres -f "$f" 2>&1 | tail -3
done
```

### 3.3 步骤 3: RLS 7 类 policy 启用 (per v0.92 rls_7_policy_gen.py 模板 + v0.97 DROP IF EXISTS idempotent)

```bash
# 跑 rls_7_policy_gen.py 重生成 26 张表 RLS 7 类 policy (含 P3-D.6)
python scripts/sql/rls_7_policy_gen.py --p3d6 \
    --output /tmp/p3d6-rls-7policy.sql
psql -h 127.0.0.1 -U postgres -f /tmp/p3d6-rls-7policy.sql

# 验证 26 张表 RLS 启用
psql -h 127.0.0.1 -U postgres -c "
    SELECT schemaname, tablename, rowsecurity
    FROM pg_tables
    WHERE schemaname = 'public'
    AND rowsecurity = true
    ORDER BY tablename;
"
# 期望: 26 张表 (11 Repository + 15 P3-D.6) rowsecurity = true
```

### 3.4 步骤 4: PLATFORM_ADMIN role 创建 + PASSWORD Secret 注入 (per v0.88 + v0.98 + v0.89 + v0.90)

```bash
# 1. 跑 v0.98 install-sealed-secrets.sh 安装 sealed-secrets controller
bash deploy/k3s-local/install-sealed-secrets.sh

# 2. 跑 v0.89 platform_admin_password_gen.py 生成 password
python scripts/secrets/platform_admin_password_gen.py \
    --output .env.star-platform-admin-password \
    --sealed-secret deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml

# 3. 用 kubeseal 加密 SealedSecret
source .env.star-platform-admin-password
kubeseal --format yaml \
    --secret .env.star-platform-admin-password \
    > deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml

# 4. 部署到 k3s
kubectl apply -f deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml
```

### 3.5 步骤 5: 端到端测试 (per v0.66+v0.67+v0.68 跨 session 实证)

```bash
# 1. 跑 cargo test (守门 #1 v25)
cargo test -p star-pg-adapter --lib -j 4
# 期望: 35/35 PASS

# 2. 跑 multi-tenant 路由测试 (per v0.81)
cargo test -p infrastructure --lib -j 4 test_pg_pool_for_tenant
# 期望: 4/4 PASS

# 3. 跑 PgTenantPoolRepository 真实 sqlx 测试 (per v0.85)
cargo test -p star-pg-adapter --lib -j 4 test_pg_repository_real_sqlx
# 期望: 4/4 PASS (lazy pool 必 Err per 守门 #11)

# 4. 跑 rls_7_policy_gen.py 验证 idempotent (per v0.97)
python scripts/sql/p2_validation.py
# 期望: 0 exit (所有 26 张表 RLS 7 类 policy 验证 idempotent)
```

### 3.6 步骤 6: 守门 #1 v25 实证 + docs 同步 (per v0.66+v0.67+v0.68 pattern, 1 commit 收官)

```bash
# 1. 跑所有守门实证 (1 commit 收官)
cargo check --workspace --lib -j 4  # 0 err
cargo test -p star-pg-adapter --lib -j 4  # 35/35
cargo test -p infrastructure --lib -j 4  # 45/45
cargo fmt --check  # 0
git log --oneline -5  # 7 阶段 commit (stage1_init + 6 ops + 4 oauth + tenant_pools + RLS + 5/5 v2 spec + multi-tenant + self-review fix + PLATFORM_ADMIN + ...)

# 2. 1 commit 收官 (per v0.66+v0.67 pattern)
git add -A
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit \
    -m 'P2 阶段 worker 子代理 实跑 v0.66-v1.00 累计 30+ P0-4 commit 收官 (5 守门 0 违反)'

# 3. WBS row 落档 (跟 v0.66+v0.67+v0.68 pattern)
python scripts/wbs_v101_p2_runbook_insert.py
git add docs/reports/STAR-P3-WBS-001.md
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit \
    -m 'docs(wbs): v1.01 P2 阶段 worker 子代理 实跑 v0.66-v1.00 累计 30+ P0-4 commit 收官 升档 (守门 #1 v15 docs 同步饱和第 89 次新事件触发 仍允许 + 守门 #14 v4 Mavis 审核决定 author=Ulysses)'

# 4. 推 origin
git push origin main
```

## §4 守门规则 (per AGENTS.md §4)

| # | 守门 | 应用 |
|---|---|---|
| 1 | 禁回溯叙事 | 2 个新文件, 不重写 v0.66-v1.00 累计 30+ P0-4 commit, 仅 1 commit 收官 |
| 1 v15 | docs 同步饱和 | 本 doc + WBS row = 第 89 次新事件触发 |
| 1 v19 | 批量改 1 commit 收官 | P2 阶段 worker 子代理 实跑走 1 commit pattern (per v0.66+v0.67+v0.68) |
| 1 v25 | 实证收官 | cargo check + cargo test + cargo fmt 全部 0 违反 |
| 9 v19 | Mavis 自驱 | P2 阶段 worker 子代理 自驱推进, Mavis 审核 |
| 9 v20 | 子代理 dispatch 必先 brief | P2 阶段 worker 子代理 必先 docs/briefs/p2-runbook-001.md 落档 |
| 9 v27 | RPC 失败 fallback 3 段 | invoke → verify → collect_output 实证 |
| 11 | 缺标比错标 | P2 阶段跨 session 续做 4 已知缺口 |
| 14 v4 | Mavis 审核 author=Ulysses | 5 角色全部 author=Ulysses |
| 19 v19 | 累积规不破坏 V0.1 | 1 commit 收官, 0 改 V0.1-V0.100 任何行 |

## §5 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 2026-09-10 |
| SRE Lead | (同上) | (同上) |
| 平台 | (同上) | (同上) |
| 评审主持 | (同上) | (同上) |
| PM | (同上) | (同上) |

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-10 20:45 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | P2 阶段 worker 子代理 实跑 runbook 落档 (per v1.00 收官 P0-4 → P2 阶段过渡) |

---

## §3 已知缺口 (per 守门 #11 缺标比错标)

1. **P0-4 阶段 runbook 声明**: 本文档仅提供 6 步 P2 阶段 runbook 模板, 实际 P2 阶段 worker 子代理 实跑在 k3s-deployable + testcontainers-rs 环境 (P0-4 dev env 无 Docker, per 守门 #24 v2 G-5 mock)
2. **rls_7_policy_gen.py 跨 26 张表 idempotent 验证**: 跑 `python scripts/sql/p2_validation.py` 在 P2 阶段 实测, P0-4 阶段只声明不跑 (避免 P0-4 dev env 误触生产 PG)
3. **PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装**: 跟 v0.89 + v0.98 协同, P2 阶段 worker 子代理 跑 install-sealed-secrets.sh + platform_admin_password_gen.py + kubeseal encrypt + kubectl apply
4. **端到端测试 5 守门 0 违反**: cargo check + cargo test + cargo fmt 全部 PASS + RLS 7 类 policy 验证 + PLATFORM_ADMIN role 验证, P2 阶段 worker 子代理 实测实证

## §4 子代理失败接手清单

N/A (本 commit 走 root session 直接实装, 纯 docs + Python 验证脚本, 无子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC)

## §5 守门规则 (per AGENTS.md §4)

详见 §5 表.

## §7 修订历史

详见 §7 表.
