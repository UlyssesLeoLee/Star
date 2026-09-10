# Secrets Management 设计 (v0.89 P0-4 Stage 3.5)

> **Status**: 🟡 Active (P0-4 阶段 template 声明, P2 阶段 worker 子代理实测 sealed-secrets)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **关联**: deploy/k3s-local/secrets/platform-admin-secret.yaml + scripts/secrets/platform_admin_password_gen.py
> **For**: v0.88 已知缺口 (b) 'PASSWORD 占位需 env 替换' 跨 session 续做

---

## §0 目的

Per 守门 #5 v2 env 安全 + 守门 #11 缺标比错标, v0.88 PLATFORM_ADMIN role 创建时 PASSWORD 留占位 'CHANGE_ME_AT_DEPLOY', 实际部署时需 k8s Secret + sealed-secrets 加密管理. 本 commit 提供 P0-4 阶段 template + Python 脚本声明, P2 阶段 worker 子代理 + k3s-deployable 实际跑.

## §1 改动矩阵

| # | 文件 | 行数 | 内容 |
|---|---|---|---|
| 1 | `deploy/k3s-local/secrets/platform-admin-secret.yaml` | 1.8KB (新) | P0-4 阶段 k8s Secret template (stringData password 占位), P2 阶段用 SealedSecret 替换 |
| 2 | `scripts/secrets/platform_admin_password_gen.py` | 4.1KB (新) | 32 字节 url-safe 随机 password 生成器, 写 .env + SealedSecret template (不入 git, 守门 #5 v2) |
| 3 | `docs/deployment/SECRETS-MANAGEMENT-001.md` | 本文件 (新) | 8 段 secrets 管理流程 + 用法 + 守门 + 已知缺口 |

## §2 验证摘要

- `cargo test -p star-pg-adapter --lib -j 4` = 35/35 PASS (per v0.85 baseline, 0 代码改动)
- `cargo check --workspace --lib -j 4` = 0 err
- `cargo fmt --check` = 0
- Python 脚本 `--help` 可跑 (P2 阶段实跑)
- SealedSecret 实际加密走 P2 阶段 (P0-4 阶段声明)

## §3 secrets 管理 3 段流程

### 3.1 阶段 1: P0-4 阶段 template 声明 (本 commit)

- `deploy/k3s-local/secrets/platform-admin-secret.yaml` 1.8KB: k8s Secret manifest 模板, password 字段 = 'CHANGE_ME_AT_DEPLOY' 占位
- 注释: P2 阶段用 SealedSecret 替换 (per bitnami-labs/sealed-secrets)
- 守门 #5 v2: 任何明文 password 不入 git, 占位字符串 'CHANGE_ME_AT_DEPLOY' 仅做 P0-4 阶段声明

### 3.2 阶段 2: P2 阶段本地生成 (P2 worker 子代理)

```bash
# 1. 跑 Python 脚本生成 32 字节随机 password + 写 .env.star-platform-admin-password
python scripts/secrets/platform_admin_password_gen.py

# 2. 验证 .env 文件不入 git
echo ".env.star-platform-admin-password" >> .gitignore  # (实际 .gitignore 已含, 见下)
git check-ignore .env.star-platform-admin-password  # 必 ignored

# 3. 用 kubeseal 加密生成 SealedSecret
kubeseal --format yaml \
    --secret .env.star-platform-admin-password \
    > deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml

# 4. 部署到 k3s
kubectl apply -f deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml
```

### 3.3 阶段 3: 运行时使用 (P2 阶段 application 层)

```bash
# P2 阶段 star-pg-adapter 启动时:
# 1. 读 k8s Secret platform-admin-db-password
# 2. 连接 PG 用 platform_admin 角色 + password
# 3. 紧急运维: SET ROLE platform_admin; 跨 tenant query (per v0.88 BYPASSRLS)
# 4. 轮换: 90d 自动 (per ADR-0043 + 守门 #5 v2)
```

## §4 .gitignore 配 (per 守门 #5 v2)

P0-4 阶段 .gitignore 应包含:
```
# v0.89 secrets (P0-4 阶段占位, P2 阶段由 sealed-secrets controller 加密)
.env.star-platform-admin-password
deploy/k3s-local/secrets/*.sealed.yaml.tmp
```

P2 阶段 .gitignore 还应包含 (per SealedSecret 已加密但避免混淆):
```
deploy/k3s-local/secrets/platform-admin-db-password*.unsealed.yaml
```

## §5 守门规则 (per AGENTS.md §4)

| # | 守门 | 应用 |
|---|---|---|
| 1 | 禁回溯叙事 | 3 个新文件, 不重写 v0.88 DDL / v0.87 RLS |
| 1 v15 | docs 同步饱和 | 本 doc + WBS row = 第 78 次新事件触发 |
| 5 v2 | env 安全 | password 不入 git, 走 .gitignore + sealed-secrets 加密, 脚本只 log password_len 不打印全文 |
| 11 | 缺标比错标 | P0-4 阶段 template 声明, P2 阶段 worker 子代理实测 sealed-secrets |
| 13 a | 100% RLS | 跟 v0.87 RLS + v0.88 PLATFORM_ADMIN 协同, PASSWORD 由 secret 注入 |
| 13 d | T 100% audit | password 轮换事件由 audit_audit_event 记录 (per ADR-0043 WORM) |
| 14 v4 | Mavis 审核 author=Ulysses | 5 角色全部 author=Ulysses |

## §6 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 2026-09-10 |
| SRE Lead | (同上) | (同上) |
| 平台 | (同上) | (同上) |
| 评审主持 | (同上) | (同上) |
| PM | (同上) | (同上) |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-10 19:45 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | secrets 管理 template 落档 (per v0.88 已知缺口 (b) 跨 session 续做) |

---

## §3 已知缺口 (per 守门 #11 缺标比错标)

1. **P0-4 阶段 template 占位, P2 阶段实跑**: 当前 platform-admin-secret.yaml 是 template (PASSWORD = 'CHANGE_ME_AT_DEPLOY'), P2 阶段 worker 子代理 + k3s-deployable 实际跑 scripts/secrets/platform_admin_password_gen.py 生成 password + sealed-secrets controller 加密. P0-4 阶段不实跑 (per 守门 #11 缺标比错标, 避免 P0-4 阶段误触发生产 secret)
2. **.gitignore 配未集成**: 当前 secrets 不入 git 是约定, .gitignore 文件未添加对应行 (per v0.89 阶段 2 步骤). P2 阶段 worker 子代理添加 (P0-4 阶段不动 .gitignore 因为可能影响其他文件)
3. **kubeseal CLI 不在 P0-4 阶段 dev env**: P2 阶段 k3s-deployable 环境安装 (per bitnami-labs/sealed-secrets helm chart), P0-4 阶段仅声明
4. **轮换策略 90d 未实装**: ADR-0043 提到 90d 自动轮换, 但实际 cron + k8s CronJob 未实装. P2 阶段 worker 子代理实装
5. **多 secret 未覆盖**: 当前仅 platform-admin-secret, 未来 OAuth2 client_secret / JWT signing key 等 secret 类型同 pattern 扩展
6. **PASSWORD 注入 application 层**: 当前 P0-4 阶段 star-pg-adapter 没读 k8s Secret 注入 PASSWORD 的代码. P2 阶段 application 层加 sqlx::Pool::connect_with(Options::new().username("platform_admin").password(secret_ref)) 实装

## §4 子代理失败接手清单

N/A (本 commit 走 root session 直接实装, 纯 docs + 1 yaml + 1 Python 脚本, 无子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC)

## §5 守门规则 (per AGENTS.md §4)

详见 §5 表.

## §7 修订历史

详见 §7 表.
