# PHASE-STAR-STABLE-W3-LEGACY-CONTEXT-20261102

> **状态：W3 该子项实现完成并通过本机定向验证；整体 v1.0.6 稳定发行仍为 NO-GO。** 本次改动只关闭 Canvas 发布上下文的静态 legacy 过滤缺口，没有构建、发布、部署任何镜像，也没有产生 SBOM、provenance、digest、attestation、消费侧或 rollback 证据。

## §0 目的

按 brief `docs/briefs/STAR-STABLE-W3-LEGACY-CONTEXT-20261102.md` 加固 Star Canvas 镜像发布上下文，使其拒绝非 Star 的 game / legacy 载荷。

已确认的缺陷（静态阅读 `.github/workflows/publish-star-canvas-services.yml` 原始内容）：

- 上下文文件检查只做 `"game" in path.lower()`，**不含 `legacy`**，因此 `crates/canvas-engine/src/legacy_compat.rs` 这类文件可以进入 BuildKit 上下文。
- Cargo 包检查只做精确等值 `identity[0] == "star-game"` 与 `"star-game" in filtered_names`，**不含子串、不含大小写变体、不含 `legacy`**，因此名为 `legacy-auth-shim`、`GAME_UTILS`、`game-legacy-core` 的包（含 registry 包）可以进入解析出的依赖闭包。

Lane 基线：worktree HEAD 与 `dev` 同为 `09b4eead99b047759be271ade2f5b06c0612ffb1`；起始工作树只含 brief 与 dispatch JSONL 两个未跟踪文件。

本 lane 与 W7 并行，**未修改共享的 stable-release WBS**（`docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md`），未 revert 或覆盖其他 lane 的工作。

## §1 改动矩阵

| 文件 | 改动 | 说明 |
|---|---|---|
| `.github/workflows/publish-star-canvas-services.yml` | 修改（+24 / −1） | 在 "Assemble and verify a minimal per-service Cargo context" 步骤内导入共享策略并从 checkout 失败关闭；新增 4 处策略调用（镜像名、锁包身份闭包、过滤后锁包名、生成上下文路径）。唯一删除行是结尾 `print` 文案由 `no game or foreign workspace sources` 改为 `no game/legacy or foreign workspace sources`。**未删除任何既有检查，未放宽 allowlist。** |
| `scripts/release/star_canvas_context_policy.py` | 新增（177 行） | 共享 fail-closed 策略：`FORBIDDEN_TERMS = ("game", "legacy")`、`CanvasContextError`、`forbidden_matches`、`ensure_allowed_image`、`ensure_clean_package_names`、`ensure_clean_package_identities`、`ensure_clean_context_paths`。规则确定性、大小写不敏感，违规项排序后一次性报出。 |
| `scripts/release/tests/test_star_canvas_context_policy.py` | 新增（502 行） | 21 个定向测试：策略规则、workflow 静态接线与顺序、allowlist 未被放宽，以及**真实执行** workflow 内联脚本的对照组 + 8 个变异组。 |
| `docs/reports/PHASE-STAR-STABLE-W3-LEGACY-CONTEXT-20261102.md` | 新增 | 本报告。 |
| `docs/reviews/WBS-STAR-STABLE-W3-LEGACY-CONTEXT-REVIEW-20261102.md` | 新增 | 本 lane review。 |
| `docs/reports/WBS-DISPATCH-STAR-STABLE-W3-LEGACY-CONTEXT-20261102.jsonl` | 追加 | dispatch / validation 审计记录。 |
| `docs/briefs/STAR-STABLE-W3-LEGACY-CONTEXT-20261102.md` | 未修改 | dispatcher 生成，创建后不编辑。 |

### 保留的既有门禁（逐项静态核对，未改动）

`allowed = {"canvas-engine", "domain-canvas", "canvas-realtime"}` 镜像 allowlist；`Refusing unapproved Canvas image`；`fromJSON('["canvas-engine","domain-canvas","canvas-realtime"]')` 矩阵；manifest name/build script 检查；`version.workspace = true` 继承检查；`Local path dependency is not allowed in isolated image context`；`Workspace dependency resolves to a local product crate`；lock 依赖引用唯一可解析检查；`Canvas package closure reaches non-Canvas workspace members`；`Canvas package closure unexpectedly includes star-game.`；`Filtered Cargo.lock failed the isolated package closure check.`；`Filtered Cargo.lock root does not match the resolved release version.`；`Unexpected files in image context`（含 `.git/`、`target/` 前缀）；`Image context contains a crate outside the allowlisted service.`；tag/version/source_sha 正则门禁。

### 新规则语义

- 匹配为整值 `casefold()` 子串匹配，**强于**逐路径分段匹配（分段匹配是整值匹配的子集）。
- `None` 视为「可选组件缺失」（workspace-local Cargo 包没有 `source`），不匹配任何 term；这是 Cargo.lock 的合法语义，不是放宽。
- 空串 / 纯空白 → `<empty-identity>` 违规；非 str 且非 tuple/list → `<unprintable-identity>` 违规。载荷无法被证明时失败关闭。
- 包身份检查覆盖 name / version / source 三个分量，因此禁止词无法通过改名包、vendored path 或非 crates.io source 引用进入闭包。
- `ensure_allowed_image` 要求名字**精确**属于调用方传入的 allowlist，策略模块自身不持有任何镜像名单，因此不可能放宽 allowlist。

## §2 验证摘要（本机实测）

环境：Windows，Python 3.13.7，PyYAML 6.0.3。仓库根 `D:\orcaWork\Star\star-stable-w3-legacy-context-20261102`。

| 检查 | 命令 | 结果 |
|---|---|---|
| 新增定向测试 | `python -m unittest scripts.release.tests.test_star_canvas_context_policy` | `Ran 21 tests in 1.506s` → `OK`（21/21） |
| 三个 release 测试模块合计 | `python -m unittest scripts.release.tests.test_star_canvas_context_policy scripts.release.tests.test_render_star_canvas_manifest scripts.release.tests.test_resolve_star_version` | `Ran 39 tests in 25.702s` → `OK`（含既有 18 个测试未回归） |
| 真实仓库上下文门（执行 workflow 内联脚本本体） | 抽取 workflow 中唯一 PowerShell here-string，以 `GITHUB_WORKSPACE` 指向真实仓库、`RUNNER_TEMP` 指向临时目录，对三个镜像各执行一次 | **3/3 exit=0**：`Verified canvas-engine: 119 locked packages; 9 files; no game/legacy or foreign workspace sources.`；`Verified domain-canvas: 119 locked packages; 9 files; …`；`Verified canvas-realtime: 136 locked packages; 9 files; …`。三者均写出 `directory=<RUNNER_TEMP>/star-canvas-oci/<image>` |
| 变异对照（守门 v30：门禁必须有变异测试且对照组保持绿） | 同一内联脚本对合成 fixture workspace 执行 | 对照组 exit=0（`2 locked packages; 8 files`）；8 个变异全部 exit≠0 且 `GITHUB_OUTPUT` 为空 |
| CypherGraph Guardian | `cgg_guard.py --root . --stamp --json scripts/release/star_canvas_context_policy.py scripts/release/tests/test_star_canvas_context_policy.py` | `"ok": true`，`"errors": []` |
| workflow YAML 可解析 | `yaml.safe_load(...)` | jobs = `['build-and-push', 'render-release-manifest', 'resolve_release', 'test']`，`build-and-push` 13 steps |
| 空白错误 | `git diff --check` | clean |

### 变异清单（对照组绿 + 变异全红）

| # | 变异 | 结果 | 由哪条规则拒绝 |
|---|---|---|---|
| 控制 | 干净 Canvas fixture（`canvas-engine` + `serde`） | exit=0，8 files，写出 `directory=` | — |
| M1 | lock 闭包加入 registry 包 `legacy-auth-shim` | exit≠0 | **新规则**（旧精确 `star-game` 检查与 `local_dependencies` 检查都放过它，因为它有 source 且名字不等于 `star-game`） |
| M2 | lock 闭包加入 registry 包 `GAME_UTILS`（大写） | exit≠0 | **新规则**（大小写不敏感） |
| M3 | lock 闭包加入 local 包 `star-game` | exit≠0 | 新规则 + 既有检查（双重） |
| M4 | 源文件 `crates/canvas-engine/src/legacy_compat.rs` | exit≠0 | **新规则**（旧 `"game" in path.lower()` 放过它） |
| M5 | 源文件 `crates/canvas-engine/src/LegacyCompat.rs`（大写 legacy） | exit≠0 | **新规则**（大小写不敏感） |
| M6 | 源文件 `crates/canvas-engine/src/GamePad.rs` | exit≠0 | 既有 game 检查先触发（证明既有检查未被移除且仍生效） |
| M7 | `CANVAS_IMAGE` 为 `star-game` / `legacy-canvas` / `canvas-engine-legacy` | exit≠0 | 既有 allowlist 检查（`Refusing unapproved Canvas image`） |
| M8 | fixture 中删除策略模块 | exit≠0 | **新规则**（导入失败即失败关闭，`Star Canvas context policy helper is unavailable`） |

M1 / M4 由 `test_mutated_payloads_are_invisible_to_the_previous_rule` 单独断言：这些载荷在旧规则下不可见（`"game" not in "crates/canvas-engine/src/legacy_compat.rs".lower()`、`"legacy-auth-shim" != "star-game"`），因此它们证明的是**新增覆盖**，而不是既有检查的重复。

### 真实闭包未被误伤的证据

根 `Cargo.lock` 第 7165 行确实存在 `name = "star-game"`，但三个 Canvas 服务的解析闭包（119 / 119 / 136 个锁定包）中均无含 `game` 或 `legacy` 的包名、版本或 source，因此新规则不改变当前发布路径的可构建性。

## §3 已知缺口

| 优先级 | 缺口 | 后续验收 |
|---|---|---|
| P0 | **未构建、未发布、未部署任何镜像。** 本次只执行了上下文组装与校验步骤的 Python 本体；没有运行 BuildKit、`docker build-push-action`、GHCR login、attestation 或 K3s 部署。 | 需要一次真实的 gated workflow run，产生三个不可变 image digest、SBOM、provenance 与 attestation，并做消费侧验证。 |
| P0 | **无 CI 覆盖这些测试。** 现有 workflow 中只有 `pre-tool-use-guard.yml` 跑 `scripts/automation/guardian/tests/`，Canvas workflow 的 `test` job 只跑 `cargo check` / `cargo test`。`scripts/release/tests/` 目前没有任何 CI job 执行，本次 39/39 只是本机证据。 | 需要由拥有相应 workflow 的 lane 把 `scripts/release/tests/` 接入 CI，否则回归只能靠人工本机执行发现。 |
| P0 | 稳定发行的其他既有阻塞未解决，且不属于本 lane 文件：`Cargo.toml` 的 `[workspace.package] license = "Apache-2.0"` 与 `LICENSE` 首行 `MIT License` 不一致，会在上下文组装之前触发 workflow 的 MIT 门禁失败；编译进二进制的版本仍是 `0.1.0` 而非 tag 派生版本。 | 由 W1 / W0 相应 lane 处理；本 lane 未修改 `Cargo.toml`、`LICENSE` 或版本解析逻辑。 |
| P1 | **策略重复未统一。** `scripts/automation/star_k3s_context.py`（W7 lane 所有）自带 `FORBIDDEN_TERMS = ("game", "legacy")` 与 `_is_forbidden()`，其逐路径分段匹配弱于本 lane 的整值匹配。两处规则现在并存。 | 跨 lane 去重需由协调者决定；本 lane 未修改 W7 拥有的文件。 |
| P1 | **新规则对第三方依赖是硬失败。** 若未来引入名字中含 `game` 或 `legacy` 的合法 crates.io 依赖，发布门禁会拒绝它。当前三个闭包无此情况（已实测），但没有例外/豁免机制。 | 这是有意的 fail-closed 行为；若真出现，需 owner 显式决策（改名、换依赖或经评审调整规则），不应静默放行。 |
| P2 | `deploy/k3s-local` 本地构建路径不在本 lane 范围，未改动；其载荷边界由 W7 lane 负责。 | 见 W7 lane 报告。 |
| P2 | 本次验证在 Windows 上以 `python -` 执行内联脚本，未在 GitHub hosted ubuntu runner 上以 `python3 -` + PowerShell here-string 的真实组合执行。PowerShell here-string 的传递语义未做端到端验证。 | 需要一次真实 hosted run。 |

## §4 子代理失败接手清单

- 本 lane 未派发子代理，全部工作由当前 worker 直接完成，无需接手。
- 未读取、移植或覆盖其他 worktree / lane 的内容；`D:\Star`、`main`、远端 ref 均未触碰。
- 未 fetch、未 push、未创建 tag / release、未 build / publish / deploy。
- 过程中发现并修复了两个自身缺陷（非上游遗留）：策略模块最初把 `None`（local 包缺失的 `source`）判为违规，导致真实仓库三个镜像全部被误拒；`_describe()` 对非序列值抛 `TypeError`。两者均由本机执行真实内联脚本与定向测试当场暴露并修复，修复后 3/3 与 21/21 通过。

## §5 守门规则

1. 只写本 lane 登记文件；brief 由 dispatcher 生成，创建后不编辑。
2. 不修改共享 stable-release WBS（W7 lane 拥有）；W3 状态以「建议值」形式记录在本报告 §6，由协调者在两个代码 lane 评审后同步。
3. 保留全部既有 game 检查与 Cargo lock 闭包 / 身份 / 版本门禁，不删除、不减弱。
4. 不放宽镜像或包 allowlist；策略模块自身不持有 allowlist，只接受调用方传入。
5. 新规则确定性、大小写不敏感、失败关闭；违规项排序后一次性报出，便于复现。
6. 载荷无法证明时（空值、非字符串）判为违规，不默认放行。
7. 门禁必须配变异测试，且对照组保持绿（守门 v30）：8 个变异全红 + 1 个对照组绿。
8. 变异必须证明是**新增覆盖**：M1 / M4 由专门测试断言其在旧规则下不可见。
9. 断言必须能区分「被测物没跑」：执行类测试断言 stdout 的 `Verified …` 计数行与 `GITHUB_OUTPUT` 的 `directory=` 写出；拒绝路径断言 `GITHUB_OUTPUT` 为空。
10. 真实仓库上下文门必须实测，不能只靠静态阅读声明通过。
11. 不把「上下文过滤通过」描述成「镜像已构建 / 已发布 / 内容已验证」。
12. 不声称 SBOM、provenance、digest、attestation、消费侧或 rollback 证据——这些操作没有运行。
13. v1.0.6 稳定发行保持 NO-GO；W3 保持未闭合，直到真实 gated run 产生制品证据。
14. 不修改其他 lane 拥有的文件（`scripts/automation/star_k3s_context.py`、`deploy/k3s-local/*`、`Cargo.toml`、`LICENSE`）。
15. `@cypher` header 必须由工具盖章，不得手写或推算 `source_sha256`。仓库内无 canonical validator，改用已安装 skill 路径的 validator（见 §2 与 §6），并在报告说明。
16. 不 fetch、不 push、不打 tag、不发布、不部署；不动 `main`、`D:\Star`、其他 worktree 与远端 ref。
17. 静态校验与本机执行不代替 hosted CI、镜像构建与发行验收。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 共享策略单点持有禁止词规则，workflow 4 处调用全部位于镜像构建之前；allowlist 与既有 lock 门禁逐项保留，diff 为纯增量（+24 / −1，唯一删除行是文案）。 |
| SRE Lead | SRE Lead（Mavis 接手审核）— 本机执行真实内联脚本 3/3 通过（119 / 119 / 136 锁定包）；未执行任何 hosted run、镜像构建或部署，运行门仍阻塞。 |
| 平台 | 平台（Mavis 接手审核）— 策略导入失败即失败关闭（M8 已实测）；`scripts/release/tests/` 目前无 CI job 执行，已登记为 P0 缺口。 |
| 评审主持 | 评审主持（Mavis 接手审核）— 8 个变异全红且对照组绿；M1 / M4 已单独断言在旧规则下不可见，证明为新增覆盖而非重复。 |
| PM | PM（Mavis 接手审核）— v1.0.6 仍 NO-GO，不创建 release，不声称任何制品证据。 |

### 建议的 W3 状态（供协调者同步 WBS，本 lane 未写 WBS）

- W3 / legacy 上下文过滤子项：**静态实现 + 本机定向验证完成**（代码已落地，39/39 测试通过，真实仓库三镜像上下文门 3/3 通过，8 变异全红 + 对照组绿）。
- W3 整体：**仍 OPEN**。缺 hosted workflow run、真实 image digest、SBOM / provenance / attestation、消费侧验证与 rollback 证据；且 CI 未覆盖 `scripts/release/tests/`。
- v1.0.6 稳定发行：**NO-GO**。
- 未触碰的既有阻塞：Cargo license `Apache-2.0` vs `LICENSE` 的 `MIT License` 不一致；二进制版本仍为 `0.1.0`。

### 工具边界说明

仓库内不存在 canonical `.agents` validator 或 `cgg_guard.py`。按既有 brief 先例（`docs/briefs/wld5-stale-session-401-regression-20261023.md` §6：canonical validator 不存在时使用已安装 skill 路径的 validator 并在报告说明），本次使用 `C:\Users\leo19\.claude\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py` 盖章并校验，结果 `"ok": true`。盖章值：`scripts/release/star_canvas_context_policy.py` = `3efc645ceabf3b0cc59264a97e2d293188c08055949193a96004ccfbdbdaa261`；`scripts/release/tests/test_star_canvas_context_policy.py` = `c5565a6f45bb9a62282c2a76d5b82f74ebbedc6802e28bd2152d33b2b0862f84`。该工具只做结构校验，语义边由本 lane 自行核对。

另记录一处工具限制：`cgg_guard.py` 的 `FILE_PATH_RE` 校验对路径做 `lstrip("./")`，会把 `.github/workflows/...` 削成 `github/workflows/...` 并报「File path target does not exist」。因此测试文件 header 中 workflow 以 `Config` 节点（`id:"publish-star-canvas-services-workflow"`）而非 `File` 节点表示。未修改该 skill 工具。

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增共享 fail-closed game/legacy 上下文策略模块与 21 个定向测试；workflow 4 处接入策略并在导入失败时失败关闭；保留全部既有 allowlist 与 Cargo lock 门禁；本机执行真实内联脚本 3/3 通过、39/39 测试通过、8 变异全红 + 对照组绿、Guardian `ok: true`；未构建/发布/部署任何镜像，W3 保持 OPEN，v1.0.6 保持 NO-GO；未修改共享 WBS。 | `docs/briefs/STAR-STABLE-W3-LEGACY-CONTEXT-20261102.md` |
