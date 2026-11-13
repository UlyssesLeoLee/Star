# PHASE-STAR-STABLE-W6-UAT-NPM-20261102

## 0. 目的

记录 W6 的本地 UAT 辅助工具从 pnpm 调用迁移到 npm 调用。此变更仅覆盖历史 UAT 文档和脚本，不代表 Web package manager/lockfile、Node 版本或 Vercel 生产闭环已经统一或验收。

固定目标基线：`dev@d3611583cacc06770cee23f2ffb49c67f8b56df9`。本 lane：`codex/star-stable-w6-uat-npm-20261102`。

## 1. 改动矩阵

| 文件 | 变更 | 状态 |
|---|---|---|
| `tools/star-flash-mock/scripts/README.md` | 将 UAT 安装示例改为 `npm ci` 与本地 `./node_modules/.bin/playwright` shim；将执行示例改为 `npm run test:e2e -- ...` | 完成 |
| `tools/star-flash-mock/scripts/uat-3000-regression.ps1` | 将 Next.js 改为显式调用 `frontend/node_modules/.bin/next.cmd`，避免 npm exec 缓存回退；Playwright 通过现有 npm script 调用，保留端口、用例路径、日志与服务检测 | 完成 |
| `docs/briefs/STAR-STABLE-W6-UAT-NPM-20261102.md` | 本 lane brief，含 Read COMMENTS | 完成 |
| `docs/reports/WBS-DISPATCH-STAR-STABLE-W6-UAT-NPM-20261102.jsonl` | 由 dispatcher 写入 brief 审计记录 | 完成 |
| 本报告与 `docs/reviews/WBS-STAR-STABLE-W6-UAT-NPM-REVIEW-20261102.md` | 阶段事实与待协调者复核记录 | 完成 |

## 2. 验证摘要

| 验证 | 结果 |
|---|---|
| PowerShell AST `Parser.ParseFile` 静态解析 UAT 脚本 | 通过，0 parse errors |
| `rtk rg -n -i pnpm` 检查两份本 lane UAT 文件 | 无命中（rg exit 1，符合零匹配结果） |
| `rtk git diff --check` | 通过 |
| 本机 UAT / Playwright / WSL / K3s / 3001 服务 | 未运行；避免触发宿主或集群操作，不能据此声称 UAT 通过 |
| Web package install / full frontend checks | 未运行；锁文件迁移尚未批准并实施 |

## 3. 已知缺口

1. W6 的核心 package manager 收敛尚未完成：`frontend/package-lock.json` 与 `frontend/pnpm-lock.yaml` 仍共存；`frontend/package.json` 未声明唯一 package manager。
2. `.github/workflows/p1e-integration.yml` 仍有 pnpm install/exec，且 Node 22 与 stable Web workflow 的 Node 24.22.2 不同。通用 CI 仍选择 Node 22。
3. 首次完整 manifest 因与两个本地 remote-tracking Dependabot refs 的文件重叠，MAGOS `check` 返回 CONFLICT；本 lane 已缩至无冲突的 UAT 文件。没有使用 override。
4. Parent review 指出 `npm exec --offline` 仍可能从 npm cache 运行二进制，不能证明来源于本项目 lockfile。已将自动启动命令改为显式 `frontend/node_modules/.bin/next.cmd`，README 浏览器安装改为显式本地 Playwright shim；缺少 shim 时命令直接失败。此修正仅有静态验证，没有运行服务或安装依赖。
5. WBS 文件由在途 Star Delivery lane 登记持有，本 lane 未改 WBS；父 agent 需在该 claim 释放后再更新 WBS 的 W6 进展。
6. Vercel production deployment、Neon/Owner runtime、真实消费者/协作路径、rollback 与稳定发行证据仍未提供。
7. UAT 脚本仍指向历史 worktree 路径，且依赖 WSL/K3s 宿主状态；按 brief 不作无关改写。

**当前判定：** 此 lane 的两份本地 UAT 文件迁移完成；W6 仍 ADAPT/OPEN，v1.0.6 仍 NO-GO。

## 4. 子代理失败接手清单

本 lane 由单一 worker 执行，无派生子代理失败项。Parent agent 已独立复核 diff、brief、审计日志、报告和审阅记录，并接受本 lane 的受限范围。该 ACCEPT 不扩大到未修改的 W6 核心包管理器或稳定版发布门。

## 5. 守门规则

| # | 守门 | 结果 |
|---:|---|---|
| 1 | 使用登记的隔离 worktree 与 W6 lane | 符合 |
| 2 | 单一 writer，不并发编辑此 lane | 符合 |
| 3 | 实施变更限于两份 manifest UAT 文件 | 符合 |
| 4 | Brief 与 dispatch audit 已通过项目 dispatcher 生成 | 符合 |
| 5 | 仅使用 npm 本地依赖命令，不添加新依赖 | 符合 |
| 6 | 保留原 UAT 用例和报告判定逻辑 | 符合 |
| 7 | 未修改 package manifest 或任一 lockfile | 符合，仍是 W6 缺口 |
| 8 | 未修改 CI workflow | 符合，仍是 W6 缺口 |
| 9 | 未修改由其他 lane 持有的 WBS | 符合 |
| 10 | 未读取或改动 `D:\Star` 的用户 dirty/untracked 文件 | 符合 |
| 11 | 未切换或修改 `main` | 符合 |
| 12 | 未 fetch、push、改远端 refs、建 tag 或 release | 符合 |
| 13 | 未启动 WSL/K3s/3001 或执行 UAT | 符合 |
| 14 | PowerShell 静态解析与 whitespace 检查通过 | 符合 |
| 15 | 未把局部迁移标成 W6 或稳定版完成 | 符合 |

## 6. 签字栏

- 架构：本 lane 不改变架构；Mavis 接手复核通过（限本 lane）。
- SRE Lead：本 lane 未运行服务或集群；无需运行审批。
- 平台：本 lane 仅调整本地 UAT 工具命令；Mavis 接手复核通过（限本 lane）。
- 评审主持：parent agent 独立审阅完成；初版 `npm exec --offline` 的缓存回退风险已修正为本地 shim。
- PM / Release owner：用户仍为 accountable release owner；此报告不构成发布审批。

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 记录受限 UAT package-manager 迁移和未关闭的 W6 发布门 | WBS W6 指出 UAT 脚本仍使用 pnpm |
| v0.2 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 记录 parent 独立复核 ACCEPT（仅本 lane），并补充 `npm exec` cache 回退问题的修正；W6 与稳定版仍 OPEN | Parent review 完成与受限范围确认 |
