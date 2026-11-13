# Star CI/CD 说明

> 版本 v0.1 · 2026-11-13 · 配套实现: `.github/workflows/`、`scripts/ci/`、`.github/rulesets/`
>
> 本文描述 **现行** 的 GitHub CI/CD。凡标注"已知问题"的条目是实测确认但尚未修复的状态，
> 不代表已验收。

## 1. 为什么重构

2026-11-13 调查时，`dev` 上的 `CI` 与 `License & Advisory Gate` 已连续 37 次全红，
且 `main`/`dev` 没有任何分支保护。红灯是 5 个互相独立的根因叠加，其中门禁本身
(许可/漏洞) 从未真正执行过：

| 现象 | 根因 | 处置 |
|---|---|---|
| Rust job 编译失败 | `--all-features` 把 tauri 桌面 crate 拉进全仓检查，runner 缺 `glib-2.0` | 全仓 job `--exclude star-desktop`，桌面 crate 单独装系统库检查 |
| Cross-platform (ubuntu) 失败 | `cli_spawn.rs` 在 Linux 分支对 `Box<dyn Future>` 直接 `.await`（非 `Unpin`，E0277）；Windows 本机开发从不编译该分支 | 修代码；该 job 保留三平台矩阵兜底 |
| BFF 失败 | `cargo fmt --check` 全仓积累 100 个文件格式债 | 独立提交一次性 `cargo fmt --all` |
| License / Advisory 失败 | cargo-deny 钉死的 SHA256 与上游不符；cargo-audit 解压路径错误 | 统一 `scripts/ci/install_pinned_binary.sh`，SHA256 取自上游 release API |
| Desktop 打包失败 | tauri 的 `before*Command` 工作目录是前端目录，`--prefix frontend` 变成 `frontend/frontend` | 配置改为 `npm run build` / `npm run dev` |
| Markdown lint 失败 | 1 处波浪线代码围栏 + 1 个 vendored 工具的模板 | 修围栏；排除 `tools/aci-emitter/**` |

## 2. Workflow 一览

| Workflow | 触发 | 作用 | 是否必需检查 |
|---|---|---|---|
| `ci.yml` | push/PR → `dev`、`main`；手动 | 主 CI，见 §3 | 是（聚合检查 `CI gate (required)`） |
| `license-gate.yml` | 同上 + 每日 03:17 UTC | cargo-deny 许可门禁、cargo-audit 漏洞门禁、cargo-machete（advisory） | 仅 `License (cargo-deny)` |
| `codeql.yml` | push/PR → `main`；每周 | CodeQL 安全扫描 | 否 |
| `coverage.yml` | push/PR；手动 | tarpaulin 覆盖率基线（advisory） | 否 |
| `release.yml` | 手动 | 统一发布编排，见 §5 | 不适用 |
| `star-stable-desktop.yml` | PR/push → `dev`（路径过滤）；手动 | 桌面三平台打包、SBOM、attestation | 否 |
| `star-stable-web.yml` | 手动 | Web 构建证据与 attestation | 否 |
| `publish-star-canvas-services.yml` | push `main`/`dev`、tag `v*`；手动 | Canvas 服务镜像构建与发布 | 否 |
| `spawn-e2e.yml` | PR → `main`；手动 | 跨平台进程 spawn/kill e2e | 否 |
| `star-api-rest` 的 2 个单测 | Linux 下失败（已在 CI 中 `--skip`） | `workspace_files.rs` 的 `resolve_blocks_parent_escape`、`resolve_rejects_path_outside_roots` 假定 Windows 盘符路径；Linux 上应先决定「盘符路径在 Linux 上是否也必须被拒绝」，再修实现或用例 | 该文件在仓库主工作区另有未提交改动，故未触碰 |
| `wasm-build.yml` | 改动 `crates/*-wasm/**` 时 | Rust→WASM 构建 | 否（**已知问题**，见 §7） |
| `pre-tool-use-guard.yml` | 改动 guardian 脚本时 | guardian 的 pytest/lint | 否（**已知问题**，见 §7） |
| `mock-switch-validate.yml` | 改动 mock 工具时 | 跨项目 mock 切换校验 | 否（**已知问题**，见 §7） |
| `p1e-integration.yml`、`star-flash-mock-regression.yml` | 路径过滤 | 专项集成/回归 | 否 |

所有 workflow：`permissions` 默认 `contents: read`；第三方 action 全部固定到 commit SHA
（行尾注释为版本号，Dependabot 会据此更新）；runner 使用带版本号的镜像
（`ubuntu-24.04` / `windows-2025` / `macos-15`），因为 `ubuntu-latest` 已宣布迁移到
Ubuntu 26，系统库包名会漂移。

## 3. 主 CI (`ci.yml`)

```text
changes ──┬─ rust-fmt ───────────────┐
          ├─ rust-ci ────────────────┤
          ├─ rust-test-workspace ────┤
          ├─ cargo-doc ──────────────┤
          ├─ bff-ci ─────────────────┤
          ├─ cross-platform (3 OS) ──┤
          ├─ desktop-rust ───────────┼─► ci-gate  ← 分支保护只要求这一个
          ├─ frontend-ci ────────────┤
          ├─ workflow-lint ──────────┤
          └─ markdownlint ───────────┘
          (advisory，不进 gate: cargo-bench-no-run)
```

- **路径过滤**：`scripts/ci/detect_changes.py` 根据改动路径决定各区域是否运行，
  纯文档提交不再跑 Rust 与前端全套。无法确定时（新分支、force push、手动触发）
  一律 fail-open，全部运行。规则与测试在 `scripts/ci/tests/`。
- **聚合门禁**：`scripts/ci/gate.py` 把 `skipped` 视为通过，`failure`/`cancelled`/缺失视为失败，
  空输入 fail-closed。这样路径过滤造成的跳过不会卡住无关 PR。
- **enforced 与 advisory**：
  - enforced：fmt、`cargo check --all-features`、clippy（deny 级 lint 为 error）、
    **全 workspace 单测**、`cargo doc`、BFF、三平台 smoke、桌面 crate、前端 typecheck/test/build、
    workflow lint、markdownlint。
  - advisory（job 内 `continue-on-error`，结果可见但不拦合并）：`cargo bench --no-run`。
- **并发**：同一 PR/分支的旧运行自动取消；`main` 上的运行不取消。
- **全 workspace 单测**：2026-11-13 在 ubuntu-24.04 实测 3033 通过 / 2 失败 / 8 忽略，约 6 分钟。
  9/5 记录的「`star-context` 并行 panic」（`actor.rs:118`，AGENTS.md §4.1 v25b）在 CI 上未复现，
  因此由 advisory 升为 enforced。唯一的 2 个失败是 `star-api-rest` 的
  `workspace_files::tests::resolve_blocks_parent_escape` / `resolve_rejects_path_outside_roots`：
  用例写死 Windows 盘符路径（`C:/Windows/system32`、`D:/elsewhere/x.md`），在 Linux 上它们是相对路径，
  解析后仍在根目录内，`is_err()` 断言不成立。这两个用例被精确 `--skip`（见 §7），其余全部强制。
  若该单测因并行 panic 偶发失败，应退回 advisory 并记录，而不是重跑到绿。

### 本地复现

```bash
python -m unittest discover -s scripts/ci/tests -v   # CI 脚本自测
cargo fmt --all -- --check
cargo check --workspace --exclude star-desktop --all-targets --all-features -j 4
cargo clippy --workspace --exclude star-desktop --all-targets -j 4
cargo test -p star-context --lib -j 4
(cd bff && cargo fmt --all -- --check && cargo check --all-targets --all-features -j 4)
(cd frontend && npm ci && npx tsc --noEmit && npx vitest run && npx next build)
```

> Linux 专属代码分支（`#[cfg(target_os = "linux")]`）在 Windows 本机永远不会被编译，
> 只能靠 `cross-platform` 的 ubuntu 矩阵项兜底。改动这类代码时以 CI 结果为准。

## 4. 供应链门禁 (`license-gate.yml`)

- 依据 AGENTS.md §0「商业开源依赖硬约束」。三个 job 互相独立，保证每个门禁都真实执行。
- 工具二进制 **版本钉死 + SHA256 校验**。SHA256 必须取自上游 release API 的 asset `digest`，
  不可取自下载物本身：

  ```bash
  gh api repos/EmbarkStudios/cargo-deny/releases/tags/0.20.2 --jq '.assets[]|{name,digest}'
  ```

- 升级工具版本 = 同时改 `*_VERSION` 与 `*_SHA256`，并用上面的命令核对。
- 不使用 workflow 级 `paths` 过滤：这些 job 是必需检查，过滤后"检查缺失"会让无关 PR 永远无法合并。
- **必需检查只含确定性项**：`License (cargo-deny)` 只跑 `licenses bans sources`（锁文件的确定性函数）。
  `advisories` 随 RustSec 数据库每日变化，一条新公告会在没有任何代码改动时把检查变红并卡住所有合并，
  因此放在非必需的 `Advisories (cargo-audit)` job 里（cargo-audit + `cargo-deny check advisories`，
  后者使用 `deny.toml` 中带复审期限的 ignore 清单），并每日定时运行。
- `cargo-machete` 为 advisory（静态分析对 feature 间接使用的依赖会误报），发现项写进 job summary。

## 5. 发布 (`release.yml`)

三个既有的发布 workflow（桌面 / Web / Canvas 镜像）已经具备 SHA 固定、SBOM、attestation 与独立
consumer 验证，**本次不改动其内部**。`release.yml` 只解决"三处分别手点、无法保证对应同一提交"：

1. 手动运行，输入 `tag`（形如 `v1.2.3`，必须已存在）、`components`、`release_confirmation`、`dry_run`。
2. `preflight`：tag 文法 → 与构建 workflow 相同的 `resolve_star_version.py` →
   提交必须在 `main` 上 → 该提交上 `CI gate (required)` 与 `License (cargo-deny)` 均为 success。
3. `dry_run=true`（默认）到此为止，不触发任何构建，也不需要 APPROVED。
4. 真实发布需 `dry_run=false` 且 `release_confirmation=APPROVED`，运行在 `star-stable-release`
   environment 下：依次以 `workflow_dispatch` 触发所选构建 workflow（各自仍要求 APPROVED），
   等待全部成功后创建 **draft** GitHub Release。公开发布仍是一次人工点击。

> **待你决定**：仓库当前没有配置 `star-stable-release` environment（GitHub 会在首次引用时自动创建
> 一个无保护规则的空环境）。要让"需要人工批准"真正生效，请在 Settings → Environments 中为它添加
> Required reviewers。

## 6. 分支保护 (`.github/rulesets/`)

| 文件 | 目标 | 规则 |
|---|---|---|
| `main.json` | `main` | 禁删除、禁 force-push、必须经 PR、必须通过 `CI gate (required)` 与 `License (cargo-deny)` |
| `dev.json` | `dev` | 仅禁删除、禁 force-push |

`dev` 故意不要求状态检查：日常流程是在本地合并 feature 分支后直接 push `dev`，
规则集的 required status checks 会拒绝这种直接 push。仓库管理员角色可绕过（避免 CI 自身故障时被锁死）。

```bash
bash scripts/ci/apply_rulesets.sh --dry-run   # 预览
bash scripts/ci/apply_rulesets.sh             # 创建/更新（按 name 幂等，从不删除）
```

要临时关闭某条规则：把对应 JSON 的 `enforcement` 改为 `disabled`（或 `evaluate`）后重新运行脚本。
仓库里早先存在两个空壳 ruleset（`StarMain`、`StarMainD`，均为 disabled，仅含删除/force-push 规则），
本方案未触碰它们。

## 7. 已知问题（实测确认、未修复）

| 项 | 现象 | 根因 | 备注 |
|---|---|---|---|
| `wasm-build.yml` | 自 2026-10-11 起失败 | `graph-core` 依赖 workspace 的 `tokio`（`features=["full"]`）拖入 `mio`，`mio` 无法编译到 `wasm32-unknown-unknown`；之后还有 `uuid` 在 wasm32 缺随机源、`relationship-engine-wasm` 直接依赖 `tokio` | 产品代码的依赖策略问题，非 CI。详见该 workflow 文件头注释 |
| `pre-tool-use-guard.yml` | 失败 | guardian 测试目录的 flake8 未使用导入；`test_comment_signing.py` 收集期出错 | 仅在改动 guardian 脚本时触发 |
| `mock-switch-validate.yml` | 失败（`fail=7`） | 校验的是跨项目 mock 集群，依赖 runner 上不存在的其他项目 | 本质是本机工具，不适合在 runner 上跑 |
| `Advisories (cargo-audit)` | 修复取工具问题后首次真正执行（2026-11-13）：cargo-audit 报 **3 个真实漏洞** | `crypto-bigint` 0.5.5 时序问题 RUSTSEC-2026-0387；`libsqlite3-sys` 0.30.1 捆绑的 SQLite RUSTSEC-2026-0459（中危 6.8，经 `rusqlite` 0.32.1 引入）；`rsa` Marvin 时序侧信道 RUSTSEC-2023-0071（`deny.toml` 已有评估后的 ignore，cargo-audit 不读该清单）。另有 4 条 unmaintained/unsound 警告（fxhash、instant、proc-macro-error、glib） | 需要你决定「升级依赖」还是「在 `deny.toml` 登记评估后的 ignore」；该 job 非必需，不会拦合并 |
| macOS 桌面打包 | 排队后 "job was not acquired" | GitHub macOS arm64 runner 容量不足 | 基础设施问题 |
| `star-stable-desktop` | 修复 tauri 工作目录后，Linux / Windows / macOS 三个平台都已能真实打出安装包（.deb / AppImage / MSI / universal DMG），但都在 **Validate … SBOMs** 步骤失败（`Error: DESCRIBES`，run 43338735139） | `cargo-sbom` 0.10.0 对 `star-desktop` 生成的 SPDX 2.3 是 `SPDXRef-DOCUMENT DESCRIBES SPDXRef-File-star-desktop`，且该 File 以 `GENERATED_FROM` 指向根 Package；而 workflow 内联 JS 校验器要求 DOCUMENT 直接 `DESCRIBES` 根 **Package**（`npm sbom` 的输出是这种形态，所以前端 SBOM 通过、Rust SBOM 不通过）。本机用同版本 cargo-sbom 复现，456 个 package、关系类型仅 `DEPENDS_ON`/`DESCRIBES`/`GENERATED_FROM` | **未修**：同一段校验逻辑复制在 6 个 job 的内联 JS（Linux/Windows/macOS 的 Validate + 三个 Attest）及 consumer 验证的 Python（约 875 行）里，属于 fail-closed 的发布证明通道，且有 `PHASE-STAR-STABLE-W2-DESKTOP-ATTEST-20261031` 记录的变异/对照验证。建议的修法：接受「DOCUMENT DESCRIBES File，且该 File GENERATED_FROM 根 Package」作为等价形态，并重跑该报告里的变异测试；需要负责人确认 |

## 8. 修订历史

| 版本 | 日期 | 说明 |
|---|---|---|
| v0.1 | 2026-11-13 | 初版：CI 重构、供应链门禁修复、发布编排、ruleset 配置 |
