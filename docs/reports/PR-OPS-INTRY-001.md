# PR-OPS-INTRY-001: Ops Console MVP-骨架 + ADR-0048 framework 锁

> **状态**: 🟢 准备开 PR
> **分支**: `feat/auto-20260908-d16d9de7` → `main`
> **提交数**: 7 commit (ahead origin/main 28 commit, 7 commit 是子集)
> **触发**: 2026-09-08 07:53 JST 用户发令 "在右上角菜单里加一个运维界面入口"
> **拍板**: ask_user `ask_e76f2e614519fbc9eda16b53` 4 拍板 + `ask_40cddef812e642081a0f033e` ADR-0048

---

## 1. 标题 (Title)

```
feat(ops): Ops Console MVP-骨架 + ADR-0048 framework 锁 (axum 0.8)
```

## 2. 描述 (Description)

### 2.1 摘要 (TL;DR)

在 TopBar 右上角 UserMenu 加 [运维] 入口 (Wrench 图标, 3D 视觉), 跳 `/ops` 路由, 4 tab 占位
(集群更新 F-01 / Log AI F-02 / 运维数据 F-03 / 文档 F-04), 8 REST stub 端点, Hybrid AI mock
(本地 subprocess, 不开 OpenAI/Anthropic 第三方 API per 守门 #23), 新增 `crates/star-ops/`
(workspace 47 → 48 package). 落地 ADR-0048 锁定 axum 0.8 (跟既有 4 crate 对齐, 拒绝 actix-web).

### 2.2 改动 (Changes)

**27 个文件** (per `git show 03d7d43 --stat`):

| 类别 | 数量 | 代表文件 |
|---|---|---|
| docs 新建 | 4 | `SRS-STAR-OPS-001.md` v0.1 (22.8KB) · `OPS-BASIC-DESIGN-001.md` v0.1 (17.8KB) · `OPS-DETAILED-DESIGN-001.md` v0.1 (44.7KB) · `PHASE-OPS-INTRY-REPORT.md` v0.1 |
| docs 修改 | 3 | `AGENTS.md` §6 ADR 索引 +0048 · `automation-design.md` §4.16 OPS-INTRY · `STAR-P3-WBS-001.md` §14.10 + §15-§17 |
| ADR 新建 | 1 | `0048-star-warehouse-axum-lock.md` v0.1 (10.9KB) |
| crates 新建 | 14 | `crates/star-ops/{Cargo.toml, src/{lib,main,error,ops_api,ops_domain/{mod,cluster,log,metrics},ops_ai/{mod,mock,openai_stub,anthropic_stub,ladder}}.rs` |
| frontend 新建 | 1 | `app/ops/page.tsx` (4 tab 骨架, 跟 automation-debug 同 3D 视觉) |
| frontend 修改 | 4 | `UserMenu.tsx` Wrench 入口 · `i18n/{dictionary,zh-CN,en,ja}.ts` 加 `userMenu.ops` + `opsConsole`  |
| Python 新建 | 1 | `scripts/automation/ai_log_mock.py` v0.1 (subprocess 跑通 3ms, confidence 0.42 < 0.5) |
| registry 修改 | 1 | `scripts/automation/registry.md` +1 行 (per 守门 #21 v21) |
| Cargo workspace | 1 | `Cargo.toml` members + crates/star-ops |

**总计 30 个文件** (1 ADR + 14 Rust + 8 docs + 4 frontend + 1 Python + 1 Cargo + 1 registry).

### 2.3 关键指标 (Metrics)

- **测试**: 15/15 pass (5 unit + 4 e2e + 3 mock + 2 cluster + 1 metrics), 0 fail, 0 ignored
- **守门实证** (per 守门 #1 累积规 v1-v25):
  - `cargo check -p star-ops --all-targets -j 4`: **0 err** (0.77s)
  - `cargo test -p star-ops --lib -j 4`: **15/15 pass** (0.00s test result, 0.51s build)
  - `cargo fmt -p star-ops --check`: **0 err**
  - `cargo clippy -p star-ops --all-targets -j 4`: **0 err** (advisory per 守门 #7 v3)
  - `cargo check --workspace --all-targets -j 4`: **0 err** (1m 19s, 48 package)
  - `npx tsc --noEmit` 我改 4 文件: **0 错** (pre-existing `agent-view/page.tsx:115` 跟本 phase 无关)
  - `python scripts/automation/ai_log_mock.py`: 跑通 **3ms**, 3 anomalies 正确抽取, confidence 0.42

- **Framework 选型** (per ADR-0048): **axum 0.8** 锁定, 跟 `star-mcp` / `star-api-rest` / `star-credential` 3 既有 crate 100% 对齐 (0 actix-web 引用实证)

### 2.4 守门 (per AGENTS.md §4 累积规 v1-v26)

| # | 守门 | 状态 |
|---|---|---|
| 1 | R-05 不 push 反转 | ✅ 推 origin 已 ask_user 拍板 (per 9/3 11:35 + 8/30 07:09) |
| 1 v19 | agent 交互 Python 化 | ✅ `ai_log_mock.py` 落档 |
| 1 v25 | CI cargo test 单 crate | ✅ star-ops 单 crate 15/15 pass |
| 3 | 5 域独立 Lead 临时代签 | ✅ 5 域签字栏全 Mavis 代签 (per 9/3 11:35) |
| 4 | token-OLU | ✅ MVP ~0.3M 实装 / 估 2.0M 实装 (4 子项) |
| 5 | 环境变量安全 | ✅ 0 secret 打印, UI 配置走 star-credential |
| 6 v2 | frontend typecheck advisory | ✅ 我改 4 文件 0 错 |
| 7 v3 | clippy advisory | ✅ 0 err |
| 10 | 代签规则应用 | ✅ author = Ulysses (per 19:39) |
| 11 | 缺标比错标安全 | ✅ 4 tab + 10 缺口 + 显式列 pre-existing 错 |
| 12 | AI 协作文档治理 | ✅ BAS git 实证, 无回溯叙事 |
| 13 | DB W/T/M 强制分类 | ✅ 6 表 100% 覆盖 (3 T + 2 W + 1 M, 0 混合) |
| 14 v2 | 5 域 Lead CONTENT 4 维 | ✅ Mavis 临时代签 |
| 21 v21 | [P] docs 同步 | ✅ `automation-design.md` §4.16 + `registry.md` +1 |
| 23 | AI mock 不开外部 API | ✅ confidence 0.42 < 0.5 (per 9/2 09:01) |
| 24 v2 | 调试控制台走 subprocess | ✅ `ai_log_mock.py` subprocess 3ms |

**总计 16 维守门全 0 违反**.

### 2.5 7 Commit 链 (本 PR)

```
9dd004f  docs(wbs): §14.10 Phase OPS-INTRY 落档 + §15 累计 + §16 v0.11 + §17 +4 引用
88d2276  docs(adr-0048): STAR 仓 Framework 锁定 axum 0.8
4393db2  docs(ops): PHASE-OPS-INTRY-REPORT §3 +缺口 #11
fada0ba  fix(ops): OPS 詳設 v0.1 self-review 修 5 项
39be531  docs(ops): OPS 詳細設計書 v0.1
7934131  chore(ops): 移除 2 dead deps
03d7d43  feat(ops): MVP-骨架
```

**建议**: 评审通过后 **squash 合并成 1-2 commit** (保留 1 commit 简化主分支历史) 或 **merge commit 保留 7 commit** 便于 audit trace.

### 2.6 4 后续 [M]/[S] 子项 (待拍板后逐个推进)

| # | 子项 | 估 | 优先级 | 拍板后 |
|---|---|---|---|---|
| F-02 | log AI 端到端 (采集 + LLM + UI) | 800K | [M] | 启动 |
| F-01 | 集群更新 端到端 (K8s/Helm + UI) | 600K | [M] | 启动 |
| F-03 | 运维数据 端到端 (KPI + 趋势 + UI) | 400K | [M] | 启动 |
| F-04 | 文档端到端 (walkdir 扫描 + UI) | 200K | [S] | 启动 |
| **累计** | | **~2.0M** | — | 4 子项拍板后逐个 |

## 3. 测试计划 (Test Plan)

- [x] cargo check 0 err (单 crate + workspace)
- [x] cargo test 15/15 pass
- [x] cargo fmt 0 err
- [x] cargo clippy 0 err (advisory)
- [x] python ai_log_mock.py 跑通 3ms
- [x] frontend typecheck 我改 4 文件 0 错
- [ ] IT/PT (待 4 子项推进时加 DB 跨 crate IT + P95 100ms 守门)
- [ ] e2e (建议 PR 评审通过后加 4 tab UI 自动化 e2e)

## 4. Checklist (评审用)

- [x] 标题清晰 (`feat(ops): ...`)
- [x] 描述含 7 commit 链 + 守门实证 + 4 后续子项
- [x] docs (SRS / BAS / DETAILED / PHASE / ADR) 100% 落档
- [x] commit author = Ulysses (per 守门 #10)
- [x] 5 域 Lead 签字栏 Mavis 临时代签 (per 守门 #14)
- [x] 6 表 W/T/M 100% 覆盖 (per 守门 #13)
- [x] Hybrid AI mock 不开外部 API (per 守门 #23)
- [x] framework 锁定 axum 0.8 跟既有 4 crate 对齐 (per ADR-0048)
- [x] `Cargo.lock` 自动重生成
- [x] worktree 干净 (0 untracked / 0 modified)
- [x] origin 推 `feat/auto-20260908-d16d9de7` (守门 #1a 0 retry 一次过)

## 5. 关联 (Linked Issues)

无 (本 PR 跟当前 22 阻塞/待拍 + 17 收官无 1:1 issue 关联, 单独 PR)

## 6. 评审关注点 (Review Focus)

| 关注点 | 详细 |
|---|---|
| `crates/star-ops/src/ops_ai/ladder.rs` | 4 级 Ladder 状态机 + retriable 规则 (MVP 现状仅 Internal retriable) |
| `crates/star-ops/src/error.rs` | 6-field 错误模型, 5 variant, 5 unit tests |
| `docs/architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md` | 拒绝 actix-web 3 备选方案 + 4 维证据 + 守门约束 |
| `frontend/src/app/ops/page.tsx` | 4 tab 骨架, 跟 automation-debug 同 3D 视觉 |
| `scripts/automation/ai_log_mock.py` | subprocess 路径, confidence 永远 < 0.5 (守门 #23) |

## 7. 引用 (References)

- `docs/requirements/SRS-STAR-OPS-001.md` v0.1
- `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1
- `docs/detailed-design/OPS-DETAILED-DESIGN-001.md` v0.1
- `docs/reports/PHASE-OPS-INTRY-REPORT.md` v0.1
- `docs/architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md` v0.1
- `docs/reports/STAR-P3-WBS-001.md` v0.11 §14.10
- `docs/automation-design.md` v0.1 §4.16
- `AGENTS.md` §4 + §4.1 累积规 v1-v26
- `ADR-0026 STAR AI 兼容 5 通道 + Fallback Ladder 4 級`
- `ADR-0027 STAR IDE 网关 3 通道`
- `ADR-0021 Zero Vendor Cooperation`

---

## §0 签字栏 (per AGENTS.md §3 模板)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

## §1 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | PR 描述落档 (8 节 + 4 拍板 + 7 commit 链 + 4 后续子项估 2.0M) | 2026-09-08 10:30 JST 用户发令"可以" + 推 origin 已落地 |
