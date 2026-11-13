# PHASE — Star Stable W5 Desktop Node 24 Update

Date: 2026-11-03
Lane: `codex/star-stable-w5-node24-20261103`
Baseline: `dev@92f98f4a8699990227465722777a9edb9baab670`

## §0 目的与范围

将桌面构建 workflow 从已结束支持的 Node 20 更新到官方维护中的 Node 24.22.2，并保持界面步骤名称与实际安装版本一致。Node 24.22.2 于 2026-10-28 发布为 LTS；Node 20 于 2026-03-24 到达 EOL。依据：[Node 24.22.2 release](https://nodejs.org/en/blog/release/v24.22.2/) 与 [Node.js EOL schedule](https://nodejs.org/en/about/eol)。`actions/setup-node` 的 `node-version` 支持精确 SemVer，参见[官方版本语法文档](https://github.com/actions/setup-node#supported-version-syntax)。

本 lane 仅修改 `.github/workflows/star-desktop-build.yml` 中两个 step labels 和对应两个 `node-version` 值，以及 WBS 中的 W5 证据和单条修订记录，并新增本阶段报告。没有修改 Dependabot PR/分支、`main`、远端或其他 worktree；没有运行产品测试或 hosted workflow。W5 保持 OPEN，v1.0.6 保持 NO-GO。

## §1 改动矩阵

| 项 | 修改 | 结果 |
|---|---|---|
| Desktop workflow | 两个步骤名称从 `Install Node.js 20` 改为 `Install Node.js 24.22.2`；两个 `node-version` 输入改为 `'24.22.2'` | 显示名称与实际安装版本一致；其他 workflow 内容保持原样 |
| Stable Release WBS | 只更新 W5 行证据并追加 v1.50 修订记录 | W5 仍 OPEN；v1.0.6 仍 NO-GO，未把本地静态检查提升为发行证明 |
| 本阶段报告 | 记录范围、静态检查、未关闭门禁和修订历史 | 不声称 hosted CI、原生打包或发行验收通过 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| PyYAML YAML 解析与固定版本断言 | 通过：workflow 可解析，两个 `node-version` 值均为 `24.22.2`；对应两个显示名称也已同步 |
| Node 20 输入检查 | 通过：workflow 中无 `node-version: '20'` |
| `rtk proxy git diff --check` | 通过；仅报告工作树 CRLF 将在 Git 写入时规范为 LF 的提示 |`r`n| 协调者独立复核 | 通过：只含 brief 授权的三个实现文件，Node 版本和标签、WBS 标题/行数/修订号正确；无范围外改动 |`r`n| CodeRabbit 第二意见 | 未能运行：CLI 启动返回 `/root/.local/bin/coderabbit: Permission denied`；由协调者直接审阅差异与静态验证 |
| `actionlint` 可用性 | 不可用：当前 PATH 没有 actionlint，未能执行 action 专项静态分析 |
| 产品测试与 hosted workflow | 未运行，按任务 brief 明确排除 |

## §3 已知缺口

1. 没有 hosted Node 24 workflow run，因此未验证 GitHub runner、缓存、矩阵、依赖安装或后续步骤的真实行为。
2. 没有 Linux、Windows、macOS 原生/Tauri 发布包及资产上传证据。
3. 签名密钥管理、代码签名、公证和可下载签名资产尚未验证。
4. Desktop BOM、签名与 provenance/attestation 及独立消费者验证仍缺。
5. 真实安装、升级、更新通道、回滚与 CSP 验收记录仍缺。
6. 因此 W5 保持 OPEN，v1.0.6 稳定发行候选保持 NO-GO。

## §4 子代理失败接手清单

本 lane 由指定 worker 直接实施，没有额外派发子代理，故无子代理失败待接手项。协调者已独立审查精确 diff 与 WBS 证据；目标 dev 快进集成及 worktree 回收仍待完成。worker commit 本身不表示已集成或已关闭 W5。

## §5 守门规则

1. 仅修改 brief 登记的 workflow、WBS 和本阶段报告。
2. Node 输入固定为精确 SemVer `24.22.2`。
3. 两个步骤显示名称必须与安装的 Node 版本一致。
4. 不将 Node 24 workflow 升级扩大为其他 action、toolchain 或依赖更新。
5. 不修改远端 Dependabot 分支或 PR。
6. 不触碰 `main`、远端 refs 或其他 worktree。
7. YAML 解析和静态断言不等同于 hosted workflow 成功。
8. 不得把本地静态检查描述为产品测试通过。
9. 不运行本任务明确排除的产品测试或 hosted workflow。
10. `actionlint` 不可用必须如实记录，不以其他工具伪称等价覆盖。
11. W5 在 hosted Node 24 与桌面发行闭环证据齐备前保持 OPEN。
12. v1.0.6 在 W0–W9 全部门禁通过前保持 NO-GO。
13. 不得声称存在本 lane 未生成的原生包、签名、公证、BOM 或 attestation。
14. 未验证安装、升级、更新、回滚与 CSP 前不得关闭相应验收项。
15. WBS 只记入本轮实际完成的 W5 证据，不重写其他工作项状态。
16. 合并前独立核对只含受托文件和允许的最小差异。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— W5 仅更新 Node 版本输入和对应步骤标签 |
| SRE Lead | SRE Lead（Mavis 接手审核）— 未运行 hosted workflow，运行态仍待验证 |
| 平台 | 平台（Mavis 接手审核）— PyYAML 与版本断言通过；actionlint 不可用 |
| 评审主持 | 评审主持（Mavis 接手审核）— worker 自检与协调者独立复核完成，dev 集成和 worktree 回收待执行 |
| PM | PM（Mavis 接手审核）— W5 OPEN，v1.0.6 NO-GO |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更新两处 Node.js 步骤标签和两个版本输入，记录静态验证与未关闭门禁。 | W5 需要将 Desktop workflow 从 EOL Node 20 转为维护中的 Node 24 LTS |
