# W3 Star Canvas legacy 上下文过滤审阅 — 2026-11-02

## 决定

**静态过滤缺口已关闭并通过本机执行验证；W3 整体仍 OPEN，v1.0.6 稳定发行仍 NO-GO。**

被审阅的缺陷是真实的，且已实测确认其影响范围：原 `.github/workflows/publish-star-canvas-services.yml` 的上下文文件检查只做 `"game" in path.lower()`，Cargo 包检查只做精确等值 `star-game`。因此 `legacy_compat.rs` 这类源文件、以及名为 `legacy-auth-shim` / `GAME_UTILS` / `game-legacy-core` 的锁包（含 registry 包）都能进入 BuildKit 上下文而不被拒绝。

修复引入单一共享策略 `scripts/release/star_canvas_context_policy.py`（`FORBIDDEN_TERMS = ("game", "legacy")`，整值 `casefold()` 子串匹配），并在 workflow 的上下文组装步骤中 4 处接入：镜像名、锁包身份闭包、过滤后锁包名、生成上下文路径。策略导入失败即失败关闭。

diff 为纯增量（+24 / −1）。唯一删除行是结尾 `print` 文案；全部既有 allowlist、path 依赖、workspace 依赖、lock 闭包唯一可解析、`star-game` 精确检查、过滤锁一致性、root 版本绑定与 `.git/` / `target/` 前缀检查逐项保留。allowlist 未被放宽——策略模块自身不持有任何镜像名单，只校验调用方传入的集合。

## 来源与边界

- 审阅对象为 lane worktree `D:\orcaWork\Star\star-stable-w3-legacy-context-20261102`，基线 HEAD 与 `dev` 同为 `09b4eead99b047759be271ade2f5b06c0612ffb1`。未 fetch、未 push、未打 tag、未发布、未部署；`main`、`D:\Star`、远端 ref 与其他 worktree 未触碰。
- 与 W7 并行。本 lane **未修改共享 WBS** `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md`，也未修改 W7 拥有的 `scripts/automation/star_k3s_context.py` 与 `deploy/k3s-local/*`。W3 状态以建议值记录在阶段报告 §6，由协调者在两个代码 lane 评审后同步。
- 验证方式是**执行 workflow 内联脚本本体**（抽取唯一的 PowerShell here-string，`textwrap.dedent` 后以 `python -` 运行），而非仅静态阅读。这是本 lane 最强的证据形式，但仍不等于 hosted run。
- 未运行 BuildKit / `docker build-push-action` / GHCR login / attestation / K3s。没有任何真实 image digest、SBOM、provenance、消费侧或 rollback 证据。
- 未在 GitHub hosted ubuntu runner 上以 `python3 -` + PowerShell here-string 的真实组合执行；here-string 的端到端传递语义未验证。

## 审阅检查

| 检查 | 结果 |
|---|---|
| 缺陷是否真实存在（legacy 路径与包名可绕过） | 是；已由 M1 / M4 变异在真实内联脚本上复现，并由专门测试断言其在旧规则下不可见 |
| 既有 game 检查是否保留 | 保留；M6（`GamePad.rs`）实测仍由既有检查先触发 |
| 既有 Cargo lock 闭包 / 身份 / 版本门禁是否保留 | 保留；逐项静态核对，且既有 18 个 release 测试未回归 |
| allowlist 是否被放宽 | 否；两处 `allowed = {…}` 与 matrix `fromJSON([...])` 经精确行断言仍为三项 |
| 大小写不敏感 | 是；M2（`GAME_UTILS`）、M5（`LegacyCompat.rs`）实测被拒 |
| 失败关闭 | 是；M8（策略模块缺失）实测 exit≠0 且 `GITHUB_OUTPUT` 为空 |
| 变异测试 + 对照组（守门 v30） | 8 个变异全红，对照组绿（`2 locked packages; 8 files`，写出 `directory=`） |
| 真实仓库三镜像上下文门 | 3/3 exit=0；119 / 119 / 136 锁定包，各 9 files |
| 真实闭包是否被误伤 | 否；根 `Cargo.lock:7165` 存在 `star-game`，但三个闭包中无含 game / legacy 的 name / version / source |
| 本机测试 | `Ran 39 tests in 25.702s` → `OK`（21 新增 + 18 既有） |
| `@cypher` header 盖章 | Guardian `"ok": true`，`"errors": []`；使用已安装 skill 路径 validator（仓库内无 canonical），已在报告 §6 说明 |
| `git diff --check` | clean |
| workflow YAML 可解析 | 4 jobs，`build-and-push` 13 steps |
| hosted workflow / 镜像构建 / 制品 | **未执行** |
| CI 是否覆盖这些测试 | **否**；无 CI job 执行 `scripts/release/tests/`，39/39 仅为本机证据 |
| W3 / v1.0.6 状态 | OPEN / NO-GO |

## 遗留判断

- 新规则对第三方依赖是硬失败：未来若引入名字含 `game` 或 `legacy` 的合法 crates.io 依赖，发布门禁会拒绝。这是有意的 fail-closed 行为，不应加静默豁免；真出现时需 owner 显式决策。
- 仓库内现有两处禁止词规则（本 lane 的整值匹配、W7 `star_k3s_context.py` 的逐段匹配），语义强度不同。跨 lane 去重应由协调者决定，本 lane 未越界统一。
- 上下文过滤通过**不等于**最终镜像内容合格。W3 闭合仍需真实 gated run 产生的三个不可变 digest、逐文件 context 清单与哈希、最终镜像 rootfs 检查、SBOM / provenance / attestation，以及 digest 固定的 Envoy 运行时依赖证据。
- 未解决且不属于本 lane 的既有阻塞：`Cargo.toml` 的 `license = "Apache-2.0"` 与 `LICENSE` 首行 `MIT License` 不一致（会在上下文组装前触发 MIT 门禁失败）；编译进二进制的版本仍为 `0.1.0` 而非 tag 派生版本。

## 修订历史

| 版本 | 日期 | 内容 |
|---|---|---|
| v0.1 | 2026-11-02 | 确认 legacy 过滤缺口真实存在并已关闭；记录纯增量 diff、8 变异全红 + 对照组绿、真实仓库三镜像 3/3 通过、39/39 本机测试与 Guardian 盖章；同时保留 hosted run、CI 覆盖、镜像/SBOM/provenance 与 license/version 阻塞缺口，W3 OPEN、v1.0.6 NO-GO。 |
