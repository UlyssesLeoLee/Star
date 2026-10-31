# PHASE-STABLE-HARDENING-DESKTOP-PACKAGING-REPORT

- **阶段**：STABLE-HARDENING / Desktop Packaging
- **状态**：`READY_FOR_INTEGRATION_WITH_RELEASE_GATES_OPEN`
- **基线**：`dev@04d8b7a84514dcb454874eb852651778e1452dea`
- **分支**：`agent/stable-hardening/desktop-packaging`

## 0. 目的

为 Star Desktop 建立可复现、按操作系统构建且制品缺失时失败的打包路径。本阶段修复了阻止前端打包的类型错误和过时的测试，并升级有安全公告的 Vite 工具链。此 lane 可集成；这不代表已生成可安装、已签名或已发布的稳定版。本阶段没有创建 tag、发布、部署或 push。

## 1. 改动矩阵

| 文件/范围 | 改动与结果 |
|---|---|
| `.github/workflows/star-stable-desktop.yml` | 增加 Linux `.deb`/AppImage、Windows MSI、macOS universal DMG job；固定 Node.js 24.22.2 与 Rust 1.99.0；Cargo 并行度为 2；actions 固定完整 SHA；三平台在生产构建前运行 Vitest；checkout 显式关闭凭据持久化；缺少或空制品时失败；仅上传标记为 unsigned、保留 7 天的验证制品；无发布权限或发布步骤。 |
| `crates/star-desktop/frontend/package.json`、`package-lock.json` | `npm ci` 锁文件安装；依赖升级至 Vite 7、Vitest 5、`@vitejs/plugin-react` 5；解决 npm 审计发现的 Vite/Vitest/esbuild 漏洞。三个直接包均为 MIT。全依赖法律闭包仍待 SPDX/SBOM 审查。 |
| `crates/star-desktop/frontend/tsconfig.json` | 保留 Vitest 与 Testing Library 全局类型，并加入 `vite/client`，使 Vite CSS Modules 声明可见。 |
| `crates/star-desktop/frontend/src/constants.ts` | 从唯一来源 `types/ids.ts` 重新导出 `DEFAULT_KANBAN_COLUMNS`，供看板消费者使用。 |
| `crates/star-desktop/frontend/src/hooks/useTauriKeyboardLayout.ts` | 从定义它的 `types/ids.ts` 导入 `DEFAULT_SWIMLANES`；运行时默认值不变。 |
| `crates/star-desktop/frontend/src/pages/WorktreeVizPage.tsx` | 从纯布局函数移除未使用参数；节点半径仍由渲染处单独计算，位置计算行为不变。 |
| `crates/star-desktop/frontend/tests/setup.ts`、`tests/BoardView.test.tsx` | 增加测试专用的窄 IPC mock 类型和每项测试后的 DOM 清理；删除行级类型抑制；将 DSL 测试改为支持的 `show` 关键字，并按三个泳道的真实 DOM 查询重复列。 |
| `crates/star-desktop/frontend/tests/KanbanBoard.test.tsx`、`tests/WorktreePage.test.tsx`、`tests/WorktreeVizPage.test.tsx` | 对齐现行看板页面和泳道结构；移除已过时的 mock 表格断言；修正可视化节点选择器，避免把节点数滑块计入 SVG 节点。 |
| `crates/star-desktop/src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`、`scripts/build-desktop.sh` | 修正前端命令路径，关闭未配置真实公钥的 updater artifacts，使用 `npm ci`，只构建当前宿主平台并检查非空安装包。 |

旧工作流 `.github/workflows/star-desktop-build.yml` 未修改；该文件仍由既有 owner 管理。

## 2. 验证摘要

| 验证 | 结果 | 证据与限制 |
|---|---|---|
| `npm ci` | 通过 | 从锁文件安装 163 个包；本机 Node 22.19.0 低于 package 要求的 Node 24，因此 npm 发出 `EBADENGINE`；另有 `whatwg-encoding@3.1.1` 弃用提示。CI 固定 Node 24.22.2，尚未取得 hosted runner 结果。 |
| `npm run build` | 通过 | `tsc --noEmit` 与 Vite 7.3.7 production build 均通过；55 个模块转换完成。 |
| `npm test -- --reporter=dot` | 通过 | Vitest 5.0.5：13 个测试文件、62 项测试全部通过。 |
| `npm audit` | 通过 | 升级前审计发现 6 个漏洞（含 2 个 Critical、1 个 High）；安全兼容升级后审计为 0 个漏洞。 |
| `npm ls vite vitest @vitejs/plugin-react` | 通过 | Vite 7.3.7、Vitest 5.0.5、`@vitejs/plugin-react` 5.1.4；Vite 7 与 Vitest 5 的最低 Node 要求满足当前 Node 22.19，产品包自身仍要求 Node 24。 |
| `bash -n scripts/build-desktop.sh` / ShellCheck 0.11.0 | 通过 | 脚本语法和静态检查无错误。 |
| JSON/TOML/YAML 解析 | 通过 | package manifest、lock、Tauri config、Cargo manifest 与 workflow 可解析。 |
| CypherGraph Guardian `--changed --stamp` | 通过 | 8 个修改的 TypeScript/TSX 源文件通过结构检查；代码关系已逐项按导入和测试引用核对。 |
| `cargo metadata --no-deps --format-version 1` | 通过 | 只证明 Cargo manifest 可解析，不证明 Rust 编译或桌面打包。 |
| Tauri CLI `--version` / `info` | 通过 | CLI 2.12.4；识别到 tauri 2.12.1、tauri-build 2.7.1、WebView2 与 MSVC。 |
| `tauri build --bundles msi` | 未通过 | 本机 `rustc` 组件不可用；CLI 在 Rust 信息读取处 panic，未生成 MSI。不能视为构建通过。 |
| 工作流安全配置审查 | 通过（静态） | 三个平台均在前端 build 前执行 `npm test -- --reporter=dot`；全部 checkout 设置 `persist-credentials: false`；尚未运行 GitHub hosted workflow。 |
| GitHub hosted CI / actionlint | 未验证 | 未运行 hosted workflow；本机未安装 actionlint。 |

## 3. 已知缺口

1. **真实打包门仍未闭合**：本机 Rust toolchain 缺少可用 `rustc`。必须由 Node 24.22.2 / Rust 1.99.0 hosted runner 实际生成 Linux、Windows、macOS 制品。
2. **安装验收未做**：`.deb`、AppImage、MSI、DMG 均未实际下载、安装、启动和卸载验证；VBSCRIPT/WiX、WebView2、Linux 系统库和 macOS Gatekeeper 仍需核对。
3. **签名和更新未就绪**：没有 Authenticode、Apple Developer ID/公证凭据或真实 updater 公钥；所有制品均未签名，updater artifact 保持关闭。
4. **旧工作流仍使用 Node 20**：`star-desktop-build.yml` 未改，需由该工作流 owner 处理或明确退休。新稳定工作流已固定 Node 24.22.2，但 hosted runner 尚未实际运行。
5. **法律闭包仍开放**：本轮确认三个新增/升级直接工具为 MIT，但尚未对 npm 完整传递依赖、分发制品、安装器和容器闭包生成 SPDX/SBOM 并完成再分发义务审核。
6. **稳定版整体仍开放**：Web/Vercel 真实部署与回滚、Canvas 镜像实际构建和 digest、最多两个 Multipass 实例上的 K3s 验收、Cargo MIT 权利闭包及 release CI 必需检查尚未完成。
7. **本地运行时与目标 CI 有差异**：本机 Node 22.19.0 低于 package 要求的 Node 24；此前的 npm ci/build/62 项测试是在 Node 22.19.0 上得到的，不能替代目标 Node 24 hosted runner 复验。当前机器无 Node 24，因此本轮没有重复运行 npm test/build。
8. **Tauri CSP 尚未配置**：现有 `src-tauri/tauri.conf.json` 中 `app.security.csp` 为 `null`（本 lane 未新增此设置）；发布前需审查前端所需来源并配置生产 CSP。
9. **checkout 凭据防护**：本轮已为新稳定工作流中的三个 checkout 设置 `persist-credentials: false`；旧 workflow 的 credential 设置仍由其 owner 单独审查。
10. **没有发布动作**：本阶段没有创建 tag、发布、部署或 push；不能宣称稳定版已发布。

## 4. 子代理失败接手清单

初始 Desktop 实施 worker 因 usage limit 中断；协调者完成原前端修复后，对当前稳定工作流又执行独立 review，并按 findings 增加 Node 24 测试步骤及 checkout 凭据限制。未把 worker 状态作为验证证据；本轮工作流 hosted 结果仍未取得。

## 5. 守门规则

| # | 守门 | 结果 |
|---:|---|---|
| 1 | 只修改 MAGOS revision 16 Desktop manifest 内文件 | 符合 |
| 2 | 基线为记录的 `dev@04d8b7a84514dcb454874eb852651778e1452dea` | 符合 |
| 3 | 保留 `star-desktop-build.yml` 现有 owner 内容 | 符合 |
| 4 | 新打包工作流使用 Node 24 | 符合，固定 24.22.2 |
| 5 | Rust 工作流使用项目 baseline | 配置 Rust 1.99.0；runner 尚未实测 |
| 6 | npm 安装锁定依赖 | 符合，`npm ci` |
| 7 | 逐平台构建，不伪装跨平台本地构建 | 符合 |
| 8 | 缺少制品时失败 | 符合 |
| 9 | 仅上传短期验证制品且标为 unsigned | 符合，保留 7 天 |
| 10 | 使用最小 GitHub 权限 | 符合，`contents: read` |
| 11 | GitHub actions 固定完整 SHA | 符合 |
| 12 | 不使用 placeholder updater key | 符合，updater artifacts 关闭 |
| 13 | 不编造签名或公证凭据 | 符合 |
| 14 | 不跳过 TypeScript 检查 | 符合，typecheck 是 build 第一门 |
| 15 | 不发布、打 tag、部署或 push | 符合 |
| 16 | 源码图关系有代码证据且 freshness stamp 通过 | 符合，8 个源文件通过 CGG |
| 17 | 真实 runner、安装、签名、许可与集群证据缺失时明确标记 | 符合，见 §2–§3 |
| 18 | 新稳定工作流每平台在 build 前运行 Vitest；checkout 不保留执行 PR 代码所需的凭据 | 符合，静态检查通过；hosted 运行仍待验证 |

## 6. 审查栏

| 角色 | 状态 |
|---|---|
| 架构 | Mavis 接手审核；按当前 dev 基线集成前复核 |
| SRE Lead | Mavis 接手审核；hosted runner、签名和安装证据待补 |
| 平台 | Mavis 接手审核；本机 Rust 构建受工具链阻塞 |
| 评审主持 | Mavis 接手审核；Desktop lane 可集成，release gates 仍开放 |
| PM | Mavis 接手审核；稳定版整体 W0–W9 未关闭 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 初始 desktop packaging 报告；记录打包和签名缺口。 |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复 Vite 类型、常量引用和 test-only 类型问题；矫正过时 UI 测试；升级 Vite/Vitest 安全版本；本地 build、62 项 Vitest 与 npm audit 通过。保留 hosted runner、平台制品、安装、签名和法律闭包缺口。 |
| v0.3 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 根据独立审查在 Node 24 工作流的 Linux/Windows/macOS job 中加入 Vitest，并对三个 checkout 均关闭凭据持久化；更新报告记录 Node 22 本地结果限制及既有 Tauri CSP 缺口。未运行 hosted CI，stable release gates 继续开放。 |
