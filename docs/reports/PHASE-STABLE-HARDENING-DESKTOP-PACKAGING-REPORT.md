# PHASE-STABLE-HARDENING-DESKTOP-PACKAGING-REPORT

**阶段**：STABLE-HARDENING / Desktop Packaging

**状态**：`IMPLEMENTED_WITH_BLOCKERS`；不得标为 READY

**基线**：`dev@04d8b7a84514dcb454874eb852651778e1452dea`

**分支**：`agent/stable-hardening/desktop-packaging`

**范围**：仅桌面构建工作流、Tauri 打包配置、前端依赖锁文件、构建脚本和本 lane 报告/派发记录。

## 0. 目的

为 Star Desktop 建立可复现、按操作系统构建并在制品缺失时失败的打包路径。CI 仅生成短期保留的未签名验证制品；本阶段没有发布、创建 tag、部署或推送，也没有宣称已形成可安装的稳定发行版。

## 1. 改动矩阵

| 文件 | 改动 | 结果与边界 |
|---|---|---|
| `.github/workflows/star-stable-desktop.yml` | 新增 Linux `.deb`/AppImage、Windows MSI、macOS universal DMG job；固定 Node.js 24.22.2 与 Rust 1.99.0；将 Cargo 并行度限制为 2；第三方 actions 以已核验的完整提交 SHA 固定；上传制品前要求目标文件存在且非空，上传缺文件设置为 `error`，保留 7 天 | 只在 `dev` PR/push 或手动触发；权限为 `contents: read`；没有 release/tag/deploy/publish job；制品名称明确标为 unsigned |
| `crates/star-desktop/frontend/package.json` | 最低 Node.js 版本改为 `>=24` | 淘汰该新打包路径中的 EOL Node 20 |
| `crates/star-desktop/frontend/package-lock.json` | 新增 npm lockfile v3 | `npm ci` 成功使用锁文件安装 174 个包；依赖许可闭包不在本 lane 的核验范围内 |
| `crates/star-desktop/src-tauri/tauri.conf.json` | 修正前端命令路径；删除启用状态却使用占位公钥的 updater 配置；显式关闭 updater artifact 创建 | 不生成伪签名更新制品；自动更新保持未配置 |
| `crates/star-desktop/src-tauri/Cargo.toml` | 将过时的 PoC 产品描述改成中性桌面客户端描述 | 没有升级 Cargo 依赖或改动 workspace license |
| `scripts/build-desktop.sh` | 改为 lockfile 驱动的 `npm ci`；只在当前宿主系统构建；将选项映射到 Tauri bundle 类型；验证所选 bundle 目录内存在非空安装包 | 删除了跨平台打包失败后仍返回成功的行为；跨系统制品交给独立 CI runner |
| `docs/reports/PHASE-STABLE-HARDENING-DESKTOP-PACKAGING-REPORT.md` | 新增本报告 | 记录真实验证结果和未关闭风险 |
| `docs/briefs/STABLE-HARDENING-DESKTOP-PACKAGING-20261031.md`、`docs/reports/WBS-DISPATCH-STABLE-HARDENING-DESKTOP-PACKAGING-20261031.jsonl` | 本 lane 原有 dispatcher brief 与派发记录 | 保留原 brief；派发记录追加本阶段结果 |

`.github/workflows/star-desktop-build.yml` 未修改；它仍使用 Node 20，因其已有 Dependabot owner，后续需由该 owner 单独收敛。

## 2. 验证摘要

| 验证 | 结果 | 实测细节 |
|---|---|---|
| `npm install --package-lock-only --ignore-scripts --no-audit --no-fund` | 通过，退出码 0 | 在 `crates/star-desktop/frontend` 生成锁文件；本机 Node.js 22.19.0/npm 10.9.3 因项目要求 Node `>=24` 打出 `EBADENGINE` 警告；命令约 6 分钟完成 |
| `npm ci --no-audit --no-fund` | 通过，退出码 0 | 安装 174 个包；同样报告本机 Node 22 低于目标版本，并报告 `whatwg-encoding@3.1.1` 弃用提示；约 2 分钟完成 |
| `npm run build` | 失败，退出码 1 | `tsc --noEmit` 报 15 个错误，涉及 6 个未修改文件，详细列于 §3；工作流会在该步骤停止，后续 Tauri 打包与上传步骤不会运行 |
| Tauri CLI `--version` | 通过，退出码 0 | `tauri-cli 2.12.4` |
| Tauri CLI `info` | 通过，退出码 0 | 识别到 `tauri 2.12.1`、`tauri-build 2.7.1`、WebView2 与 MSVC；本机 Rust 信息中 `rustc` 为空，只有 Cargo 1.99.0 |
| `tauri build --bundles msi` | 失败，退出码 1 | 本机 CLI 在 `crates\tauri-cli\src\interface\rust.rs:1145:14` 因 `Option::unwrap()` panic；未生成 MSI。环境信息显示 `rustc` 组件不可用，不能视为构建通过 |
| `bash -n scripts/build-desktop.sh` | 通过，退出码 0 | Shell 语法通过 |
| ShellCheck 0.11.0 | 通过，退出码 0 | 无 shell lint 错误 |
| JSON/TOML/YAML 解析 | 通过，退出码 0 | `package.json`、lockfile、Tauri JSON、Cargo TOML 与工作流 YAML 均成功解析 |
| CypherGraph Guardian `--changed` | 通过，退出码 0 | 1 个变更源文件通过；工作流 YAML 的关系头另按同一哈希算法校验 |
| `cargo metadata --no-deps --format-version 1` | 通过，退出码 0 | 只证明 workspace manifest 可解析，不证明编译或打包 |
| GitHub Actions hosted CI | 未运行 | 没有本地 hosted runner 结果；本地没有安装 `actionlint`，因此只做 YAML 语法解析，未完成 GitHub 工作流语义 lint |

### 前端 TypeScript 错误清单

| 文件 | 实测问题 |
|---|---|
| `crates/star-desktop/frontend/src/App.tsx` | 找不到 `./App.module.css` 模块声明（TS2307） |
| `crates/star-desktop/frontend/src/components/KanbanBoard.tsx` | `DEFAULT_KANBAN_COLUMNS` 未导出（TS2459）；若干参数隐式 `any`（TS7006）；若干 `Record<WorkItemStatus, WorkItem[]>` 索引为 `any`（TS7053） |
| `crates/star-desktop/frontend/src/hooks/useTauriKeyboardLayout.ts` | `constants` 未导出 `DEFAULT_SWIMLANES`（TS2305） |
| `crates/star-desktop/frontend/src/pages/WorktreeVizPage.tsx` | `visualDistance` 声明后未使用（TS6133） |
| `crates/star-desktop/frontend/tests/BoardView.test.tsx` | `globalThis` 属性索引签名错误（TS7017） |
| `crates/star-desktop/frontend/tests/setup.ts` | `globalThis` 属性索引签名错误（TS7017） |

这些文件不在 desktop packaging lane manifest 内，本 lane 未修改它们。工作流保留严格的 frontend build 门，不通过跳过 typecheck 来掩盖这些问题。

## 3. 已知缺口

1. **当前阻断项**：上述 15 个 TypeScript 错误使 frontend build 返回非零；CI 在制品上传前停止。应由拥有相关前端文件的 lane 修复并重新运行该工作流。
2. **Rust 构建证据缺失**：本机 Tauri build 因 `rustc` 组件缺失而 panic；必须由 CI 中 Rust 1.99.0 的真实 runner 重新验证。
3. **平台产物尚未验证**：尚未运行 Linux、Windows、macOS hosted jobs；没有 `.deb`、AppImage、MSI 或 DMG 的真实构建、上传、下载、安装/卸载和启动冒烟证据。
4. **签名与更新未就绪**：没有 Windows Authenticode 证书、Apple Developer ID/签名/公证凭据或真实 updater 公钥。本阶段移除了占位 updater 配置并关闭 updater artifact 创建；所有工作流制品均未签名，不能对外称为可自动更新的稳定发行版。
5. **旧工作流仍使用 Node 20**：`star-desktop-build.yml` 保持不变，需在原 owner 允许的独立 lane 中更新或退休。
6. **商业许可闭包未审计**：本 lane 生成了 npm lockfile，但没有对 npm 完整传递依赖做 SPDX/SBOM 与商业再分发许可审核；根级许可证闭包 lane 仍须给出结论。
7. **包的实际安装策略未验收**：Windows WebView2、Windows MSI 所需的 VBSCRIPT optional feature/WiX 环境、Linux 系统库和 macOS Gatekeeper 体验仍需在各平台安装验证中确认。Tauri 官方文档说明构建 MSI 需要启用 VBSCRIPT（[Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/#vbscript-for-msi-installers)）；当前 hosted runner 尚未实测此条件。
8. **稳定版本未发布**：没有发布、tag、部署或 push 动作。

## 4. 子代理失败接手清单

本 lane 未派生子代理。实现工作由本 lane worker 完成；工作流语法校验工具 `actionlint` 不在当前环境中，因此未声称通过该检查。根协调者在接收前仍需复核当前前端 blocker、Cargo/Rust 构建缺口、旧 Node 20 工作流 owner 与签名/许可门。

## 5. 守门规则

| # | 本 lane 执行的守门规则 | 结果 |
|---:|---|---|
| 1 | 仅改 MAGOS manifest 内的 desktop packaging 文件与本 lane 报告/派发记录 | 符合 |
| 2 | 以记录的 `dev@04d8b7a84514dcb454874eb852651778e1452dea` 为基线 | 符合 |
| 3 | 不改由 Dependabot owner 占用的 `star-desktop-build.yml` | 符合 |
| 4 | 新工作流不使用 Node 20 | 符合，固定 Node 24.22.2 |
| 5 | Cargo build 使用仓库 Rust baseline | 配置为 Rust 1.99.0；hosted runner 尚未验证 |
| 6 | npm 构建严格使用 lockfile | 符合，使用 `npm ci` |
| 7 | 不将跨操作系统打包伪装为一次本地 build | 符合，每个平台由本地 runner 构建 |
| 8 | 缺少任一选定产物时 job 必须非零失败 | 符合，构建后显式检查非空文件，上传使用 `if-no-files-found: error` |
| 9 | 工作流只上传短期验证 artifact | 符合，保留 7 天，名称标注 unsigned |
| 10 | 仓库权限采用最小权限 | 符合，`contents: read` |
| 11 | GitHub actions 固定到完整 commit SHA | 符合，checkout/setup-node/upload-artifact 均固定 |
| 12 | 不把 updater placeholder 当成有效密钥 | 符合，删除 updater placeholder 配置并关闭 updater artifacts |
| 13 | 不生成或声称拥有签名凭据 | 符合 |
| 14 | 不以跳过 TypeScript 检查换取表面打包成功 | 符合，失败会终止 job |
| 15 | 不发布、打 tag、部署或 push | 符合 |
| 16 | 代码/脚本 Cypher 图关系与源码一致 | 符合，CypherGraph Guardian 检查通过 |
| 17 | 对未执行的平台、签名、安装与授权审查明确标缺口 | 符合，详见 §2–§3 |

## 6. 审查栏

| 角色 | 状态 |
|---|---|
| 架构 | Mavis 接手审核；等待根协调者按当前 dev 基线复核 |
| SRE Lead | Mavis 接手审核；真实 hosted runner 与签名证据待验证 |
| 平台 | Mavis 接手审核；Cargo build 与平台依赖待验证 |
| 评审主持 | Mavis 接手审核；本报告未将 lane 标为 READY |
| PM | Mavis 接手审核；稳定版本整体闭环仍依赖其他 lane |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 desktop packaging 实施与验证报告；记录前端 typecheck、Rust toolchain、hosted CI、签名和许可闭环缺口 |
