# Changelog

All notable changes to `aci-emitter` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-23 (ULYS-191.1 / ULYS-224)

### Added

- 独立 Rust `aci-emitter` crate, 跨 7 项目共享 (Star / IDE1.0 / RGS / CATs / IM1.0 / GitGit / Ada).
- Workspace 结构: `resolver = "2"`, members = `["crates/aci-emitter"]`, rust-version = 1.77 (MSRV).
- 11 公共 API:
  - `AciEmitter::new(layer)` / `with_project(name)` / `layer()` / `project()`
  - `AciEmitter::build(...)` / `build_with(...)` (5 + 10 字段, 10 必填 + 可选)
  - `AciEmitter::with_tags(assertion, tags)` (dedupe 便利方法)
  - `AciEmitter::to_json(assertion)` (sort_keys + ensure_ascii=false)
  - `AciEmitter::to_value(assertion)` (programmatic 消费)
  - `AciEmitter::write(assertion, path)` (UTF-8 + 末尾换行, 自动 mkdir parents)
  - `AciEmitter::from_file(path)` (JSON 反序列化, 10 必填字段强类型保证)
- 4 强类型 enum:
  - `Layer` (4: ut / it / st / e2e)
  - `Status` (4: PASS / FAIL / WARN / SKIP)
  - `Severity` (5: critical / high / medium / low / info)
  - `ExpectValueType` (17: response_within_ms / response_status_2xx / ... / no_resource_leak)
- `ScopeMap` = `BTreeMap<String, String>` (有序 + dedupe, 保证序列化稳定)
- 10 validator (断言 ID 含 `:` / 6 scope dims / expect.type ∈ 17 / status ∈ 4 / severity ∈ 5 / layer ∈ 4 / FAIL 必填 reasoning / scope-unknown-dim / scope-empty-value / layer-default-scope-setdefault)
- 2 错误链:
  - `AciEmitError` (用户输入 / IO / JSON 解析, 用于 CLI 层)
  - `AciValidationError` (字段级验证, 用于 `build()` 返回)
- CLI binary `aci-emit` (clap 4.5 derive):
  - `schema` 子命令: 打印 10 必填字段 + 4+4+5+17+6 枚举清单 (跟 Python `_lib_aci_emit.py schema` 1:1)
  - `emit` 子命令: 14 CLI 参数 (--layer / --assertion-id / --scope / --expect-type / --expect-value / --expect-description / --actual-type / --actual-value / --actual-description / --status / --severity / --reasoning / --suggested-fix / --tags / --captured-at / --output)
  - 退出码: 0 = OK, 2 = schema / emit 校验失败
- 18 测试 (12 unit + 6 integration):
  - 10 必填字段构造 (PASS minimal / FAIL with reasoning + suggested_fix + tags)
  - 6 validation 错误路径 (assertion_id empty / missing-colon / scope empty / unknown-dim / FAIL no reasoning)
  - 4 强类型 enum 序列化 (`status="PASS"` / `layer="it"` / `severity="info"` / `expect_type="response_within_ms"`)
  - JSON 序列 / 反序列化 round-trip (含 sort_keys + ensure_ascii=false + 中文 description)
  - with_tags dedupe
  - write 自动 mkdir parents
  - 17 ExpectValueType 全部跑一遍 build + serialize
  - 10 必填字段名与 Python emitter 1:1
- Cargo workspace 标准 (resolver=2, MSRV 1.77, edition 2021, dual license MIT OR Apache-2.0).
- `rust-toolchain.toml` 锁 stable (CI 用 stable, 不绑 1.98.1).
- `.gitignore` (/target/, Cargo.lock.bak, .DS_Store, .idea/, .vscode/).

### Notes

- 字段命名严格按 `tools/star-flash-mock/.aci.json` schema 1:1 (Python emitter 是规范).
- Scope 用 `BTreeMap` 而非 `HashMap`, 保证序列化顺序稳定 (便于与 Python dict cross-diff).
- clap 4.5 derive 引入为 CLI-only 依赖, 库使用者不强制依赖 (虽然 clap 列在 `[dependencies]` 但仅 bin target 使用).
- 守门 #7 0 unsafe: `#![deny(unsafe_code)]` 在 lib.rs 顶部.

### Known gaps

- **G-ACI-03.a**: Rust emitter 完成 ≠ Python emitter 完成 (Python 已 ship Stage 1, 本 crate 是 Stage 3 启动).
- **G-ACI-03.b**: TS emitter 还未启动 (GitGit MSW 适配需), 留 ULYS-191.8 brief.
- **G-ACI-03.c**: 跨项目 `aci-summary` CLI 还未统一, 留 ULYS-191.9 brief.

[0.1.0]: https://github.com/UlyssesLeoLee/aci-emitter/releases/tag/v0.1.0