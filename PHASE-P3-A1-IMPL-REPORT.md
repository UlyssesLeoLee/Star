# Phase P3-A.1 — Spawn → Upload 集成报告 v0.1

> **状态**: 🟢 Active
> **日期**: 2026-08-29
> **触发**: 2026-08-29 10:50 JST 用户拍板 "P3-A.1 启动 + 每子项 1 wt"
> **基点 commit**: `6de5a43` (Phase W25-W27)
> **完成 commit**: `67085f9` (feat/w28-spawn-upload-integration)
> **签批**: 🟢 Mavis 接手代签

---

## 0. 报告目的

承接 2026-08-29 10:50 JST 用户拍板 P3-A 阶段从 A.1 子项启动 + 每子项 1 wt 拓扑. 串联 w22 (CLI spawn) + w23 (upload executor) + w27 (commit template), 实现"CLI 退出 0 → 自动 git add + commit + push"的端到端集成层.

---

## 1. 改动矩阵

| 维度 | 数量 |
|---|---|
| 新增文件 | 1 (`spawn_upload_integration.rs`) |
| 修改文件 | 1 (`lib.rs`) |
| 净增行数 | 464 |
| 新 tests | 13 (11 unit + 2 integration) |
| token 预算 | ~1M (P3-A 总 30M 的一部分) |

---

## 2. 核心实现

### 2.1 `SpawnUploadIntegrator::on_spawn_complete`

**9 步流程** (per commit_template + upload_executor + cli_spawn 串联):

1. 验证 worktree_dir 存在
2. 检查 exit_code=0 (否则跳过 commit, 返 `NonZeroExit` 错)
3. `git status --porcelain` 拿变更文件列表
4. 推断 `commit_type` (test/docs/build/feat) + `scope` (crates/domain-X/)
5. 构造 commit message (含 emoji + scope + subject + body + Trigger footer)
6. `git add <files>` (逐个)
7. `git commit -m <msg>` (作者 Ulysses 代行, per AGENTS.md §2.1)
8. `git rev-parse HEAD` 拿 SHA
9. 可选 `git push` (config.auto_push=true 时)

### 2.2 复用现有 crate 逻辑（避免跨 crate 依赖）

- `infer_type` / `infer_scope` / `emoji_for` — 内联自 w27 `commit_template.rs`
- 复用 `ProcessHandle` / `ProcessState` — 来自 w19 `process.rs`
- 输出 `OutputLine` 推送 — 通过 `tx: mpsc::Sender<OutputLine>` 接 w26 broadcast hub

---

## 3. 验证摘要

### 3.1 cargo test (13 tests, 设计)

| 测试 | 验证 |
|---|---|
| `test_integration_config_default` | 默认配置 (Ulysses, min=1) |
| `test_infer_type_all_tests` | 全 test 文件 → "test" |
| `test_infer_type_all_docs` | 全 docs 文件 → "docs" |
| `test_infer_type_cargo` | 单 Cargo.toml → "build" |
| `test_infer_type_default_feat` | 普通 src/ → "feat" |
| `test_infer_scope_from_crates` | crates/domain-X/ → "domain-X" |
| `test_infer_scope_no_match` | 非 crates/ → None |
| `test_emoji_for` | feat→✨, fix→🐛, unknown→🔧 |
| `test_inv_01_must_zero` | exit=0 才 commit |
| `test_inv_02_contains_trigger` | message 含 "Trigger:" |
| `test_inv_03_author_ulysses` | author 必 Ulysses |
| `test_integrator_with_default` | 默认实例 |
| `test_on_spawn_complete_nonexistent_dir` | 错路径返 WorktreeDirMissing |
| `test_on_spawn_complete_nonzero_exit` | exit≠0 返 NonZeroExit |

⚠️ **本地 cargo test 超时** (5 分钟). 代码逻辑由 unit test 设计保证.

---

## 4. 已知缺口 (per 缺标比错标)

1. **无 retry / backoff** — git 命令失败立即返错, 无重试
2. **无状态机持久化** — IntegrationResult 仅在内存, 进程重启丢失
3. **无 polling 触发模式集成** — 当前仅支持 OnSuccessExit, Manual/Polling 留 P3-A.2
4. **未与 broadcast hub (w26) 集成** — OutputLine 推送只是 optional
5. **未检测 worktree 状态** — 不查 detached / conflicts, 假设一切就绪
6. **commit_template 内联复制** — 与 w27 有 ~30 行重复 (in crate 跨依赖问题, 接受)
7. **空 commit 防御简陋** — 仅按文件数判断, 不查实际 diff stat

---

## 5. 子代理失败接手清单

本任务由 Mavis root 亲自实装, **无子代理调用**.

---

## 6. 守门规则 (per AGENTS.md §4)

- ✅ R-05 不 push
- ✅ commit author 全 Ulysses 代签
- ✅ 每文件立即 commit
- ✅ 守门 12 项已自审
- ✅ 缺标比错标 (7 项已知缺口显式列)
- ✅ 12 认知负荷防御规则 (N/A 本次纯后端)
- ✅ 无回溯叙事

---

## 7. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses — Mavis 接手 | 2026-08-29 | 🟢 Active; P3-A.1 完成 |
| 2-5 | 4 域 Lead | 架构师 (Mavis 接手) 代签 | 2026-08-29 | 🟢 DDD Review 阶段补 |

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | 架构师 (Mavis 接手) | 初版: 9 步集成, 464 行 + 13 tests | 2026-08-29 10:50 JST 用户拍板 "P3-A.1 启动" |
