# STAR-RF-001 — 代码健康度重构 Spec (T1/T2/T3 三档)

**Status**: 🟡 Draft (待 Ulysses review)
**Created**: 2026-09-02
**Authority**: Ulysses（一人公司 12 角色 per DEC-008）
**起草**: 本 session AI (Claude Code, per 2026-09-02 用户发令"制定完整的重构 spec 和实施计划")
**For**: 下游 AI (任意 worker/子代理/新 session) 执行代码健康度重构时的范围与验收依据
**Pair with**: `docs/refactor/WBS-001-refactor.md` (任务分解 + token 估算) / `docs/refactor/HANDOFF-001.md` (下游 AI 执行清单)

---

## 0. 背景

2026-09-02 JST, Ulysses 与本 session AI 讨论"本项目可以从哪些角度重构", 经过多轮收敛 (设计模式/算法/原子化解耦/去重/数据优化/提取方法/可读性/软件工程学 + AI 补充的 panic 安全/所有权优化/可观测性/lint 严格度/死代码清理/测试架构/多协议去重/事务边界/命名一致性/编译时间/前端状态管理), 拍板"制定完整重构 spec 和实施计划, 并更新 handoff 要求下游 AI 执行"。

**范围决策 (per 2026-09-02 讨论)**: 按风险分 3 档 (T1 机械级 / T2 行为保持 / T3 需拍板), 三件套文档结构照搬 `docs/batch/{spec, WBS-001-domain-batch.md, HANDOFF-001.md}` 既有模式。

### 0.1 现状实测 (2026-09-02, 本 session, 前置于本 spec 的证据)

| 检查项 | 命令 | 结果 |
|---|---|---|
| lib 编译 | `cargo check --workspace --lib` | 0 err (跟 AGENTS.md v0.35 最新记录一致) |
| all-targets 编译 | `cargo check --workspace --all-targets` | **~460 err, 跨 11 crate** (domain-feedback 77 / domain-integration 76 / domain-validation 66 / domain-development 63 / domain-workflow 54 / domain-worktree 51 / domain-notification 45 / star-mcp 26 / domain-relation 4 / api 1) |
| 根因 | — | `TenantId`/`UserId` 强类型 vs `Uuid` 的 ActorContext 类型不统一, 属于 `HANDOFF-ST-001.md` H2/H2-EXT 已跟踪工作, 卡在等 Ulysses 对 domain-work-item 等的类型语义拍板 |
| unwrap 计数 | `grep -ro "\.unwrap()" crates --include=*.rs \| wc -l` | 1339 处 |
| clone 计数 | `grep -ro "\.clone()" crates --include=*.rs \| wc -l` | 730 处 |
| tracing 埋点覆盖 | `grep -rl "tracing::" crates --include=*.rs \| wc -l` / 总 rs 文件数 | 7 / 288 文件 |
| 根目录报告文件 | `git ls-files \| grep -E "^(PHASE\|STAR\|QA\|HANDOFF\|REQUIREMENTS\|DDD)"` | 98 个已跟踪 md |
| 根目录脚本/日志散件 | `git ls-files \| grep -E "^_"` | 14 个已跟踪文件 (`_c2.sh` `_c3.sh` `_c4.sh` `_c5.sh` `_commit_fix_jql.sh` `_fix_jql.py` `_fix_viz.py` `_msg_audit.txt` `_msg_automation.txt` `_msg_scm.txt` `_wt_audit/*` 4 个) |
| 孤儿 crate | `Cargo.toml` workspace.members 对照 `crates/` 目录 | `crates/star-vcs` 存在磁盘但未列入 workspace.members, 不参与编译/测试 |
| 最大 3 个 domain crate | `find crates/domain-*/src -name "*.rs" \| xargs wc -l` | domain-local-runtime 5182 / domain-integration 3962 / domain-cli 3683 行 |
| lint 严格度 | 根 `Cargo.toml` `[workspace.lints.rust]` | `missing_docs`/`rust_2018_idioms`/`unreachable_pub` 均为 `"warn"`, 非 `"deny"` |

---

## 1. 目标与非目标

### 1.1 目标
1. 降低仓库认知负担 (根目录治理), 不影响任何运行时行为
2. 消除已确认的死代码/未注册资产 (`star-vcs` 孤儿 crate)
3. 提升代码健壮性 (panic 安全、所有权效率), 保持外部行为不变
4. 补齐可观测性缺口, 为后续故障排查/审计打基础
5. 给 T3 (需人工决策) 条目**提交选项报告**, 不擅自替 Ulysses 做架构决策

### 1.2 非目标 (显式排除, 避免与现有 HANDOFF 冲突)
- **不做** H2/H2-EXT ActorContext (`TenantId`/`UserId` vs `Uuid`) 统一 — 已由 `HANDOFF-ST-001.md` §1/§5/§8 跟踪, 卡在等 Ulysses 拍板 `domain-work-item`/`domain-identity` 类型语义, 本 spec 不重复/不越权处理
- **不做** `docs/batch` (domain-batch) 相关工作 — 独立 WBS 已存在
- **不做**"事件驱动 app 集群 + 上传界面"新子系统 — 该子项目单独立 spec `docs/specs/domain-app-spec.md`(2026-09-03 起草, Q1-Q4 拍板记录见该文档状态头), 依赖本 spec 先完成 T1 打底(该 spec 编写不受阻塞, 仅其 T 任务分解受阻塞)
- **不新增业务功能** — 本 spec 全部条目是重构, 禁止顺手加 feature

---

## 2. 范围 (T1/T2/T3)

### T1 — 机械级/零行为风险 (下游 AI 可直接执行, 无需等拍板)

| # | 条目 | 现状证据 | 目标 | 验收标准 |
|---|---|---|---|---|
| T1.1 | 根目录报告文件归档 | 98 个 `PHASE-*/STAR-*/QA-*/HANDOFF-*/DDD-*` md 混在根目录 | 移动到 `docs/reports/`, 根目录只留 `README.md`/`CHANGELOG.md`/`AGENTS.md`/`LICENSE` 等入口文件; 所有内部相对链接同步更新 | `git mv` 后 `grep -rl "PHASE-.*-REPORT.md" --include=*.md .` 引用路径全部指向新位置; `cargo check --workspace --lib` 仍 0 err (纯文档移动不应影响编译) |
| T1.2 | 根目录散件清理 | 14 个已跟踪 `_*.sh/_*.py/_*.txt` + `_wt_audit/` | 有效脚本移入 `scripts/`, 一次性调试产物 (`_msg_*.txt`) 确认无引用后删除, `_wt_audit/` 若仍是活跃审计产物则移入 `docs/reports/wt-audit/` | `git status` 干净, 无孤立引用 |
| T1.3 | 孤儿 crate 处理 | `crates/star-vcs` 不在 `Cargo.toml` workspace.members | 先确认是否废弃 (查 git log / 内容完整度), 若是遗漏则补 `Cargo.toml` 注册 + `cargo check -p star-vcs`; 若确认废弃则删除目录 | `cargo metadata` 里 `star-vcs` 状态与 Cargo.toml 一致, 二者不再矛盾 |
| T1.4 | 死依赖清理 | 未跑过 `cargo machete`/`cargo udeps` | 跑一遍, 清理未使用依赖 | 工具输出 0 未使用依赖, 或每条保留项有注释说明原因 (如 feature-gated) |
| T1.5 | lint 严格度提升 | `missing_docs`/`rust_2018_idioms`/`unreachable_pub` = `warn` | 改 `deny`, 逐个 crate 修复触发项直到能改 (先修完再切 deny, 避免 CI 直接爆红) | `cargo clippy --workspace --all-targets` 0 warning (跟改动相关的), `cargo build --workspace` 通过 |

### T2 — 行为保持, 改动面大, 需测试兜底 (下游 AI 执行, 每步跑 gate)

| # | 条目 | 现状证据 | 目标 | 验收标准 |
|---|---|---|---|---|
| T2.1 | unwrap 收敛 | 1339 处 `.unwrap()` | 先处理**库代码** (非测试) 里的 unwrap: 能返回 `Result` 的改 `Result` + `thiserror`; 确定不可能失败的标注 `// SAFETY:` 注释说明; 测试代码里的 unwrap 保留 (测试失败即崩溃是合理行为) | `clippy::unwrap_used` 在非测试代码路径降到 0 (可用 `#[cfg(not(test))]` 范围lint 或按 crate 分批); 每个 crate 改完跑 `cargo test -p <crate>` 全过 |
| T2.2 | clone 审计 | 730 处 `.clone()` | 用 `clippy::redundant_clone` 找出可去除的克隆, 优先处理热路径 (被 `domain-local-runtime`/`domain-integration` 等高频调用的函数); 非必要不改无关 clone | 每个改动点前后跑对应 crate 单测确认行为不变; `clippy::redundant_clone` 命中数下降有 commit 记录 |
| T2.3 | 可观测性埋点 | `tracing::` 仅 7/288 文件 | 为跨 crate 边界调用 (port trait 实现、跨 domain 调用点) 补 `tracing::instrument` / `tracing::info!`/`warn!`/`error!`; 不要求覆盖内部私有函数 | 抽样检查 5 个高频跨域调用路径, 均能看到 span/event; `cargo build` 0 新增 warning |
| T2.4 | 大 crate 拆分评估 | `domain-local-runtime` 5182 / `domain-integration` 3962 / `domain-cli` 3683 行 | **先只做评估报告**(不强制拆): 逐个分析模块边界是否有明确子领域可拆, 若有则提拆分方案 (新 crate 名 + 迁移文件清单), 若无明确边界则说明原因保留现状 | 每个 crate 产出 1 份评估小节 (可并入本 spec 附录), 含"拆/不拆 + 理由" |

### T3 — 跨切面设计决策, 需 Ulysses 拍板 (先出选项报告, 不擅自动手)

| # | 条目 | 现状证据 | 需要拍板的问题 | 输出物 |
|---|---|---|---|---|
| T3.1 | 多协议 DTO 去重 | `star-api-rest`/`star-mcp`/`star-sse` 三个协议入口各自定义请求/响应结构, 初步 grep 命中率低但需人工复核确认重复面 | 是否新增共享 `star-dto`/`star-contract` crate 承载公共 DTO? 还是保持三层各自定义 (代价是改一处要改三处)? | 选项对比表 (方案 A: 共享 crate / 方案 B: 保持现状 + 加契约测试 / 方案 C: 仅共享 serde derive 宏), 各自代价与收益, 推荐项 |
| T3.2 | Saga 覆盖率审计 | `star-saga` 已存在, 但未核实所有跨域写操作是否都走它 | 哪些跨域写路径应该强制走 saga 但目前绕过了? 是否需要 lint/CI 规则强制? | 跨域写调用点清单 + 现状 (走/不走 saga) + 风险评级 |
| T3.3 | 领域统一语言审计 | 未系统检查, 只是怀疑 (DDD 项目常见腐化点) | 审计范围多大 (全 22 domain vs 抽样)? 发现冲突后是否要改代码 (有 API 兼容成本) 还是只记录? | 术语对照表 (概念 → 各 crate 用词), 冲突点清单 |

---

## 2.4 设计文档同步映射表 (per 2026-09-02 用户发令"相关设计书也要用重构后的设计更新", v0.3 改写: 补齐上游 V-model 详设 + 纠正两处已核实错误)

**总规则**: 任何 T1/T2/T3 任务如果改变了 crate 的职责边界、错误类型、公共接口签名、或架构决策, **必须在同一个 commit 里同步对应设计文档**, 不允许"代码先改, 文档以后再补" —— 对应本仓库既有的"守门 #12 commit-time docs 同步"惯例 (见 `AGENTS.md` 修订历史里每条 commit 都带的 §"文档同步"记录)。纯移动文件/清理死代码/不改签名的性能优化 (如多数 T2.2 clone 审计) 不强制要求文档变更, 但要在 commit message 里注明"无设计文档影响, 因为 XXX"。

**v0.2 → v0.3 纠错说明** (本次逐条核实全仓 `docs/` 树后发现):
1. ~~"star-vcs 不在 `basic-design.md` §2.1 的 25-Module 表里, 佐证它是废弃代码"~~ — **撤回**。该表只收录 `domain-*` Module (25 个), 从未覆盖任何 `star-*` 基础设施层 crate (含 star-vcs/star-cache/star-saga/star-mcp 等), 表里没有一条 star-* 的库存条目。star-vcs 缺席对"保留/删除"判断**不构成任何方向的证据**, T1.3 判定时不要引用这张表作依据。
2. ~~"T2.3 应改用 `docs/operation-design.md` 已有 tracing 章节, 不要新建 `observability.md`"~~ — **撤回**。核实后 `operation-design.md` §6.3 只讲**可观测性后端管线** (Tempo/Jaeger 选型、OTLP endpoint、`tracing_enabled` 配置项), 不涉及**代码级埋点覆盖率/span 命名规范**; 通读 `internal-design.md` 全文也没有对应章节。T2.3 新建 `docs/architecture/observability.md` 的结论不变, 但文中必须反向链接 `operation-design.md` §6.3 (两者分工不同, 避免看起来像重复权威源)。

**核实到的新发现** (下游 AI 执行 T1.3/T2.4/T3.2 前必读):
- `basic-design.md` §2.1 "25 个 Module" 清单本身已过期于本次重构之前: 当前 `Cargo.toml` `workspace.members` 实际有 **34 个 `domain-*` crate**, §2.1 表缺 9 个 (`domain-batch`/`domain-kms`/`domain-theme`/`domain-report`/`domain-dashboard`/`domain-form`/`domain-ai`/`domain-cli`/`domain-agent-windows`)。这个缺口**不属于本 spec 任何任务的必做范围** (与重构无关的历史遗留), 引用 §2.1 时只能当"曾经完整、现已部分过期"的参照, 不能当权威全集。
- **`docs/architecture/2026-08-26-upgrade/` 不是历史归档, 是当前活跃目录** —— 目录名带日期极易被误判为快照, 已核实为误判来源之一。实测: 该目录 `adr/0040-domain-batch.md` 最后提交于 2026-09-01 (比 `docs/adr/` 最新文件晚 6 天); `git log --all` 还能在未合并 worktree 里找到 `adr/0041-arch-agent-graph-viewer.md`, 说明 ADR 编号至少用到 0041+ 且仍在增长。`docs/adr/`(仅 0021-0025) 与 `docs/architecture/2026-08-26-upgrade/adr/`(0021-0040, 含前者 5 篇的重复副本) 是**同一条编号序列**, 后者是当前活跃位置。**任何时候新增 ADR 前必须重新执行 §2.4.2 检索步骤现场算编号, 不允许沿用本文档任何地方写过的数字** (哪怕是这次修正后写的, 写下来那一刻就可能过期)。
- Saga 的真正权威设计文档**不是** `docs/integration-design.md` (那里只有一行指针引用), 而是 `docs/architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md` (270 行, Saga 契约/协调角色/8 步编排的实际定义, `crates/star-saga` 的实施依据) + `docs/ddd/03-match-bc.md` (match 域 = workflow/saga bounded context 边界文档)。

### 2.4.1 文档权威分层 (下游 AI 判断"要不要改"的依据)

| 层级 | 目录/文件 | 处理方式 |
|---|---|---|
| **主权威 (改动即必须同步)** | `docs/{requirements,basic-design,api-design,data-design,security-design,runtime-design,external-design,internal-design,integration-design,test-design,operation-design}.md`; `docs/specs/domain-*-spec.md`; `docs/architecture/2026-08-26-upgrade/spec/**` (**当前活跃, 非归档**); `docs/architecture/2026-08-26-upgrade/adr/**` 与 `docs/adr/**` (同一编号序列); `docs/architecture/*.md` (非 upgrade 子目录下的独立架构文档) | 任务改了这些文档描述的职责边界/接口/决策, 必须同 commit 更新 |
| **附带引用 (只核实断言未被改错, 不主动扩写)** | `docs/ddd/*-bc.md`; `docs/rfcs/rfc-*.md` (多数 `Proposed`/`Accepted` 状态, 记录决策当时的理由); `docs/governance/**`; `docs/plan*/**`、`docs/briefs/**`、`docs/batch/**` | 任务落地后若这些文档里的具体断言 (文件名/行为描述) 因改动变成假的, 最小化修一处即可, 不必展开重写整份文档 |
| **历史记录 (不动)** | 已归档进 `docs/reports/` 的 `PHASE-*-REPORT`/`*-HANDOFF` 类文件; status 已是 `Superseded`/`Deprecated` 的 RFC/ADR | 这些是"当时发生了什么"的记录, 不因后续重构改写 |

### 2.4.2 检索流程 (每个 T1/T2/T3 任务动手前必跑, 替代任何静态清单)

```bash
# 1. 找出所有提及目标 crate/概念的文档 (含 2026-08-26-upgrade 目录, 它是活跃目录, 不要排除)
grep -rli "<crate 名或概念关键词>" docs --include=*.md

# 2. 对每个命中文件, 按 §2.4.1 分层表判断层级, 只对"主权威"层做同 commit 编辑
# 3. 对"附带引用"层, 读一遍确认没有断言被改动打脸, 有则最小化修正
# 4. 涉及新建 ADR 时, 现场重新数编号 (不要复用本文档任何位置写过的数字):
grep -rohE "ADR-[0-9]{4}" docs | sort -u | tail -1        # 当前 HEAD 文本引用过的最大号
git log --all --diff-filter=A --name-only -- "**/adr/00*.md" | grep -oE "[0-9]{4}" | sort -u | tail -1   # 含未合并分支/worktree 的最大号
# 取两者较大值 +1, 新文件放 docs/architecture/2026-08-26-upgrade/adr/ (当前活跃位置, 除非执行时该目录已合并回 docs/adr/, 需现场确认)
```

### 2.4.3 逐任务映射表 (v0.3 修正版)

| 任务 | 受影响设计文档 | 同步要求 |
|---|---|---|
| T1.3 (star-vcs 处理) | 判定"保留/废弃"**不参考** `basic-design.md` §2.1 (该表不收录 star-* crate, 见上); 若保留注册: 新建 `docs/specs/domain-vcs-spec.md`(骨架); 若删除: 新增 ADR, 文件名/编号按 §2.4.2 步骤 4 现场算, **不要**使用 "0026" (已被 `docs/architecture/2026-08-26-upgrade/adr/0026-star-ai-compat.md` 占用) | 二选一必做; ADR 编号现场算, 不沿用任何文档写死的数字 |
| T2.1 (unwrap 收敛) | 若新增/调整了某 crate 的 `thiserror` 错误变体, 同步该 crate 对应 `docs/specs/domain-*-spec.md` 的错误类型章节 (若该 spec 文件有此章节) | 仅当错误类型有实质变化时触发, 纯 unwrap→expect 文案改动不算 |
| T2.2 (clone 审计) | 一般不触发; 若某函数签名从取值改引用/`Arc`, 涉及公共 API 变化的, 同步对应 spec 的接口签名描述 | 按需, 大多数情况可在 commit message 注明"无设计文档影响" |
| T2.3 (tracing 埋点) | 新建 `docs/architecture/observability.md` (代码级埋点覆盖范围 + span/event 命名规范), 文中反向链接 `docs/operation-design.md` §6.3 (后端管线配置, 分工不同, 不要合并); 相关 crate 已有 `docs/architecture/<crate>.md` 的追加一节 | 必做 (T2.3 本身就是补可观测性, 不留档等于没做完) |
| T2.4 (大 crate 拆分评估) | 评估结论直接写入 `docs/architecture/<crate>.md` (`domain-local-runtime.md` 已存在则追加; `domain-integration`/`domain-cli` 若无对应文档则新建)。**附带提醒**: `basic-design.md` §2.1 已知过期缺 9 个 crate (含 domain-cli 本身不在表里), 评估报告里提一句这个已知缺口即可, **不要求**顺带修复整张表 (超出本任务范围) | 必做, 评估报告本身就是设计文档产出; 过期表格问题仅需提及不需修复 |
| T3.1 (DTO 去重) | 拍板前: 在报告里列出"若选 A/B/C 各自需要改哪些文档"; 拍板落地后: 同步 `docs/api-design.md` + 视方案新建/更新协议层 spec | 拍板前只列清单不动手, 拍板后必做 |
| T3.2 (Saga 覆盖率) | 拍板落地后**主要同步** `docs/architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md` (Saga 契约实际定义所在) + `docs/ddd/03-match-bc.md` (match 域边界文档); `docs/integration-design.md` 只有一行指针引用, 若拍板结论与其矛盾则一并修正, 不必展开 | 拍板后必做, 以 saga spec 为主、`integration-design.md` 为辅 |
| T3.3 (统一语言审计) | 拍板落地后同步各 `docs/specs/domain-*-spec.md` 术语描述, 或新建 `docs/ubiquitous-language.md` 作为跨 domain 词典 | 拍板后必做, 词典建议独立新文件, 不塞进单个 domain spec |

---

## 3. 风险与回滚

- T1 全部是文件移动/元数据变更/新增 lint, **无运行时行为变更**; 每步 `git commit` 独立, 出问题可单独 `git revert` 该 commit
- T2 有代码语义变更风险, 强制要求"改一个 crate → 跑该 crate 测试 → commit → 下一个", 不允许攒一大批改动再测
- T3 不动代码, 纯分析产出, 无风险
- 任何 T1/T2 步骤如果发现改动面超出预估 (类似 HANDOFF-ST-001 H2 从 0.3-0.5M 暴涨到 1.1-1.6M 的先例), 立即停止、记录实测数据到 WBS, 不要在预算失控的情况下硬着头皮做完

---

## 4. 与现有 HANDOFF-ST-001 (H2/H2-EXT) 的边界声明

本 spec 的 T2.1 (unwrap 收敛) 可能触及 `domain-feedback`/`domain-integration`/`domain-validation` 等同样是 H2/H2-EXT 覆盖范围的 crate。**规则**: 如果某处 unwrap 修复需要动到 `ActorContext` 相关类型转换代码, 视为 H2/H2-EXT 范围, 跳过并在 WBS 里标注"阻塞于 H2/H2-EXT", 不在本 spec 下顺带解决类型统一问题 (避免两条 handoff 互相踩)。

---

## 5. 验收总标准 (对应 `docs/refactor/HANDOFF-001.md` §5 守门)

- `cargo check --workspace --lib` 0 err (不允许退化, T1/T2 完成后必须仍是 0 err)
- `cargo check --workspace --all-targets` err 数不劣化 (基线 ~460, 若因 H2/H2-EXT 之外原因新增 err 需当场修)
- `cargo clippy --workspace --lib` 0 warning (T1.5 完成后的新基线)
- `cargo fmt --all --check` exit 0
- `cargo test --workspace --lib` 全过 (数量不少于改动前)
- author = Ulysses (per 代签规则), 每个 commit message 标明对应 WBS 任务号 (如 `T1.1`)

---

## 6. 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-09-02 | 本 session AI | 初版: 4 轮讨论收敛后的 T1/T2/T3 三档 spec, 排除 H2/H2-EXT + app 集群子项目 (另立 spec), 现状实测数据 (all-targets ~460 err / unwrap 1339 / clone 730 / tracing 7/288 / 根目录 98+14 散件 / star-vcs 孤儿 crate) |
| v0.2 | 2026-09-02 | 本 session AI | 加 §2.4 设计文档同步映射表 (per 用户发令"相关设计书也要用重构后的设计更新"): 每个 T1/T2/T3 任务标注受影响的 `docs/specs/`/`docs/architecture/`/`docs/adr/` 文档 + 同步要求, 总规则对齐既有"守门 #12 commit-time docs 同步"惯例 |
| v0.3 | 2026-09-02 | 本 session AI | 逐条核实全仓 `docs/` 树后改写 §2.4: 补齐 v0.2 遗漏的 11 份 V-model 上游详设文档 (`requirements.md` 等); 撤回两处已证伪的 v0.2 结论 (star-vcs 缺席 §2.1 表不构成证据 / T2.3 该新建 observability.md 而非改用 operation-design.md); 新增 §2.4.1 文档权威分层 + §2.4.2 检索流程 (替代静态清单, 可重复执行); 修正 T1.3 ADR 编号硬编码 bug ("0026" 实际已被占用, 且 `docs/architecture/2026-08-26-upgrade/` 核实为当前活跃目录而非归档, ADR 编号序列跨两个目录且仍在增长); T3.2 同步目标从 `integration-design.md` 改为实际权威的 saga spec + match-bc 文档; 记录 `basic-design.md` §2.1 已知过期 9 个 crate 的既有缺口 (不纳入本 spec 修复范围) |
