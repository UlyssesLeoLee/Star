# ULYS-154 回归测试覆盖率基线 — 操作手册

> **Status**: PR-1 draft (workspace baseline + tarpaulin CI 接入)
> **Ticket**: https://github.com/UlyssesLeoLee/Star/issues/154
> **Last updated**: 2026-09-26 (Mavis 代签 Ulysses)

## 目标

把 Star monorepo (88 个 Rust crate) 的回归测试覆盖率从"无统一指标"推进到 **workspace 总 line coverage ≥ 90%**, 按价值密度分 4 个 PR 推进。

## 路线图

| PR | scope | 状态 |
| --- | --- | --- |
| **PR-1** (本 PR) | cargo-tarpaulin CI 接入 + baseline 报告 | ✅ 当前 |
| PR-2 | release-s4-5/s4-4/c1b 涉及的 ~10 crate (worktree-shared-dir, ai-vault, terminal-stack, star-flash-mock 等) 补到 90% | ⏳ 待 PR-1 跑出真实数字后定 scope |
| PR-3 | domain-* / star-* 核心库 (~40 crate) 补到 90% | ⏳ |
| PR-4 | bff / application / star-cli 外壳层 + mock 脚本 (~38 crate) 补到 90% | ⏳ |

## 工具选型: 为什么 cargo-tarpaulin

| 工具 | 优点 | 缺点 | 选/不选 |
| --- | --- | --- | --- |
| **cargo-tarpaulin** | 成熟、CI 集成简单、输出 cobertura 标准格式 | 仅 Linux (macOS 不全), Windows ptrace 限制 | ✅ **选** |
| cargo-llvm-cov | 基于 LLVM PGO, 跨平台 (Linux+macOS+Win) | 需要 nightly Rust + 复杂 setup | ❌ PR-1 不上 (后续可加对照) |
| grcov | 灵活, 跨平台 | 需 cargo + 链接时插桩, setup 复杂 | ❌ |

## 当前 CI 接入

`/.github/workflows/coverage.yml`:
- **触发**: push main/dev + 任意 PR + workflow_dispatch
- **平台**: `ubuntu-latest` (tarpaulin 在 Windows/macOS runner 跑不动或统计偏差大)
- **advisory**: `continue-on-error: true`, 失败不挡 PR 合入
- **artifact**: `coverage-*` (14 天保留), 含 `cobertura.xml` + `baseline.txt`
- **PR 评论**: 跑完自动 comment 一份 summary

## 本地复跑

```bash
# 装 tarpaulin (锁版本, 跟 CI 一致)
cargo install --locked cargo-tarpaulin --version "^0.31"

# 全 workspace 跑 (首次约 30 分钟, 受守门 #1 v19 互锁风险影响)
cargo tarpaulin --workspace --lib --timeout 120 --skip-clean --output-dir coverage

# 单 crate 跑 (PR-2+ 主推模式, 跟守门 #1 一致)
cargo tarpaulin -p star-context --lib --timeout 120 --skip-clean --output-dir coverage

# 解析 baseline (复用 coverage/, actions/ 守门 #2 工具)
python3 scripts/coverage_summary.py coverage/cobertura.xml
```

## 解读 tarpaulin 输出

`coverage/baseline.txt` 例子:
```
star-context          lines=450   covered=405   rate=90.00%
star-eventbus         lines=320   covered=288   rate=90.00%
worktree-shared-dir   lines=890   covered=445   rate=50.00%   ← 需 PR-2 重点补
...
── workspace total ──
lines=12345  covered=7890  rate=63.91%
```

每行 4 列: crate 名 / total lines / covered lines / rate%。

**注意**: tarpaulin 对 `async fn` + `tokio::spawn` 的覆盖统计偏低 (分支覆盖不到 spawn 内闭包), 这是工具已知限制, 路线图 PR-2+ 会针对这点加 mock runtime 测试来补。

## 不在 PR-1 scope

- ❌ 不改业务代码
- ❌ 不强制覆盖率门槛 (PR-1 跑完看真实数字再说)
- ❌ 不替代 `cargo test` (ci.yml 的 cargo test 继续守门)
- ❌ 不上 cargo-llvm-cov (待 PR-1 跑通后 PR-2+ 评估是否并行)
- ❌ 不在 macOS / Windows 跑 tarpaulin (后续可选对照)

## 与现有守门的关系

| 守门 | 关系 |
| --- | --- |
| #1 cargo check / test / clippy / fmt | 互不替代 — tarpaulin 是度量, 不挡 PR |
| #6 cargo fmt enforced | tarpaulin 不影响 fmt |
| #13 a/b/c/d 跨域协调 / RLS / 13 类 | 测试覆盖率目标是为了 13 类审计可追溯 |
| #DB-13 W/T/M 持久化测试 | tarpaulin 不替代集成测试, 只是单测维度 |

## 关联

- Issue: https://github.com/UlyssesLeoLee/Star/issues/154
- Parent: ULYS-141 (回归测试总集, 已 closed 但路线图延续)
- 关联 release: #153 (s4-5), #152 (s4-4), #150 (c1b)