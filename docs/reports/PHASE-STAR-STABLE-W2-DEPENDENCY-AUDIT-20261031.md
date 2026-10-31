# PHASE-STAR-STABLE-W2-DEPENDENCY-AUDIT-20261031

**状态：完成只读审查；不建议当前合并任一 PR。** 本报告记录截至 2026-10-31 对 Dependabot PR #296 与 #292 的实时元数据、文件差异、可见 CI runs 和官方迁移资料。没有修改 PR、远端分支或依赖文件。

## §0 目的

判断两个现有依赖 owner PR 是否适合进入当前 `dev`，并说明阻止稳定版 W2/W4 闭环的证据与后续 owner 动作。Star 自有项目许可证已由用户确认为 MIT；这不解决第三方依赖的许可证、NOTICE、SBOM 或再分发义务。

## §1 PR 审查矩阵

| PR | 当前状态与范围 | 检查证据 | 决定 |
|---|---|---|---|
| [#296 frontend dependencies](https://github.com/UlyssesLeoLee/Star/pull/296) | 仍为 open，base=`main`，head=`dependabot/npm_and_yarn/frontend/npm-dependencies-4ec8768635`，head SHA `e6534bcd5090da2c37c5270034f740bd881bad7f`。Dependabot 标注 21 项更新；改动 3 个文件：`frontend/package.json`、`frontend/package-lock.json`、`frontend/pnpm-lock.yaml`；GitHub 统计 2,104 additions / 2,577 deletions。 | Frontend typecheck/test/build、CodeQL 和 advisory coverage 成功（[CI run 40248864873](https://github.com/UlyssesLeoLee/Star/actions/runs/40248864873)、[CodeQL run 40248864905](https://github.com/UlyssesLeoLee/Star/actions/runs/40248864905)、[coverage run 40248864948](https://github.com/UlyssesLeoLee/Star/actions/runs/40248864948)）；spawn E2E 的 Linux jobs 失败（[run 40248864876](https://github.com/UlyssesLeoLee/Star/actions/runs/40248864876)），CI 中 Rust check/build、BFF `cargo fmt` 与 Markdown lint 失败，License & Advisory Gate 失败（[run 40248864926](https://github.com/UlyssesLeoLee/Star/actions/runs/40248864926)）。PR head 的 CodeRabbit combined status 为 success，但这不能覆盖失败的 Actions。未发现 review submission。 | **暂缓合并。** 当前 head 的真实检查未通过；PR 还同时更新 npm 与 pnpm 两份锁文件，必须由 Web owner 确认发行构建的唯一 package-manager/lockfile 契约，并在当前 `dev` 基线上重新运行 Web/发行验证。 |
| [#292 Rust toolchain](https://github.com/UlyssesLeoLee/Star/pull/292) | 仍为 open，base=`main`，head=`dependabot/github_actions/dtolnay/rust-toolchain-1.120.0`，head SHA `f6bede9301572ddd249ffd8c2baa39af4f106c0e`。将 8 个 workflow 中的 `dtolnay/rust-toolchain@1.99.0` 改为 `@1.120.0`，共 19 处增删；PR 有一个 commit，未见 review submission。 | Actions 日志直接显示该 action 为 `1.120.0` 安装 Rust `1.120.0`，rustup 返回 “could not download nonexistent rust version” 和 HTTP 404。Rust CI、跨平台、E2E 与 license/advisory jobs 因 toolchain 安装失败或跳过（[CI run 39180145160](https://github.com/UlyssesLeoLee/Star/actions/runs/39180145160)、[spawn E2E run 39180145158](https://github.com/UlyssesLeoLee/Star/actions/runs/39180145158)、[License & Advisory run 39180145061](https://github.com/UlyssesLeoLee/Star/actions/runs/39180145061)）；CodeQL 和 advisory coverage 成功（[CodeQL run 39180145142](https://github.com/UlyssesLeoLee/Star/actions/runs/39180145142)、[coverage run 39180145041](https://github.com/UlyssesLeoLee/Star/actions/runs/39180145041)），Frontend job 成功。 | **拒绝当前 head。** 仓库 `rust-toolchain.toml` 与用户规定的最低版本仍为 `1.99.0`。不得把 action ref 的改动描述成单纯 action 实现升级；该 PR 实际把多数 jobs 的 Rust toolchain 切到了当前日志中不存在的 `1.120.0`。 |

### PR #296：版本变化与兼容性检查

Dependabot 描述中的 21 项版本变化为：

- Runtime/dependencies：`@react-three/drei` 10.7.8→10.7.9；`@react-three/fiber` 9.8.1→9.8.2；`@tanstack/react-query` 5.103.1→5.104.5；`cytoscape` 3.34.3→3.34.4；`lucide-react` 1.47.0→1.61.0；`monaco-editor` 0.56.0→0.57.0；`motion` 13.4.0→14.6.0；`next` 16.3.5→16.4.2；`react` / `react-dom` 19.3.0→19.3.1；`three` 0.186.0→0.186.1；`zustand` 5.0.15→5.0.16。
- Test/build dependencies：`@playwright/test` 1.63.0→1.64.1；`@testing-library/user-event` 14.6.7→14.6.8；`@types/node` 26.6.1→26.6.5；`fake-indexeddb` 6.2.5→6.2.6；`jsdom` 30.1.0→30.1.3；`msw` 2.15.0→3.0.3；`postcss` 8.5.28→8.5.31；`vitest` 5.0.1→5.0.5；`vite` 6.4.3→8.3.5。

官方升级资料指出，MSW 3 包含 Node.js 最低版本提升至 22、入口拆分及 GraphQL handler 变化；Vite 8 继承 Vite 7 的 Node.js 最低版本要求（20.19+ 或 22.12+），并涉及构建器与迁移注意事项；Motion 14 官方说明没有 breaking changes，但弃用部分内部 API 且收紧内部依赖版本关系。依据：[MSW 2.x→3.x migration](https://mswjs.io/docs/migrations/2.x-to-3.x)、[Vite migration guide](https://vite.dev/guide/migration)、[Motion for React upgrade guide](https://motion.dev/docs/react-upgrade-guide)。因此需验证现有 handlers、Vite config/plugin、构建目标和运行时版本；本次只确认版本差异和 CI 结果，没有推断代码已不兼容。

锁文件方面，当前 CI 的 Web job 使用 `npm ci`，另有 P1E workflow 使用 `pnpm install --frozen-lockfile`。该 PR 同时改两个锁文件；稳定版 W2 要求 Web/Desktop 分别确定唯一 manager 与锁文件，当前结果尚未满足该决定。

### PR #292：工具链证据

PR patch 将 `.github/workflows/ci.yml`、`coverage.yml`、`license-gate.yml`、`p1e-integration.yml`、`publish-canvas-game.yml`、`spawn-e2e.yml`、`star-desktop-build.yml` 和 `wasm-build.yml` 的 action ref 替换为 `1.120.0`。PR head 的 [CI run](https://github.com/UlyssesLeoLee/Star/actions/runs/39180145160)、[spawn E2E run](https://github.com/UlyssesLeoLee/Star/actions/runs/39180145158) 与 [License & Advisory Gate run](https://github.com/UlyssesLeoLee/Star/actions/runs/39180145061) 中，失败步骤均可追溯到安装 `1.120.0`。例如 Linux 日志明确记录 rustup 对 `1.120.0-x86_64-unknown-linux-gnu` 返回 404。官方 action 仓库说明见 [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain)；本报告对失败结论以 PR 自身日志为直接证据。

## §2 验证摘要

- 通过已连接 GitHub 只读接口重新读取两个 PR 的 head/base/state、文件清单、reviews、combined status、PR-triggered workflow runs 和失败 job/step 摘要；PR #296 当前 head `e6534bc`，PR #292 当前 head `f6bede9`。
- 对失败 job 读取了 bounded workflow logs。PR #292 的核心失败原因有直接 404 证据；PR #296 的失败项记录为 head 上观察到的结果，不将其归因于这 21 项 npm 更新。
- 使用 Context7 查阅 MSW、Vite、Motion 的官方升级资料，并按其官方来源链接记录版本迁移边界。
- 本 lane 仅写审计报告和 dispatcher 文档；没有运行 Cargo/npm 构建或测试，也没有更改远端资源。
- GitHub connector 未提供可确认 `main` branch protection required-check 配置的证据；因此本报告不把任何 observed check 标为 branch-protection 必需项。

## §3 已知缺口

1. PR #296 仍需 Web owner 决定 npm/pnpm 双锁文件是否都属当前 Web 交付闭包；将 PR 同步到 `dev` 后重跑锁一致性、typecheck/test/build 和实际 Vercel consumer verification。外部部署仍未由此 PR 证明。
2. PR #296 的 Rust、Markdown 与 license/advisory failures 尚未通过基线对比确认因果；其依赖升级不能代替稳定版第三方许可证、NOTICE、SBOM 和 advisory 审核。
3. PR #292 需重新生成到真实存在且与 `rust-toolchain.toml` 一致的 toolchain；若未来要升 Rust baseline，必须单独同步 workspace MSRV、CI pin、锁文件和验证结果。当前 PR 不适合 cherry-pick 到 `dev`。
4. 两个 PR 都以 `main` 为 base，且本 lane 未对其进行 rebase/合并性验证；GitHub 报告的 `mergeable=true` 只说明其对当前 PR base 的状态，不证明可安全进入本地 `dev`。
5. 实际第三方分发权利、近 12 个月维护证据、完整依赖闭包和许可/NOTICE 履约仍属稳定版 W1/W2/W4 开放事项。

## §4 子代理失败接手清单

- 无子代理派发；审查由本 lane 单一 owner 执行。
- 未观察到远端修改、PR 操作或工作树外写入。

## §5 守门规则

- 只读审查：未变更依赖清单、锁文件、workflow、PR、分支或 WBS。
- 失败检查不因 CodeRabbit success、coverage success 或 PR mergeable 状态而被掩盖。
- 对 unknown 的 required-check 配置、CI 失败因果和 Vercel 部署状态明确保留为未证实。
- 只对已登记的 brief/report/dispatcher 文件执行提交；不合并、不清理、不推送。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 审查范围为依赖 owner 与发行门，不批准任何 PR 合并。 |
| SRE Lead | SRE Lead（Mavis 接手审核）— 要求在当前 `dev` 重跑失败的 CI 和发布验证。 |
| 平台 | 平台（Mavis 接手审核）— 工具链应与仓库 Rust 1.99.0 基线一致。 |
| 评审主持 | 评审主持（Mavis 接手审核）— PR #296、#292 均暂不集成。 |
| PM | PM（Mavis 接手审核）— 稳定版 WBS 维持 OPEN。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对 PR #296、#292 作只读版本、owner、CI 与官方升级资料审查；记录两者均不适合当前合并，并保留稳定版法律/部署证据缺口。 | stable-release WBS wave 2 |
