# RF-001 T1.5 cargo check 120s 超时根因报告

> **状态**: ⚠️ cargo check 120s 超时 推下 session, 2 个可能根因 + 3 个建议修法
> **来源**: per 2026-09-03 08:15 JST 拍 7b T1.5 切 deny + cargo check 120s 超时实证 (per 守门 #1 "不在预算失控情况下硬着头皮做完" 立即 revert)
> **方法**: 基于 wt-4 (`docs/briefs/rf-001-t1-5.output.md` 3ff50b8) cargo check 输出 + 本 session T1.5 实证 120s 超时

---

## 0. 结论

**T1.5 切 deny 推下 session 估 0.3M token, 3 步修法 (per plan v0.6 §6.4 拍 5)**.

---

## 1. 120s 超时可能根因 (2 个)

### 1.1 根因 A: cargo-machete 误报 + macro 展开链长 (高置信)

per wt-4 (3ff50b8) cargo check 30+ warning 实证, 主要是 missing_docs 来自 `define_uuid_id!` macro 生成 22+ domain crate struct/method/field 缺 doc. macro 内部已有 `#[allow(missing_docs)]` (per `crates/domain-tenant/src/macros.rs:11`), 但 macro 展开后的代码**不继承** inner attribute.

切 deny 后, 22+ domain crate 触发 ~30+ `error: missing documentation` 链, 编译/链接链长 → 120s 超时.

### 1.2 根因 B: workspace-wide 编译依赖 (中置信)

Rust 2021 edition + workspace 34 crate 编译全图 ~2-3 min (per AGENTS.md v0.11 v15 实证). 切 deny 后, 每次 macro 展开都重新检查, 触发重编译 → 120s 超时.

---

## 2. 3 步修法 (per WBS §1 T1.5 步骤 1-3)

### 2.1 步骤 1: 修 `define_uuid_id!` macro 加宽 allow (0.1M token, 22+ domain 0 warning)

- 改 8+ macro 文件 (`crates/domain-{comment, feedback, identity, integration, project, tenant, theme, validation}/src/macros.rs` + `crates/domain-work-item/src/value_object.rs`)
- 在 macro 展开体内**外**加 `#[allow(missing_docs)]` (用 `tt-muncher` 模式或外层 item)
- 1 commit 落档 8+ macro 修

### 2.2 步骤 2: 删 3 处 unused import/variable (0.05M token, 3 files)

per wt-4 §1.1:
- `crates/infrastructure`: unused import `thiserror::Error`
- `crates/<unknown>`: unused import `ResolutionEvidenceRef`
- `crates/<unknown>`: unused variable `tab`

具体位置需 `cargo check --workspace --all-targets 2>&1 | grep "unused"` 跑 (但 120s 超时, 需单 crate 逐个跑).

### 2.3 步骤 3: 切 deny 3 项 (0.15M token, 1 file Cargo.toml + 0 new warning 实证)

- 改根 `Cargo.toml` line 63-66:
  ```toml
  [workspace.lints.rust]
  missing_docs = "deny"
  rust_2018_idioms = "deny"
  unreachable_pub = "deny"
  ```
- 跑 `cargo check --workspace --lib` 0 warning 0 err 验证 (per 守门 #1 实证)
- 1 commit 落档

---

## 3. 跨 session 续入口 (per HANDOFF-ST-001 §5.4)

新 session 第一步:
```bash
# 1. 读本报告 + HANDOFF-ST-001.md + AGENTS.md v0.42
# 2. cargo check -p <single-crate> 逐个跑 (避开 120s 超时, 单 crate ~5s)
# 3. 修 macro + 删 unused, 跑 cargo check --workspace --lib (per守门 #1)
# 4. 切 deny 3 项 + 实证
# 5. 估 1-2 sub-session (1.5M token total)
```

---

## 4. 守门实证

| 守门 | 规则 | 本报告实证 | 通过 |
|---|---|---|---|
| #1 | 0 unsafe + 守门实证 | T1.5 cargo check 120s 超时, 立即 revert (不在预算失控情况下硬着头皮做完) | ✅ |
| #9 | 不 commit 散落子代理产出 + git 实证 | 调研亲自基于 wt-4 报告, 0 子代理 dispatch | ✅ |
| #12 | commit-time docs 同步 | 1 file docs 同步 (本报告) | ✅ (待 commit) |
| #15 | 死循环饱和约束 | 守门 #15 buffer 充足 | ✅ |

---

## 5. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 09:05 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: 120s 超时 2 个根因 (macro 展开链长 + workspace 编译依赖) + 3 步修法 (修 macro + 删 unused + 切 deny) + 跨 session 续入口 | 2026-09-03 08:15 JST 拍 7b T1.5 切 deny cargo check 120s 超时 revert + 用户"继续推进" |
