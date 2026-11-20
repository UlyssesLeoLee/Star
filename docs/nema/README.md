# docs/nema — NEMA 语法快照与解释器插件追溯

本目录保存 P0 .nema lane 的可追溯语法快照、样例与已知缺口记录。

## 布局

| 路径 | 内容 |
|---|---|
| `grammar-snapshot-2026-11-20.html` | NEMA 官网语法页原文 (实网抓取, HTTP 200) |
| `grammar-snapshot-2026-11-20.md` | 结构化摘要: 语法规则 / 续行语义 / 值类型 |
| `manifest.json` | 全部文件的 SHA-256 记录 + 抓取来源与本地 pin 对照 |
| `examples/*.nema` | v1/v2 × (普通值 / Null / Empty / Missing) + 续行差异样例 |
| `examples/expected/*.ast.json` | 每份样例的期望 AST (Rust strict 模型 JSON) |
| `KNOWN-GAPS.md` | 隔离解释器运行时的诚实缺口标注 (CONTRACT 风格) |

## 追溯链

1. 线上页面 → 原文快照 (sha256 记录于 manifest.json)。
2. 原文快照 → 结构化摘要 (同 manifest)。
3. 样例 .nema → 期望 AST JSON → `crates/nema-codec/tests/plugin_runtime.rs` 的
   `examples_conform_to_pinned_snapshot` 逐一断言 (parse == 期望 AST,
   format→parse 往返无损)。
4. 解释器身份: 每份样例可绑定到 `InterpreterProfile`
   (grammar_revision + plugin_version + abi_version + ast_schema_version +
   grammar_source_sha256 + artifact_sha256), 迁移见
   `crates/nema-codec/src/plugin_runtime.rs`。

## 更新规则 (与 NEMA_INTERPRETER_PLUGIN_CONTRACT.md 一致)

- 线上页面更新 → 新增快照, 不覆盖旧快照; manifest 记录新旧对照与抓取时间。
- 语法行为变化 → 新解释器 profile + 新样例; 旧 profile 与旧样例保持可用。
- 任何 .nema 文件只在其绑定的 profile 下解析, 不静默升级; 迁移必须显式调用。
