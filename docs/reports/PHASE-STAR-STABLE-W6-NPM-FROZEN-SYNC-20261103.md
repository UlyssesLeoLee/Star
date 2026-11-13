# PHASE-STAR-STABLE-W6-NPM-FROZEN-SYNC-20261103

## §0 目的

将已集成的 W6 npm 冻结安装本机验收记录同步到稳定版 WBS，保留通过与失败两类证据及未关闭门禁。源证据为 `docs/reports/PHASE-STAR-STABLE-W6-NPM-FROZEN-ACCEPTANCE-20261102.md`，其提交 `789392b67a90998bfa59ed49c3deaab31a012269` 已在本 lane 基线 dev@`f467b5bae68f5c372d67d1cc78c1ae470376ace0` 的历史中。本 lane 仅更新已登记的文档，不重跑 npm 命令。

## §1 改动矩阵与验收状态

| 项目 | 结果 | 证据 / 说明 |
|---|---|---|
| WBS W6 状态 | 已同步 | 追加 frozen-install acceptance 数值与源报告链接；W6 保持 ADAPT/OPEN，稳定版 v1.0.6 保持 NO-GO |
| 冻结安装 | 本机通过 | 源报告记录 `npm ci --no-audit --no-fund` exit 0，新增 329 packages |
| package-lock 不变 | 本机通过 | 安装前后 SHA-256 均为 `798BDF3188080F80F8EBC7F68A01F8116D180F49E7C203D6EEF0C7C311B64C09` |
| typecheck | 通过 | 源报告记录 `npm run typecheck` exit 0 |
| 默认测试配置 | 失败，保留 | 源报告记录 `npm test` exit 1；错误前 109 files / 936 tests passed，随后 2 个未处理 worker 错误，包括 V8 `AlignedAlloc Allocation failed` |
| 有界 worker 重跑 | 通过 | 源报告记录 `npm test -- --maxWorkers=4` exit 0，111 files / 950 tests passed |
| Node 与 npm | 记录 | Node `v22.19.0`；npm `10.9.3`；engine 与弃用警告保留 |
| 包管理器收敛 | 未完成 | `frontend/package.json` 仍无 `packageManager`；`pnpm-lock.yaml` 和 P1E workflow 的 pnpm consumer 仍存在 |
| 生产发布证据 | 未验证 | Node 24 hosted run、Vercel/Neon/Star owner production、consumer verification 与 rollback 仍开放 |

本报告仅把已提交源报告的本机结果传递至 WBS，不把历史验收描述成当前重新运行的测试。

## §2 验证摘要

| 检查 | 结果 | 范围 |
|---|---|---|
| 源报告与 WBS 数值交叉核对 | 通过 | 仅核对已集成的 W6 acceptance 报告与新增 WBS 文案 |
| `rtk proxy git diff --check` | 通过 | staged diff exit 0；仅检查本 lane 的文档差异 |
| npm install/typecheck/Vitest | 未运行 | 本同步 lane 不重跑源报告命令 |
| Vercel、Neon、owner、E2E、生产构建、rollback | 未运行 | 本同步 lane 不产生部署或 hosted 证明 |

## §3 已知缺口

1. `frontend/package.json` 未固定 `packageManager`，且 npm 与 pnpm lockfile 仍并存。
2. `.github/workflows/p1e-integration.yml` 仍使用 pnpm setup/cache、frozen install 与 pnpm 脚本。
3. Node `v22.19.0` 低于源报告所记录的若干依赖 Node 22 最低范围 `22.22.2`；Node 24 hosted 验收仍缺。
4. 默认 Vitest 并行配置曾发生 worker/V8 OOM；`--maxWorkers=4` 重试通过不能证明默认配置在目标机器稳定。
5. Web production Vercel deployment identity、source commit、build/runtime config、Neon/Star owner profile、协作路径、consumer 与 rollback 均未验证。
6. 本 WBS 同步不修改 package manifest、lockfile 或 workflow，不能关闭 W6 实施工作或稳定版发布门。

## §4 子代理失败接手清单

本同步 lane 没有重新派发 npm 测试子代理。源报告中的默认 Vitest 执行失败及随后有界 worker 重试均完整保留；本报告不删除或弱化该失败证据。本 lane 的文档编辑由 worker 执行，提交前差异由 worker 自审，独立协调者审查仍需完成。

## §5 守门规则

1. 将 npm 冻结安装描述为 `npm ci`，不称作包管理器收敛。
2. 保留 329 packages 与安装退出码的源报告范围。
3. 保留 lockfile SHA-256，不推断锁文件已被所有消费者使用。
4. 如实记录 `npm run typecheck` 的退出码。
5. 如实保留默认 `npm test` 失败状态。
6. 不省略两项 worker 错误及 V8 OOM 事实。
7. 明确区分 `--maxWorkers=4` 的 950/950 有界重跑结果。
8. 不把有界重跑外推为默认并行配置稳定。
9. 记录实测 Node/npm 版本，不宣称 Node 24 hosted 通过。
10. 保留依赖 Node engine warning。
11. 保留 `packageManager` 字段缺失状态。
12. 保留 pnpm lockfile 与 P1E workflow consumer。
13. 不将本机结果描述成 Vercel、Neon 或 Star owner 的生产验收。
14. 不声称已运行 E2E、production build、consumer verification 或 rollback。
15. 本 lane 不改应用代码、package manifest、lockfile 或 workflow。
16. 只改 MAGOS 登记的 WBS、brief、phase report、review、dispatch receipt 五个文件。
17. 不修改 main、D:\Star 用户内容、远端分支/PR，不 push、tag、release 或 deploy。

## §6 签字栏

| 角色 | 记录 |
|---|---|
| 架构 | Worker 文档自审；协调者独立集成审查待完成 |
| SRE Lead | 未执行 hosted、集群、容量或恢复验收 |
| 平台 | 本 lane 未修改 CI、package manager 或供应链配置 |
| 评审主持 | 协调者在集成前复核 WBS 与源报告对应关系 |
| PM / Release Owner | Ulysses 为 accountable release owner；此同步不构成稳定版批准 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将已集成的 W6 npm frozen-install 本机证据同步到 WBS；明确 package manager、Node 24 hosted 与生产部署门仍开放 | STAR-STABLE-W6-NPM-FROZEN-SYNC-20261103 brief |