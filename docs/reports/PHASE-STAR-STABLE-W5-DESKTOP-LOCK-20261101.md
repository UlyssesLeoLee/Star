# PHASE-STAR-STABLE-W5-DESKTOP-LOCK-20261101

## §0 目的

为稳定版 Desktop 的 Linux、Windows、macOS 三个打包步骤强制 Cargo 使用现有 `Cargo.lock`。本 lane 基于 `dev@2a57fa0d329be830735d6657677236e195673362`，只修改登记的 `.github/workflows/star-stable-desktop.yml`；不创建 tag/release、不构建或上传安装包、不部署。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `.github/workflows/star-stable-desktop.yml` | Linux deb/AppImage、Windows MSI、macOS universal DMG 三条 Tauri build 命令都在 Tauri 选项后追加 `-- --locked`，把 `--locked` 作为 runner 参数传给 Cargo。 | 已修改 |
| `docs/briefs/STAR-STABLE-W5-DESKTOP-LOCK-20261101.md` | 记录 lane 范围、参数传递核验条件和禁止的发布动作。 | 已由 dispatcher 落档 |
| `docs/reports/WBS-DISPATCH-STAR-STABLE-W5-DESKTOP-LOCK-20261101.jsonl` | dispatcher brief 事件及本 lane 静态验证结果均已记录。 | 已完成 |

**参数依据：** 当前 `frontend/package-lock.json` 把 `@tauri-apps/cli` 固定为 `2.12.4`，workflow 以 `npm ci` 安装。该上游版本的 [build.rs](https://raw.githubusercontent.com/tauri-apps/tauri/tauri-cli-v2.12.4/crates/tauri-cli/src/build.rs) 将 `build` 的 `args` 定义为传给 runner 的命令行参数，并注明用 `--` 标记参数起点；同一版本的 [desktop.rs](https://raw.githubusercontent.com/tauri-apps/tauri/tauri-cli-v2.12.4/crates/tauri-cli/src/interface/rust/desktop.rs) 把 `options.args` 交给 Cargo command。工作流中的 `-- --locked` 因而由 Tauri CLI 传给其 Cargo runner，而不是作为 Tauri 自身选项。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk proxy git -C C:\Users\leo19\.codex\worktrees\star-stable-release-wave4-desktop-lock diff --check` | PASS；无空白错误。 |
| `rtk proxy rg -n 'tauri(\.cmd)? build' C:\Users\leo19\.codex\worktrees\star-stable-release-wave4-desktop-lock\.github\workflows\star-stable-desktop.yml` | PASS；三条 build 命令均包含 `-- --locked`。 |
| `rtk proxy python -c "import yaml; from pathlib import Path; p=Path(r'C:\Users\leo19\.codex\worktrees\star-stable-release-wave4-desktop-lock/.github/workflows/star-stable-desktop.yml'); data=yaml.load(p.read_text(encoding='utf-8'),Loader=yaml.BaseLoader); assert isinstance(data,dict); assert 'jobs' in data; print('YAML parse PASS; top-level jobs:',len(data['jobs']))"` | PASS；YAML 解析成功，顶层包含 7 个 jobs。 |
| `actionlint` | 未运行；本机未安装该工具。 |
| workflow 的本地 pinned Tauri CLI help | 未运行；lane 中未安装 `node_modules/.bin/tauri`，没有为此安装依赖。参数转发由该 CLI 的对应上游 tag 源码核实。 |
| Rust/Tauri 原生打包、hosted workflow、签名/公证、制品安装升级 | 未运行；本 lane 不证明这些发布能力。 |

曾用 PyYAML `safe_load` 做显示键排序，YAML 1.1 将 `on` 解析为布尔值导致排序表达式报 `TypeError`；YAML 文件已被 `BaseLoader` 成功解析，前一错误是诊断显示代码问题，不是 YAML 解析失败。

## §3 已知缺口

1. 这只强制 Desktop Tauri 的 Cargo 命令锁定依赖；不表示 Rust 1.99.0 基线、workspace all-targets、license/advisory、签名、公证或安装回滚验收已通过。
2. `actionlint` 和本机安装的 Tauri CLI 不可用；仅执行了 YAML 解析、静态命令检查与 diff 检查。
3. 无候选 source commit 的 hosted run；本地 diff 不能替代三个操作系统的真实构建与消费者安装验证。
4. 用户指定的 canonical tag `v1.0.6` 与 accountable release owner 已记录于主 WBS；本 lane 不产生 tag，也不证明包元数据与该 tag 已一致。

## §4 子代理失败接手清单

- 本 lane 没有再派生子代理；无失败、超时或未接收的子代理结果需要接手。
- 本 lane 的结果仍需协调者独立 review，确认参数顺序与 workflow 的三个操作系统 job 一致后才可标为 ready。

## §5 守门规则

1. 只修改已登记的稳定 Desktop workflow 与 lane 报告文件。
2. 只给三个正式 Desktop 打包步骤加锁定参数，不扩大产品范围。
3. 以 lockfile 中实际安装的 Tauri CLI `2.12.4` 源码证明 runner 参数转发。
4. `--locked` 必须位于 Tauri CLI 的 `--` 参数分隔符之后。
5. 不修改 `Cargo.lock`、依赖版本或 package manager。
6. 不改 release trigger、审批、artifact 命名或 provenance 条件。
7. 不创建 `v1.0.6` 或其他 Git tag。
8. 不创建 GitHub Release。
9. 不构建或上传 Desktop artifact。
10. 不使用或读取任何发布凭据。
11. 不部署、不安装、不升级用户设备。
12. 不写入 `dev` 或 `main`。
13. `git diff --check` 必须通过。
14. YAML 静态解析和三个命令的锁参数检查必须通过。
15. 未执行的 actionlint、hosted workflow、原生构建和安装验收必须保持标记为未验证。
16. 不把本 lane 的成功写成稳定版发布批准或完整 W5 关闭。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | Mavis 接手审核：静态源码证明 Cargo passthrough 路径；不批准发布。 |
| SRE Lead | Mavis 接手审核：hosted build 与签名/公证 evidence 缺失；W5 仍 OPEN。 |
| 平台 | Mavis 接手审核：三平台 Tauri build 命令均传递 `--locked`；actionlint 未安装。 |
| 评审主持 | Mavis 接手审核：仅 workflow 静态改动；需协调者审阅与宿主运行验证。 |
| PM | Mavis 接手审核：不授权提前创建 tag、release 或制品。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 给三个稳定 Desktop 打包命令追加 Cargo `--locked` runner 参数，并记录 pinned CLI 上游源码依据与未完成的 hosted/原生打包验收。 | W5 stable release workflow lock audit |
