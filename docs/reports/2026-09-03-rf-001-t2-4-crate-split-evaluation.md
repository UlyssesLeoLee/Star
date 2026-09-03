# RF-001 T2.4 大 crate 拆分评估报告

> **状态**: 🟡 3 crate (domain-local-runtime 5182 行 / domain-integration 3962 行 / domain-cli 3683 行) 评估完成, 0 行代码改动, 1 评估输出
> **来源**: per 2026-09-03 09:54 JST 用户发令"开始重构" (Phase 5 启动) + 5.4 RF-001 T2.4 大 crate 拆分评估 (per plan v0.6 §2.6)
> **方法**: 跑 `cargo modules structure` 不可用 (无 cargo-modules 工具), 手工 read `crates/<crate>/src/lib.rs` 画模块依赖图, 判断可拆信号

---

## 0. 结论

**3 crate 评估结论**: 都**不建议拆分**, 保持现状. 拆分需拆 3 大独立 crate + 跨域 trait 提取, 风险大 + 跨域类型重构 (H2) 成本 0.3M+ token, 不在 Phase 5 范围.

---

## 1. 3 crate 评估表 (per RF-001 WBS §2 T2.4 步骤 1-3)

| crate | 总行数 | 模块数 | 主要模块 | 可拆信号 | 评估结论 |
|---|---|---|---|---|---|
| `domain-local-runtime` | 5182 | 5+ | runtime / runtime_command / runtime_observation / port / service / entity | 中 (runtime 与其他 crate 高度耦合, 内部边界不清) | ❌ 不建议拆 |
| `domain-integration` | 3962 | 4+ | integration / sync_state / port / service / link / mirror / bidirectional | 低 (内部 integration/sync_state 强耦合, 拆 风险大) | ❌ 不建议拆 |
| `domain-cli` | 3683 | 6+ | cli / command / flag / port / service / registry | 中 (cli 与 domain-* path dep 14 个, 跨域边界清) | ⚠️ 边界清但拆需重新解 14 path deps, 估 0.2-0.3M token |

**3 crate 都不建议拆**, 保持现状.

---

## 2. 不拆的具体原因 (per WBS §2 T2.4 步骤 3 判断标准)

### 2.1 domain-local-runtime 5182 行

- 5+ 模块: runtime / runtime_command / runtime_observation / port / service / entity
- 内部模块互相依赖 (per `lib.rs` 引用): runtime → runtime_command → runtime_observation → port → service → entity 形成单向链
- 无明确"互相独立, 仅通过少数 trait 交互"的可拆信号
- 拆分需新独立 crate (e.g. `domain-runtime-observation` + `domain-runtime-command`) + 重新定义跨 crate trait, 估 0.3-0.5M token + 跨域类型重构 (H2 风险)

### 2.2 domain-integration 3962 行

- 4+ 模块: integration / sync_state / port / service
- 内部 integration/sync_state 强耦合 (sync_state 是 integration 的核心数据结构, 拆 sync_state 需拆整个 integration)
- 拆 sync_state 独立 crate 风险大, 需保证零成本抽象 (Runtime/RuntimeCommand 同步)
- 估 0.2-0.3M token

### 2.3 domain-cli 3683 行

- 6+ 模块: cli / command / flag / port / service / registry
- 14 path dep 跨域 (per Cargo.toml: domain-agent + domain-feedback + domain-identity + domain-permission + domain-project + domain-tenant + domain-work-item + domain-workspace + domain-worktree + ...), 边界清晰
- 拆 cli 独立 crate 可行, 但需重新解 14 path deps (估 0.2-0.3M token), 与 Phase 5 5.4 (T2.4) 5.5 (T3) 5.6 (H2 原 3 domain) 改造互斥
- 风险高, 优先做 Phase 5 其他项

---

## 3. 3 crate 模块依赖图 (per 守门 #1 实证)

### 3.1 domain-local-runtime 5 模块依赖

```
runtime → runtime_command → runtime_observation → port → service → entity
(单向链, 无可拆信号)
```

### 3.2 domain-integration 4 模块依赖

```
integration ↔ sync_state (双向强耦合)
        ↓
port → service
(双向 + 单向, 无可拆信号)
```

### 3.3 domain-cli 6 模块依赖

```
cli → command → flag → port → service
  ↓               ↑
registry ─────────┘
(command 复用 registry, 6 模块边界清但 cli 高度依赖 14 域)
```

---

## 4. 守门实证

| 守门 | 规则 | 本评估实证 | 通过 |
|---|---|---|---|
| #1 | 0 unsafe + 守门实证 | 0 行代码改动, cargo check 0 err baseline 保持 (3 crate cargo check 0 err 跑过) | ✅ |
| #9 | 不 commit 散落子代理产出 + git 实证 | 评估亲自 read + grep, 0 子代理 dispatch | ✅ |
| #12 | commit-time docs 同步 | 1 file docs 同步 (本报告) | ✅ (待 commit) |
| #15 | 死循环饱和约束 | 守门 #15 buffer 充足 | ✅ |
| #19 | agent 交互 Python 化 | docs 改动不算 agent 外部交互 | ✅ |

---

## 5. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 10:00 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: 3 crate 评估 ❌ 不建议拆 (5.1.1 domain-local-runtime 5182 行 / 5.1.2 domain-integration 3962 行 / 5.1.3 domain-cli 3683 行 14 path deps), 0 行代码改动, 3 模块依赖图 | 2026-09-03 09:54 JST 用户发令"开始重构" + Phase 5 5.4 启动 |
