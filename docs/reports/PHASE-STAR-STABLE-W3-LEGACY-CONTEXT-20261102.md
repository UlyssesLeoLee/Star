# PHASE-STAR-STABLE-W3-LEGACY-CONTEXT-20261102

> **状态：W3 该子项实现完成并通过本机定向验证；整体 v1.0.6 稳定发行仍为 NO-GO。** 本次改动只关闭 Canvas 发布上下文的静态 legacy 过滤缺口，没有构建、发布、部署任何镜像，也没有产生 SBOM、provenance、digest、attestation、消费侧或 rollback 证据。

## §0 目的

按 brief `docs/briefs/STAR-STABLE-W3-LEGACY-CONTEXT-20261102.md` 加固 Star Canvas 镜像发布上下文，使其拒绝非 Star 的 game / legacy 载荷。

已确认的缺陷（静态阅读 `.github/workflows/publish-star-canvas-services.yml` 原始内容）：

- 上下文文件检查只做 `"game" in path.lower()`，**不含 `legacy`**，因此 `crates/canvas-engine/src/legacy_compat.rs` 这类文件可以进入 BuildKit 上下文。
- Cargo 包检查只做精确等值 `identity[0] == "star-game"` 与 `"star-game" in filtered_names`，**不含子串、不含大小写变体、不含 `legacy`**，因此名为 `legacy-auth-shim`、`GAME_UTILS`、`game-legacy-core` 的包（含 registry 包）可以进入解析出的依赖闭包。
- **（P1，独立评审追加）** 上下文组装用 `shutil.copytree(source_crate / "src", destination / "src", dirs_exist_ok=True)` 与 `shutil.copy2(...)` 复制源码树，两者默认**跟随链接**并把目标内容写成普通文件。因此一个名字无害的链接（例如 `crates/canvas-engine/src/vendor`）可以把退役 game/legacy 内容搬进上下文，而生成路径 `crates/canvas-engine/src/vendor/lib.rs` 能通过全部基于名字的检查。本机实测复现：junction 指向外部 `star-game-src` 后，`copytree` 产出 `vendor/lib.rs`，内容含走私标记。

链接绕过还有两个使名称检查与朴素 symlink 检查双双失效的细节，均已实测：

- Windows 目录 junction 是 reparse point，`Path.is_symlink()` 返回 **False**，但 `copytree` 依然跟随；只检查 symlink 会完全漏掉它。
- 若链接位于**祖先目录**（如 `repo/crates` → `repo/legacy-source`，仍在 checkout 内），则 `realpath` 仍落在 checkout 内、`os.walk` 只看到普通子项，容器包含性检查与遍历检查都不会报警。

Lane 基线：worktree HEAD 与 `dev` 同为 `09b4eead99b047759be271ade2f5b06c0612ffb1`；起始工作树只含 brief 与 dispatch JSONL 两个未跟踪文件。

本 lane 与 W7 并行，**未修改共享的 stable-release WBS**（`docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md`），未 revert 或覆盖其他 lane 的工作。

## §1 改动矩阵

| 文件 | 改动 | 说明 |
|---|---|---|
| `.github/workflows/publish-star-canvas-services.yml` | 修改（累计 +36 / −1，541 行） | 在 "Assemble and verify a minimal per-service Cargo context" 步骤内导入共享策略并从 checkout 失败关闭；新增 **5** 处策略调用（镜像名、锁包身份闭包、过滤后锁包名、**复制前的链接自由性**、生成上下文路径）。链接门禁位于全部 `copy2` / `copytree` 之前，输入清单覆盖被复制的 source crate 与被复制的三个根文件（`rust-toolchain.toml`、`LICENSE`、`.dockerignore`），`allowed_root=repo`。唯一删除行是结尾 `print` 文案由 `no game or foreign workspace sources` 改为 `no game/legacy or foreign workspace sources`。**未删除任何既有检查，未放宽 allowlist。** |
| `scripts/release/star_canvas_context_policy.py` | 新增（345 行） | 共享 fail-closed 策略：`FORBIDDEN_TERMS = ("game", "legacy")`、`CanvasContextError`、`forbidden_matches`、`ensure_allowed_image`、`ensure_clean_package_names`、`ensure_clean_package_identities`、`ensure_clean_context_paths`，以及链接规则 `_link_kind` / `_descendant_chain` / `find_link_violations` / `ensure_link_free_sources` 与平台能力位 `_IS_WINDOWS` / `_JUNCTION_API_AVAILABLE`。规则确定性、大小写不敏感，违规项排序后一次性报出。 |
| `scripts/release/tests/test_star_canvas_context_policy.py` | 新增（841 行） | **35** 个定向测试：策略规则、链接/祖先链接规则、**平台能力缺失时的 fail-closed 行为**、workflow 静态接线与顺序、allowlist 未被放宽，以及**真实执行** workflow 内联脚本的对照组 + 11 个变异组（含 2 个「无门禁时确实能走私」的对照测试）。 |
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
- **链接规则（P1 修复）**：`ensure_link_free_sources(paths, allowed_root=...)` 在任何复制之前对每个被复制的源做三重判定 —— (a) 从 `allowed_root` 到该源的**每一个词法路径分量**都不得是 symlink 或 junction（链式检查在 `realpath` 之前进行，否则链接会被提前折叠而看不见）；(b) 该源及其所有后代的 `realpath` 必须留在 `allowed_root` 内；(c) 源自身与任何后代不得是链接。`allowed_root` 自身**不**参与链接判定，避免 runner 工作区位于系统 symlink 之下时误报（包含性另由 `realpath` 双侧归一化保证）。
- junction 检测独立于 `is_symlink()`：Windows 目录 junction 由 `Path.is_junction()` 识别，无法分类的链接（`OSError`）记为 `unresolved-link` 并失败关闭。
- **平台能力缺失时失败关闭（P2 修复）**：`Path.is_junction()` 仅存在于 Python 3.12+。原实现用 `getattr(Path, "is_junction", None)` 探测，API 缺失时直接当作「不是 junction」返回 `None`，即在**恰恰存在 junction 的操作系统上失败开放**。现改为两个模块级能力位 `_IS_WINDOWS = os.name == "nt"` 与 `_JUNCTION_API_AVAILABLE = hasattr(Path, "is_junction")`；当 `_IS_WINDOWS` 为真且 API 不可用时，`_link_kind()` 返回 `junction-check-unavailable`，该路径被判为违规、门禁拒绝组装上下文。非 Windows 平台不存在 junction，因此 API 缺失**不**视为违规，普通路径照常通过——这条边界是必需的，否则实际发布用的 Linux runner 会把所有输入判为违规。
- 遍历中对「是链接」或「realpath 逃逸」的目录**就地剪枝**（`dirnames.remove`），因此指向大树的链接不会把门禁变成无界扫描。
- 违规消息使用相对于 `allowed_root` 的 posix 路径并排序去重，因此拒绝原因确定可复现。

## §2 验证摘要（本机实测）

环境：Windows，Python 3.13.7，PyYAML 6.0.3。仓库根 `D:\orcaWork\Star\star-stable-w3-legacy-context-20261102`。下表为**链接修复后**的最终实测。

| 检查 | 命令 | 结果 |
|---|---|---|
| 新增定向测试 | `python -m unittest scripts.release.tests.test_star_canvas_context_policy -q` | `Ran 35 tests in 2.320s` → `OK`（35/35，0 skip） |
| 三个 release 测试模块合计 | `python -m unittest scripts.release.tests.test_star_canvas_context_policy scripts.release.tests.test_render_star_canvas_manifest scripts.release.tests.test_resolve_star_version -q` | `Ran 53 tests in 46.192s` → `OK`（35 新增 + 18 既有，无回归） |
| 平台能力位实测值 | `python -c "…print(m._IS_WINDOWS, m._JUNCTION_API_AVAILABLE)"`（加载已提交策略模块） | `is_windows: True junction_api: True`（本机 Python 3.13.7，走真实 junction 检测路径） |
| 能力缺失 fail-closed（模拟 Python ≤3.11 on Windows） | 单元：置 `_IS_WINDOWS=True` / `_JUNCTION_API_AVAILABLE=False`；端到端 M11：改写 fixture 中被复制的策略模块两个能力位后跑真实内联脚本 | 单元：`_link_kind()` 返回 `junction-check-unavailable`，`ensure_link_free_sources` 抛 `CanvasContextError`。端到端：exit≠0、报 `not link-free` + `junction-check-unavailable`、`GITHUB_OUTPUT` 为空、`crates/` 下 0 个文件被复制 |
| 能力缺失不得误伤非 Windows | 单元：置 `_IS_WINDOWS=False` / `_JUNCTION_API_AVAILABLE=False` | `_link_kind()` 返回 `None`，`find_link_violations` 返回 `()`，`ensure_link_free_sources` 正常返回 1 条 clean —— 即实际发布用的 Linux runner 不会被该分支拒绝 |
| 模拟确实生效（防「被测物没跑」） | M11 内断言 `assertNotEqual(patched, text)` + `assertIn("_IS_WINDOWS: bool = True", patched)` + `assertIn("_JUNCTION_API_AVAILABLE: bool = False", patched)` | 通过；若将来常量声明形式变更导致替换失效，该断言会立刻失败而不是静默通过 |
| workflow 内联脚本端到端执行（含链接门禁） | 由已提交的 `WorkflowExecutionTests` 抽取 workflow 唯一 here-string、`textwrap.dedent` 后以 `python -` 对 tempfile fixture 执行 | 15 个执行类测试全过 = 1 个干净对照组（exit=0）+ 2 个「无门禁确实能走私」链接对照 + 1 个旧规则不可见性断言 + **11 个变异（M1–M11）全部 exit≠0 且 `GITHUB_OUTPUT` 为空** |
| 真实仓库链接门禁无误报 | 直接调用**已提交**策略模块：`ensure_link_free_sources([crates/canvas-engine, crates/domain-canvas, crates/canvas-realtime, rust-toolchain.toml, LICENSE, .dockerignore], allowed_root=repo)` | `inputs checked: 6` / `clean real paths returned: 6` / 未抛异常 |
| 真实仓库无被跟踪 symlink | `git ls-files -s \| findstr /r "^120000"` | 空（索引中无 mode 120000 条目） |
| 链接绕过可复现性（对照） | fixture 中不经门禁直接 `shutil.copytree(crate/"src", dst)` | 外部载荷确实被复制为 `vendor/lib.rs`，内容含 `SMUGGLED-GAME-LEGACY-PAYLOAD`；内部祖先链接场景复制出 `payload.rs` 同样含标记 |
| junction 语义实测 | 探针：`_winapi.CreateJunction` 后检查 | `is_symlink()=False`、`os.path.islink()=False`、`Path.is_junction()=True`、`realpath` 解析到外部目标、`os.walk(followlinks=False)` 仍进入、`copytree` 仍跟随并复制内容 |
| 本机 symlink 权限 | `os.symlink(...)` | `OSError [WinError 1314] 客户端没有所需的特权` → 本机无法创建真 symlink，因此链接类测试在 Windows 上以 junction 构造（更难的情形），POSIX 上走真 symlink |
| CypherGraph Guardian | `cgg_guard.py --root . --stamp --json scripts/release/star_canvas_context_policy.py scripts/release/tests/test_star_canvas_context_policy.py` | `"ok": true`，`"errors": []` |
| workflow YAML 可解析 | `yaml.safe_load(...)` | jobs = `['build-and-push', 'render-release-manifest', 'resolve_release', 'test']`，`build-and-push` 13 steps |
| 空白错误 | `git diff --check` | clean |
| 范围核对 | `git diff --name-only -- docs/implementation-plans AGENTS.md` | 空（共享 WBS 与 AGENTS.md 未改动） |

> **证据边界（诚实记录）**：修复前曾用一次性 scratch harness 以真实仓库为 `GITHUB_WORKSPACE` 执行内联脚本，得到 3/3 exit=0（`canvas-engine` 119 包 / 9 文件、`domain-canvas` 119 / 9、`canvas-realtime` 136 / 9）。按协调者范围指令，该 scratch harness 已删除且不再重建；**链接修复后未重跑真实仓库的完整内联脚本端到端**，改用「已提交 unittest 套件（tempfile fixture）+ 直接对 6 个真实输入调用已提交策略模块 + 索引无 symlink」三项组合作为等价证据。完整真实仓库端到端重跑登记为 §3 缺口。

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
| M9 | `crates/canvas-engine/src/vendor` 链接到 checkout **外部**的 `star-game-src`（名字无害） | exit≠0，且生成上下文中检索不到走私标记 | **新链接规则**（`image context sources are not link-free`，报出 `src/vendor (junction)`） |
| M10 | `repo/crates` 链接到 checkout **内部**的 `repo/legacy-source`，内含良性名 `payload.rs` | exit≠0，且**拒绝发生在任何载荷复制之前** | **新链接规则的祖先链检查**（报出 `crates (junction)`）；`realpath` 包含性检查与 `os.walk` 都看不见它 |
| M11 | fixture 中被复制的策略模块两个能力位改写为 `_IS_WINDOWS=True` / `_JUNCTION_API_AVAILABLE=False`（模拟 Python ≤3.11 on Windows） | exit≠0，`GITHUB_OUTPUT` 为空，`crates/` 下 0 文件被复制 | **新能力 fail-closed 分支**（报 `junction-check-unavailable`）；修复前该条件下 `getattr` 探测返回 `None`，junction 完全不被检测 → 失败开放 |

M1 / M4 由 `test_mutated_payloads_are_invisible_to_the_previous_rule` 单独断言：这些载荷在旧规则下不可见（`"game" not in "crates/canvas-engine/src/legacy_compat.rs".lower()`、`"legacy-auth-shim" != "star-game"`），因此它们证明的是**新增覆盖**，而不是既有检查的重复。

M9 / M10 各配一个「无门禁对照」测试（`test_benign_named_link_would_smuggle_content_without_the_gate`、`test_internal_parent_link_would_smuggle_content_without_the_gate`）：在同一 fixture 上直接调用 workflow 使用的 `shutil.copytree`，断言走私标记**确实**被复制进来、且生成名 `vendor/lib.rs` / `payload.rs` 不含 `game` 或 `legacy`。这证明被拒绝的是一个真实可复现的绕过，而不是一个本来就无害的树；若将来该对照不再复现，M9/M10 的拒绝测试就失去意义，测试会立刻暴露这一点。

M10 的对照测试还显式断言了两个使朴素检查失效的前提：`realpath(crate)` 仍在 `realpath(workspace)` 之内（包含性检查会通过），且 `crates` 的 `is_symlink()` 为 False（朴素 symlink 检查会通过）。

### 链接类测试的平台行为

本机（Windows，非 Developer Mode）`os.symlink` 返回 `WinError 1314`，无法创建真 symlink；因此 `_create_directory_link()` 在 Windows 上用 `_winapi.CreateJunction` 构造 junction，在 POSIX 上用 `os.symlink(target_is_directory=True)`。**junction 是更难的情形**（`is_symlink()` 为 False），所以本机的 32/32 全过意味着链接规则在最容易漏检的形态下也成立；在 Linux CI runner 上同一批测试会走真 symlink 路径。若某平台两种链接都无法创建，测试以明确原因 `skipTest`，而本机实测 **0 skip**。

### 真实闭包未被误伤的证据

根 `Cargo.lock` 第 7165 行确实存在 `name = "star-game"`，但三个 Canvas 服务的解析闭包（119 / 119 / 136 个锁定包）中均无含 `game` 或 `legacy` 的包名、版本或 source，因此新规则不改变当前发布路径的可构建性。

## §3 已知缺口

| 优先级 | 缺口 | 后续验收 |
|---|---|---|
| P0 | **未构建、未发布、未部署任何镜像。** 本次只执行了上下文组装与校验步骤的 Python 本体；没有运行 BuildKit、`docker build-push-action`、GHCR login、attestation 或 K3s 部署。 | 需要一次真实的 gated workflow run，产生三个不可变 image digest、SBOM、provenance 与 attestation，并做消费侧验证。 |
| P0 | **无 CI 覆盖这些测试。** 现有 workflow 中只有 `pre-tool-use-guard.yml` 跑 `scripts/automation/guardian/tests/`，Canvas workflow 的 `test` job 只跑 `cargo check` / `cargo test`。`scripts/release/tests/` 目前没有任何 CI job 执行，本次 50/50 只是本机证据。 | 需要由拥有相应 workflow 的 lane 把 `scripts/release/tests/` 接入 CI，否则回归只能靠人工本机执行发现。 |
| P0 | **链接修复后未重跑真实仓库的完整内联脚本端到端。** 修复前的 3/3 真实仓库内联执行证据由一次性 scratch harness 产生；按协调者范围指令该 harness 已删除且不重建，改用「已提交 unittest 套件 + 直接对 6 个真实输入调用策略模块 + 索引无 symlink」作为等价证据（见 §2 证据边界）。 | 由拥有 CI 的 lane 在一次真实 hosted run 中覆盖；或由协调者授权后在 lane 内重建一次性 harness 重跑三镜像。 |
| P0 | 稳定发行的其他既有阻塞未解决，且不属于本 lane 文件：`Cargo.toml` 的 `[workspace.package] license = "Apache-2.0"` 与 `LICENSE` 首行 `MIT License` 不一致，会在上下文组装之前触发 workflow 的 MIT 门禁失败；编译进二进制的版本仍是 `0.1.0` 而非 tag 派生版本。 | 由 W1 / W0 相应 lane 处理；本 lane 未修改 `Cargo.toml`、`LICENSE` 或版本解析逻辑。 |
| P1 | **策略重复未统一。** `scripts/automation/star_k3s_context.py`（W7 lane 所有）自带 `FORBIDDEN_TERMS = ("game", "legacy")` 与 `_is_forbidden()`，其逐路径分段匹配弱于本 lane 的整值匹配。两处规则现在并存。 | 跨 lane 去重需由协调者决定；本 lane 未修改 W7 拥有的文件。 |
| P1 | **链接门禁只覆盖 workflow 显式复制的 6 个输入**（source crate + `rust-toolchain.toml` + `LICENSE` + `.dockerignore`）。若将来上下文组装新增复制项而未同步加入该清单，新输入就不受链接检查保护。清单完整性目前由静态接线测试断言（`ensure_link_free_sources(` 与 `allowed_root=repo,` 必须出现），但不会自动跟随新增复制项。 | 后续如新增复制项，须同步扩清单并加对应变异测试。 |
| P1 | **真 symlink 路径未在 POSIX 上实测。** 本机 `WinError 1314` 无法创建 symlink，链接类测试实际以 Windows junction 执行（更难的情形）；`os.symlink` 分支（`is_symlink()` → `"symlink"`）只有代码路径，无本机运行证据。Linux CI runner 上可无特权创建 symlink，届时会覆盖该分支。 | 需要一次 Linux runner 上的 `scripts/release/tests/` 执行，确认 symlink 分支与 junction 分支同样为 0 skip。 |
| P2 | 检查与复制之间不是原子的（TOCTOU）。CI checkout 在门禁运行时是静态的，实际风险低；未加锁或二次校验。 | 如上下文组装改为并发或在可变工作区运行，需要在复制后复验。 |
| P2 | **能力 fail-closed 是有意的「宁可拒绝」行为，会阻断旧解释器上的本地组装。** 在 Windows + Python ≤3.11 上，`_JUNCTION_API_AVAILABLE` 为 False，门禁会把**全部**输入判为 `junction-check-unavailable` 并拒绝组装上下文（M11 实测）。这是刻意的：无法排除 junction 时不得放行。代价是该组合下无法本地产出 Canvas 上下文。 | 若需在旧 Windows 解释器上支持本地组装，正确做法是补一个不依赖 3.12 的 reparse-point 检测（`os.lstat().st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT`，Windows 上自 Python 3.5 可用），而不是放宽门禁；本次未实现，属独立改动。发布路径本身跑在 Linux runner（`os.name != "nt"`），不受影响。 |
| P1 | **新规则对第三方依赖是硬失败。** 若未来引入名字中含 `game` 或 `legacy` 的合法 crates.io 依赖，发布门禁会拒绝它。当前三个闭包无此情况（已实测），但没有例外/豁免机制。 | 这是有意的 fail-closed 行为；若真出现，需 owner 显式决策（改名、换依赖或经评审调整规则），不应静默放行。 |
| P2 | `deploy/k3s-local` 本地构建路径不在本 lane 范围，未改动；其载荷边界由 W7 lane 负责。 | 见 W7 lane 报告。 |
| P2 | 本次验证在 Windows 上以 `python -` 执行内联脚本，未在 GitHub hosted ubuntu runner 上以 `python3 -` + PowerShell here-string 的真实组合执行。PowerShell here-string 的传递语义未做端到端验证。 | 需要一次真实 hosted run。 |

## §4 子代理失败接手清单

- 本 lane 未派发子代理，全部工作由当前 worker 直接完成，无需接手。
- 未读取、移植或覆盖其他 worktree / lane 的内容；`D:\Star`、`main`、远端 ref 均未触碰。
- 未 fetch、未 push、未 rebase、未 merge、未创建 tag / release、未 build / publish / deploy。
- 过程中发现并修复了四个自身缺陷（非上游遗留）：
  1. 策略模块最初把 `None`（local 包缺失的 `source`）判为违规，导致真实仓库三个镜像全部被误拒；
  2. `_describe()` 对非序列值抛 `TypeError`；
  3. **链接规则最初只检查源自身与后代，未检查 `allowed_root` 与源之间的祖先分量**（独立评审 P1 指出）：`repo/crates` → `repo/legacy-source` 这类**内部**祖先链接会让 `realpath` 仍留在 checkout 内、`os.walk` 只看到普通子项，从而绕过包含性与遍历两道检查。已改为对每个词法路径分量做链接判定，并补 M10 端到端变异与「无门禁对照」测试。
  4. **`_link_kind()` 用 `getattr(Path, "is_junction", None)` 探测能力，API 缺失时返回 `None` 即「不是链接」**（独立评审 P2 指出）：`Path.is_junction()` 仅 Python 3.12+，因此在 Windows + ≤3.11 上 junction 完全不被检测 —— 在恰恰存在 junction 的平台上失败开放。已改为 `_IS_WINDOWS` / `_JUNCTION_API_AVAILABLE` 两个能力位，Windows 且 API 缺失时返回 `junction-check-unavailable` 并拒绝；同时用测试锁定「非 Windows 不受影响」这条边界，避免反过来打断 Linux 发布 runner。补 M11 端到端变异。
  四者均由本机执行与定向测试当场暴露并修复，修复后 35/35 与 53/53 通过。
- **范围纠正记录**：本 lane 曾越界写入两处仓库外的持久内容 —— 全局记忆索引 `C:\Users\leo19\.qwen\memories\MEMORY.md`（新增 2 个条目并一度产生 1 行重复）与两个记忆文件（`feedback/no-interactive-pager-for-validation.md`、`reference/cyphergraph-guardian.md`），另在 `%TEMP%` 下重建了一次性 harness。按协调者指令已全部撤销：`MEMORY.md` 已还原为原始 2 条目，两个记忆文件与其空目录已删除，`%TEMP%\w3fix\` 已删除。**未误删任何属于本 lane 或他人的仓库文件**；被删除的都是本 lane 本会话自己创建的仓库外 scratch 内容。`AGENTS.md` 与全局 settings 全程未改动。

## §5 守门规则

1. 只写本 lane 登记文件；brief 由 dispatcher 生成，创建后不编辑。
2. 不修改共享 stable-release WBS（W7 lane 拥有）；W3 状态以「建议值」形式记录在本报告 §6，由协调者在两个代码 lane 评审后同步。
3. 保留全部既有 game 检查与 Cargo lock 闭包 / 身份 / 版本门禁，不删除、不减弱。
4. 不放宽镜像或包 allowlist；策略模块自身不持有 allowlist，只接受调用方传入。
5. 新规则确定性、大小写不敏感、失败关闭；违规项排序后一次性报出，便于复现。
6. 载荷无法证明时（空值、非字符串）判为违规，不默认放行。
7. 门禁必须配变异测试，且对照组保持绿（守门 v30）：11 个变异全红 + 1 个对照组绿。
8. 变异必须证明是**新增覆盖**：M1 / M4 由专门测试断言其在旧规则下不可见；M9 / M10 各配「无门禁时确实能走私」的对照测试；M11 断言能力位改写**确实生效**，避免「被测物没跑」而静默通过。
9. 断言必须能区分「被测物没跑」：执行类测试断言 stdout 的 `Verified …` 计数行与 `GITHUB_OUTPUT` 的 `directory=` 写出；拒绝路径断言 `GITHUB_OUTPUT` 为空。
10. 真实仓库不得只靠静态阅读声明通过：修复前以真实仓库执行内联脚本 3/3；修复后按范围指令改用「已提交 unittest 套件 + 直接对 6 个真实输入调用策略模块（6/6 clean）+ 索引无 mode 120000」组合证据，并把未重跑的完整端到端登记为 P0 缺口，不冒充已覆盖。
11. 不把「上下文过滤通过」描述成「镜像已构建 / 已发布 / 内容已验证」。
12. 不声称 SBOM、provenance、digest、attestation、消费侧或 rollback 证据——这些操作没有运行。
13. v1.0.6 稳定发行保持 NO-GO；W3 保持未闭合，直到真实 gated run 产生制品证据。
14. 不修改其他 lane 拥有的文件（`scripts/automation/star_k3s_context.py`、`deploy/k3s-local/*`、`Cargo.toml`、`LICENSE`）。
15. `@cypher` header 必须由工具盖章，不得手写或推算 `source_sha256`。仓库内无 canonical validator，改用已安装 skill 路径的 validator（见 §2 与 §6），并在报告说明。
16. 不 fetch、不 push、不 rebase、不 merge、不打 tag、不发布、不部署；不动 `main`、`D:\Star`、其他 worktree 与远端 ref。
17. 静态校验与本机执行不代替 hosted CI、镜像构建与发行验收。
18. **链接门禁必须在任何复制之前执行**，且必须检查从 `allowed_root` 到源的每一个词法分量；只检查源自身与后代不足以拦住内部祖先链接。
19. 复制类 API 默认跟随链接，因此不得以「生成路径名无害」作为载荷干净的证据；必须检查链接本身。
20. 范围严格限于本 lane 登记清单；不改 `AGENTS.md`、不改全局记忆 / settings、不在 lane 文件之外新建持久产物或 harness。越界写入一经发现立即撤销并在 §4 记录，不静默保留。
21. 重跑验证不得经由交互式分页器（`more` 等）：本机 `cmd` 下分页器会等待按键而使已成功的运行被报成超时。改用 `-q` 或重定向到日志后按退出码判定。
22. **可选 API 探测不得退化为「检测通过」。** 用 `getattr(obj, "method", None)` 探测能力时，缺失分支必须失败关闭而不是当作否定结果；且 fail-closed 范围要精确限定在真正存在该风险的平台（此处为 `os.name == "nt"`），否则会反过来打断正常平台（Linux runner）的发布。两条边界都必须有测试。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 共享策略单点持有禁止词与链接规则，workflow **5** 处调用全部位于镜像构建之前，链接门禁位于全部 `copy2` / `copytree` 之前；allowlist 与既有 lock 门禁逐项保留，diff 为纯增量（累计 +36 / −1，唯一删除行是文案）。 |
| SRE Lead | SRE Lead（Mavis 接手审核）— 修复前以真实仓库执行内联脚本 3/3 通过（119 / 119 / 136 锁定包）；修复后对 6 个真实输入直接调用策略模块 6/6 clean、索引无 mode 120000；未执行任何 hosted run、镜像构建或部署，运行门仍阻塞。 |
| 平台 | 平台（Mavis 接手审核）— 策略导入失败即失败关闭（M8 已实测）；链接与祖先链接均失败关闭并剪枝，避免无界扫描（M9 / M10 已实测）；`Path.is_junction` 缺失时于 Windows 失败关闭、于非 Windows 不误伤（M11 + 单元双向实测）；`scripts/release/tests/` 目前无 CI job 执行，已登记为 P0 缺口。 |
| 评审主持 | 评审主持（Mavis 接手审核）— 11 个变异全红且对照组绿；M1 / M4 断言在旧规则下不可见，M9 / M10 各配「无门禁确实能走私」的对照，M11 断言能力位改写确实生效；独立评审提出的祖先链接绕过（P1）与能力探测失败开放（P2）均已修复并回归，评审确认当前实现无其他链接 / junction / 祖先绕过。 |
| PM | PM（Mavis 接手审核）— v1.0.6 仍 NO-GO，不创建 release，不声称任何制品证据；越界写入的全局记忆与 scratch harness 已全部撤销并在 §4 记录。 |

### 建议的 W3 状态（供协调者同步 WBS，本 lane 未写 WBS）

- W3 / legacy 上下文过滤子项：**静态实现 + 本机定向验证完成**（代码已落地，53/53 测试通过含 35 个新增，真实仓库 6/6 输入链接自由、修复前三镜像上下文门 3/3 通过，11 变异全红 + 对照组绿）。
- W3 整体：**仍 OPEN**。缺 hosted workflow run、真实 image digest、SBOM / provenance / attestation、消费侧验证与 rollback 证据；且 CI 未覆盖 `scripts/release/tests/`。
- v1.0.6 稳定发行：**NO-GO**。
- 未触碰的既有阻塞：Cargo license `Apache-2.0` vs `LICENSE` 的 `MIT License` 不一致；二进制版本仍为 `0.1.0`。

### 工具边界说明

仓库内不存在 canonical `.agents` validator 或 `cgg_guard.py`。按既有 brief 先例（`docs/briefs/wld5-stale-session-401-regression-20261023.md` §6：canonical validator 不存在时使用已安装 skill 路径的 validator 并在报告说明），本次使用 `C:\Users\leo19\.claude\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py` 盖章并校验，结果 `"ok": true`。**P2 修复后的最终盖章值**：`scripts/release/star_canvas_context_policy.py` = `f817fa711856a6bdf02e9c7f59f43436a0946e093a8efa7096d8b592b80a6da7`；`scripts/release/tests/test_star_canvas_context_policy.py` = `4435f818ae636e316207576da5317c04c8567382aa780158d6acac7895f5a2a3`。（v0.1 的 `3efc645c…` / `c5565a6f…` 与 v0.2 的 `44a4f339…` / `a9cfb55c…` 均已随代码改动失效。）该工具只做结构校验，语义边由本 lane 自行核对。

另记录一处工具限制：`cgg_guard.py` 的 `FILE_PATH_RE` 校验对路径做 `lstrip("./")`，会把 `.github/workflows/...` 削成 `github/workflows/...` 并报「File path target does not exist」。因此测试文件 header 中 workflow 以 `Config` 节点（`id:"publish-star-canvas-services-workflow"`）而非 `File` 节点表示。未修改该 skill 工具。

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增共享 fail-closed game/legacy 上下文策略模块与 21 个定向测试；workflow 4 处接入策略并在导入失败时失败关闭；保留全部既有 allowlist 与 Cargo lock 门禁；本机执行真实内联脚本 3/3 通过、39/39 测试通过、8 变异全红 + 对照组绿、Guardian `ok: true`；未构建/发布/部署任何镜像，W3 保持 OPEN，v1.0.6 保持 NO-GO；未修改共享 WBS。 | `docs/briefs/STAR-STABLE-W3-LEGACY-CONTEXT-20261102.md` |
| v0.2 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | **修复独立评审提出的 P1 链接绕过**：`copytree` / `copy2` 默认跟随链接，良性名链接可把退役 game/legacy 内容搬进上下文而通过全部名称检查。新增 `ensure_link_free_sources` / `find_link_violations` / `_link_kind` / `_descendant_chain`，在**任何复制之前**判定 symlink 与 Windows junction，并对 `allowed_root`→源的**每个词法祖先分量**做链接检查（拦住 `repo/crates` → `repo/legacy-source` 这类 realpath 仍在 checkout 内的内部祖先链接）；逃逸目录就地剪枝避免无界扫描。workflow 第 5 处接入，输入清单为被复制的 source crate + 三个根文件。测试 21→32、release 套件 39→50 全过（0 skip），新增 M9 / M10 两个端到端变异及各自「无门禁确实能走私」对照，真实仓库 6/6 输入链接自由、索引无 mode 120000，Guardian 重新盖章 `ok: true`，`git diff --check` clean。allowlist 与既有门禁未改动；未 push / rebase / merge；共享 WBS、`AGENTS.md`、W7 文件未触碰。同时记录范围纠正：撤销本 lane 越界写入的全局记忆条目与文件、删除 `%TEMP%` scratch harness，验证改走已提交 unittest 套件。 | 协调者转达独立评审 P1（symlink/junction 遍历）+ 范围纠正指令 |
| v0.3 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | **修复独立评审提出的条件性 P2：能力探测失败开放。** `_link_kind()` 原以 `getattr(Path, "is_junction", None)` 探测，API 缺失（Windows + Python ≤3.11，`Path.is_junction` 自 3.12 起）时返回 `None` 即「不是链接」，导致在恰恰存在 junction 的平台上完全不检测 junction。改为模块级能力位 `_IS_WINDOWS = os.name == "nt"` 与 `_JUNCTION_API_AVAILABLE = hasattr(Path, "is_junction")`：Windows 且 API 不可用时返回 `junction-check-unavailable` 并判为违规、拒绝组装上下文；**非 Windows 不视为违规**，以免反过来打断实际发布用的 Linux runner（该边界由专门测试锁定）。新增 3 个测试（Windows 无 API → fail closed；非 Windows 无 API → 普通路径照常通过；端到端 M11 改写 fixture 内策略副本两个能力位后跑真实内联脚本，断言 exit≠0 + `junction-check-unavailable` + `crates/` 下 0 文件被复制，并断言改写确实生效）。测试 32→35、release 套件 50→53 全过 0 skip，变异 10→11 全红且对照组绿；真实仓库 6/6 输入仍 clean（实测 `_IS_WINDOWS=True`、`_JUNCTION_API_AVAILABLE=True`）；Guardian 重新盖章 `ok: true`（`f817fa71…` / `4435f818…`）；`git diff --check` clean。**workflow 文件本次未改动**，改动仅在策略模块与其测试。新增 P2 缺口：Windows + ≤3.11 下门禁会拒绝全部输入（刻意的宁可拒绝），正确解法是补 `st_file_attributes & FILE_ATTRIBUTE_REPARSE_POINT` 检测而非放宽门禁，本次未实现。allowlist 与既有门禁未改动；未 push / rebase / merge；共享 WBS、`AGENTS.md`、W7 文件未触碰；manifest 仍为 7 个路径。 | 协调者转达独立评审条件性 P2（`Path.is_junction` 缺失时失败开放）|
