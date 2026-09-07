# Brief: OPT-A4 — 质量 / 技术债 / 优化项 全面扫描

**Agent**: explorer
**Phase**: OPT-WBS
**Created**: 2026-09-07 11:53 JST
**Author**: Mavis 接手 (per 8/27 19:39 JST Ulysses 授权代签)

---

## 1. 任务目标

扫描 `D:\Star` 主仓的**质量与技术债**,识别**优化空间**(非未实装功能),生成结构化清单。

**扫描维度**:
1. **Lint 警告** (守门 #1 v26 + #7 v3 + #1 v26 advisory)
   - `cargo clippy -- -W clippy::all` warning
   - `cargo doc` 警告 (RUSTDOCFLAGS 移除后)
   - `cargo fmt --check` 偏离
2. **missing_docs 警告** (per Cargo.toml lint, 47 crate 警告总量)
3. **unreachable_pub / unsafe_code 违规** (per AGENTS §4.2 实装前一致性门)
4. **测试覆盖缺口** (per守门 #1 v12 100% 守门覆盖里程碑 A.24 实证; 部分域覆盖 < 100%)
5. **CI 实证缺口** (PR #12 9/9 pass 已落地, 但 advisory 4 守门待补 e2e)
6. **e2e / 集成测试阻塞** (per HANDOFF H4 + HANDOFF H1)
7. **性能 hot path 实证缺口** (per守门 #1 v14 release mode 53.7s, 单 crate 100/100 0.51s, 但 workspace timeout 历史问题)
8. **frontend 残留 mock / .test 实证** (per `ux-frontend-fix` worktree)
9. **wiki / docs drift 缺口** (per `f26fa7f` fix(wiki): P0-1 修)
10. **Worktree 清理缺口** (per AGENTS §7 #1 21+ wt pending)

## 2. 已知输入

**基线 (per 9/3-9/6 实证)**:
- workspace 47 packages, 41/41 crate 100% 守门覆盖 (守门 #1 v12)
- release mode test: 41 crate 53.7s 实证 (守门 #1 v14)
- `cargo check --workspace --lib -j 4` 0 err 32.27s (守门 #1 v19, 9/3 实证)
- `cargo check --workspace --all-targets -j 4` 716 err 5+ sub-session 实证 (HANDOFF H1)
- CI PR #12 9/9 pass (advisory 模式 4 守门, per守门 #26)

**Lint 派生约束 (per守门 #1 v26 / #7 v3)**:
- cargo clippy: `-D warnings` → advisory (warning 透传)
- cargo doc: RUSTDOCFLAGS 移除 → advisory
- frontend typecheck/test/build: `continue-on-error: true` advisory
- Setup Node.js: 20 → 22 LTS

**剩余 hot path 缺口**:
- 469 err 待补 (per AGENTS §7 v0.65 "280 err 待补 + 280 err 待分支" 类累计 716 err)
- 8 domain ActorContext 类型不兼容 (H2-EXT #4 #5)
- 280+ err 待 5 域 Lead 真人到位后才能正式化

## 3. 范围与边界

**in-scope**:
- `Cargo.toml` 全部 (workspace + crate 级别 lint 配置)
- `crates/*/src/lib.rs` (lint 警告 main 来源)
- `crates/*/tests/` (测试覆盖)
- `.github/workflows/ci.yml` (CI 守门)
- `frontend/` (per守门 #6 v2 advisory 实证)
- `frontend/src/**/__tests__/` 与 `*.test.tsx`
- `.worktrees/` 列表 (清理缺口, per `git worktree list`)
- `docs/wiki/pgwiki/` + `docs/wiki/docswiki/` (drift 缺口, per `f26fa7f` P0-1 修)

**out-of-scope**:
- 任何 RGS 仓 (`D:\RustGameServer` 完全独立, per AGENTS §5 硬约束)
- 第三方依赖 `Cargo.lock` / `package-lock.json`
- 已经按守门 #1 v12 100% 守门覆盖的 41 crate (无新增 lint warning)

## 4. 交付物

**输出文件**: `docs/briefs/OPT-A4-quality-debt-scan.output.md`

**Schema**:

```markdown
# OPT-A4 质量 / 技术债 / 优化项 扫描报告

> **Created**: <timestamp> JST | **Authority**: Mavis 接手
> **扫描范围**: 47 crate + CI + frontend + worktree + wiki
> **守门基线**: 守门 #1 v19 + #7 v3 + #26 + #6 v2

## §0 摘要
- lint 警告 总数: <N>
- missing_docs 警告: <N>
- 716 err 5+ sub-session 剩余: <N>
- worktree 清理缺口: <N>
- wiki drift 缺口: <N>

## §1 Lint 警告矩阵 (按 crate 维度)
| crate | clippy warning | doc warning | fmt 偏离 | 严重度 |
|---|---|---|---|---|

## §2 missing_docs 警告 Top 10
| crate | 警告数 | 代表 file:line | 修复估计 token |
|---|---|---|---|

## §3 --workspace --all-targets 剩余 err 矩阵
| crate | err 数 | 主要 E-code | 修复路径 |
|---|---|---|---|

## §4 CI advisory 守门实证
| 守门 | 当前 | 缺口 |
|---|---|---|

## §5 测试覆盖缺口 (per 守门 #1 v12 100% 实证)
| crate | 当前覆盖率 | 缺口 |
|---|---|---|

## §6 Frontend 残留 mock / test
| file:line | mock 内容 | 用途 | 替换路径 |
|---|---|---|---|

## §7 Worktree 清理缺口
| wt 路径 | 状态 | 建议处理 |
|---|---|---|

## §8 Wiki / docs drift 缺口
| 文档 | drift 类型 | 修复估计 |
|---|---|---|

## §9 性能 hot path 优化空间
| 路径 | 当前耗时 | 优化点 | 估计加速 |
|---|---|---|---|

## §10 已知缺口
- 本次未扫到的 lint 子集 / 隐藏技术债显式列入
- 任何"per 守门"未量化标"未量化"
```

## 5. 接受标准

- [ ] §1 覆盖 workspace 47 packages
- [ ] §3 引用 `cargo check --workspace --all-targets -j 4` 实证
- [ ] §7 worktree 列表 = `git worktree list` 当前值
- [ ] 全部 commit 引用 7 字符 hash
- [ ] 估算条目数: ≤ 600 行

## 6. 报告语言

中文。

## 7. 时间预算

≤ 250K token (单 sub-session, 因 lint 扫描量大)。

## 8. 失败处理

若 lint 扫描超时 (per 9/2 实证 5min timeout), 拆 crate 分批, 失败子集标"超时未扫" 到 §10。
