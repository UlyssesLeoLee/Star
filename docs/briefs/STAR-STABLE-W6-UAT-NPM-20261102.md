# Brief: STAR-STABLE-W6-UAT-NPM-20261102

**Agent**: worker
**Phase**: STAR-STABLE-W6-UAT-NPM-20261102
**Created**: 2026-11-02 11:16:57

---

## 任务目标
在已登记的 W6 UAT package-manager lane 中，只把下列现行 UAT 辅助脚本和配套 README 内的 pnpm 调用迁移为已有 npm package-lock 工作流下的等价 npm 命令，保持既有测试目标、路径、端口和日志行为。

## 用户与产品约束
- 稳定发布范围：Desktop、Web/Vercel、Star Canvas 三个镜像；MIT；Git tag 唯一版本源，目标 v1.0.6；用户是 release owner。此 lane 不改发布状态。
- 只包含 Star 产品资产；不引入 game/旧业务载荷，不接触 main、远端、tag 或 release。
- 只写入本 lane worktree；不得读取、覆盖、修改或清理 D:\Star 的 dirty/untracked 用户文件。
- 项目历史导航架构与 Rust/K3s 等规范不在本 lane 变更范围内。

## 所有权与文件范围
唯一实现所有者：本 worker。只允许修改：
1. tools/star-flash-mock/scripts/README.md
2. tools/star-flash-mock/scripts/uat-3000-regression.ps1
3. docs/briefs/STAR-STABLE-W6-UAT-NPM-20261102.md（本 brief）
4. docs/reports/PHASE-STAR-STABLE-W6-UAT-NPM-20261102.md
5. docs/reports/WBS-DISPATCH-STAR-STABLE-W6-UAT-NPM-20261102.jsonl（dispatcher 自动写入）
6. docs/reviews/WBS-STAR-STABLE-W6-UAT-NPM-REVIEW-20261102.md
不得修改任何其他路径。

## 审查证据与限制
当前 dev@d3611583cacc06770cee23f2ffb49c67f8b56df9 已集成 W6 四份候选审查文档（W6 review commit 6f7cbc88 与基线注记 66610de4 均在 dev ancestry）。WBS v1.42 已记录 npm ci 与 pnpm 混用；pnpm 执行点包括 P1E workflow 和此 UAT 脚本。MAGOS 拒绝把 P1E workflows、frontend/package.json 与 pnpm-lock.yaml 纳入同一 lane，因为与本地 remote-tracking Dependabot refs 有文件重叠；不得为此使用 override。WBS 文件被仍登记的 Star Delivery lane 占有，不要修改它。
CodeGraph 对这些配置/PowerShell 文本返回了无关代码符号；MCP 没有 check_index_coverage 工具。父 agent 已使用 `rtk rg` 在目标 dev checkout 精确定位本 lane 内全部 pnpm 调用点，此类非代码配置/文档使用受限 rg fallback。

## 实施指引
- 把脚本中的 `pnpm exec next dev` 改为调用锁文件下安装的 Next CLI 的 npm 等价形式（例如 `npm exec -- next dev`），把 Playwright 调用改为 npm 等价形式，兼顾 PowerShell/cmd 参数转义。
- README 的安装/执行示例改用 `npm ci`、`npm exec -- playwright install chromium` 与 `npm run test:e2e -- ...`；不要写入 pnpm 操作命令。
- 不改 `frontend/package.json`、`package-lock.json`、`pnpm-lock.yaml`、GitHub workflows、WBS 或其他发布文件，因为它们与在途 Dependabot claim 重叠或由其他 lane 持有。
- 不趁机修复脚本里无关的过时工作树路径、业务名或旧注释。

## 验收标准
- 仅 manifest 中 6 个路径发生变化；实现路径最多为两份指定 UAT 文件。
- `rg -n -i pnpm` 在两份指定文件中无命中；相关命令只使用 npm。
- 用可用的 PowerShell parser 做静态语法验证；`git diff --check` 通过。不要连接或启动本机 K3s/WSL/3001 服务，也不要声称 UAT 已执行。
- 生成阶段报告，明确本 lane 只是本地 UAT 消费者迁移；P1E workflow、两个 lockfile、Node 22/24 对齐和 Vercel/Neon/Owner 运行证据仍开放，W6 与 v1.0.6 不可标完成。
- 写入独立审阅记录，列出实际 diff、验证命令/结果、未运行 UAT 的原因、scope 限制，并由 parent agent 审核后再集成。
- 提交仅含本 lane manifest 内文件。不得在 dev 或 main 直接提交。


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
