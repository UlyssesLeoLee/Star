# W3 Star Canvas legacy 上下文过滤审阅 — 2026-11-02

## 决定

**静态过滤缺口与独立评审提出的 P1 链接绕过均已关闭并通过本机执行验证；W3 整体仍 OPEN，v1.0.6 稳定发行仍 NO-GO。**

被审阅的缺陷是真实的，且已实测确认其影响范围：原 `.github/workflows/publish-star-canvas-services.yml` 的上下文文件检查只做 `"game" in path.lower()`，Cargo 包检查只做精确等值 `star-game`。因此 `legacy_compat.rs` 这类源文件、以及名为 `legacy-auth-shim` / `GAME_UTILS` / `game-legacy-core` 的锁包（含 registry 包）都能进入 BuildKit 上下文而不被拒绝。

修复引入单一共享策略 `scripts/release/star_canvas_context_policy.py`（`FORBIDDEN_TERMS = ("game", "legacy")`，整值 `casefold()` 子串匹配），并在 workflow 的上下文组装步骤中 **5** 处接入：镜像名、锁包身份闭包、过滤后锁包名、**复制前的链接自由性**、生成上下文路径。策略导入失败即失败关闭。

**P1 链接绕过（独立评审追加）已确认并修复。** `shutil.copytree(source_crate / "src", …)` 与 `shutil.copy2(…)` 默认跟随链接并把目标内容写成普通文件，因此一个名字无害的链接可以把退役载荷搬进上下文，而生成路径能通过全部名称检查——本机以 junction 实测复现（`copytree` 产出 `vendor/lib.rs`，内容含走私标记）。修复采用 `ensure_link_free_sources(paths, allowed_root=repo)`，在**任何复制之前**执行，并且：

- 同时识别 symlink 与 Windows junction。junction 是必须单独处理的形态：实测 `is_symlink()` 与 `os.path.islink()` 均为 `False`，`Path.is_junction()` 为 `True`，而 `copytree` 依然跟随——只做 symlink 检查会完全漏掉。
- 检查从 `allowed_root` 到源的**每一个词法祖先分量**，而不只是源自身与后代。这是评审第二轮指出的关键缺口：`repo/crates` → `repo/legacy-source`（仍在 checkout 内）会让 `realpath` 保持包含、`os.walk` 只看到普通子项，包含性与遍历两道检查同时失效。链式检查在 `realpath` 之前进行，否则链接会被提前折叠。
- `allowed_root` 自身不参与链接判定，避免 runner 工作区位于系统 symlink 之下时误报；包含性另由 `realpath` 双侧归一化保证。
- 对「是链接」或「realpath 逃逸」的目录就地剪枝，指向大树的链接不会把门禁变成无界扫描。
- 输入清单覆盖全部被复制项：source crate（含 `src/`、`Cargo.toml`、`Dockerfile`）与三个根文件 `rust-toolchain.toml`、`LICENSE`、`.dockerignore`。

diff 为纯增量（累计 +36 / −1）。唯一删除行是结尾 `print` 文案；全部既有 allowlist、path 依赖、workspace 依赖、lock 闭包唯一可解析、`star-game` 精确检查、过滤锁一致性、root 版本绑定与 `.git/` / `target/` 前缀检查逐项保留。allowlist 未被放宽——策略模块自身不持有任何镜像名单，只校验调用方传入的集合。

## 来源与边界

- 审阅对象为 lane worktree `D:\orcaWork\Star\star-stable-w3-legacy-context-20261102`，基线 HEAD 与 `dev` 同为 `09b4eead99b047759be271ade2f5b06c0612ffb1`。未 fetch、未 push、未 rebase、未 merge、未打 tag、未发布、未部署；`main`、`D:\Star`、远端 ref 与其他 worktree 未触碰。
- 与 W7 并行。本 lane **未修改共享 WBS** `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md`，未修改 `AGENTS.md`，也未修改 W7 拥有的 `scripts/automation/star_k3s_context.py` 与 `deploy/k3s-local/*`（`git diff --name-only -- docs/implementation-plans AGENTS.md` 为空）。W3 状态以建议值记录在阶段报告 §6，由协调者在两个代码 lane 评审后同步。
- 改动范围严格限于 MAGOS manifest 的 7 个路径；未在 lane 文件之外新建持久产物。曾越界写入全局记忆索引与两个记忆文件、并在 `%TEMP%` 重建一次性 harness，已按协调者指令全部撤销（索引还原为原始 2 条目、记忆文件与空目录删除、`%TEMP%\w3fix\` 删除）；未误删任何属于本 lane 或他人的仓库文件。
- 验证方式是**执行 workflow 内联脚本本体**（由已提交的 `WorkflowExecutionTests` 抽取唯一 PowerShell here-string，`textwrap.dedent` 后以 `python -` 对 tempfile fixture 运行），而非仅静态阅读。这仍不等于 hosted run。
- 修复前曾以真实仓库为 `GITHUB_WORKSPACE` 跑过完整内联脚本（3/3 exit=0）；该 scratch harness 已按范围指令删除且不重建，修复后的真实仓库证据改为「直接对 6 个真实输入调用已提交策略模块（6/6 clean）+ 索引无 mode 120000 + 50/50 测试」。完整真实仓库端到端重跑登记为 P0 缺口，未冒充已覆盖。
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
| 变异测试 + 对照组（守门 v30） | 10 个变异全红，对照组绿（`2 locked packages; 8 files`，写出 `directory=`） |
| 链接绕过是否真实可复现（对照） | 是；不经门禁直接 `copytree` 时，外部 junction 产出 `vendor/lib.rs`、内部祖先链接产出 `payload.rs`，两者内容均含走私标记，且生成名不含 `game` / `legacy` |
| 良性名链接是否被拒（M9） | 是；exit≠0，报 `image context sources are not link-free` 与 `src/vendor (junction)`，且生成上下文中检索不到走私标记 |
| 内部祖先链接是否被拒（M10） | 是；`repo/crates` → `repo/legacy-source` 场景 exit≠0，报 `crates (junction)`；对照测试同时断言 `realpath` 仍在 checkout 内且 `is_symlink()` 为 False，即包含性与朴素 symlink 检查都会放过它 |
| 拒绝是否发生在载荷复制之前 | 是；M9 / M10 均断言生成上下文内检索不到走私标记，`GITHUB_OUTPUT` 为空 |
| junction 是否被单独识别 | 是；实测 `is_symlink()=False` / `islink()=False` / `is_junction()=True`，`_link_kind()` 返回 `junction` |
| 真实仓库是否被新门禁误伤 | 否；对 6 个真实输入调用 `ensure_link_free_sources` 返回 6/6 clean 且未抛异常；`git ls-files -s` 无 mode 120000 |
| 真实仓库三镜像上下文门 | 修复前 3/3 exit=0（119 / 119 / 136 锁定包，各 9 files）；修复后未重跑完整内联脚本，见 P0 缺口 |
| 真实闭包是否被误伤 | 否；根 `Cargo.lock:7165` 存在 `star-game`，但三个闭包中无含 game / legacy 的 name / version / source |
| 本机测试 | 定向 `Ran 32 tests in 1.869s` → `OK`（0 skip）；release 三模块合计 `Ran 50 tests in 40.034s` → `OK`（32 新增 + 18 既有） |
| `@cypher` header 盖章 | Guardian `"ok": true`，`"errors": []`；最终盖章 `44a4f339…` / `a9cfb55c…`；使用已安装 skill 路径 validator（仓库内无 canonical），已在报告 §6 说明 |
| `git diff --check` | clean |
| workflow YAML 可解析 | 4 jobs，`build-and-push` 13 steps |
| 范围核对 | `git diff --name-only -- docs/implementation-plans AGENTS.md` 为空；未 push / rebase / merge |
| hosted workflow / 镜像构建 / 制品 | **未执行** |
| CI 是否覆盖这些测试 | **否**；无 CI job 执行 `scripts/release/tests/`，50/50 仅为本机证据 |
| W3 / v1.0.6 状态 | OPEN / NO-GO |

## 遗留判断

- 新规则对第三方依赖是硬失败：未来若引入名字含 `game` 或 `legacy` 的合法 crates.io 依赖，发布门禁会拒绝。这是有意的 fail-closed 行为，不应加静默豁免；真出现时需 owner 显式决策。
- 链接门禁只覆盖 workflow 显式复制的 6 个输入。若将来上下文组装新增复制项而未同步扩清单，新输入不受链接检查保护；目前清单完整性仅由静态接线断言保证，不会自动跟随新增项。
- 真 symlink 分支未在 POSIX 实测：本机 `WinError 1314` 无特权创建 symlink，链接类测试实际以 Windows junction 执行（`is_symlink()` 为 False 的更难形态）。`os.symlink` → `"symlink"` 分支只有代码路径，需一次 Linux runner 执行确认 0 skip。
- 检查与复制之间非原子（TOCTOU）。CI checkout 在门禁运行时静态，风险低，未加二次复验。
- 仓库内现有两处禁止词规则（本 lane 的整值匹配、W7 `star_k3s_context.py` 的逐段匹配），语义强度不同；且 W7 侧无链接检查。跨 lane 去重与链接规则推广应由协调者决定，本 lane 未越界统一。
- 上下文过滤通过**不等于**最终镜像内容合格。W3 闭合仍需真实 gated run 产生的三个不可变 digest、逐文件 context 清单与哈希、最终镜像 rootfs 检查、SBOM / provenance / attestation，以及 digest 固定的 Envoy 运行时依赖证据。
- 未解决且不属于本 lane 的既有阻塞：`Cargo.toml` 的 `license = "Apache-2.0"` 与 `LICENSE` 首行 `MIT License` 不一致（会在上下文组装前触发 MIT 门禁失败）；编译进二进制的版本仍为 `0.1.0` 而非 tag 派生版本。

## 修订历史

| 版本 | 日期 | 内容 |
|---|---|---|
| v0.1 | 2026-11-02 | 确认 legacy 过滤缺口真实存在并已关闭；记录纯增量 diff、8 变异全红 + 对照组绿、真实仓库三镜像 3/3 通过、39/39 本机测试与 Guardian 盖章；同时保留 hosted run、CI 覆盖、镜像/SBOM/provenance 与 license/version 阻塞缺口，W3 OPEN、v1.0.6 NO-GO。 |
| v0.2 | 2026-11-02 | 确认并关闭独立评审提出的 P1 链接绕过：`copytree` / `copy2` 跟随链接，良性名链接可走私退役载荷。新增复制前的 `ensure_link_free_sources` 门禁，识别 symlink 与 Windows junction，并检查 `allowed_root`→源的每个词法祖先分量（拦住 realpath 仍在 checkout 内的内部祖先链接），逃逸目录剪枝。变异 8→10（M9 外部链接 / M10 内部祖先链接，各配「无门禁确实能走私」对照），定向测试 21→32、release 套件 39→50 全过 0 skip，真实仓库 6/6 输入链接自由且索引无 mode 120000，Guardian 重新盖章 `ok: true`。记录范围纠正（撤销越界全局记忆写入、删除 `%TEMP%` scratch harness）与新增缺口：完整真实仓库端到端未重跑、真 symlink 分支未在 POSIX 实测、链接清单不自动跟随新增复制项、TOCTOU。allowlist 与既有门禁未改动；未 push / rebase / merge；共享 WBS、`AGENTS.md`、W7 文件未触碰。W3 仍 OPEN、v1.0.6 仍 NO-GO。 |
