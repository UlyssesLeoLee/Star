# STAR-P4-CREDENTIAL-INVENTORY P4 阶段 外部凭证清单 v0.1

> **Status**: 🟡 Draft v0.1
> **Created**: 2026-09-04 09:00 JST
> **Authority**: Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手代签 (per 8/27 19:39 JST + 21:59 JST 用户授权)
> **承接**:
> - `STAR-P4-UNIMPL-WBS-001.md` v0.1 §2 Phase A.4 外部凭证收集
> - `HANDOFF-ST-001.md` v0.7 §9.5 Blocker #5 (P3-B 凭证)
> - `STAR-P3-WBS-001.md` v0.2 §7 阻塞项汇总 (5/8 项凭证)
> - `2026-09-03-rf-001-blockers-4items-board.md` v0.1 拍板 A (可无限期维持 mock)
> **配套脚本**: `scripts/automation/credential_collect.py` v0.1 (--status / --list / --check)

---

## §0 目的

P4 阶段 Phase A.4 落地 5 项外部凭证收集清单,显式列 mock 备选状态 + 切真操作 + 阻塞影响,避免下游 AI 误把"等凭证"当独立工作项。本清单配套 `credential_collect.py` 脚本可重放,符合守门 #1 v19 [P] 任务卡 4 维要求。

---

## §1 凭证清单(5 项)

### 1.1 B.5 OpenClaw HTTP API endpoint + API key

| 维度 | 内容 |
|---|---|
| **WBS 引用** | `STAR-P3-WBS-001.md:73,205` |
| **影响阶段** | P3-B OpenClaw 真实集成 e2e (5M token 估) |
| **mock 备选** | wiremock 模式 (per commit `29692a7`) |
| **mock 状态** | ✅ 已落地, `docs/frontend/design/mock-msw-handlers.md` |
| **切真条件** | Ulysses 提供真实 endpoint + API key |
| **切真操作** | Mavis 替换 config + cargo test e2e (守门 #5 secret 不进 git) |
| **阻塞** | 🟡 mock 备选可长期维持,不阻塞 P3-B 推进 |
| **可访问性验证** | `python scripts/automation/credential_collect.py --check` 验证 mock_msw_handlers.md 存在 |

### 1.2 B.6 Hermes HTTP API endpoint + API key

| 维度 | 内容 |
|---|---|
| **WBS 引用** | `STAR-P3-WBS-001.md:74,206` |
| **影响阶段** | P3-B Hermes 真实集成 e2e (5M token 估) |
| **mock 备选** | wiremock 模式 (per commit `29692a7`) |
| **mock 状态** | ✅ 已落地, 同 B.5 共享 mock-msw-handlers.md |
| **切真条件** | Ulysses 提供真实 endpoint + API key |
| **切真操作** | 同 B.5 |
| **阻塞** | 🟡 同 B.5 |
| **可访问性验证** | 同 B.5 |

### 1.3 E.4 KMS 集成(Vault / AWS KMS 凭证)

| 维度 | 内容 |
|---|---|
| **WBS 引用** | `STAR-P3-WBS-001.md:151,207` |
| **影响阶段** | P3-E KMS 集成 (5M token 估) |
| **mock 备选** | LocalMockKms (per commit `5ea9611`) |
| **mock 状态** | ✅ 已实装, `crates/domain-kms` |
| **切真条件** | Ulysses 提供 Vault / AWS KMS 凭证 |
| **切真操作** | Mavis 替换 domain-kms + KMS rotation test |
| **阻塞** | 🟡 mock 备选可长期维持,不阻塞 P3-E 推进 |
| **可访问性验证** | `python scripts/automation/credential_collect.py --check` 验证 crates/domain-kms 存在 |

### 1.4 D.2 GitHub Actions runner(windows/macos 跨平台 e2e)

| 维度 | 内容 |
|---|---|
| **WBS 引用** | `STAR-P3-WBS-001.md:128,209` |
| **影响阶段** | P3-D 跨平台 e2e 矩阵 (5M token 估) |
| **mock 备选** | integration_e2e.py (per `docs/automation-design.md` v0.1 §4.5 共享脚本, commit `8ace1d5` 拍板) |
| **mock 状态** | ✅ 已实装, `scripts/automation/integration_e2e.py` |
| **切真条件** | Ulysses 配 GitHub repo 管理员 + 真 runner |
| **切真操作** | Mavis 替换 .github/workflows/ + integration_e2e.py |
| **阻塞** | 🟡 stub 可长期跑,真实跨平台需 Ulysses 权限 |
| **可访问性验证** | `python scripts/automation/credential_collect.py --check` 验证 integration_e2e.py 存在 |

### 1.5 D.6 markdownlint + cargo doc CI job runner

| 维度 | 内容 |
|---|---|
| **WBS 引用** | `STAR-P3-WBS-001.md:132,209` |
| **影响阶段** | P3-D CI job (3M token 估) |
| **mock 备选** | saga_e2e.py (per `docs/automation-design.md` v0.1 §4.5 共享脚本, commit `8ace1d5` 拍板) |
| **mock 状态** | ✅ 已实装, `scripts/automation/saga_e2e.py` |
| **切真条件** | Ulysses 配 GitHub repo 管理员 + 真 runner |
| **切真操作** | Mavis 加 markdownlint + cargo doc CI job |
| **阻塞** | 🟡 stub 可长期跑 |
| **可访问性验证** | `python scripts/automation/credential_collect.py --check` 验证 saga_e2e.py 存在 |

---

## §2 累计统计

| 维度 | 数据 |
|---|---|
| 凭证总数 | 5 |
| mock 备选已落地 | **5/5 (100%)** |
| 阻塞阶段 | 0(全部 mock 可长期跑) |
| 切真操作 | 5(每项需 Ulysses 启动) |
| 影响 token 估 | 23M (5 × ~5M 含 P3-B 5+5 + P3-E 5 + P3-D 5+3) |

---

## §3 拍板请求(per 9/1 14:58 JST "决策必须用选项")

| # | 决策 | 选项 A | 选项 B | 推荐 |
|---|---|---|---|---|
| 1 | 5 项凭证切真时机 | 立即切真(需 Ulysses 提供 B.5/B.6/E.4 凭证) | **维持 mock 长期跑**(per 29692a7 + 5ea9611 + 8ace1d5) | **B**(per 9/3 11:35 JST 拍板 A 已生效) |
| 2 | mock 备选维护节奏 | Mavis 每次 WBS 更新同步重测 | 季度 review 一次 | **A**(守门 #12 commit-time 同步) |
| 3 | 凭证存放方式 | $env:VAR 引用(per 守门 #5) | 加密 vault 集成(需 KMS 凭证) | **A**(永 mock) / **B**(切真后) |

---

## §4 守门规则(本文件专属)

| # | 规则 | 出处 |
|---|---|---|
| 1 | 5 项凭证 mock 备选可长期维持,不阻塞 P3 推进 | 9/3 11:35 JST 拍板 A |
| 2 | 凭证切真需 Ulysses 启动,Mavis 接收后落地 | 守门 #1 + 9/1 14:58 JST 拍板 |
| 3 | secret 不进 git,只走 $env:VAR(per 守门 #5 11:06 JST hard ban) | AGENTS §4 #5 |
| 4 | 配套 `credential_collect.py` 脚本可重放 | 守门 #1 v19 [P] 任务卡 |
| 5 | WBS §7 阻塞项汇总同步更新(per 守门 #12 commit-time 同步) | AGENTS §4 #12 |

---

## §5 签字栏(5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008)— Mavis 接手代签 | 2026-09-04 | 🟡 凭证清单 v0.1 落档, 5/5 mock 已落地 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008)— Mavis 接手代签 | 2026-09-04 | 🟡 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008)— Mavis 接手代签 | 2026-09-04 | 🟡 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008)— Mavis 接手代签 | 2026-09-04 | 🟡 Mavis 接手代签 |
| 5 | 项目负责人(PM)| 架构师 (Mavis 接手 agent per DEC-008)— Mavis 接手代签 | 2026-09-04 | 🟡 Mavis 接手代签 |

---

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-04 09:00 JST | 架构师 (Mavis 接手 agent per DEC-008)— Mavis 接手代签 Ulysses | 初版: 5 项凭证清单(B.5/B.6/E.4/D.2/D.6)+ mock 备选 5/5 已落地 + 切真操作 5 项 + 拍板 3 项 + 守门 5 项 + 5 签字栏 | 2026-09-04 09:00 JST 严格 IPA 7 阶段 Phase A 推进, A.4 凭证清单落档(per 9/4 08:59 JST 拍板) |
