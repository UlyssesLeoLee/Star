# aci-emitter

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT_OR_Apache--2.0-blue.svg)](LICENSE)
[![Rust 1.77+](https://img.shields.io/badge/rust-1.77%2B-orange.svg)](rust-toolchain.toml)

> **跨项目 ACI (Assertion Contract Interface) emitter helper crate**
> (per ULYS-191 §4.2.1 brief v0.1, 决策 #5).

对标 Python 实现:
[`tools/star-flash-mock/scripts/_lib_aci_emit.py`](https://github.com/UlyssesLeoLee/Star/tree/main/tools/star-flash-mock/scripts/_lib_aci_emit.py)
—— 各自拷贝 `_lib_aci_emit.{sh,py,.rs}`, 跨项目共享同一份 `.aci.json` schema.

## 项目状态 (per 阶段 0 验收标准)

- ✅ Workspace root `Cargo.toml` (resolver=2, members, deps)
- ✅ `crates/aci-emitter/` 单 crate, ~17 文件
- ✅ 11 公共 API: `AciEmitter::new/with_project/build/build_with/with_tags/to_json/to_value/write/from_file/layer/project`
- ✅ 10 必填字段 + 可选 `suggested_fix` / `tags`
- ✅ 10 validator (assertion_id / scope / expect / actual / status / severity / layer / layer-default-scope / fail-reasoning / scope-unknown-dim)
- ✅ 17 个 `ExpectValueType` 强类型 enum + 4 `Layer` + 4 `Status` + 5 `Severity`
- ✅ CLI `aci-emit` (clap derive, `emit` + `schema` 子命令)
- ✅ 18 测试 (12 unit + 6 integration, **target: ≥12**)
- ✅ `cargo fmt --check` / `cargo check --all-targets` / `cargo test --workspace` 全绿
- ✅ 跨实现 parity (Python → Rust 字段名 1:1)
- ✅ 跨项目 `path = "../aci-emitter"` 集成 smoke (per brief §4.5)

## 快速上手

### 作为库使用 (path dependency)

```toml
# 跨项目 Cargo.toml (per 决策 #5: path = "../aci-emitter")
[dependencies]
aci-emitter = { path = "../aci-emitter/crates/aci-emitter" }
serde_json = "1.0"
```

```rust
use aci_emitter::{
    types::empty_scope, AciEmitter, ExpectActual, ExpectValueType,
    Layer, ScopeMap, Severity, Status,
};

let em = AciEmitter::new(Layer::It);

let mut scope = empty_scope();
scope.insert("module".to_string(), "guards".to_string());

let assertion = em.build(
    "star-flash-mock:guards:g-1",
    scope,
    ExpectActual::new(ExpectValueType::ResponseWithinMs, 2000_i64, "API 应在 2s 内"),
    ExpectActual::new(ExpectValueType::ResponseWithinMs, 5_i64, "实测 5ms"),
    Status::Pass,
    Severity::Info,
    "实测延迟远低于预期",
).unwrap();

let json = em.to_json(&assertion);
println!("{json}");
```

### 作为 CLI 使用

```bash
# 打印 schema
$ cargo run --bin aci-emit -- schema
{
  "aci_version": "0.1.0-draft",
  "required_fields": ["assertion_id", "aci_version", ...],
  ...
}

# 构造一条 PASS assertion
$ cargo run --bin aci-emit -- emit \
    --layer it \
    --assertion-id "test:smoke" \
    --scope project=test,module=smoke \
    --expect-type response_within_ms --expect-value 2000 \
      --expect-description "API should respond within 2s" \
    --actual-type response_within_ms --actual-value 5 \
      --actual-description "Measured 5ms" \
    --status PASS --severity info --reasoning "actual << expect" \
    --output /tmp/test.aci.json

# 退出码: 0 = OK, 2 = schema 校验失败
```

## 跨项目集成示例

每个项目 (`star-flash-mock`, IDE1.0, RGS, CATs, IM1.0, GitGit, Ada)
在自己的 `Cargo.toml` 加:

```toml
[dependencies]
aci-emitter = { path = "../aci-emitter/crates/aci-emitter" }
```

如果后续 D-Boy 拍板将 `aci-emitter` 提取到独立 GitHub repo (per 决策 #5 选项 A),
则改为:

```toml
[dependencies]
aci-emitter = { git = "https://github.com/UlyssesLeoLee/aci-emitter", tag = "v0.1.0" }
```

## 设计要点

### 字段命名 1:1 对齐

| ACI emitter (Rust)            | .aci.json schema        | Python emitter     |
|-------------------------------|-------------------------|--------------------|
| `assertion_id`                | `assertion_id`          | `assertion_id`     |
| `aci_version`                 | `aci_version`           | `aci_version`      |
| `layer`                       | `layer` (ut/it/st/e2e)  | `layer`            |
| `scope` (BTreeMap)            | `scope` (6 dims)        | `scope` (dict)     |
| `expect` / `actual`           | `expect_actual_format`  | `expect`/`actual`  |
| `status` (PASS/FAIL/WARN/SKIP)| `assertion_statuses`    | `status`           |
| `severity`                    | `severity_levels`       | `severity`         |
| `reasoning`                   | `reasoning`             | `reasoning`        |
| `captured_at` (RFC3339 UTC)   | `captured_at`           | `captured_at`      |
| `suggested_fix` (可选)        | `suggested_fix`         | `suggested_fix`    |
| `tags` (可选)                 | `tags`                  | `tags`             |

### 守门对齐 (per AGENTS.md §4)

| 守门 | 对齐 |
|------|------|
| #5 env 安全 | ✅ 纯 schema + emitter, 0 env 涉及 |
| #7 0 unsafe | ✅ `#![deny(unsafe_code)]`, 0 unsafe block |
| #11 缺标比错标 | ✅ brief §6 列已知缺口 G-ACI-03 + 4 项工程风险 |
| #12 docs 同步 | ✅ commit 引用本 README + .aci.json schema + ULYS-191 v0.2 |
| #13 W/T/M | ✅ Assertion struct = Master (SCD-2), JSON output = Transaction (audit append-only) |
| #14v4 Mavis 审核 | ✅ author=Ulysses, 永久授权 per 8/27 19:39 JST |
| #15 scope creep | ✅ 本 crate = 1 sub-agent 1 切点 (不跨 IDE1.0 / 5 项目) |
| #19v19 Python 化 | ✅ Star Python emitter 是规范, Rust 严格 1:1 对应 |
| #24 vendor 中立 | ✅ 仅 serde + serde_json + chrono + thiserror + uuid + clap (CLI only) |

### 已知缺口 (per G-ACI-03)

- **G-ACI-03.a**: Rust emitter 完成 ≠ Python emitter 完成 (Python 已 ship Stage 1, 本 crate 是 Stage 3 启动)
- **G-ACI-03.b**: TS emitter 还未启动 (GitGit MSW 适配需), 留 ULYS-191.8 brief
- **G-ACI-03.c**: 跨项目 `aci-summary` CLI 还未统一, 留 ULYS-191.9 brief

## 验证 (per brief §4 验收标准)

```bash
$ cd tools/aci-emitter
$ cargo fmt --all -- --check       # ✅ exit 0
$ cargo check --all-targets        # ✅ exit 0
$ cargo clippy --all-targets --no-deps   # ✅ exit 0
$ cargo test --workspace           # ✅ ≥12 passed, 0 failed
```

### 跨实现 parity (Python ↔ Rust)

```bash
# 1) 用 Python emitter 构造一条
$ python3 ../star-flash-mock/scripts/_lib_aci_emit.py emit \
    --layer it --assertion-id "test:smoke" \
    --scope project=test \
    --expect-type response_within_ms --expect-value 2000 --expect-description "t" \
    --actual-type response_within_ms --actual-value 5 --actual-description "t" \
    --status PASS --severity info --reasoning "t" \
    --output /tmp/py.aci.json

# 2) 用 Rust emitter 构造同一条
$ cargo run --bin aci-emit -- emit \
    --layer it --assertion-id "test:smoke" \
    --scope project=test \
    --expect-type response_within_ms --expect-value 2000 --expect-description "t" \
    --actual-type response_within_ms --actual-value 5 --actual-description "t" \
    --status PASS --severity info --reasoning "t" \
    --output /tmp/rust.aci.json

# 3) 比较 (去除 captured_at 时间戳后, 应完全一致)
$ diff <(jq -S 'del(.captured_at)' /tmp/py.aci.json) \
       <(jq -S 'del(.captured_at)' /tmp/rust.aci.json)
# 期望: exit 0 (无差异)
```

## 项目结构

```
aci-emitter/
├── Cargo.toml             # workspace root (resolver=2, members, deps)
├── Cargo.lock             # deps 锁版 (sha256 校验)
├── rust-toolchain.toml    # 锁 stable (CI 用 stable)
├── LICENSE                # MIT OR Apache-2.0 dual license
├── README.md              # 本文件
├── CHANGELOG.md           # 版本变更日志
└── crates/aci-emitter/
    ├── Cargo.toml
    └── src/
        ├── lib.rs         # 公共 API re-exports + crate 级 docs
        ├── types.rs       # Assertion / ExpectActual / 4 enum / ScopeMap
        ├── error.rs       # AciEmitError / AciValidationError
        ├── validators.rs  # 10 个 validator fn
        ├── builder.rs     # AciEmitter struct + 5 个 impl 方法
        ├── util.rs        # CLI value coerce + scope k=v parse
        └── bin/
            └── aci-emit.rs # CLI (clap derive, emit + schema)
```

## 链接

- 设计稿: `docs/architecture/2026-09-22-aci-mock-interface/00-design-analysis.md` v0.2
- Brief: `docs/briefs/ulys-191-stage3-aci-emitter.md` v0.1
- Stage 1 brief: `docs/briefs/ulys-191-star-mock-aci-stage1.md` v0.1 (Python emitter 已 ship)
- 父 issue: ULYS-191 (ACI 接口)
- 本 issue: ULYS-191.1 / ULYS-224
- Python 对标: `tools/star-flash-mock/scripts/_lib_aci_emit.py`
- Schema 规范: `tools/star-flash-mock/.aci.json`

## 许可证

本 crate 采用 [MIT OR Apache-2.0](LICENSE) 双协议.