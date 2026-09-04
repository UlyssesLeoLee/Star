# HANDOFF-RF-001 — 代码健康度重构 下游 AI 执行清单

> **来源**: `docs/refactor/RF-001-spec.md` + `docs/refactor/WBS-001-refactor.md` (2026-09-02)
> **目的**: 把重构范围里"下游 AI 可直接执行"和"必须等 Ulysses 拍板"的部分分开, 避免下游 AI 把决策类问题当工作项直接动手
> **触发**: 2026-09-02 用户发令"制定完整的重构 spec 和实施计划, 并更新 handoff 要求下游 AI 进行重构"

---

## §0 开工前必读 (下游 AI 进入本任务时第一步)

1. 读 `docs/refactor/RF-001-spec.md` §1.2 非目标 —— **本任务不碰 H2/H2-EXT ActorContext 统一**, 那是 `..\..\reports\HANDOFF-ST-001.md` 的范围, 两份 handoff 不要互相踩
2. 读 `docs/refactor/WBS-001-refactor.md` §0 总览表, 认领任务前先看依赖关系 (§4 执行顺序)
3. 重新实测 `cargo check --workspace --all-targets` err 数, **不得沿用本文档写的 ~460 这个数字** (数字有时效性, 参考 `..\..\reports\HANDOFF-ST-001.md` Q9-T/A9 先例)
4. 每完成一个 WBS 任务号, 回本文件对应 §1/§2 条目打勾 + commit message 标注任务号 (如 `refactor(T1.3): 注册 star-vcs 到 workspace members`)

---

## §1 下游 AI 可直接执行 (T1 全部 5 项, 无需等拍板)

### RF-T1.1 — 根目录报告文件归档
98 个 `PHASE-*/STAR-*/QA-*/HANDOFF-ST-001/DDD-LEAD-REVIEW-PROCESS/REQUIREMENTS-THREAD-C-HANDOFF` md 文件混在根目录。
**动作**: 按 `docs/refactor/WBS-001-refactor.md` §1 T1.1 步骤, `git mv` 到 `docs/reports/`, 同步全部内部相对链接。`..\..\reports\HANDOFF-ST-001.md` 移动前先确认它没有正在被别的活跃 session 引用中 (检查是否有 `.worktrees/*/..\..\reports\HANDOFF-ST-001.md` 处于未合并状态), 若有则本条先跳过, 待那些 worktree 合并/清理后再做。

### RF-T1.2 — 根目录散件清理
14 个已跟踪 `_*.sh/_*.py/_*.txt` + `_wt_audit/`。
**动作**: 按 WBS §1 T1.2 步骤逐个确认引用后移动/删除。

### RF-T1.3 — `star-vcs` 孤儿 crate 处理
`crates/star-vcs` 存在磁盘但未列入 `Cargo.toml` `workspace.members`。
**动作**: 按 WBS §1 T1.3 步骤判定"遗漏注册"还是"废弃", 二选一落地。**这是 T1 里最该先做的一项** —— 其判定结果 (crate 是否存在/是否编译) 会影响后续 T1.5 lint 全量检查范围。
**文档同步 (必做, 同一 commit)**: 注册 → 新建 `docs/specs/domain-vcs-spec.md`(骨架); 删除 → 新建 ADR, 编号**不要用 0026** (已被 `docs/architecture/2026-08-26-upgrade/adr/0026-star-ai-compat.md` 占用), 按 `docs/refactor/RF-001-spec.md` §2.4.2 检索流程现场算编号, 新文件放 `docs/architecture/2026-08-26-upgrade/adr/`(当前活跃目录, 非 `docs/adr/`)。判定"保留/废弃"**不要**参考 `basic-design.md` §2.1 的 25-Module 表——该表只收录 `domain-*`, 从不覆盖 `star-*` crate, star-vcs 缺席对判断不构成任何方向的证据。

### RF-T1.4 — 死依赖清理
**动作**: 按 WBS §1 T1.4 步骤跑 `cargo machete`, 逐条核实清理。

### RF-T1.5 — lint `warn` → `deny`
`missing_docs`/`rust_2018_idioms`/`unreachable_pub` 目前是 `warn`。
**动作**: 按 WBS §1 T1.5 步骤, 本地改完全部触发点后一次性 commit 切 `deny`, 不要中间态推送。

---

## §2 下游 AI 执行, 但每步必须过 gate (T2 全部 4 项)

### RF-T2.1 — unwrap 收敛
**先做**: 按 crate 精确统计 (库代码 vs 测试代码), 更新 WBS §0 token 估算区间为实测值, 再动手改。
**规则**: 落在 H2/H2-EXT 8 个 domain (per `..\..\reports\HANDOFF-ST-001.md` §1 H2 表) 里、且 unwrap 在 ActorContext 类型转换代码路径上的, 标记"阻塞于 H2/H2-EXT"跳过, 不要顺手改。
**gate**: 每个 crate 改完 `cargo test -p <crate> --lib` 全过才能 commit。

### RF-T2.2 — clone 审计
**gate**: 每处改动前后跑对应 crate 单测确认行为不变。

### RF-T2.3 — tracing 埋点补齐
**gate**: `cargo build --workspace` 0 新增 warning。
**文档同步 (必做, 同一 commit)**: 新建 `docs/architecture/observability.md`(埋点范围 + 命名规范), 这是本任务的交付物之一, 不是可选项。文中反向链接 `docs/operation-design.md` §6.3——那是可观测性后端管线 (Tempo/Jaeger 配置), 与本文档的代码级埋点规范分工不同, 不要合并/不要重复权威源。

### RF-T2.4 — 大 crate 拆分评估
**注意**: 本项**只产出评估报告, 不动代码**。若评估结论是"应该拆", 不要在本任务里顺手拆, 拆分是否执行需要 Ulysses 单独拍板 (涉及对外接口路径变化, 升级为 T3 性质)。
**文档同步 (必做)**: 评估结论写入 `docs/architecture/domain-local-runtime.md`(追加) + `domain-integration`/`domain-cli` 对应架构文档(新建), 不能只写在 commit message 里。评估报告里顺带提一句 `basic-design.md` §2.1 的 25-Module 表已知过期 (实缺 9 个 crate, 含 `domain-cli` 本身), 但**不要求**顺带修复整张表 (超出本任务范围)。

---

## §3 需 Ulysses 拍板 (T3 全部 3 项, 先出报告不擅自动手)

### RF-T3.1 — 多协议 DTO 去重
**产出**: 方案 A (共享 `star-dto` crate) / B (保持现状 + 契约测试) / C (仅共享 serde 宏) 对比表 + 推荐项, **报告里必须列出每个方案拍板后要改哪些设计文档** (`docs/api-design.md` + 协议层 spec)。**不要**在拍板前新建任何共享 crate 或改动 3 个协议 crate 的现有结构体, 也不要提前改文档。

### RF-T3.2 — Saga 覆盖率审计
**产出**: 跨域写调用点清单 + 是否走 saga + 风险等级, 审计对照的权威口径是 `docs/architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md`(Saga 契约/8 步编排实际定义, **不是** `docs/integration-design.md`——那里只有一行指针引用), 一并核对 `docs/ddd/03-match-bc.md` 是否一致; 报告里列出拍板后需要同步的具体章节。**不要**在拍板前强制改任何调用路径去走 saga。

### RF-T3.3 — 领域统一语言审计
**产出**: 术语对照表 + 冲突点清单, 报告里注明拍板后建议新建 `docs/ubiquitous-language.md` 作为跨 domain 词典。**不要**在拍板前重命名任何现有类型/字段 (属于破坏性 API 变更)。

---

## §4 边界声明 (与 `..\..\reports\HANDOFF-ST-001.md` 的关系)

- 本 handoff (RF-001) 与 `..\..\reports\HANDOFF-ST-001.md` (ST-001, H2/H2-EXT ActorContext 统一) 是**两条独立任务线**, 可能在同一 session 里都有下游 AI 在跑, 注意不要在同一次 commit 里混改两边范围的文件
- 如果执行 RF-T2.1 (unwrap 收敛) 时发现某处修复**必须**先解决 ActorContext 类型统一才能继续, 停止该处改动, 在本文件 §5 记录阻塞点, 转而处理 WBS 里其他不冲突的任务
- 两条 handoff 都完成 T1 级别工作后, 建议下一次 session 再评估是否合并成一份统一的 `HANDOFF-ST-002.md` (避免长期维护 2 份平行文档), 但这个合并动作本身需要 Ulysses 拍板, 不在本次范围内

---

## §5 验收/守门标准 (对应 `RF-001-spec.md` §5)

- `cargo check --workspace --lib` 0 err (不允许因本次重构退化)
- `cargo check --workspace --all-targets` err 数不劣化 (基线按开工时重测的数字, 不用本文档写的 ~460)
- `cargo clippy --workspace --lib` 0 warning (T1.5 落地后的新基线)
- `cargo fmt --all --check` exit 0
- `cargo test --workspace --lib` 全过, 数量不少于开工前
- **设计文档同步 (per `RF-001-spec.md` §2.4)**: 任何标了"必做"的任务, 对应 `docs/specs/`/`docs/architecture/`/`docs/adr/` 文件必须在同一 commit 里出现, `git show --stat <commit>` 里看不到对应文档变更的, 视为该任务未完成, 不能打勾
- author = Ulysses (per 代签规则), commit message 标注 WBS 任务号

---

## §6 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-09-02 | 本 session AI | 初版: 从 `RF-001-spec.md`/`WBS-001-refactor.md` 拆出下游 AI 可执行项 (§1 T1 5 项 / §2 T2 4 项需 gate) vs 需拍板项 (§3 T3 3 项) vs 边界声明 (§4, 与 HANDOFF-ST-001 划清范围) |
| v0.2 | 2026-09-02 | 本 session AI | 加设计文档同步要求 (per 用户发令"相关设计书也要用重构后的设计更新"): T1.3/T2.3/T2.4 标注必做的文档新建/追加项, T3 系列报告需列拍板后文档同步清单, §5 加"设计文档同步"验收项 (commit 里看不到对应文档变更视为未完成) |
| v0.3 | 2026-09-02 | 本 session AI | 核实全仓 `docs/` 树后修正三处 bug: T1.3 的 ADR 文件名 "0026" 实际已被占用 (改为现场检索编号 + 目标目录改到当前活跃的 `docs/architecture/2026-08-26-upgrade/adr/`) 且判定不应参考 `basic-design.md` §2.1 表; T2.3 `observability.md` 补充与 `operation-design.md` §6.3 的分工/反链要求; T2.4 补充 §2.1 表已知过期 9 crate 的提醒 (不要求修复); T3.2 的权威同步目标从 `integration-design.md` 改为实际的 saga spec + match-bc 文档。详见 `docs/refactor/RF-001-spec.md` §2.4 v0.3 |
| v0.4 | 2026-09-05 | 本 session AI | RF-T1.5 `unreachable_pub`(`bef2d60`) + `rust_2018_idioms`(`d9f65b3`) deny 已落地确认存活(跨外部 TMO rebase/merge 验证); `missing_docs` 步骤两次 85-way 并行 workflow 尝试均因账号级 5 小时 rate limit 中途击杀, 且未提交的部分完成编辑(含 100% 完成的 `domain-batch/src/domain.rs` 120 行)在同期另一条独立 "TMO" worktree 编排工作线的 `rebase`+5 次 merge 序列中丢失; 已测量 missing_docs 单项花费 >1.3M token 仅落地 1/85 file, 超 T1.5 全项 0.3M 预算 4 倍以上, 触发 `RF-001-spec.md` §3 "改动面超出预估" 停止规则, 已记录到 `WBS-001-refactor.md` T1.5 明细段。抽查 `domain-scm`/`domain-agent` 确认其 `pub` 项是策展过的 domain 模型公开面(非 `pub use *` 顺带导出), 收窄可见性非可行捷径, 补文档仍是唯一路径。**后续继续时改为逐 file/crate 顺序处理 + 每完成一个立即 commit**, 不再攒批并行, 以将"未提交编辑因外部并发活动丢失"的风险窗口从 85 file 收窄到 1 file |
