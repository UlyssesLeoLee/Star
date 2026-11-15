# Star 功能与文档批评性评估 (STAR-FEATURE-DOC-CRITIQUE-20261115)

> **Review scope**: Star 仓库全量功能与文档体系 (crates/ 118 workspace package + bff 独立 workspace + frontend/ + docs/ 2217 md + CI/deploy 栈)，对应 Multica issue ULYS-285「批评与建议」。
> **Reviewer**: Multica agent (qwen3-8, on behalf of Ulysses)，独立于原实现会话。
> **方法**: 只读实证。依赖图用 `cargo metadata --format-version 1` resolve 图逐 package 反向遍历 (含 bff 独立 workspace 的 path 依赖合并)；CI 状态用 `gh run list / gh run view --log-failed`；文件计数用 `find | wc -l`；文档声明与实测逐项对账。所有数字均可用文中给出的命令复现。
> **Base**: 本任务分支自 `08ef56e4` (dev 谱系)；`origin/main = caa0ca7e` (PR #303)。评估结论对该两点均适用 (差异为 frontend 3 commit)。
> **日期**: 2026-11-15

---

## §0 Verdict (先给结论)

**ACCEPT WITH SERIOUS RESERVATIONS.**

Star 的文档纪律、风险披露文化、CI 工程化 (action SHA 固定、ci-gate 聚合、license/SPDX 证据链、canvas 服务 digest+provenance 发布) 在单人项目里属于罕见的高水准。但本次评估发现一个**结构性问题压过所有其他问题**：

> **约 1/4 的 Rust 代码 (40 个 lib crate, 79,929 行) 没有任何消费者** —— 不被任何可部署二进制 (star-api-rest / star-mcp / star-cli / star-ops / canvas 3 服务 / sandboxd / bff / star-desktop) 依赖，只被自己的测试引用。与此同时 WBS 把对应条目标为「100% 已完成」。

这不是「代码质量差」——这些 crate 大多有测试、有文档头、守门合规。这是**验收语义失真**：「crate 写完 + 单测绿」被记为「已完成」，而产品意义上的完成 (被服务消费、部署、端到端验收) 没有发生。项目最大的风险不是某个 bug，而是**账面进度与可运行产品之间的差距没有被任何一份现行文档量化**。本报告 §2 给出这个差距的精确测量。

---

## §1 实测数字基线 (与 README/WBS 对账)

| 指标 | README/WBS 声明 | 2026-11-15 实测 | 复现命令 |
|---|---|---|---|
| workspace crate | 105 (README, 2026-10-02 审计) | **118** workspace package + bff 独立 workspace + platform-messaging-proto (excluded) | `cargo metadata --no-deps --format-version 1` |
| Rust 源文件 | 736 (README) | **949** | `find crates -name '*.rs' \| wc -l` |
| Rust 代码行 | — | **354,501** (crates/) | `find crates -name '*.rs' -exec cat {} + \| wc -l` |
| 前端源文件 | 496 (README) | **594** (341 tsx + 253 ts), **108,431** 行 | `find frontend/src -name '*.tsx' -o -name '*.ts'` |
| 设计文档 | 1392 (README) | **2217** | `find docs -name '*.md' \| wc -l` |
| main HEAD | `bd97197f` (PR #271) (README) | **`caa0ca7e`** (PR #303)，README 落后 32+ PR | `git log origin/main -1` |
| Rust 测试 | 「756 tests」(守门 v12, 41 crate 时代) | **4,734** 个 `#[test]/#[tokio::test]`；另有 **65** 个 `#[ignore]` (多为需 PostgreSQL) | `grep -rc '#\[test\]\|#\[tokio::test\]' crates` |
| helm chart bug | README §5 风险#1「高」仍标 open | Chart.yaml description 已改为 "Deprecated Star Helm placeholder" —— 用弃用修复了 bug，但 README 风险表未同步 | `head -3 deploy/helm/star/Chart.yaml` |
| License & Advisory Gate | — | **dev 每次 push 都红**: `unsafe-libyaml 0.2.11` unmaintained (RUSTSEC-2024-0449)，根因 `crates/sandboxd` 依赖已弃用的 `serde_yaml 0.9` | `gh run view 43717656049 --log-failed` |

README 的「投资人 90 秒版」数字全部停在 2026-10-02，且 §5 风险表与仓库现状脱节 (风险#1 已处置未销账)。README §4 对 k3s 不可启动的**完整披露是正面典范**——问题只在于这种诚实没有贯彻到进度数字上。

---

## §2 P0 — 结构性发现：40 个孤儿 lib crate (79,929 行, 占 crates/ 代码 22.5%)

### 2.1 测量方法

对 `cargo metadata` resolve 图做反向遍历：一个 crate 是「孤儿」当且仅当 (a) 不是二进制/不是 *-wasm (它们是被部署/被前端直接消费的合法叶子)，且 (b) 没有任何 workspace 内 package、也没有 bff 依赖它。结果 (完整清单按 LOC 降序)：

| 孤儿 crate (前 15) | LOC | 备注 |
|---|---|---|
| domain-local-runtime | 12,554 | **CI 有专属 job (含 e2e-integration serial 跑)**，但没有任何服务暴露它 |
| run-owner-runtime | 10,027 | owner/Run 核心语义，无消费者 |
| terminal-stack | 4,697 | 前端已有 `useTerminalStackWs.ts` + 2 个 e2e spec，但**没有服务端进程挂这个 crate** |
| domain-report | 4,435 | |
| star-arg-effect | 4,387 | |
| star-saga | 3,656 | WBS 5.5.1 标 100% |
| star-agent-runtime | 3,455 | WBS 6.2 标 90% |
| star-dispatcher | 3,239 | |
| domain-audit | 2,534 | 审计域未接线 (api-rest 的 audit 走 domain-* 其他路径) |
| domain-collaboration | 2,140 | |
| terminal-protocol | 2,075 | |
| domain-planning | 2,056 | |
| star-taskqueue | 1,915 | WBS 5.5.1 标 100% |
| domain-workflow | 1,849 | 与 star-workflow (762, 同孤儿) **双双无人消费** |
| domain-relation | 1,813 | |

其余 25 个：domain-development, star-eventbus, domain-context, application, domain-agent-windows, star-task, star-mutex, star-workflow, star-canvas, star-scheduler, star-sa, domain-form, star-cache, ai-vault, star-vcs, leads, domain-ai, star-context-tiering, star-memory, star-taskgraph, domain-dashboard, star-sse, domain-mcp, domain-task, worktree-canvas (facade, 96 行, 宣称「14 crate 单 import 入口」但 bff 直接依赖 graph-core/git-adapter/git-observer，无人走 facade)。

### 2.2 与 WBS「已完成」声明的冲突 (核心批评)

- **WBS 5.5.1**「Saga / EventBus / TaskQueue / Cache / SSE / Webhook / Quota / Telemetry 等基础服务 crate — 100% 已完成」：8 个中 **5 个 (saga/eventbus/taskqueue/cache/sse) 零消费者**，只有 star-webhook 进入 star-api-rest 依赖闭包。
- **WBS 5.1.1**「25 个 domain-* crate 骨架与领域逻辑 — 100% 已完成」：约半数 domain crate (local-runtime/report/audit/collaboration/planning/workflow/relation/development/context/form/ai/dashboard/mcp/task/agent-windows) 不在任何部署闭包内。
- **AGENTS.md 硬约束自己写了**：「仅有 UI、mock、seed、预览适配器或 REST 契约不得标为可用闭环」——同一条纪律应当适用于后端 crate：**仅有 crate + 单测，不得标为「已完成」**。WBS 目前的「已完成」不区分这两者。

### 2.3 为什么这是 P0 而不是「技术债」

1. **测试与覆盖率给了假信号**。4,734 个测试里相当一部分在孤儿 crate 内；tarpaulin 覆盖率把它们计入分母分子，「绿」的观感掩盖了「没接上」。守门 #19 v30 自己总结过这个模式：「报告绿 ≠ 跑过」「只比较最终状态的断言测不出代码没跑」——同样的逻辑适用于进度表。
2. **编译/CI 成本为死代码支付**。`cargo check/test --workspace` 每轮为 8 万行无人消费的代码花时间，Windows 上还需要 `-j 4` 规避资源耗尽 (守门 v19)——孤儿面越大，这个税越重。
3. **它直接推迟产品核心承诺**。AGENTS.md 的产品叙事核心是「Agent/Task CLI 闭环」「Lane/Worktree 本地执行」。而这条链路上的三大件 domain-local-runtime (spawn/kill 跨平台已 100%, WBS 5.5.5)、terminal-stack、run-owner-runtime 全部是孤儿——**能力存在，产品面不存在**。

### 2.4 建议 (P0)

- **WBS 增加「接线」列**：每个「已完成」条目必须给出消费者 (哪个二进制/哪个 route/哪个前端页面) 与验收证据；给不出的降级为「crate 就绪，未接线」。一次性对账成本约 1-2 天，之后每次升版沿用。
- **对 40 个孤儿逐个判决** (三选一)：(a) 接线——列入 W7/W9 等对应 WBS 条目；(b) 合并——见 §5 合并地图；(c) 退役——移入 `attic/` 或删除 (git 历史即归档)。判决本身可以按域批量做，不需要逐个重写代码。
- **优先接线三件套**：domain-local-runtime + terminal-stack + run-owner-runtime。它们是「Agent CLI 执行闭环」硬约束的物质基础，且前端 (useTerminalStackWs、任务卡 CLI 面板「输入禁用+未接入提示」) 已经在等它们。

---

## §3 P1 — 功能与工程发现

### 3.1 dev 分支 License Gate 常红 (可 1 小时内修复)

`serde_yaml 0.9` (deprecated) → `unsafe-libyaml` (RUSTSEC-2024-0449 unmaintained)，唯一引用方是 `crates/sandboxd/Cargo.toml:44`。deny.toml 把 unmaintained 设为阻断，于是 dev 每次 push 该 job 必红 (run 43717656049 实证)。门禁常红比没有门禁更糟——它训练所有人忽略红色。修法：sandboxd 迁 `serde_yaml_ng` (API 兼容) 或 `serde_yml`；或者如果短期不迁，在 deny.toml 给该 advisory 加带到期日的豁免。**不建议**长期豁免。

### 3.2 E2E 与 PostgreSQL 验收不在持续门禁里

- frontend 有 26 个 playwright spec，但 `ci.yml` 的 frontend-ci 只跑 typecheck/vitest/build；playwright 只在 `p1e-integration.yml` 跑，而其触发条件是**特定 agent 分支 push + PR 改 terminal-stack 相关路径**。也就是说绝大多数 PR 不会跑任何浏览器 e2e。
- 65 个 `#[ignore]` 测试大多要求真实 PostgreSQL (device outbox、advisory-lock 并发、schedule ACL fixture——全是资金级/一致性级路径)。CI 没有 PG service container job，这些路径的回归靠人工 (Windows 上还需专用 harness)。
- 建议：(a) 从 26 个 spec 里挑 5-8 个冒烟级进 ci.yml (chromium only, 约 4 min)；(b) 加一个 `postgres:16` service container job 跑迁移 + 全部 ignored PG 测试，nightly 或 PR 标签触发。db/migrations 现有 122 个脚本、三种命名风格 (`001_`, `2026-09-08-`, 混合)，先统一为单列有序 + 一个 runner (sqlx migrate 或 refinery)，否则 PG job 无从自动搭库。

### 3.3 前端 mock-first 的「real 接线」缺口没有产品面兜底

`frontend/src/mocks/real-mode.ts` 头注释自认：「仅 cli.ts 一个 handler 改, 其他 handler 留 TODO」「agents/analytics/inbox 暂未 real-mode 化」。实测 19 个 handler 域中 **7 个**有 realFetch 短路 (audit/cli/design-artifacts/incidents/validation/worktree-shared/worktrees)，**12 个**没有 (agents/analytics/annotations/billing/comments/graph/inbox/kms/providers-llm/tenants/workspaces/worktree-canvas)。production build 关闭 MSW 后，这些面的数据来源取决于对应 REST 是否已在 star-api-rest/bff 实现并部署——而部署面本身 (见 3.4) 只有 canvas 3 服务是维护中的。AGENTS.md 硬约束「未接通授权数据的功能须显示未接入状态，不得以示例数据冒充」应当落实为**每个未接线域的显式未接入态**，而不是依赖 mock 关不关。

### 3.4 部署栈四轨并行，主产品 API 无维护中的发布路径

现状：`deploy/k3s-local` (kustomize, README 自证 9/14 后未复现) + `deploy/helm` (已弃用占位) + `deploy/gitops` + `deploy/multipass` + `star-canvas-services-k3s.yaml`。CI 的 publish workflow **只发布 canvas-engine / canvas-realtime / domain-canvas 三个镜像** (带 digest+provenance 校验，这条流水线本身是亮点)。star-api-rest——42K 行、429 个测试的主产品 API——和 bff 没有任何维护中的镜像构建路径，只有 k3s-local 脚本 (未验证)。建议：弃用的 helm chart 整目录删除 (占位 chart 会被误认为可用)；把 publish workflow 的 allowlist 构建模式复制到 star-api-rest + bff，这是现成模板。

### 3.5 双 i18n、双锁文件、demo 路由混入产品面

- `frontend/src/i18n/` (charts, 仅 zh-CN, 15 文件引用) 与 `frontend/src/lib/i18n/` (en/zh-CN/ja 三语, 116 文件引用) 并存。前者应并入后者的 dictionary 体系，否则「三语 i18n 全覆盖 (WBS 8.1.6 已 100%)」在 charts 域不成立。
- `package-lock.json` 与 `pnpm-lock.yaml` 同时被 track (同日各自更新过)。CI 用 `npm ci`。pnpm-lock 是漂移陷阱，应删除并在 package.json 加 `packageManager` 字段钉死 npm。
- app/ 下 wasm-demo、terminal-stack-demo、chrono-vibe、automation-debug、refactor 等 demo/调试路由与产品路由平级。建议归入 `(dev)/` route group 并用 env flag 门禁，产品导航硬约束 (Project→Lane→Worktree→Branch) 不应与 demo 面混杂。

---

## §4 P2 — 文档与仓库卫生

1. **AGENTS.md 351 KB / 1102 行**，其中 122 行修订历史、守门规则多次重号 (文件自己承认 v25 重号、v27-v30 两套编号并存)、已作废政策 (v30/v31) 原文内联保留、守门 #6「PowerShell only」与当前 bash 工具链矛盾。新 agent 的入坑成本主要花在这里。建议拆三层：**硬约束正文 (≤10 页, 现在时)** + `docs/governance/ledger.md` (修订历史与已作废政策, 只增不改, 满足禁回溯) + 各主题 runbook (k3s 运维累积规等)。守门 #6 按当前 runtime 实况改写或删除。
2. **README 数字与风险表过期** (§1 表)。README 是「投资人 90 秒版」，过期数字直接损伤其唯一用途。建议：数字段改为脚本生成 (计数命令 README 自己都列了) 或至少每次 merge train 后刷新；§5 风险表加「状态/销账」列。
3. **CHANGELOG.md 自 2026-08-30 起是占位**，且内容是 RGS 游戏 5 域 (player/economy/match/social/admin) 叙事，放在 Star 根目录会让任何新读者困惑两个项目的关系。要么写真实的 Star changelog，要么删除并在 docs/INDEX.md 说明历史归属。
4. **ADR 三处重复**：`docs/adr/0021-0028` 与 `docs/architecture/2026-08-26-upgrade/adr/` 同文件双份，`docs/wiki/pgwiki/30-architecture/adr/` 又一份；README 说 28 篇，`find` 到 81 个 adr 路径下的 md。建议单一 canonical 目录 + 其他位置只留链接存根。`docs/plan` vs `docs/plans` vs `docs/implementation-plans`、`docs/rfc` vs `docs/rfcs` 同理 (INDEX.md 已标 archive 的，物理归档掉)。
5. **仓库垃圾被 track**：根目录空文件 `0`、`.coverage` (53KB 二进制)、`demo/server.err`、`deliverables/kanban-vmodel-jp/server.log.err`。删除并补 .gitignore 条目。
6. **scripts/ 88 个 py 中 25 个是 `p0_1_*` 一次性迁移脚本** (含 `p0_1_ENDGAME.py`、`p0_1_LAST.py`、`p0_1_real_last.py`、`p0_1_final_attempt.py`)。它们是 P0-1 审计的历史证据，但混在活动脚本目录里。建议整体移入 `scripts/archive/p0-1/` 并在 automation-design.md 注记位置——证据保留，噪音出清。
7. **markdownlint 关掉了约 30 条规则** (.markdownlint-cli2.jsonc)，门禁接近无牙。可以接受 (存量太大)，但应写明「只防新增」的 ratchet 策略，否则该 job 只产生虚假安心。

---

## §5 可合并功能地图 (issue 要求项)

| # | 合并对象 | 现状 | 建议 |
|---|---|---|---|
| 1 | `event-bus` (1523) + `star-eventbus` (1714) | 两套 EventBus trait + 实现，都是孤儿 (event-bus 仅被孤儿 worktree-canvas facade 引用)；star-eventbus 还带 Mailbox 3 模式 | 保留 star-eventbus 设计 (Valkey backend + Mailbox)，吸收 event-bus 的 15 事件类型与 4 消费者语义，成为唯一 bus；或整体让位 platform-messaging-runtime (已接线, 11K LOC) |
| 2 | `star-saga` + `star-dispatcher` + `star-taskqueue` + `star-scheduler` + `star-task` + `star-taskgraph` + `domain-task` (212) + `shared-task` (242) | 8 个任务/编排 crate，7 个孤儿；实际在跑的调度语义写在 star-api-rest `group_api/schedule_rules.rs` (20K 行 group_api 的一部分) | 这是全仓最大冗余簇。先判决 schedule_rules 与 star-scheduler/star-taskqueue 谁是正统，再合并为一层「编排」+ 一层「任务域模型」；domain-task/shared-task 并入 domain-work-item (3758, 已接线) |
| 3 | `star-workflow` (762) + `domain-workflow` (1849) | 双双孤儿 | 二合一后接 star-api-rest workflow 路由，或明确 workflow 语义由 group_api + SRS-WORKFLOW-TEMPLATE 承载后退役两者 |
| 4 | `star-vcs` (536) | 孤儿；git 能力已由 git-adapter + star-git-core + git-observer (全部已接线) 覆盖 | 退役或并入 star-git-core |
| 5 | `terminal-protocol` (2075) + `terminal-stack` (4697) | 协议与栈分离但都无人消费；前端 e2e 已就绪 | 合一，并作为 §2.4 三件套接线的一部分挂到 bff (WS) |
| 6 | `star-cache` + `star-memory` + `star-context-tiering` (1509 合计) | 全孤儿；star-context (949, 39 个依赖方) 是事实上的共享核心 | 把仍有价值的语义 (tiering 策略、valkey 选型) 并入 star-context 家族，其余退役 |
| 7 | `worktree-canvas` facade (96) | 「单 import 入口」无人使用；bff 直接依赖 3 个子 crate | 要么让 bff/api-rest 改走 facade，要么删 facade——两者取一，不要保留「为统一而统一」的空壳 |
| 8 | `api` + `application` + `infrastructure` 三层 | application 仍孤儿 (P0-1 审计 2026-08-31 就点名过三者 0 引用；现在 api 被 bff 用、infrastructure 被 api-rest 用，application 依旧) | application 层要么承载 §5#2 合并后的编排职责 (给它真实用途)，要么退役 |
| 9 | frontend 双 i18n (§3.5) | | src/i18n 并入 lib/i18n |
| 10 | deploy 四轨 (§3.4) | | kustomize (k3s-local) + multipass 二选一为主轨，helm 删除 |

---

## §6 缺失待补全 (issue 要求项)

按产品承诺 (AGENTS.md 硬约束 + SRS) 与现状的差集排序：

1. **本地执行闭环的服务面** (§2.4)：domain-local-runtime/terminal-stack/run-owner-runtime 的宿主进程与 REST/WS 端点。这是「W7 Agent CLI 执行闭环」从 crate 变成产品的最后一公里。
2. **star-api-rest / bff 的发布路径** (§3.4)：复用 canvas publish workflow 的 allowlist+digest 模式。
3. **PG 持续验收** (§3.2)：迁移 runner + CI service container + 65 个 ignored 测试常态化。
4. **e2e 冒烟门禁** (§3.2)：5-8 个 spec 进 PR 门禁。
5. **未接线域的「未接入」显式态** (§3.3)：12 个 mock 域逐个标注或接线，兑现「不得以示例数据冒充」。
6. **License gate 修复** (§3.1)：serde_yaml 迁移。
7. **W15 (商业发布验收) 与 W16-D (草稿冲突闭环)**：WBS 标 0% 未开始，而 W16 的实时协作硬约束 (冲突 p99 ≤1s、resource-fence seal/pin) 是 AGENTS.md 里最重的产品承诺之一——建议要么排期，要么在需求文档降级并记录拍板，避免硬约束与路线图脱节。
8. **inbox/notification 的真实投递验收**：SRS-INBOX 存在、domain-notification 已接线，但前端 inbox handler 无 real 短路——端到端 (事件→通知→inbox 可读) 缺一次贯通验收。
9. **README/WBS 数字自动化** (§4.2)：计数脚本化，杜绝「审计一次、过期半年」。
10. **仓库卫生包** (§4.5/4.6)：垃圾文件、一次性脚本归档、锁文件单一化。

---

## §7 应当保留的优点 (客观性要求)

1. **诚实披露文化**：README §4「不对 k3s 可启动性作任何保证」+ 探针表、review 文档的「ACCEPT WITH LIMITATIONS」判词、「缺标比错标」原则——这是多数团队做不到的。
2. **CI 工程化**：actions 全部 SHA 固定、changes 检测 fail-open、ci-gate 单检查名聚合、license gate 每日 cron、SPDX/SBOM 证据、canvas 发布的 digest+provenance 独立校验。
3. **测试体量与守门演进**：4,734 个 Rust 测试、127 个前端测试文件、`unsafe_code = "forbid"` workspace 级、守门 v30「门禁必须有变异测试」这类元认知规则——方向完全正确，本报告 §2 正是把同一把尺子 («绿 ≠ 跑过») 用到进度表上。
4. **文档追溯体系**：SRS→BD→DD 的 IPA 风格链路 + 决策表 + 修订历史，单人项目里维持这个纪律罕见。问题在体量与重复 (§4)，不在体系本身。

---

## §8 Limitations (本评估的边界)

- 未启动任何服务做运行时验证 (本机无 k3s/docker，与 README §4 披露一致)；「孤儿」判定基于静态依赖图，不排除仓外消费者 (未发现任何证据)。
- 未跑 `cargo test --workspace` 全量 (Windows 资源约束 + 与 CI 重复)；测试计数是 attribute 静态计数。
- 前端未做像素级/浏览器验证；mock/real 接线判定基于源码 grep 与 real-mode.ts 自述。
- dev 与 origin/main 相差 3 个 frontend commit，结论对两者均成立；如 dev 后续合入新接线，§2 清单应按同法复测 (命令已给出，可脚本化)。

## §9 修订历史

| ver | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-15 | Multica agent (qwen3-8, on behalf of Ulysses) | 初版：ULYS-285 批评与建议 | ULYS-285 |
