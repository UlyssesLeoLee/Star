# 发行许可证 / 锁文件 / Rust 门禁基线证据报告 (2026-11-22)

> 范围: 许可证声明 + 锁文件消费入口 + Rust 全目标门禁 + W9 制品证据矩阵。
> 基线: `D:/Star` dev@`f25e621cca68a82dbe10e8ffbed291a7f9cceb47` (2026-11-21)。
> 方法: 只读核对 + `cargo metadata` / `cargo deny` / 静态 grep。**未运行 `cargo check`/`cargo build`（慢）**, 未修改任何 Cargo.toml / 业务代码, 未提交、未 push。
> 本报告结论以命令实跑输出为准; 每节附命令摘录。

---

## 0. 执行摘要

1. **许可证冲突仍在**: 根 `LICENSE` 正文是 **MIT**, 但根 `Cargo.toml:214` 的 `[workspace.package].license` 仍是 **`Apache-2.0`**, 且 99 个 member crate 通过 `license.workspace = true` 继承它。cargo metadata 实测: 119 个 member 中 **Apache-2.0 ×101、MIT ×12、`MIT OR Apache-2.0` ×6**。W1 的 BLOCKED 理由在本基线依旧成立。
2. **cargo-deny 门禁是绿的但不覆盖此冲突**: `cargo deny --all-features check licenses bans sources` 本地实跑 exit 0 (`bans ok, licenses ok, sources ok`), 但 deny.toml `private = { ignore = true }` 把全部 workspace crate 排除在许可白名单判定外, 所以门禁**看不见** workspace 元数据与 MIT 决策的冲突。
3. **锁文件**: 所有 CI/Vercel 构建面统一消费 `npm ci` (package-lock.json); `frontend/pnpm-lock.yaml` 仍被 git 跟踪但**零消费者**; `frontend/package.json` 未声明 `packageManager`、未声明 `license`。Rust 侧: 根 `Cargo.lock` 是唯一 workspace 锁, 另有 `bff/Cargo.lock`、`tools/aci-emitter/Cargo.lock`(独立 workspace) 与 `crates/star-agent-runtime/Cargo.lock`(member 内嵌套锁, 异常)。
4. **Rust 门禁命令**: CI 实行为 `cargo check --workspace --exclude star-desktop --all-targets --all-features -j 4` (ci.yml:126) + `cargo check -p star-desktop --all-targets -j 4` (ci.yml:322)。本任务未实跑该门禁; 廉价 sanity `cargo metadata --offline --locked --no-deps --format-version 1` **exit 0**, 119 个 member 全枚举。
5. **W9 证据矩阵**: 5 个发行面(Desktop 安装包、Web/Vercel、3 份 Canvas 镜像)的构建入口与锁文件均已静态确认; **真实制品的 digest/SBOM/provenance/部署/回滚证据全部缺失**, 详见 §5/§6。

---

## 1. Workspace 枚举与 Rust 基线

### 1.1 实测命令与结果

```bash
cargo metadata --offline --locked --no-deps --format-version 1
# exit=0; 371,097 bytes JSON
```

解析结果 (cargo metadata 1.99.1 实跑输出):

```
workspace_members: 119
packages: 119
license dist:     {'Apache-2.0': 101, 'MIT': 12, 'MIT OR Apache-2.0': 6}
rust_version dist: {'1.99.1': 114, '(none)': 5}
```

- 与发行报告陈述一致: **119 个 workspace member**(根为 virtual workspace, 无 root package)。
- 命令告警(2 条, 非失败): `profiles for the non root package will be ignored` — `crates/nema-codec-wasm`、`crates/physis-wasm` 在自身 manifest 里声明 profile。
- **Rust 基线 1.99.1**: `rust-toolchain.toml` → `channel = "1.99.1"`; 根 `Cargo.toml:212` → `rust-version = "1.99.1"`; 本机 `cargo 1.99.1 (5f94df478 2026-08-27)` / `rustc 1.99.1 (58cf8f9c3 2026-10-13)`。
- **5 个 member 缺 `rust-version` 字段**: `argo-core`(MIT)、`ide-collab-core`(dual)、`physis-wasm`(dual)、`sandtree-core`(Apache-2.0)、`star-git-core`(dual)。其余 114 个均为 `1.99.1`。这 5 个恰是自带显式 license 的"旧导入"crate。

### 1.2 119 成员枚举与 workspace 边界

根 `Cargo.toml:192` 显式排除: `exclude = ["target", "crates/platform-messaging-proto", "crates/sqlx-sqlite-stub"]`(后者即 `[patch.crates-io]` 的 sqlx-sqlite stub)。metadata 中位于 `crates/` 之外(如 `tools/aci-emitter`、`bff/`)的 package 数为 **0** —— 它们是不属于 119 成员范围的独立 workspace, 各自带独立 Cargo.lock, 闭包审查范围需单独界定(W2/RC-04)。

---

## 2. 许可证事实

### 2.1 根 LICENSE 正文 — MIT

`LICENSE` 前 10 行:

```
1| MIT License
2|
3| Copyright (c) 2026 Ulysses Leo Lee
4|
5| Permission is hereby granted, free of charge, to any person obtaining a copy
...
```

→ 根 LICENSE 实际是 **MIT License**。

### 2.2 Workspace 元数据 — 仍是 Apache-2.0(冲突未修)

根 `Cargo.toml`:

```
209| [workspace.package]
210| version = "0.1.0"
211| edition = "2021"
212| rust-version = "1.99.1"
213| authors = ["Star Team"]
214| license = "Apache-2.0"
```

→ **根 Cargo workspace 仍声明 Apache-2.0, 与根 LICENSE(MIT)冲突**。W1 在 WBS 与 PHASE-DELIVERY-SYNC-20261118 §4.1 中登记为 BLOCKED, 本基线证实该事实未变化。按任务约束, 本报告只登记、不改 Cargo.toml。

### 2.3 per-crate license 分布(cargo metadata 实测, 119 成员)

| license 字段 | crate 数 | 说明 |
|---|---:|---|
| `Apache-2.0` | **101** | 99 个经 `license.workspace = true` 继承根声明 + 2 个显式: `sandtree-core`、`star-agent-runtime` |
| `MIT` | **12** | `argo-core, domain-agent-windows, domain-ai, domain-cli, domain-dashboard, domain-form, domain-kms, domain-report, domain-theme, star-credential, star-taskgraph, star-treesitter` |
| `MIT OR Apache-2.0` | **6** | `ide-collab-core, ide-kernel, nema-codec, nema-codec-wasm, physis-wasm, star-git-core` |
| (缺失) | 0 | 无 license 字段的 member 为 0 |

与根 LICENSE(MIT)不一致的 crate = **107 个**(101 Apache-2.0 + 6 双许可声明; 双许可中 MIT 分支与根 LICENSE 兼容, 但 SPDX 表达式不等价)。grep 层面的显式声明文件数(22)与 metadata 差 2, 原因是 `crates/platform-messaging-proto`(MIT)与 `crates/sqlx-sqlite-stub`(dual)被 workspace `exclude`(见 §1.2), 不计入 119。

### 2.4 cargo-deny 实跑(本机 cargo-deny 0.20.2)

CI 门禁等价命令(local 全 feature 图):

```bash
cargo deny --all-features check licenses bans sources
# 最终输出: bans ok, licenses ok, sources ok ; exit=0
```

单项复跑 `cargo deny check licenses` 亦 exit 0、`licenses ok`, 仅 11 条 `warning[license-not-encountered]`(白名单中当前依赖图未出现的许可, 属告警)。

**关键局限**: `deny.toml` 配置

```
83| private = { ignore = true }   # workspace crate (publish=false) 不参与第三方白名单判定
```

全部 119 个 member 均 `publish = false`(deny.toml §87-98 注记), 因此 **cargo-deny 不检查 workspace 自身元数据** —— MIT 决策 vs `Apache-2.0` 声明的冲突无法被该门禁捕获。deny.toml 第 83 行注释同时写明 "Star canonical license 已由用户确认为 MIT", 即配置作者已知此冲突, 门禁语义上不覆盖首方元数据。

### 2.5 NOTICE.md 状态

根目录存在 `NOTICE.md`(git 跟踪), 但:

```
5| > **生成日期**: 2026-10-05        (数据: cargo deny list, 1265 个 crate)
79| 该适配尚未完成构建与分发依赖闭包审计; 完成前不得据此宣称 release bundle/SBOM 已放行。
```

→ NOTICE 存在但**过期**(2026-10-05 生成, 与 release commit 不同步), 且 RustGameServer adapter 闭包审计未完成(对应 RC-03)。发行前须按同一 commit + lock + all-features 口径重生成。

### 2.6 "发行包继承的许可证"逐面核对结果

| 发行面 | 制品内声明 | 实际随包文件 | 元数据(工具读取) | 结论 |
|---|---|---|---|---|
| Canvas 镜像 ×3 | Dockerfile `LABEL org.opencontainers.image.licenses="MIT"` | `COPY LICENSE /LICENSE` → **MIT 正文** | crate `license.workspace = true` → **Apache-2.0**(canvas-engine/domain-canvas/canvas-realtime) | 镜像 label+LICENSE 文件与 MIT 决策一致; 但 Cargo 元数据仍是 Apache-2.0, cargo-sbom 等工具会报错许可 |
| Desktop 安装包 | Tauri 打包, 无静态可见 license 声明 | 未验证 | `star-desktop` 与 `star-desktop/src-tauri` 均 `license.workspace = true` → Apache-2.0 | 安装包内嵌许可材料**无证据**; Cargo 元数据 Apache-2.0 |
| Web/Vercel | `frontend/package.json` **无 `license` 字段** | 无 | npm sbom 将产出无许可/NOASSERTION | 首方 Web 包许可未声明, 证据缺 |

---

## 3. 锁文件消费入口(逐构建面)

### 3.1 前端锁文件现状

```bash
ls frontend/package-lock.json frontend/pnpm-lock.yaml frontend/yarn.lock
# frontend/package-lock.json   (204,643 B, git 跟踪)
# frontend/pnpm-lock.yaml       (129,787 B, git 跟踪)
# frontend/yarn.lock            -> No such file or directory
```

- **npm 与 pnpm 双锁文件并存且均被 git 跟踪**, yarn.lock 不存在。
- `frontend/package.json` **无 `packageManager` 字段**、**无 `license` 字段**。
- 最近一次锁文件提交: `cf430eb4 chore(deps): resync package-lock.json with upgraded package.json (CI/Vercel npm ci)` —— **维护中的活跃锁是 package-lock.json**; pnpm-lock.yaml 无对应 resync 提交。

### 3.2 各构建面实际使用的包管理器(全仓 grep)

grep `.github/workflows/*.yml` + `frontend/vercel.json`:

| 构建面 | 位置 | 实际命令 | 锁文件 | Node |
|---|---|---|---|---|
| 通用 CI (ci.yml) | ci.yml:378-379 | `npm ci --no-audit --no-fund` (frontend/) | package-lock.json (`cache-dependency-path: frontend/package-lock.json`) | **22** |
| P1E integration ×2 | p1e-integration.yml:123,152 | `npm ci` | package-lock.json | 24.22.2 |
| Web stable build (star-stable-web.yml) | :107 | `npm ci --no-audit --no-fund` → `npm run build` → `npm sbom --sbom-format=spdx` | package-lock.json | 24.22.2 |
| **Vercel** | frontend/vercel.json | `"installCommand": "npm ci"`, `"buildCommand": "npm run build"`, framework `nextjs` | package-lock.json | Vercel 平台 Node(未在仓内锁定) |
| Desktop (star-stable-desktop.yml) | :137,260,389 | `npm ci --no-audit --no-fund` (**crates/star-desktop/frontend**, 自带 package-lock.json) → `tauri build --bundles msi|dmg|deb,appimage -- --locked` | 桌面前端 package-lock.json + 根 **Cargo.lock**(`--locked`) | 24.22.2 |

- 全仓 workflow 中 **pnpm 引用 = 0**; 仅历史文档(scripts/automation/registry.md)留有 pnpm 记录。→ **pnpm-lock.yaml 当前无任何消费者, 是孤儿锁文件**(RC-09 的直接证据)。
- 桌面安装包三种平台: Linux `deb,appimage`(:160)、Windows `msi`(:282)、macOS `dmg`(:412), 全部经 Tauri CLI 透传 `--locked` 到 Cargo。

### 3.3 Rust 锁文件(Cargo.lock)全景

```
./Cargo.lock                          (235,341 B, v4) <- 119 member workspace 唯一锁
./bff/Cargo.lock                      <- 独立 workspace (不在 119)
./tools/aci-emitter/Cargo.lock        <- 独立 workspace (不在 119)
./crates/star-agent-runtime/Cargo.lock <- ⚠️ member crate 内的嵌套锁 (git 跟踪, 疑似遗留)
```

- 三份 Canvas 镜像 Dockerfile 均 `COPY Cargo.toml Cargo.lock rust-toolchain.toml ./` + `cargo build --locked --release --package <crate>`, 上下文由 publish-star-canvas-services.yml 的过滤脚本按 Cargo 闭包组装并重生成锁(`matrix.image = ["canvas-engine","domain-canvas","canvas-realtime"]`, :113)。
- `crates/star-agent-runtime/Cargo.lock` 是 119 成员之一却自带独立锁, cargo 会忽略成员嵌套锁, 该文件为**待清理遗留**。

---

## 4. Rust 全目标门禁(命令登记, 未实跑)

发行门禁要求的实际命令(CI 源码, 非推测):

```yaml
# .github/workflows/ci.yml:122-126
- name: cargo check
  run: cargo check --workspace --exclude star-desktop --all-targets --all-features -j 4
# .github/workflows/ci.yml:321-322 (star-desktop 单独 job)
- name: cargo check -p star-desktop
  run: cargo check -p star-desktop --all-targets -j 4
# .github/workflows/publish-star-canvas-services.yml:60 (Canvas 发布前置)
run: cargo check -p canvas-engine -p domain-canvas -p canvas-realtime --all-targets --locked -j 4
```

本任务按约束**未运行**上述 check(慢); 已运行廉价 sanity 检查:

- `cargo metadata --offline --locked --no-deps --format-version 1` → **exit 0**, 119 member 全枚举(§1.1)。
- `cargo deny --all-features check licenses bans sources` → **exit 0**。
- 结论: 元数据层 sanity 通过; **`--workspace --all-targets --all-features` 的实际编译门禁仍无本机/hosted 证据**(对应 W2 "Rust 1.99.1 all-targets 未实跑"缺口)。

---

## 5. W9 发行面证据矩阵(5 个发行面)

依据 W0/RC-01 锁定的发行面: Desktop(Windows/macOS/Linux)、Web/Vercel、三份 Star Canvas 镜像。

| # | 发行面 | 构建入口 | 锁文件 | 随制品许可声明 | 已有证据 | 缺失证据 |
|---|---|---|---|---|---|---|
| 1 | Desktop 安装包 (msi/dmg/deb+appimage) | star-stable-desktop.yml: `npm ci`(crates/star-desktop/frontend) + `tauri build --bundles ... -- --locked` | 桌面前端 package-lock.json + 根 Cargo.lock | crate 元数据 Apache-2.0(license.workspace); 安装包内嵌 LICENSE 未见 | 静态: 三平台 bundle 命令、`--locked` 透传、Node 24.22.2、SPDX 校验器与 tag-only provenance job 定义 | 真实 hosted build、签名/公证、安装/升级/回滚原始记录、制品 SHA-256、BOM/attestation、内嵌许可材料、CSP |
| 2 | Web / Vercel | star-stable-web.yml: `npm ci` → `npm run build` → `npm sbom`; vercel.json: `npm ci` + `npm run build` | frontend/package-lock.json | **无**(package.json 无 license 字段) | workflow 明确产出 "build evidence bundle"(**不部署 Vercel**, 文件头注释); npm sbom SPDX-2.3 校验步骤定义 | 真实 Vercel production deployment、build/runtime config 记录、Neon/Owner runtime、consumer 与 rollback、首方 Web 包许可声明 |
| 3 | Canvas 镜像 canvas-engine | publish-star-canvas-services.yml(过滤上下文) + crates/canvas-engine/Dockerfile: `cargo build --locked --release --package canvas-engine` | 过滤重生成的 Cargo.lock(根锁投影) | 镜像 `LABEL ...licenses="MIT"` + `COPY LICENSE /LICENSE`(MIT 正文) | 上下文过滤器静态审计(W3/W7 lane 已审)、pinned rust:1.99.1-bookworm base、固定 tag 策略定义 | 真实 BuildKit archive/镜像层、镜像 digest、SBOM/provenance/attestation、consumer 验证、K3s rollout/rollback、crate 元数据 Apache-2.0 vs 镜像 MIT 的冲突修复 |
| 4 | Canvas 镜像 domain-canvas | 同上 (`--package domain-canvas`, HTTP_PORT/PORT=8081) | 同上 | 同上 | 同上 | 同上 |
| 5 | Canvas 镜像 canvas-realtime | 同上 (`--package canvas-realtime`, WS 8083/HTTP 8082) | 同上 | 同上 | 同上 | 同上 |

附: `deploy/k3s-local` 与 Multipass guest image 属测试/执行基础设施, 不进公开 payload(W0); 但 `deploy/star-canvas-services-k3s.yaml` 中三份镜像 digest 仍是占位符。

---

## 6. 缺失证据清单(合并 W1–W9 / RC-01–RC-10)

1. **[W1/RC-02, 许可证]** 根 `Cargo.toml:214` `[workspace.package].license = "Apache-2.0"` 未改为 MIT → 101 个 member 元数据仍是 Apache-2.0; MIT 决策与元数据冲突未修复(本报告只登记, 未修改)。
2. **[W1/RC-03]** NOTICE.md 过期(2026-10-05 生成、1265 crates、RustGameServer adapter 闭包未审计); 未按 release commit + lock + all-features 口径重生成。
3. **[RC-09]** `frontend/pnpm-lock.yaml` 与 package-lock.json 并存且均被 git 跟踪; pnpm 锁零消费者; `package.json` 未声明 `packageManager`; 未选定唯一包管理器。
4. **[Web 许可]** `frontend/package.json` 无 `license` 字段 → Web 制品无首方许可声明。
5. **[W2/RC-04, Rust 门禁]** `cargo check --workspace --exclude star-desktop --all-targets --all-features -j 4` 与 `-p star-desktop --all-targets` 未实跑(本任务仅 metadata sanity, exit 0)。
6. **[W2]** 5 个 member 缺 `rust-version`: argo-core、ide-collab-core、physis-wasm、sandtree-core、star-git-core。
7. **[W2/锁文件卫生]** `crates/star-agent-runtime/Cargo.lock`(member 内嵌套锁)待清理; `bff/`、`tools/aci-emitter/` 独立 workspace 闭包未纳入本次审查范围, 需明确处理(ACI MSRV 1.77 vs 1.99.1 边界)。
8. **[W4/RC-08]** license-gate(cargo-deny)/cargo-audit 的 hosted run 证据缺; 本机 `cargo deny --all-features check licenses bans sources` exit 0 仅为本地证据; 且 deny.toml `private={ignore=true}` 决定该门禁无法发现首方元数据冲突。
9. **[W5/RC-06]** Desktop: 无真实三平台 hosted build、签名/公证、updater 密钥管理、安装/升级/回滚原始记录、制品 SHA-256。
10. **[W6/RC-09]** Web/Vercel: star-stable-web.yml 文件头明确 "does not deploy to Vercel"; 无 production deployment、build/runtime config、Neon/Owner、consumer、rollback 证据。
11. **[W7/RC-07]** Canvas ×3: 无真实 image digest、SBOM/provenance/attestation、consumer 验证、K3s rollout/rollback; crate Cargo 元数据(Apache-2.0)与镜像 MIT label 冲突未解决。
12. **[W9 前提]** 无 v1.0.6 tag、无制品、无远端 run IDs —— 全发行面真实验证均为 0。
13. **[次要]** ci.yml frontend job Node "22" 与其余面 24.22.2 不一致, 锁文件解析行为(lockfileVersion 3)在 Node 22 下可用但口径不统一。

---

## 附录 A: 关键命令与输出摘录

```bash
$ cargo metadata --offline --locked --no-deps --format-version 1   # exit=0
$ python parse -> workspace_members: 119
$ python parse -> license dist: {'Apache-2.0': 101, 'MIT': 12, 'MIT OR Apache-2.0': 6}
$ python parse -> rust_version dist: {'1.99.1': 114, '(none)': 5}

$ cargo deny check licenses                # exit=0, "licenses ok", 11× license-not-encountered
$ cargo deny --all-features check licenses bans sources   # exit=0, "bans ok, licenses ok, sources ok"

$ grep -n "^license\|license" Cargo.toml   # 214: license = "Apache-2.0"
$ head LICENSE                             # 1: MIT License / 3: Copyright (c) 2026 Ulysses Leo Lee

$ ls frontend/*lock*                       # package-lock.json + pnpm-lock.yaml (yarn.lock 不存在)
$ cat frontend/vercel.json                 # "installCommand": "npm ci"
$ grep -rn pnpm .github/workflows/*.yml    # (零命中)
$ grep -n "npm ci" .github/workflows/*.yml # ci.yml:379, p1e:123/152, desktop:137/260/389, web:107

$ grep -n "cargo check" ci.yml             # 126: --workspace --exclude star-desktop --all-targets --all-features -j 4
                                           # 322: -p star-desktop --all-targets -j 4
$ grep -n "image:" publish-star-canvas-services.yml  # 113: ["canvas-engine","domain-canvas","canvas-realtime"]

$ head crates/canvas-engine/Dockerfile     # cargo build --locked --release --package canvas-engine
                                           # COPY LICENSE /LICENSE + LABEL org.opencontainers.image.licenses="MIT"
```

## 附录 B: 本任务边界声明

- 未修改任何 Cargo.toml / 前端业务代码 / workflow; 未提交、未 push。
- 未运行 `cargo check` / `cargo build`(慢); 编译门禁结论引用 CI 源码命令而非实跑。
- 本报告为纯登记性基线证据; 修复决定(尤其根 license 字段、pnpm 锁去留)由 parent/owner 拍板。
