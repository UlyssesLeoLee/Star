# PHASE-STAR-STABLE-W6-NPM-FROZEN-ACCEPTANCE-20261102

## §0 目的

在当前隔离 lane 对前端 npm 冻结安装路径做本地验收，并记录可复核的命令、工具版本、锁文件状态、测试结果和发布门缺口。本报告只证明当前环境的一次本地安装与检查，不代表 hosted Vercel/Neon/Star owner 验收，也不关闭 W6 或稳定版发布门。

工作目录：`D:\\orcaWork\\Star\\star-stable-w6-npm-frozen-acceptance-20261102\\frontend`
基线：`dev` 创建时 HEAD `9726a955c4bac6637952fa198d3346b9c905e7f4`。
边界：仅本 lane 的 brief、phase report、review、dispatch JSONL；不修改源代码、package manifests、锁文件、workflow、WBS、main 或远端。

## §1 改动与验收矩阵

| 项目 | 结果 | 证据 / 说明 |
|---|---|---|
| npm 冻结安装 | 通过 | `npm ci --no-audit --no-fund` exit 0；新增 329 packages，约 41 秒 |
| 锁文件不可变 | 通过 | `frontend/package-lock.json` 安装前后 SHA-256 均为 `798BDF3188080F80F8EBC7F68A01F8116D180F49E7C203D6EEF0C7C311B64C09` |
| Node / npm 版本 | 记录 | Node `v22.19.0`；npm `10.9.3` |
| TypeScript 检查 | 通过 | `npm run typecheck` exit 0 |
| 默认 Vitest 测试 | 失败，按原样保留 | `npm test` exit 1；终止前显示 109 files / 936 tests passed，随后有 2 个未处理 worker 错误，其中包括 V8 `AlignedAlloc Allocation failed` 致命内存分配失败 |
| 有界 worker 重试 | 通过 | `npm test -- --maxWorkers=4` exit 0；111 files / 950 tests passed，94.33 秒 |
| package manager 收敛 | 未完成 | `frontend/package.json` 无 `packageManager` 字段；`frontend/pnpm-lock.yaml` 仍存在 |
| pnpm CI 消费者 | 未完成 | `.github/workflows/p1e-integration.yml` 仍配置 pnpm setup/cache，并在 frontend build 与 e2e job 中使用 `pnpm install --frozen-lockfile`；另有 pnpm lint、Playwright 命令 |
| hosted 验收 | 未验证 | 本 lane 未运行 Vercel、Neon、Star Project owner 的 hosted 接受检查；也未运行 Playwright E2E |

安装时 npm 报告以下引擎兼容警告：`@asamuzakjp/css-color@7.0.0`、`@asamuzakjp/dom-selector@9.1.4` 与 `jsdom@30.1.0` 声明的 Node 范围为 `^22.22.2 || ^24.15.0 || >=26`，当前 Node 为 `22.19.0`。另有 `@types/cytoscape@3.31.0` 弃用警告。安装仍 exit 0；这些警告不等于引擎兼容性已验收。

## §2 验证摘要

命令均在上述 `frontend` 目录执行：

| 命令 | Exit | 结果 |
|---|---:|---|
| `node --version` | 0 | `v22.19.0` |
| `npm --version` | 0 | `10.9.3` |
| `npm ci --no-audit --no-fund` | 0 | 329 packages；约 41 秒；有上文记录的 Node engine 与弃用警告 |
| `npm run typecheck` | 0 | `tsc --noEmit` 通过 |
| `npm test` | 1 | 109 files / 936 tests 在 worker 错误前通过；2 个未处理 worker 错误，包括 V8 内存分配失败 |
| `npm test -- --maxWorkers=4` | 0 | 111 files / 950 tests 通过；94.33 秒 |

本轮确认 `frontend/package-lock.json` 安装前后 SHA-256 一致。E2E、生产构建、部署、hosted 集成与容量验收均未执行；不得将这些本地结果扩展解释为它们已通过。

## §3 已知缺口

1. 前端 manifest 没有固定 `packageManager` 元数据；本地通过 npm 执行不能单独决定 CI 与贡献者的唯一包管理器。
2. pnpm 锁文件仍在，且 `p1e-integration.yml` 仍消费 pnpm。完成 npm 单路径验收不等于 pnpm 消费者已迁移或移除。
3. 当前 Node `22.19.0` 低于三项依赖声明的 Node 22 最低版本 `22.22.2`；需在批准的目标 Node 版本上重验。
4. 默认 Vitest 并行运行发生 worker 内存故障；4 worker 重跑通过，但该结果不证明默认并行配置在目标 CI/用户机器上可靠。
5. `test:e2e`、正式 build、Vercel、Neon 与 Star Project owner 的 hosted 验收不在本次实测范围内。
6. W6 与稳定版发布仍保持 OPEN。release owner 尚需按既定发布矩阵完成其余证据与审批。

## §4 子代理失败接手清单

本 lane 未派发子代理。第一次默认并行 Vitest 执行发生 2 个未处理 worker 错误，包含 V8 内存分配失败；保留该失败证据后，以 `--maxWorkers=4` 进行受限重试并通过 950/950。此项属于运行资源限制的恢复验证，不清除也不覆盖首次失败记录。

## §5 守门规则

1. npm 安装使用 lockfile 冻结路径 `npm ci`。
2. 安装前后逐字节校验 package-lock SHA-256。
3. 记录 Node 与 npm 的实际版本。
4. 不把退出码 0 的安装描述成整个包管理体系已收敛。
5. 原样记录默认测试失败。
6. 不以有界 worker 重试掩盖默认并行 OOM。
7. 不把受限重试通过外推为默认 CI 并行稳定。
8. 保留 packageManager 缺失状态。
9. 保留 pnpm lockfile 和工作流消费者的事实。
10. 不在本 acceptance lane 编辑 manifest、锁文件或 workflow。
11. 不把未运行的 E2E 或 build 标为通过。
12. 不把本地结果冒充 Vercel、Neon 或 Project owner 线上证据。
13. 不运行历史 star-flash-mock/WSL UAT 作为产品验收。
14. 不部署、不发布、不创建 tag、不操作 VM。
15. 不修改 WBS、main、远端或其他 worktree。
16. W6 / release 状态在上述缺口闭合前保持 OPEN。

## §6 签字栏

| 角色 | 记录 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008）—本 lane worker 自审；集成前由协调者独立复核 |
| SRE Lead | 不适用：本 lane 未运行 hosted、集群或容量验收 |
| 平台 | 不适用：本 lane 未改动 CI 或 package manager 配置 |
| 评审主持 | 协调者独立复核待完成；本报告不宣称已获独立审阅 |
| PM / Release Owner | Ulysses 为 accountable release owner；本 lane 未作稳定版批准 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 npm frozen install、锁文件哈希、typecheck、默认 Vitest OOM 与 4-worker 重试；明确 package manager、hosted 验收与 W6 状态仍开放 | STAR-STABLE-W6-NPM-FROZEN-ACCEPTANCE-20261102 brief |
