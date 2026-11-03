# PHASE — Star Stable W6 P1E npm Workflow Migration

Date: 2026-11-03
Lane: codex/star-stable-w6-p1e-npm-20261103
Baseline: dev@67755e31b5693e59678679c60277648fd478cd09

## §0 目的与范围

将 P1E 集成工作流中最后的前端 pnpm CI consumer 迁移为使用既有 frontend/package-lock.json 的 npm 流程，并统一到 Node 24.22.2。范围仅限 p1e-integration.yml 的 frontend-build 与 frontend-e2e 两个 job、稳定版 WBS 的 W6 证据，以及本阶段报告。

本 lane 未修改 frontend/package.json、package-lock.json 或 pnpm-lock.yaml，也未修改 Dependabot PR #292、远端或 main。未运行产品测试或 hosted workflow。静态配置修正不表示 P1E、W6 或稳定版发行门禁通过。

## §1 改动矩阵

| 项 | 修改 | 结果 |
|---|---|---|
| P1E frontend-build | 移除 pnpm setup；使用 Node 24.22.2、npm cache 与 frontend/package-lock.json；改用 npm ci 及 npm exec | 最后一个 CI 构建/静态检查 consumer 使用 npm 冻结安装 |
| P1E frontend-e2e | 移除 pnpm setup；使用 Node 24.22.2、npm cache 与 frontend/package-lock.json；Playwright 安装与执行改用 npm exec | 保留原有 Playwright specs、Chromium 项目、reporter 与 2 workers |
| Stable Release WBS | 仅追加 W6 lane 证据和 v1.51 修订记录 | W6 保持 ADAPT/OPEN；v1.0.6 保持 NO-GO |
| 本阶段报告 | 记录变更、静态验证及未关闭门禁 | 不声称 Node 24 hosted、P1E E2E、Vercel 或生产协作验证通过 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| PyYAML workflow parse | 通过 |
| 两个 actions/setup-node consumer 结构断言 | 通过：两个 job 均 pin 24.22.2、使用 npm cache，cache dependency 为 frontend/package-lock.json |
| pnpm/npm 静态断言 | 通过：workflow 中不再有 pnpm；两个 npm ci 和 npm exec 调用均存在 |
| rtk proxy git diff --check | 通过 |
| actionlint | 不可用：当前 PATH 未找到 actionlint |
| 产品测试与 hosted workflow | 未运行；依 brief 排除 |

既往本机 npm 冻结安装验收记录在 Node 22.19.0/npm 10.9.3 上；不是 Node 24.22.2/npm 11.19.1 或 GitHub hosted runner 的实测。本 lane 没有执行 npm ci 或 P1E 浏览器测试。

## §3 已知缺口

1. frontend/package.json 仍未声明 packageManager。
2. frontend/pnpm-lock.yaml 仍保留；因此仓库还未完成单一 package-manager/lockfile 的最终闭合。
3. 本地既往 npm ci 证据不验证 Node 24 hosted runner、缓存行为或 P1E E2E 成功。
4. Vercel production deployment、Neon/Owner profile、RLS/pooler/migration、协作更新、consumer 与 rollback 证据仍缺。
5. 因上述缺口，W6 仍为 ADAPT/OPEN，v1.0.6 仍为 NO-GO。

## §4 子代理失败接手清单

本 lane 由指定 worker 实施，没有额外派发子代理，故无子代理失败待接手项。协调者仍须独立审查确切差异、WBS 状态和 dev 集成状态；worker 提交不表示 W6 关闭。

## §5 守门规则

1. 只修改 MAGOS manifest 登记的 P1E workflow、W6 WBS 行和本报告。
2. 两个前端 job 固定使用 Node 24.22.2。
3. npm cache 必须引用 frontend/package-lock.json。
4. 使用 npm ci，不使用可改写 lockfile 的安装命令。
5. Next 与 Playwright 的执行使用 npm exec。
6. 保留现有 job 名称、测试规格、Chromium 项目、reporter 与 worker 数。
7. 不修改 frontend/package.json、package-lock.json 或 pnpm-lock.yaml。
8. 不修改、合并、关闭或清理 PR #292；该 PR 保持 HOLD。
9. 不修改 main、远端 refs、远端分支或其他 worktree。
10. PyYAML 与文本断言仅证明静态结构，不证明 Actions 执行成功。
11. 不将 Node 22 上既有 npm ci 记录声称为 Node 24 实测。
12. 不运行本任务 brief 明确排除的产品测试或 hosted workflow。
13. actionlint 不可用需如实记录，不以其他工具声称等价覆盖。
14. packageManager 未声明且 pnpm lock 保留时，不得声称 package manager/lockfile 已唯一化。
15. W6 在 Vercel/Neon/Owner、协作、consumer 与 rollback 证据闭合前保持 OPEN。
16. W0–W9 门禁全部通过并最终复核前，稳定版保持 NO-GO。
17. 合并前独立核对文件范围与 WBS 中的证据边界。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 仅将 P1E CI consumer 迁至 npm；完整部署架构仍待验证 |
| SRE Lead | SRE Lead（Mavis 接手审核）— 未运行 hosted workflow 或浏览器 E2E |
| 平台 | 平台（Mavis 接手审核）— PyYAML 与静态配置断言通过；actionlint 不可用 |
| 评审主持 | 评审主持（Mavis 接手审核）— worker 自检已完成，协调者独立复核待执行 |
| PM | PM（Mavis 接手审核）— W6 ADAPT/OPEN，v1.0.6 NO-GO |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 P1E workflow 的两个前端 job 从 pnpm 迁移到 Node 24.22.2/npm ci，记录静态验证与未关闭门禁。 | W6 需要将 P1E pnpm consumer 迁移到已验证的 npm lockfile |
