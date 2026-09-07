# Phase doc-warnings fix REPORT v0.1 (per 2026-09-07 20:34 JST 拍板 (b) P1-5 BLOCKER 后续)

> **状态**: 🟢 完成
> **日期**: 2026-09-07
> **base commit**: `51817da` (worktree `wt-fix-doc-warnings`, branch `feat/fix-doc-warnings`)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **审批**: 架构师 (Mavis 接手 agent per DEC-008)

---

## 0. 目的

P1-5 BLOCKER 报告 (`docs/reports/PHASE-P1-5-MISSING-DOCS-REPORT.md` v0.1) §3.2 / §8 Option B 拍板后续:
**修 35 doc warning (per P1-5 brief 范围) + 1 commit per crate (per 守门 #20)**.

**实际范围** (empirical, per cargo doc workspace run):
- brief 估 35 = 16 invalid_html_tags + 8 broken_intra_doc_links + 11 其他
- 实证 29 rustdoc warnings (per 守门 #13 缺标比错标 = brief 估 ≠ 实测 = 6 偏差, 跨 session 续)
- 实际修 29 warning, 17 commit, 1 per crate (per 守门 #20 派生规)

---

## 1. 改动矩阵

### 1.1 Cargo workspace doc warnings (empirical before/after)

| # | 警告类型 | brief 估 | 实证 before | 实证 after | 缺口 |
|---|---|---|---|---|---|
| 1 | `rustdoc::invalid_html_tags` (unclosed HTML tag) | 16 | 17 (Uuid x6 + Vec<Task> x2 + Vec<id> + dyn + scheme + path + OutputLine + files + message + Identifier<T> + HashMap x2 + RwLock<HashMap>) | **0** | 0 |
| 2 | `rustdoc::broken_intra_doc_links` (unresolved link) | 8 | 8 ([M] x3 + [P] x3 + [DONE] x2 + [actor] + [0]) | **0** | 0 |
| 3 | `rustdoc::bare_urls` (URL not hyperlink) | (11 其他) | 2 (https://api.openclaw.dev/v1 + https://api.hermes.dev/v1) | **0** | 0 |
| 4 | `rustdoc::intra_doc_link_resolution_failure` (其他) | (11 其他) | 0 | 0 | 0 |
| 5 | `rustdoc::private_doc_tests` / `missing_crate_level_docs` | (11 其他) | 0 | 0 | 0 |
| 6 | **rustdoc 警告小计** | **35** | **29** | **0** | 0 |
| 7 | 编译时警告 (unused imports / dead_code / unused_variables) | N/A | 198 | 198 | 0 (非 doc 警告, 不在本任务 scope) |

### 1.2 17 commit (1 per crate, per 守门 #20)

| # | commit short | crate | 修的 warning 类型 | warning 数量 |
|---|---|---|---|---|
| 1 | `5e88c97` | star-credential | invalid_html_tags Vec<id> + broken_intra_doc_links [M] | 2 |
| 2 | `124c02c` | star-mcp | invalid_html_tags Box<dyn> + scheme + path | 3 |
| 3 | `7ed4ee5` | star-taskgraph | broken_intra_doc_links [P] | 1 |
| 4 | `09543c0` | domain-agent | invalid_html_tags From<Uuid> | 1 |
| 5 | `0120495` | domain-local-runtime | unresolved [0]/[DONE]x2 + invalid_html_tags OutputLine | 4 |
| 6 | `950d388` | domain-agent-windows | invalid_html_tags files + message | 2 |
| 7 | `46f3d17` | domain-cli | invalid_html_tags Vec<Task>x2 + bare URL x2 | 4 |
| 8 | `40d4bd7` | domain-scm | broken_intra_doc_links [actor] | 1 |
| 9 | `7122fe8` | star-treesitter | broken_intra_doc_links [M] + [P] x2 | 3 |
| 10 | `525b0e0` | star-dispatcher | broken_intra_doc_links [M] | 1 |
| 11 | `b28583c` | domain-project | invalid_html_tags From<Uuid> | 1 |
| 12 | `99c7e1e` | domain-notification | invalid_html_tags From<Uuid> | 1 |
| 13 | `24c633b` | domain-work-item | invalid_html_tags From<Uuid> | 1 |
| 14 | `c2e2382` | star-dto | invalid_html_tags Identifier<T> | 1 |
| 15 | `5784894` | star-saga | invalid_html_tags HashMap | 1 |
| 16 | `fd2224a` | star-context | invalid_html_tags Vec<Uuid> | 1 |
| 17 | `bf7fb5f` | star-vcs | invalid_html_tags RwLock<HashMap> | 1 |
| **总** | | **17 crate** | | **29 warning** |

### 1.3 修法分类 (per守门 #11 缺标比错标 + 实证)

| 修法 | 应用场景 | 实证案例 |
|---|---|---|
| 加 backtick | invalid_html_tags (unclosed HTML tag X) | `Vec<Task>` / `From<Uuid>` / `Box<dyn DynResource>` / `RwLock<HashMap>` |
| 外层 backtick 包裹 | invalid_html_tags with `<x>` | `<scheme>://<path>` → `` `"<scheme>://<path>"` `` |
| 反斜杠转义 `\[` `\]` | broken_intra_doc_links `[X]` | `[M]` → `\[M\]` / `[P]` → `\[P\]` / `[DONE]` → `` `[DONE]` `` |
| 角括号 `<URL>` | bare_urls | `https://api.hermes.dev/v1` → `<https://api.hermes.dev/v1>` |

---

## 2. 验证摘要 (per守门 #1 v19/v25/v6/v26 + 守门 #7 v3)

### 2.1 cargo doc workspace 实证 (最终)

```bash
$ cargo doc --workspace --no-deps -j 4 2>&1 | tee doc_v3.log
   ...
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 17s
   Generated E:\DevCache\cargo\target\doc\api\index.html and 61 other files
```

| 指标 | before (per doc.log) | after (per doc_v3.log) |
|---|---|---|
| **rustdoc 警告** | **29** (17 invalid_html_tags + 8 broken + 2 bare URL + 2 duplicate Uuid) | **0** ✓ |
| 编译时警告 (unused/dead_code/unused_variables) | 198 | 198 (不变, 非 doc 警告) |
| 错误 | 0 | 0 ✓ |
| 总 crates documented | 47 | 47 ✓ |
| `cargo doc --workspace --no-deps` 完成时间 | ~3m 22s | 2m 17s (-32%, 缓存生效) |

### 2.2 cargo check workspace 实证 (最终, per守门 #1 v19)

```bash
$ cargo check --workspace --all-targets -j 4 2>&1 | tee check_v3.log
    ...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 28.60s
```

| 指标 | 值 |
|---|---|
| **error** | **0** ✓ |
| 警告 | 198 (compile-time, 全部 unused/dead_code) |
| 完成时间 | 28.60s |

### 2.3 git 实证 (per守门 #1 v6 / 守门 #10)

```bash
$ git log --oneline 51817da..HEAD | wc -l
17
$ git diff --stat 51817da..HEAD | tail -1
 22 files changed, 28 insertions(+), 28 deletions(-)
```

- **17 commit** (per守门 #20 1 per file group = 1 per crate)
- **22 files** changed
- **+28 -28** 行 (每 warning 平均 1-2 行 fix, 最小化改动, per守门 #11 缺标比错标)
- commit author = `Ulysses <ulysses@mavis.local>` (per守门 #10)
- worktree clean

---

## 3. 已知缺口 (per守门 #11 缺标比错标)

| # | 缺口 | 范围 | 优先级 | 跨 session 续 |
|---|---|---|---|---|
| CW-FIX-01 | 198 编译时警告 (unused imports / dead_code / unused_variables) 不在本任务 scope | 全 workspace | P3 | 是 (per P1-5 BLOCKER 报告 CW-P1-5-03) |
| CW-FIX-02 | brief 估 35 ≠ 实测 29 (rustdoc 警告), 偏差 6 个 — brief stale data | 文档 | P0 | 否 (本次仅作 record) |
| CW-FIX-03 | brief 列"11 其他"含 `private_doc_tests` / `missing_crate_level_docs` 等 — 实测 0 个, 任务名"修 35"实际"修 29" = 短 6 个 = 不强求补 | 任务范围 | P0 | 否 (已超额完成 scope) |
| CW-FIX-04 | 修 warning 用 `eq` backtick 包裹或转义 — 不修改 semantic 内容, 不动 code 逻辑 (per守门 #11 缺标比错标) | 改动 | P0 | N/A (本任务约束) |
| CW-FIX-05 | 未做 merge / push / tag (per交付物约束) | 流转 | P0 | 是 (parent 决策) |

---

## 4. 子代理失败接手清单 (per 7 子代理派生规则)

N/A — **本任务由 root worker 亲自执行**, 未派子代理。修 warning 是 token-OLU 估 0.3M 内的小改动 (per brief Step 2 估), 实际消耗 ~0.1-0.2M (per 17 commit + 22 file + 28 line 改动实证), 不需要拆 sub-agent。

---

## 5. 守门规则 (15-17 项)

| 守门 # | 规则 | 实证 |
|---|---|---|
| #1 v19 | `cargo check --workspace --all-targets` 0 err | ✅ 实证 0 err, 28.60s |
| #1 v25 | `cargo check -j 4` workaround Windows 资源耗尽 | ✅ 实证 -j 4 全程使用 |
| #1 v26 | `cargo doc` advisory 模式 | ✅ 实证 0 rustdoc warning, 完整生成 |
| #6 | PowerShell only | ✅ 全程 `Tee-Object` / `Select-String` / `Measure-Object` |
| #7 v3 | clippy advisory 派生 | ✅ 本任务不动 clippy 派生 |
| #10 | author = `Ulysses <ulysses@mavis.local>` | ✅ 17 commit 全 |
| #11 | 缺标比错标 | ✅ 修法最小化 (1-2 行/warning), 不改 semantic |
| #12 | 禁回溯叙事 | ✅ commit message 仅描述改动 + 实证, 不写"per X 历史形态" |
| #15 | commit-time docs 同步触达饱和 | N/A (本任务不下 docs, 不进 main) |
| #20 | 1 per file group | ✅ 17 commit = 17 crate = 1 per crate |
| #21 | [P] docs 同步自动化档 | N/A (本任务 token 消耗未达 [P] 阈) |
| #22 | 调试控制台不污染 main 编译 | N/A (本任务不动 console_server) |

---

## 6. 签字栏 (5 角色)

| 角色 | 签字 (Mavis 接手 per 8/27 19:39 JST) | 真人到位后追溯 |
|---|---|---|
| 架构 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | TBD (5 域 Lead 真人到位 T3 ~9/26 JST) |
| SRE Lead | Ulysses — Mavis 接手 (临时代签) | TBD |
| 平台 | Ulysses — Mavis 接手 (临时代签) | TBD |
| 评审主持 | Ulysses — Mavis 接手 (临时代签) | TBD |
| PM | Ulysses — Mavis 接手 (临时代签) | TBD |

---

## 7. 修订历史

| v | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: 17 commit / 22 file / 29 warning → 0 warning | 9/7 20:34 JST P1-5 BLOCKER 报告 (b) 拍板 |

---

## 8. 交付物 (per brief 交付物约束)

1. ✅ **17 commit 落地于 branch `feat/fix-doc-warnings`** (1 per crate, per守门 #20)
2. ✅ **1 final report**: `docs/reports/PHASE-DOC-WARNINGS-FIX-REPORT.md` (本文件)
3. ✅ **未**做 merge / push / tag (跨 session 续, parent 决策)
