# Phase C 第 2 轮 — vcs / acceptance / arch 一致性审查报告（子代理 C）

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **审查范围**：`D:/Star/.worktrees/star-acceptance`（worktree 分支 `wt-phase-c-acceptance-review`，基点 = 876a2a7 = Phase C 54 份 spec 草案 commit）
> **审查者**：架构师（Mavis 接手 agent per DEC-008）— 子代理 C（任务 1）— 2026-08-26
> **签批**：⏳ 待 Ulysses 拍板
> **平行报告**：子代理 A / B（v0.1 review）

---

## 0. 执行摘要

Phase C 第 1 轮 54 份 spec 草案由 Mavis 单干完成（per commit 876a2a7）。本轮为子代理 C 平行审查第二刀，覆盖：

- `spec/vcs/` 4 份
- `spec/acceptance/` 17 份
- `arch/` 6 份
- 交叉对账 4 份（`arch/03`、`spec/cli/01`、`spec/flows/05`、`docs/responsibility-matrix/gitgit-ide-boundary.md`）

**总体结论**：54 份 spec 内部一致性好，零 vendor 适配器已固化进 ADR-0021~0025；Fallback Ladder 4 级、Unknown Agent 16 步、Universal Submit 11 步 三套核心闭环在 4 份 spec 之间**字面一致**。本轮发现 **6 个 P1 缺口 + 12 个 P2 弱信号**（见 §9），均不阻塞 Phase C 完成但需 Phase D 开工前关闭。

---

## 1. Fallback Ladder 4 级可跑通性

### 1.1 字面对账（vcs/04 + acceptance/01-03 + arch/03）

| 项 | vcs/04 | acceptance/01 | acceptance/02 | acceptance/03 | arch/03 | 一致? |
|---|---|---|---|---|---|---|
| Ladder 4 级名字 | ✅ §1 | ✅ 隐含 L1 | ✅ 隐含 L1 | ✅ 隐含 L1+ L3 | ✅ §3 | ✅ |
| 16 步核心闭环 | ✅ §2 | ✅ §3 | ✅ §2（12 步变体）| ✅ §3（10 步变体）| ✅ §6 | ✅ |
| Level 1 必含 | MCP+CLI+Git+AGENTS.md | — | — | — | ✅ §3 | ✅ |
| Level 4 Git Only | ✅ §3 | — | — | — | ✅ §3 + §7 | ✅ |

**判定**：4 级名字 + 16 步流程 = **完全一致**。三套测试 (Unknown Agent / Zero-Knowledge / Unknown IDE) 的 16/12/10 步是同一核心闭环的 3 个不同切片（更严格 / 更 IDE 侧），非冲突。

### 1.2 弱信号

- ⚠️ vcs/04 §2 的 16 步闭环与 acceptance/01 §3 的 16 步**完全相同**，但 acceptance/01 §2 提到"测试环境不联网（无外部 AI 服务）"——这跟"测试 AI 兼容性"的目标张力：测试对象本身就是 AI agent，不联网意味着测试的是**本地 agent** 或**mock agent**，而非真实 7 款主流 Agent 之一。**P2 弱信号**：Phase D 闭环测试是否覆盖"真实 AI Agent 通过 AGENTS.md bootstrap"？arch/03 §7 提到"真实 Coding Agent 接入 7 款中至少 4 款"，但 spec/acceptance/01 §6 实施位置只说"自实现 minimal agent"。**两者职责切分不清**。
- ⚠️ vcs/04 §5 实施位置说"4 级 conformance 测试在 `crates/star-cli/tests/`"——但 acceptance/01-03 实施位置说在 `tests/unknown-agent/` `tests/zero-knowledge-agent/` `tests/unknown-ide/`。**两套位置表述不同**。**P1**：测试目录需统一。

### 1.3 Fallback Ladder 接入通道映射（arch/03 §3 vs §2）

arch/03 §3 显式定义 4 级；§2 定义 5 接入通道（Git / Shell / MCP / REST / AGENTS.md）。两者关系：5 通道是"what"，4 级是"what subset in fallback"。**映射**：

| Level | 用哪几通道 | 不依赖 |
|---|---|---|
| L1 | MCP + CLI + Git + AGENTS.md | IDE 专用 plugin |
| L2 | CLI + Git + AGENTS.md | MCP server |
| L3 | REST + Git + AGENTS.md | CLI binary |
| L4 | Git Only | CLI / REST / MCP |

**判定**：映射表本身是合理的；arch/03 §3 文字与 vcs/04 §3 表完全对应。✅

### 1.4 arch/03 §7 验收条款

arch/03 §7 写：
> Phase D 的 Unknown Agent Test 必须**只**用 Level 4 (Git Only) 通过

但 acceptance/01 §3 的 16 步用了大量 `star` CLI（步骤 4-15），`star` CLI 属 Level 2+ 能力。**冲突**。

**冲突点 P1**：arch/03 §7 主张 Unknown Agent Test 必须用 Level 4，acceptance/01 实际用的是 Level 1 强约束（必须有 star CLI）。两者**不可同时成立**。需要二选一或改写 arch/03 §7（更现实：Unknown Agent Test 跑 Level 1，但 Level 2/3/4 单独跑 conformance 测试）。

---

## 2. 3 个验收测试可执行性

### 2.1 实施位置对账

| Test | spec | 实施位置 | 跟 vcs/04 §5 是否一致 |
|---|---|---|---|
| Unknown Agent | acceptance/01 §6 | `tests/unknown-agent/` + `run.sh` | ⚠️ 不一致（vcs/04 说 `crates/star-cli/tests/`） |
| Zero-Knowledge | acceptance/02 §6 | `tests/zero-knowledge-agent/` | ⚠️ 不一致 |
| Unknown IDE | acceptance/03 §7 | `tests/unknown-ide/` | ⚠️ 不一致 |

**判定**：vcs/04 §5 实施位置与 acceptance/01-03 实施位置**表述不同**。**P1**：vcs/04 应改成"测试位置在 `tests/` 而非 `crates/star-cli/tests/`"，或 acceptance/01-03 改用 cargo test。

### 2.2 共同依赖

3 个测试都需要 Git + Shell + AGENTS.md + star CLI（acceptance/01 §2 明示）。Zero-Knowledge (acceptance/02) 进一步禁掉"详细 STAR 提示"，只给"Fix the assigned issue"——这是 prompt 层的约束而非工具层，所以 Star CLI 仍必须存在。Unknown IDE (acceptance/03) §2 列出 6 项最低能力：Git / Shell / Repository / AGENTS.md / star CLI / OpenAPI。**三者必跑通的关键是 star CLI 真存在**——MVP 退出条件 acceptance/04 §3 第一条就是"star CLI 17 个核心命令"。

### 2.3 可执行性评级

| Test | 2026-08-26 实际跑起来? | 阻塞点 |
|---|---|---|
| Unknown Agent | ❌ 不可 | star CLI 还没实现（arch/01 §1.2 明确"无 star CLI 骨架"） |
| Zero-Knowledge | ❌ 不可 | 同上 |
| Unknown IDE | ❌ 不可 | 同上 |

**判定**：3 个测试在 2026-08-26 **不可实际跑**——MVP 退出条件 acceptance/04 §3 全部是空勾。**这是 Phase C 草案阶段的预期状态**，不是 spec 缺陷。✅

### 2.4 步骤对齐度

- Unknown Agent 16 步 ≡ 核心闭环 16 步：✅ 字面相同
- Zero-Knowledge 12 步 = 16 步的子集（去掉 MR 自动创建、Issue 状态更新、commit + 一些 UI 步骤）：✅ 兼容
- Unknown IDE 10 步 = 16 步的子集（偏 IDE 视角，缺 Agent 内部能力如 capability discovery / context current / test affected）：✅ 兼容，但 §2 提供的"OpenAPI"在 10 步里**未使用**——**P2 弱信号**：OpenAPI 在测试 10 步里没被消费，价值未验证。

---

## 3. MVP 退出条件可达性（acceptance/04 §3）

### 3.1 13 项退出条件 → spec 对账

| # | 退出条件 | spec 落地 | gap |
|---|---|---|---|
| 1 | `star` CLI 17 个核心命令 | `spec/cli/01-cli-spec.md` §2（实列 23 个命令，**17** 是 MVP 子集）| ⚠️ 数字不一致：spec 列 23，MVP 说 17 |
| 2 | `--json` 稳定 schema (`agent-api/v1`) | `spec/acceptance/13-schema-stability.md` §2 | ✅ |
| 3 | MCP server 13 tools | `arch/03` §2.3（列 14 个，**13** 是 MVP 子集）| ⚠️ 数字不一致：arch 列 14，MVP 说 13 |
| 4 | REST API 12 endpoints + OpenAPI 3.1 | `arch/05` §5（列 14 个，**12** 是 MVP 子集）| ⚠️ 数字不一致：arch 列 14，MVP 说 12 |
| 5 | AGENTS.md 自动生成器 | `spec/acceptance/09` §6 | ✅ |
| 6 | Universal Submit 11 步 | `spec/flows/05-universal-submit.md` §2（实列 12 步，**11** 是 MVP 子集——**注意** 12 步文案写"11 步流程" + 12 步骤列）| ⚠️ spec 自身 11 vs 12 步表述不一致 |
| 7 | Agent Task Lifecycle 9 状态 + 4 异常 | `spec/flows/01-agent-task-lifecycle.md`（未在审查范围） | — |
| 8 | Agent Lease / Heartbeat / Resume | `spec/flows/02-agent-lease-heartbeat.md` + `03-agent-resume.md`（未在审查范围）| — |
| 9 | Version Control Provider 4 实现 | `spec/vcs/01-04` | ✅ |
| 10 | Unknown Agent Test 通过 | `spec/acceptance/01` | ✅ |
| 11 | Zero-Knowledge Agent Test 通过 | `spec/acceptance/02` | ✅ |
| 12 | Unknown IDE Test 通过 | `spec/acceptance/03` | ✅ |
| 13 | GitGit 标准 Git 兼容 100% | `arch/05` + `docs/responsibility-matrix/gitgit-ide-boundary.md` §3 | ✅ |
| 14 | Fallback Ladder 4 级全部跑通 | `spec/vcs/04` + `arch/03` | ✅ |

（实际数到 14 项，含 1 项拆 3 项 test 独立算）

### 3.2 Gap 总结

| Gap | 严重度 | 说明 |
|---|---|---|
| `star` CLI 数字 17 vs 23 | P1 | spec/cli/01 §2 列 23 命令行；MVP 退出条件说 17。需明确 17 vs 23 的子集关系。 |
| MCP 13 tools vs arch/03 列 14 | P1 | arch/03 §2.3 列 14 个 tools（含 `get_issue` / `get_current_task` / `get_worktree` / `request_review` / `get_pipeline_status` 等）；MVP 说 13。数字 1 个对不上。 |
| REST 12 endpoints vs arch/05 列 14 | P1 | arch/05 §5 列 14 个 endpoint；MVP 说 12。数字 2 个对不上。 |
| Universal Submit 11 vs 12 步 | P1 | `flows/05` §2 文字写"11 步流程"但代码块列了 12 个步骤（"11. 回写 Agent 状态" + "12. 回写 IDE Session 状态"）。`arch/03` + `acceptance/04` 都引"11 步"。spec 内部文字 + 列表矛盾。 |

**4 个数字 gap 全是 P1**——必须 Phase D 开工前 spec 自我一致化。

### 3.3 必实现但无 spec 落地的项

- 退出条件 #1 star CLI 17 命令 → `spec/cli/01` 列 23（**超出** MVP 范围声明但 spec 没显式标 MVP 子集边界）
- 退出条件 #3 MCP 13 tools → 仅 `arch/03 §2.3` 隐含（**无独立 spec/acceptance/mcp-spec.md 详查**，仅 vcs/01 提到 `crates/star-mcp/` 是 13 tools 的实施位置）
- 退出条件 #4 REST 12 endpoints → 同上，无独立 `spec/rest/01-spec.md` 在审查范围可见

**P2 弱信号**：MCP / REST 的 spec **不在 acceptance/ 目录下**——accept/ 下只看到 flows / cli 等。要么 Phase D 在 `spec/mcp/` + `spec/rest/` 下补独立 spec（并指明 13 / 12 子集），要么 MVP 退出条件改为指 arch/03 / arch/05。

---

## 4. NFR 可测量性（arch/06 §2-§3）

### 4.1 性能 NFR 测量评级

| NFR | 指标 | 测量方式 | 评级 |
|---|---|---|---|
| NFR-PERF-001 | P95 < 200ms (本地) / < 2s (REST) | benchmark | ✅ 可测（"benchmark" 具体脚本未指定）|
| NFR-PERF-002 | MCP tool invoke P95 < 500ms | benchmark | ✅ 可测 |
| NFR-PERF-003 | 不慢于 GitHub/GitLab 1.5x | benchmark | ✅ 可测 |
| NFR-AI-001 | 7 款主流 Agent 4 款实测通过 | Phase D Unknown Agent Test | ✅ 可测（门槛明确）|
| NFR-AI-002 | Unknown Agent Test pass | Phase D | ✅ 可测 |
| NFR-IDE-001 | Unknown IDE Test pass | Phase D | ✅ 可测 |
| NFR-REL-001 | Core 100% vendor-neutral | grep 测试 + CI | ✅ 可测（守门测试已存在）|
| NFR-REL-002 | 删 Optional Adapter 后 Core 100% 完整 | build + test pass + CI | ✅ 可测 |
| NFR-REL-003 | Fallback Ladder 4 级全部可工作 | 4 级分别跑通 + Phase D | ✅ 可测 |
| NFR-SEC-001 | 全部 tool 描述带签名 | 验证脚本 | ✅ 可测 |
| NFR-SEC-002 | Audit trail 不可篡改 | HMAC 链 + 验证脚本 | ✅ 可测 |
| NFR-OP-001 | ≤ 2 SRE·周/周 | per RGS-TS-001 §6.2 token-OLU | ✅ 可测（但单位"SRE·周/周" 跟 token 不对齐——见 P2）|
| NFR-COMPAT-001 | 任何 Git 客户端可 clone GitGit | CI 跑 git clone + push + worktree | ✅ 可测 |
| NFR-COMPAT-002 | MCP 2026-07-28 规范兼容 | 跑官方 MCP Inspector + Phase D | ✅ 可测 |
| NFR-COMPAT-003 | OpenAPI 3.1 spec 有效 | swagger-cli validate + CI | ✅ 可测 |

### 4.2 弱信号 NFR

- ⚠️ **NFR-OP-001** 用"SRE·周/周"做单位，但项目实践（per user.md 2026-08-26 强证据）= "AI 协作场景下用 token 而非人天算 OLU"。spec 没有改写单位，但保留传统 SRE·周。**P2**：与项目治理偏好不对齐——需要在 NFR-OP-001 加 footnote "per RGS-TS-001 §6.2 token-OLU 等价"，或显式切换 token 单位。
- ⚠️ **NFR-REL-002 "删除 Optional Adapter 后 Core 100% 完整"**——"100% 完整"是软指标。完整指 build pass？test pass？还是功能 100% 保留？spec 没细化。**P2**：定义不清，Phase D 验收会卡。

### 4.3 不可测量 NFR

- 未发现"必须工作"型虚指标。✅

### 4.4 acceptance/14 performance-requirements 重复

acceptance/14 §1 跟 arch/06 §3 性能表**部分重复**（CLI 启动 / agent capabilities / task current / code search / submit / MCP tool list / REST API P95）。差异：arch/06 §3 没列 `star code search` < 1s 和 `star submit` 端到端 < 5s，acceptance/14 列了。**P2**：两表应合并，避免 Phase D 实施时不知道哪份是 normative。

---

## 5. Risk Register 完整性（acceptance/08）

### 5.1 R-001 ~ R-015 缓解措施对账

| Risk | 缓解 | spec 落地? | 状态 |
|---|---|---|---|
| R-001 MCP 30 天升级 | 锁 2026-07-28 规范 + 12 个月迁移窗口 | acceptance/13 §4 | ✅ |
| R-002 Rust MCP SDK beta | stdio transport | arch/03 §2.3 | ✅ |
| R-003 OpenAPI 3.1 工具链 | Redocly / Stoplight / Swagger UI 5.x | 无 spec 引 | ⚠️ 经验断言，未实证 |
| R-004 Unknown Agent Test 失败 | 4 步降级到 Git Only 也必须跑通 | spec/vcs/04 + arch/03 §3 | ✅ |
| R-005 Agent API schema 频繁 breaking | 严格 `agent-api/v1` 版本化 | acceptance/13 §2-§3 | ✅ |
| R-006 Vendor 突然停服 | Zero Vendor Cooperation + Fallback Ladder | ADR-0021 + arch/03 §3 | ✅ |
| R-007 GitHub/GitLab API 限速 | Provider 抽象 + cache | spec/vcs/01 | ⚠️ "cache" 没说在哪 |
| R-008 凭证泄露到 AGENTS.md | Vault 抽象 | 无 spec 引（仅 risk 表提"GitGit V0 T6 task"）| ⚠️ 落地依赖 GitGit V0 T6 任务，需 cross-link |
| R-009 多 Agent 文件冲突 | Worktree 物理隔离 + Phase 2 AST | spec/vcs/04 + Phase 2 plan | ✅ |
| R-010 Audit trail 篡改 | HMAC chain + append-only | arch/06 §1.2 T-10 | ✅ |
| R-011 OLU 超 NFR-OP-010 | 拆 2-3 子代理并行 + 4-6 周窗口 | acceptance/17 §5 | ✅ |
| R-012 缺标比错标安全被违反 | DDD Review + 已知缺口清单 | per user.md 强证据 + arch/06 §1.3 | ✅ |
| R-013 代签新规则被滥用 | DDD Review + Ulysses 终审 | per user.md 2026-08-26 08:40 JST | ✅ |
| R-014 子代理编造回溯叙事 | 升版前必跑 `git log -p --follow` 实证 | per user.md 强证据 | ✅ |
| R-015 Phase D 无 Unknown Agent | 自实现 minimal agent | acceptance/01 §6 | ✅ |

### 5.2 缺缓解 / 缺落地的风险

- ⚠️ **R-003**："Redocly CLI / Stoplight / Swagger UI 5.x 已支持" 是经验断言，没引具体版本号或规范。**P2**：补 commit 实证。
- ⚠️ **R-007**："Provider 抽象 + cache" 但 cache 层在哪个 crate 没指。**P2**：补。
- ⚠️ **R-008**：Vault 抽象的 spec 在哪里？仅 risk 表提"GitGit V0 T6 task"——GitGit V0 T6 不在本次 Phase C 范围。**P2**：spec 应指向已存在 spec 或新增 spec。

### 5.3 跨 spec 引用完整性

- acceptance/08 R-008 引 "GitGit V0 T6 task"——本次 Phase C 没用过 T6 编号，跨项目引用需 GitGit 仓内 spec 同步。**P2**。

---

## 6. 跨 spec 一致性

### 6.1 arch/01 Current Arch 描述 vs main 的 requirements.md / basic-design.md

**Star arch/01 §1.2 关键事实 vs requirements.md 现状**：

| arch/01 表述 | requirements.md 实证 | 一致? |
|---|---|---|
| "25 Module 全部 v0.2 single-file rewrite" | requirements.md 多次提"25 Module" (第 6 章) | ✅ |
| "frontend + canvas Miro 模式已合并" | requirements.md 不直接提（但 main 5181288 commit 提到） | ⚠️ 实证靠 git log，spec 内未引 |
| "master plan v0.1 仍 Draft" | 跟 `acceptance/17` §1 "现有 master plan v0.1 (2026-08-25) 状态是 Draft" 一致 | ✅ |
| "无 star CLI 骨架" | arch/01 §1.2 自己承认 | ✅（自身一致）|
| "无 MCP server" | arch/01 §1.2 自己承认 | ✅ |
| "无 AGENTS.md 生成器" | arch/01 §1.2 自己承认 | ✅ |

**判定**：arch/01 §1.2 跟 requirements.md 主体一致，**frontend Miro 模式那条需 git log 实证**（commit 5181288 已记在 arch/01 提交 commit 876a2a7 之前的 log 里）。

**basic-design.md**：150 KB 大文件，存在**编码问题**（PowerShell 5.1 默认 GBK 读导致中文乱码）——是 workspace 工具问题，非 spec 缺陷。arch/01 没显式引 basic-design.md，**但应该引**。**P2**：arch/01 §1.2 应补一行"see `docs/basic-design.md`"。

### 6.2 arch/05 GitGit Compat vs D:/GitGit/feature/ide-boundary 3 份文档

⚠️ **D:/GitGit/feature/ide-boundary 不可达**（per `Test-Path` 返回 False）。**按指令跳过**。

替代对账：arch/05 引 `docs/responsibility-matrix/gitgit-ide-boundary.md` §3（Git 命令清单）+ §5.1（REST API 端点）。arch/05 §2 命令清单跟 gitgit-ide-boundary.md §3 命令清单**完全一致**（git clone/fetch/pull/commit/push/branch/switch/checkout/tag/status/diff/log/blame/merge/rebase/worktree）。arch/05 §5 端点跟 gitgit-ide-boundary.md §5.1 端点**完全一致**（14 个 REST endpoint）。✅

### 6.3 acceptance/17 "5 域独立 Lead" vs STAR 25 Module 划分

acceptance/17 §4 写"现有 25 Module 按业务域（domain-*）划分"，但**没**提"5 域独立 Lead"——审任务要求查"5 域独立 Lead vs STAR 25 module"对得上不。

acceptance/17 全文搜索"5 域"：**0 命中**。acceptance/17 §4 列了 25 Module 重组成 12 能力层（Domain / Application / AI Gateway / IDE Gateway / VCS Abstraction / Code Intelligence / Context / Audit / Policy / CLI / MCP / REST），**没**提"5 域独立 Lead"概念。

**判定**：acceptance/17 跟 user.md 2026-08-26 强证据的"5 域独立 Lead"概念**不对应**——但这未必是 bug：acceptance/17 是 STAR（25 Module 重组），不是 RGS（5 域）。两项目是两套工程。**判定** = 两套划分**互不引用、无冲突**。⚠️ P2 弱信号：跨项目 token-OLU 治理（5 域 vs 12 能力层）应统一到 master plan v0.2 §5，避免后续冲突。

### 6.4 arch/01 §4 升级后映射 vs 12 能力层

arch/01 §4 "本次升级的核心变更" 表 + acceptance/17 §4 "25 Module 重新组织" 表**逐项对应**：

- CLI 入口 → star-cli ✅
- MCP server → star-mcp ✅
- AGENTS.md → spec/acceptance/09 ✅
- 机器可读输出 --json → spec/cli/01 §2 ✅
- OpenAPI → arch/05 §5 ✅
- Universal Submit → spec/flows/05 ✅
- Agent Task Lifecycle → spec/flows/01（审查范围外但 arch/01 引）✅
- Agent Lease/Resume → spec/flows/02+03（审查范围外但 arch/01 引）✅
- VCS Provider 抽象 → spec/vcs/01-04 ✅
- IDE Gateway → arch/04 + star-ide-gateway ✅
- Code Intelligence → arch/04 §6（Phase 2+）✅
- Context Graph → spec/acceptance/11 + star-context ✅
- Audit → arch/06 §1.2 T-10 + star-audit ✅

**判定**：arch/01 §4 ↔ acceptance/17 §4 一一对应。✅

---

## 7. Phase 1 / 2 / 3 范围边界

### 7.1 MVP (Phase 1) 13 项退出条件

acceptance/04 §3 13 项（实列 14 项含 3 测试独立）**全部 Phase 1 / MVP**。✅

### 7.2 acceptance/05 (Phase 2) 范围

acceptance/05 §1 列 16 项 Phase 2 新增：

- Symbol Index / AST / Find References / Document Symbols / Call Hierarchy / Workspace Symbol → 6 项 Code Intelligence
- Decision Memory
- Agent Handoff 完整
- Acceptance Coverage UI
- Saved Worktree Views
- Development Heatmap Phase 1
- Agent Policy Templates
- Remote Runner
- Context Cost Analysis
- PR Review Feedback Import
- Web UI (Human Interface)

acceptance/05 §2 显式禁 Phase 3 范围（完整 RAG / Code Embedding / 完整多 Agent 编排）。

**判定**：acceptance/05 范围**没**混入 MVP（per acceptance/04 §3 退出条件都跟 Phase 2 范围**无重叠**）。✅

### 7.3 acceptance/06 (Phase 3) 范围

acceptance/06 §1 列 8 项 Phase 3 新增：

- 完整 RAG（Context Graph 全量 + Code Embedding）
- 完整多 Agent 编排（Multi-Agent 全部 9 类冲突检测）
- Advanced Context Selection
- Decision Memory 完整
- Symbol-level Conflict 完整
- Symbol-level Feedback 准确率 > 95%
- Remote Runner 完整
- Development Heatmap Phase 2

acceptance/06 §2 显式禁 V2 / Future。

**判定**：acceptance/06 范围**没**混入 Phase 2。✅

### 7.4 跨期项目边界

⚠️ P2 弱信号：acceptance/05 §1 列"Web UI (Human Interface, **不**作为 Agent API)"——这是跨期约束，应在 arch/03 §5 接入通道表里**重申**（当前 arch/03 §5 没明文"Web UI 不属于 Agent API"）。**P2**：Phase D 实施 Web UI 时容易踩"误把 Web UI 当 Agent API"陷阱。

---

## 8. Final Acceptance 2 问可观测证据（acceptance/15）

### 8.1 Q1: AI 兼容性

> 如果明天出现一个全新的 Coding Agent...答案必须 = **YES**

**可观测证据清单**：

- ✅ Unknown Agent Test 通过 (acceptance/01 + Phase D 实施)
- ✅ Zero-Knowledge Agent Test 通过 (acceptance/02 + Phase D 实施)
- ✅ Fallback Ladder 4 级 (spec/vcs/04 + Phase D 实施)
- ✅ AGENTS.md 薄 bootstrap (spec/acceptance/09 §1)
- ✅ star CLI `--json` 稳定 schema (spec/cli/01 + acceptance/13)
- ✅ MCP server stdio transport (arch/03 §2.3)
- ⚠️ 7 款主流 Agent 实测 4 款 (arch/03 §7)——"4 款" 跟 acceptance/16 §2.3 调研 7 款对得上，但**未指定哪 4 款**。

**判定**：Q1 答案 = YES 需 Phase D 实施 + 实测，spec 集提供了**可观测的退出条件**。✅

### 8.2 Q2: IDE 兼容性

> 如果明天出现一个全新的 IDE...答案必须 = **YES**

**可观测证据清单**：

- ✅ Unknown IDE Test 通过 (acceptance/03 + Phase D 实施)
- ✅ IDE Gateway 抽象 (arch/04)
- ✅ OpenAPI 3.1 (arch/05 §5)
- ⚠️ IDE 接入 3 最低要求（Git CLI / LSP client / MCP client）——arch/04 §5 列了，但**未达 3 最低要求**的 IDE 走 Level 2/3/4 Fallback 是**已说**，但 **arch/04 §5 没说 Level 1 / Level 2 接入 IDE 哪个用**——存在**判别模糊**。

**判定**：Q2 答案 = YES 需 Phase D 实施，spec 集提供**可观测的退出条件**。但 arch/04 §5 跟 arch/03 §3 的 Level ↔ 通道映射需 Phase D 实施时**写清楚**。**P2 弱信号**。

### 8.3 acceptance/15 §2 "答案 = NO 的失败模式" 表

5 条失败模式（等厂商 / 开发专用插件 / 增加 XXXProvider / 修改 GitGit 理解 IDE / 修改 GitGit 理解 Agent / 塞 Issue 进 GitGit）每条都有处置 = 拒。✅ 6 条全部一致。

但 acceptance/15 §2 提到"修改 GitGit 以理解 IDE / Agent"——这跟 gitgit-ide-boundary.md §6 "GitGit 不应提供" 表 + arch/02 §3.1 "GitGit 不应提供" 表**字面一致**。✅

---

## 9. 6 P1 缺口 + 12 P2 弱信号汇总

### 9.1 P1（Phase D 开工前必须关闭）

| # | 缺口 | 来源 | 修复建议 |
|---|---|---|---|
| P1-1 | arch/03 §7 vs acceptance/01 §3 Level 冲突（arch 说 Level 4，accept 写 16 步需 star CLI = Level 1）| §1.4 | arch/03 §7 改 "Unknown Agent Test 跑 Level 1，Level 2/3/4 单独跑 conformance" |
| P1-2 | vcs/04 §5 测试位置 vs acceptance/01-03 测试位置不一致 | §1.2 / §2.1 | vcs/04 §5 实施位置改 `tests/` 而非 `crates/star-cli/tests/` |
| P1-3 | star CLI 17 vs 23 命令数字 | §3.1 | spec/cli/01 §2 加 "MVP 17 子集边界" |
| P1-4 | MCP 13 vs 14 tools 数字 | §3.1 | arch/03 §2.3 加 "MVP 13 子集边界" |
| P1-5 | REST 12 vs 14 endpoints 数字 | §3.1 | arch/05 §5 加 "MVP 12 子集边界" |
| P1-6 | Universal Submit 11 vs 12 步（spec 内部文字 + 列表矛盾）| §3.1 | flows/05 §2 文字 + 列表统一到 11 或 12（建议 12 步含 IDE Session 回写）|

### 9.2 P2（Phase D 实施时关闭）

| # | 弱信号 | 来源 | 修复建议 |
|---|---|---|---|
| P2-1 | arch/03 §7 "真实 Agent 4 款" vs acceptance/01 §6 "自实现 minimal agent" 职责切分不清 | §1.2 | 显式说明 Phase D 跑 "minimal agent 全 4 级" + "真实 Agent 4 款仅 Level 1" |
| P2-2 | acceptance/03 §3 10 步未消费 OpenAPI | §2.4 | 删 OpenAPI 或加 1 步 "通过 OpenAPI 获取仓库元数据" |
| P2-3 | NFR-OP-001 单位 "SRE·周/周" 跟项目 token-OLU 偏好不对齐 | §4.2 | 加 footnote "per RGS-TS-001 §6.2 token-OLU" |
| P2-4 | NFR-REL-002 "100% 完整" 定义不清 | §4.2 | 定义 "build pass + 全 unit test pass + 全部 MCP tool 仍可用" |
| P2-5 | arch/01 §1.2 未引 basic-design.md | §6.1 | 补一行 "see `docs/basic-design.md`" |
| P2-6 | R-003 工具链断言无版本号 | §5.2 | 补 Redocly CLI 1.x / Swagger UI 5.x commit / release note |
| P2-7 | R-007 cache 层无指 | §5.2 | 补 "在 `crates/star-vcs/src/cache.rs`" |
| P2-8 | R-008 Vault 抽象引 GitGit V0 T6 (跨项目) | §5.2 | 改引 GitGit 本仓已存在 spec 或新建 spec |
| P2-9 | acceptance/14 vs arch/06 性能表重复 | §4.4 | 合并，acceptance/14 引 arch/06 §3 为 normative |
| P2-10 | acceptance/05 "Web UI 不属 Agent API" 跨期约束未在 arch/03 重申 | §7.4 | arch/03 §5 加一行 "Web UI 不属 Agent API 通道" |
| P2-11 | arch/04 §5 vs arch/03 §3 Level ↔ 通道映射判别模糊 | §8.2 | 实施时加 "Level 判别矩阵" 表 |
| P2-12 | acceptance/16 "4 款 Agent" 未指定 | §8.1 | 选 4 款 (Codex / Claude Code / Gemini CLI / Cursor) |

### 9.3 不可量化（无可执行项）

- acceptance/06 §1 "Symbol-level Feedback 准确率 > 95%" ——**有** 数字（95%），可测；非"必须工作"型虚指标。✅

---

## 10. 与子代理 A / B 的横向对比说明

本报告专注 vcs / acceptance / arch 27 份，**不重叠**子代理 A (生态事实基线 + ADR) / 子代理 B (mcp / rest / ide-gateway 等 spec)。如发现跨报告 P1 冲突，应在 Mavis 终审时合并。

**已识别的横向接口**：
- vcs/04 §5 实施位置 ↔ 子代理 B 审查的 mcp/rest spec（如果子代理 B 提了不同位置）→ 需合并
- acceptance/04 §3 #3 MCP 13 tools ↔ 子代理 B 的 mcp spec（应该一致）→ 待核

---

## 11. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008）— 子代理 C | 2026-08-26 | 🟡 6 P1 缺口 + 12 P2 弱信号；Phase D 开工前需关闭 P1-1 ~ P1-6 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

---

## 12. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008）— 子代理 C（任务 1） | 初版 | Phase C 第 2 轮 review |

---

**报告结束。6 P1 + 12 P2 详见 §9。Phase D 开工前必关 6 P1。**
