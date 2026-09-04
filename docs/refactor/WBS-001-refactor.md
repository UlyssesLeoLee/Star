# RF-001 WBS-001 — 代码健康度重构任务分解表

**Status**: 🟡 Draft (待 Ulysses review)
**Created**: 2026-09-02
**Authority**: Ulysses（一人公司 12 角色 per DEC-008）
**Pair with**: `docs/refactor/RF-001-spec.md` (范围与验收标准) / `docs/refactor/HANDOFF-001.md` (下游 AI 执行清单)

---

## 0. 总览表

| # | 任务 | 档位 | 估 token | 依赖 | 同步设计文档 | 状态 |
|---|---|---|---|---|---|---|
| T1.1 | 根目录报告文件归档 (98 个 md → `docs/reports/`) | T1 | 0.15M | 无 | 无 (纯路径移动) | ✅ 完成 (per 2026-09-04 本 session: 实测根目录 tracked .md 共 74, 除 README/CHANGELOG/AGENTS 3 项例外共 71 项 `git mv` 至 `docs/reports/`, 与 WBS 估计"98"不符, 以本次 `git ls-files` 实测为准; `HANDOFF-ST-001.md` 经查已在前份 session 移至 `docs/reports/`, 特例条款已 moot; 另有 4 项不匹配 WBS 命名 pattern 的报告文件 `P3-C-D-SELECTION-RESULT.md`/`P3-E-F-SELECTION-RESULT.md`/`RGS-LEAD-ROSTER.md`/`remediation-plan.md` 经内容核实均为拍板/报告类文档, 按 T1.1 任务标题范围一并归档; 全仓 `]\(...\)` markdown 链接语法精确正则匹配 71 个文件名 0 命中, 无需改链接) |
| T1.2 | 根目录散件清理 (14 个 `_*` 文件) | T1 | 0.05M | 无 | 无 | ✅ 完成 (per 2026-09-03 本 session: 已追踪 `_*` 散件此前 session 已清空; 本 session 另清理 2 个新增 untracked 一次性文件 `scripts/automation/_find_fail.py` + `docs/architecture/preview/1m-orchestrator-l0-l1.html`, `git status` 验证干净) |
| T1.3 | `star-vcs` 孤儿 crate 处理 (注册或删除) | T1 | 0.1M | 无 | `docs/specs/domain-vcs-spec.md`(新建骨架) 或新 ADR (编号现场算, 见 spec §2.4.2, **不是** 0026——已被占用) | ✅ 完成 (per commit `b7ec06e`: 注册入 workspace.members + 同 commit 新建 `docs/specs/domain-vcs-spec.md` (101 行), 本 session 复核 `cargo check -p star-vcs` 0 err) |
| T1.4 | `cargo machete`/`udeps` 死依赖清理 | T1 | 0.15M | 无 | 按需 (若依赖在某 spec 里被引用为计划依赖) | ✅ 完成 (per `docs/reports/2026-09-03-rf-001-t1-4-correction.md`: 12/12 findings 实测均为真死依赖, 订正前份报告"0 实际死依赖"误判) |
| T1.5 | lint `warn` → `deny` (3 项) | T1 | 0.3M | T1.3 (先确定 crate 数) | 无 (强制不强制, 视情况) | 🟡 2/3 步完成 (`unreachable_pub` deny, commit `bef2d60`, 后续发现 31 处回归由 `850d71c` 修复; `rust_2018_idioms` deny, commit `d9f65b3`); `missing_docs` 步进行中, 现测 11/87 file 已清零、剩 76 file / 1683 行 (原 87/3916), **改在隔离 worktree `.worktrees/wt-t15-missing-docs`(分支 `rf001-t15-work`) 里做, 不再直接写共享 `main`**(根因见 §T1.5 明细段第 3 次尝试记录) |
| T2.1 | unwrap 收敛 (库代码, 排除 H2/H2-EXT 相关) | T2 | 1.5-2.0M | T1.5 (lint 基线先立) | 按需: 受影响 crate 的 `docs/specs/domain-*-spec.md` 错误类型章节 | ⚪ 未开始 |
| T2.2 | clone 审计 (热路径优先) | T2 | 0.5-0.8M | 无 (可与 T2.1 并行, 不同文件) | 按需: 仅当函数签名变化触发 spec 接口描述同步 | ⚪ 未开始 |
| T2.3 | tracing 埋点补齐 (跨域调用路径) | T2 | 0.4M | 无 | **必做**: `docs/architecture/observability.md`(新建, 代码级埋点规范, 反向链接 `docs/operation-design.md` §6.3 后端管线) + 相关 `docs/architecture/<crate>.md` 追加小节 | ⚪ 未开始 |
| T2.4 | 大 crate 拆分评估 (3 个, 仅报告) | T2 | 0.3M | 无 | **必做**: `docs/architecture/domain-local-runtime.md`(追加) + `domain-integration`/`domain-cli` 对应架构文档(新建); 顺带提一句 `basic-design.md` §2.1 已知过期 9 个 crate, 不要求修复 | ⚪ 未开始 |
| T3.1 | 多协议 DTO 去重选项报告 | T3 | 0.2M | 无 | 拍板前列清单; 拍板后**必做** `docs/api-design.md` + 协议层 spec | ⚪ 未开始 |
| T3.2 | Saga 覆盖率审计报告 | T3 | 0.2M | 无 | 拍板后**必做**同步 `docs/architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md`(主) + `docs/ddd/03-match-bc.md`; `docs/integration-design.md` 仅指针引用, 按需修正 | ⚪ 未开始 |
| T3.3 | 领域统一语言审计报告 | T3 | 0.3M | 无 | 拍板后**必做** 新建 `docs/ubiquitous-language.md` + 各 domain spec 术语章节 | ⚪ 未开始 |
| **小计** | | | **~4.15-4.95M** | | | **4/12 完成 (T1.1/T1.2/T1.3/T1.4) + 1/12 部分完成 (T1.5 2/3 步, `missing_docs` 停止记录中) (per 2026-09-05 本 session 实测)** |

> 设计文档同步的 token 已并入各任务估算, 不单列; 详细同步规则见 `docs/refactor/RF-001-spec.md` §2.4。

> token 估算方法同 `STAR-OLU-001.md` 换算基线; T2.1 区间大是因为 1339 处 unwrap 里库代码 vs 测试代码占比未精确拆分, 需 T2.1 启动后第一步先跑分类统计再收窄估算 (同 H2 先例: 先精确测量再报数字, 不得沿用本表估算值当实测值引用)。

---

## 1. T1 详情 (可直接执行, 建议顺序 T1.3 → T1.1/T1.2 并行 → T1.4 → T1.5)

### T1.3 `star-vcs` 孤儿 crate 处理
1. `git log --follow -- crates/star-vcs` 查历史, 判断是否废弃在建
2. 检查 `crates/star-vcs/src` 是否有可编译的骨架 (`cargo check -p star-vcs --manifest-path crates/star-vcs/Cargo.toml` 独立验证, 若 crate 本身没有独立 Cargo.toml 则说明从未真正建立)
3. 若是遗漏注册: 加入根 `Cargo.toml` `workspace.members`, 跑 `cargo check --workspace`, **同一 commit** 新建 `docs/specs/domain-vcs-spec.md`(骨架即可, 职责章节可写"待补", 但文件必须存在)
4. 若确认废弃: `git rm -r crates/star-vcs`, **同一 commit** 新建 ADR 记录依据 (哪次 commit 后停止维护、为什么不补而是删)。ADR 编号**不要用 0026**——已被 `docs/architecture/2026-08-26-upgrade/adr/0026-star-ai-compat.md` 占用; 执行时先跑 `docs/refactor/RF-001-spec.md` §2.4.2 的编号检索命令现场算出下一个可用号 (含跨分支/worktree 检查, 已知 2026-09-02 当天至少用到 0041+), 新文件放 `docs/architecture/2026-08-26-upgrade/adr/`(当前活跃位置, 非 `docs/adr/`)

### T1.1 根目录报告文件归档
1. `mkdir docs/reports`
2. `git mv PHASE-*.md STAR-*.md QA-*.md ..\..\reports\HANDOFF-ST-001.md DDD-LEAD-REVIEW-PROCESS.md REQUIREMENTS-THREAD-C-HANDOFF.md docs/reports/`
   - **例外**: `..\..\reports\HANDOFF-ST-001.md` 若仍在被跨 session 活跃引用 (per 其 §8 状态), 先跟 Ulysses 确认是否此时移动会打断正在进行的 H2/H2-EXT 续接流程, 建议放最后一步单独 commit 便于 revert
3. 全仓 `grep -rl "\](\.\./)*\(PHASE\|STAR\|QA\|HANDOFF\)-[A-Za-z0-9.-]*\.md" --include=*.md .` 找出所有内部相对链接引用, 逐个更新路径
4. `README.md`/`CHANGELOG.md`/`AGENTS.md` 里若有指向这些文件的链接同步更新

### T1.2 根目录散件清理
1. 对每个 `_*.sh`/`_*.py` 文件: 检查是否被其他脚本/CI 引用 (`grep -rl "<filename>" . --include=*.{sh,py,yml,yaml}`), 无引用则移入 `scripts/` 或删除
2. `_msg_*.txt` 类明显是一次性 commit message 草稿, 确认后删除
3. `_wt_audit/` 目录: 若是 worktree 审计的活跃产物, 移入 `docs/reports/wt-audit/`; 若已过期, 删除

### T1.4 死依赖清理
1. 安装/确认 `cargo-machete` 可用 (`cargo install cargo-machete` 若未装)
2. `cargo machete` 跑一遍, 逐条核实 (排除 build.rs / feature-gated 误报)
3. 清理确认的死依赖, 分 crate 提交, 每个 commit 后跑 `cargo check -p <crate>`

### T1.5 lint 严格度提升
1. 根 `Cargo.toml` `[workspace.lints.rust]` 三项改 `"deny"` (先在本地分支改, 不直接推)
2. `cargo build --workspace 2>&1 | grep error` 收集触发点, 逐 crate 修 (主要是补 `missing_docs`, 少量 `unreachable_pub` 需要判断是否该私有化)
3. 全部修完后再 commit 这次 lint 提升 (不要中间态推到 main, 否则下游 AI 新 commit 会红)

**`missing_docs` 子步骤实测记录 (2026-09-05, per §3 "改动面超出预估" 停止规则触发)**:

- 范围: T1.5 原估 0.3M token 覆盖 3 项 lint 合计; `missing_docs` 单项在 85 file / 3591 行缺文档的 scope 下, 两次 85-way 并行 workflow 尝试均在批次中途被账号级 5 小时 rate limit 击杀, 已测量花费 >1.3M token 才落地 1/85 个 file (`domain-batch/src/domain.rs` 120 行) —— 单项子步骤已超原 T1.5 全项预算 4 倍以上, 且未算入两次失败批次里其余部分完成但未提交的产出。
- 根因 1 (rate limit): 5 小时会话限速为账号级、非 session 级, 主 session 与所有 workflow subagent 共享同一配额, 一次命中会同时杀死整批 `parallel()` 调用, 造成大量已完成的部分编辑仍停留在未提交状态。
- 根因 2 (并发写入丢失): 本仓库同期有另一条独立 "TMO" (Task↔Worktree Orchestration) 工作线在同一 `main` 分支上做 `rebase` + 连续 5 次 worktree 分支 merge; 该操作序列要求/强制了干净工作区, 两次 workflow 批次遗留的未提交编辑 (含已 100% 完成的 `domain-batch/src/domain.rs`) 在此过程中丢失, 需整段重做。已提交内容 (`bef2d60`/`d9f65b3`) 不受影响。
- 已排除的备选方案: 曾评估"收窄可见性代替补文档" (即让触发 `missing_docs` 的 `pub` 项变为非 `pub`, 从根源减少警告数), 但抽查 `domain-scm`/`domain-agent` 等 crate 后确认其 `lib.rs` 是单文件、直接声明的 `pub struct`/`pub enum`/`pub fn`(即被策展过的 domain 模型公开面), 并非通过 `pub use xxx::*` 通配符导致的"顺带公开", 因此收窄可见性不是可行杠杆, 补文档是唯一路径。
- **纠正后的执行方式 (下次继续时使用)**: 不再一次性 85-way 并行发起; 改为逐 crate/逐 file 顺序处理, 每个 file/crate 一完成立即 `git commit`(不攒批), 把"未提交编辑因外部并发活动丢失"的风险窗口从 85 file 收窄到最多 1 file。继续前先重新跑一遍 `cargo check --workspace --lib -j 4 --keep-going --message-format=json` 拿当前 HEAD 下的准确基线(树在两次尝试之间已被外部 merge 推进, 旧行号列表失效)。

**第 3 次尝试记录 (2026-09-05, 同一 session 内, 发现新根因)**:

- 按上条"纠正后的执行方式"跑了第 3 次 (35 crate 分组顺序处理 + 每 crate 完成立即 commit 的自定义 workflow), 12/35 crate (17 file, ~1938 行) 在 rate limit 击杀前已本地 `git commit` 成功 (`git reflog show main` 逐条核实, 顺序/内容均正确), 相比前两次"0 存活"是实质性改进。
- **新根因发现**: 即便本地 commit 成功, 这些 commit 后来仍从 `main` 分支消失 (`git merge-base --is-ancestor` 确认非当前 HEAD 祖先)。`git reflog show main` 显示紧跟在这 13 个本地 commit 之后有一条 `reset: moving to origin/main` —— 本仓库 `D:\STAR` 是**多个并发 agent session 共享的同一个工作目录**(而非各自独立 worktree), 其它并发 session/进程会周期性把本地 `main` reset 到 `origin/main`, 未推送的本地 commit 因此被丢弃(commit 对象本身未被 gc, 可通过 reflog/hash 找回, 但分支指针不再包含它们)。核查 `.git/hooks/` 无相关本地钩子, 判定这是**另一个并发 session 自身的同步逻辑**所致, 而非本仓库配置的自动化。
- 额外确认: `git reset` 是 `--mixed` 语义(只重置分支指针和暂存区, 不清工作目录) —— 因此发生 reset 后, 未 commit 的工作区编辑(以及已 commit 但被 reset 丢弃分支指针的那些文件的内容,若和 reset 前一致)仍原样留在共享工作目录里, 表现为一堆 uncommitted modified file。本次由此在事故现场直接"捞回"了 18 个 file(含 7 个原本以为进度落后的 crate, 实测 cargo check 后发现其实多数已经是 0 剩余警告)。
- **恢复动作 (已执行)**: (1) `git branch rf001-t15-recovered <tip-commit>` 锚定 13 个悬空 commit, 防止被 gc; (2) 把共享工作目录里散落的 18 个 file 的未提交编辑 commit 到新分支 `rf001-t15-worktree-content`(未推 `main`); (3) 把共享根目录 `D:\STAR` checkout 切回 `main`(避免继续占用其它并发 session 期望的根目录状态); (4) 用 `git worktree add .worktrees/wt-t15-missing-docs -b rf001-t15-work rf001-t15-worktree-content` 建立**专用隔离 worktree**, 后续 T1.5 `missing_docs` 剩余工作全部在这个 worktree 里做, 不再直接碰共享 `main`/根目录。
- **`domain-scm` 意外发现**: 该 crate 原本是 3 次尝试中第一批完成/提交的 (`b6d587b`), 该 commit 一度显示悬空, 但复核当前 `main` HEAD 时发现 `domain-scm/src/lib.rs` 已经是 0 剩余警告且内容与 `b6d587b` 逐字节相同 —— 说明该内容已通过某条未追踪清楚的路径(疑似另一并发 workstream 的 merge)独立进入 `main`, 无需重做。
- **`cargo check` 重测 (2026-09-05, 隔离 worktree 内, 复用共享 `CARGO_TARGET_DIR`)**: 87 file / 3916 行原始基线 → 现 11 file 已清零 (`domain-scm`/`domain-context`/`domain-notification`/`domain-search`(仅 lib.rs)/`domain-tenant`/`domain-batch`(仅 domain.rs)/`domain-comment`/`domain-automation`/`domain-project`/`domain-collaboration`/`domain-cli`), 剩 76 file / 1683 行, 分布在 27 个 crate, 大头为 `domain-report`(345)/`domain-local-runtime`(194)/`domain-agent-windows`(124)/`domain-planning`(114)。
- **累计成本 (3 次尝试合计)**: 远超原 T1.5 全项 0.3M 预算, 精确 token 数未逐次记录, 但已确认是**流程性根因**(账号级 rate limit + 共享目录并发 reset)而非单纯"文档量大", 故不作为"继续加预算"的理由, 而是要求**换隔离 worktree 执行**后再继续测量。

---

## 2. T2 详情 (需测试兜底, 逐 crate 提交)

### T2.1 unwrap 收敛
1. 第一步 (纯统计, 不改代码): 按 crate 拆分 unwrap 计数, 区分 `src/` 非测试代码 vs `#[cfg(test)]`/`tests/` 代码, 产出精确表格替换本 WBS 的估算区间
2. 排除掉落在 H2/H2-EXT 覆盖范围 crate (`domain-feedback`/`domain-validation`/`domain-integration`/`domain-comment`/`domain-identity`/`domain-project`/`domain-tenant`/`domain-work-item`, per `..\..\reports\HANDOFF-ST-001.md` §1 H2 表) 里、且 unwrap 出现在 ActorContext 类型转换代码路径上的条目 —— 这些标记"阻塞于 H2/H2-EXT", 跳过
3. 剩余 crate 按 unwrap 数量从少到多排序, 逐个改: 可恢复错误 → `Result` + 该 crate 现有 `thiserror` enum 加变体; 真正不可能失败 (如 `Uuid::parse_str` 对着字面量常量) → 保留但加 `// SAFETY:` 注释
4. 每个 crate 改完: `cargo test -p <crate> --lib` 全过 + `cargo clippy -p <crate> -- -D clippy::unwrap_used` (仅对该 crate 的 `src/`, 用 `#[cfg(not(test))]` 限定范围或 clippy.toml 排除 tests 目录) 0 触发 → commit

### T2.2 clone 审计
1. `cargo clippy --workspace --lib -- -W clippy::redundant_clone` 收集命中列表
2. 优先处理 `domain-local-runtime`/`domain-integration`/`domain-cli` (最大 3 个 crate, 也是热路径集中地) 里的命中
3. 每处改动前后跑该 crate 单测, 确认无行为变化, 逐 commit

### T2.3 tracing 埋点补齐
1. 列出所有 port trait 实现 (`impl ... for ... { ... }` 跨 crate 边界的) 和跨 domain service 调用点
2. 抽样 5 个高频路径 (可参考 `star-mcp/src/handlers/` 下最常调用的 handler) 先补, 验证模式跑通后再铺开
3. 补 `#[tracing::instrument(skip(self))]` + 关键分支 `tracing::warn!`/`error!`
4. 新建 `docs/architecture/observability.md`, 说明埋点范围 (哪些路径必须埋、哪些不要求) + span/event 命名规范, 供后续任务参照而非各自发明一套; 文中反向链接 `docs/operation-design.md` §6.3 (那里是可观测性后端管线/Tempo-Jaeger 配置, 与本文档的代码级埋点规范分工不同, 不要合并成一份也不要互相矛盾)
5. `cargo build --workspace` 确认 0 新增 warning, commit (含代码改动 + `observability.md` 在同一 commit)

### T2.4 大 crate 拆分评估
1. 对 `domain-local-runtime`/`domain-integration`/`domain-cli` 各自跑 `cargo modules structure` (若无该工具, 手工读 `lib.rs` mod 声明) 画出内部模块依赖图
2. 判断标准: 若存在一组模块互相之间无依赖、且对外只通过少数几个 trait/函数交互, 视为可拆信号
3. 产出评估结论 (拆/不拆 + 理由), 写入对应架构文档: `domain-local-runtime` → 追加到已有 `docs/architecture/domain-local-runtime.md`; `domain-integration`/`domain-cli` → 若无对应文档则各新建一份, 结构参照 `domain-local-runtime.md` 现有格式。**本任务不动代码**, 拆分本身是否执行留给 Ulysses 之后单独立项, 但评估结论必须落档, 不能只在 commit message 里一笔带过

---

## 3. T3 详情 (仅产出报告, 不动代码)

### T3.1 多协议 DTO 去重选项报告
1. 列出 `star-api-rest`/`star-mcp`/`star-sse` 三个 crate 里所有对外暴露的 request/response 结构体
2. 按语义分组, 标出真正重复 (字段/校验规则一致) vs 貌似相似但语义不同的
3. 输出方案 A/B/C 对比表 (见 spec T3.1), 推荐项 + 理由

### T3.2 Saga 覆盖率审计报告
1. 列出所有跨 domain 的写操作调用点 (grep 跨 crate 的 service 方法调用 + 检查是否经过 `star-saga`)
2. 标注每条"走 saga"/"直接跨 crate 调用"及一致性风险等级
3. 审计对照的权威口径是 `docs/architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md`(Saga 契约/8 步编排实际定义) 而非 `docs/integration-design.md`(那里只有一行指针引用); 一并核对 `docs/ddd/03-match-bc.md` 的 match 域 (workflow/saga) 边界描述是否与审计结论一致

### T3.3 领域统一语言审计报告
1. 抽样 8-10 个高频业务概念 (如 work-item/task, actor/user, tenant/workspace)
2. 列出各 domain crate 对应的类型名/字段名, 标出不一致点

---

## 4. 执行顺序建议

```
T1.3 (孤儿 crate 判定) ─┐
T1.1 (根目录归档)       ├─→ T1.4 (死依赖) ─→ T1.5 (lint deny) ─┬─→ T2.1 (unwrap, 最大头)
T1.2 (散件清理)         ┘                                      ├─→ T2.2 (clone, 可并行 T2.1)
                                                                 ├─→ T2.3 (tracing, 独立)
                                                                 └─→ T2.4 (拆分评估, 独立)
T3.1 / T3.2 / T3.3 (报告类, 全程可并行, 不阻塞任何 T1/T2)
```

---

## 5. 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-09-02 | 本 session AI | 初版: 12 任务 WBS, token 估算 ~4.15-4.95M, 执行顺序建议, T2.1 明确标注 token 区间需第一步精确统计后收窄 |
| v0.2 | 2026-09-02 | 本 session AI | 加"同步设计文档"列 (per 用户发令"相关设计书也要用重构后的设计更新"): T1.3 加 spec/ADR 新建步骤, T2.3 加 `observability.md` 新建步骤, T2.4 评估结论改为落档到 `docs/architecture/`, T3 系列标注拍板后必做的文档同步项 |
| v0.3 | 2026-09-02 | 本 session AI | 核实全仓 `docs/` 树后修正: T1.3 的 ADR 文件名 "0026" 是实际 bug (已被占用), 改为现场检索编号, 新文件位置改到 `docs/architecture/2026-08-26-upgrade/adr/`(核实为当前活跃目录, 非归档); T2.3 `observability.md` 补充与 `operation-design.md` §6.3 的分工说明 + 反向链接要求; T2.4 补充 `basic-design.md` §2.1 已知过期 9 crate 的提醒 (不要求修复); T3.2 同步目标从 `integration-design.md` 改为实际权威的 saga spec + match-bc 文档。详见 `docs/refactor/RF-001-spec.md` §2.4 v0.3 |
| v0.5 | 2026-09-05 | 本 session AI | `missing_docs` 第 3 次尝试: 12/35 crate 本地 commit 成功后被发现共享工作目录 `D:\STAR` 存在并发 session 周期性 `git reset --mixed origin/main`, 丢弃未推送本地 commit(commit 对象未 gc, 可 reflog 找回); 已恢复悬空 commit 到 `rf001-t15-recovered` 分支 + 从工作区捞回未提交编辑到 `rf001-t15-worktree-content` 分支, 并建立专用隔离 worktree `.worktrees/wt-t15-missing-docs`(分支 `rf001-t15-work`) 承载后续全部 T1.5 `missing_docs` 工作, 不再直接写共享 `main`; 重测当前进度 11/87 file 清零, 剩 76/1683 行, 详见 §T1.5 明细段"第 3 次尝试记录" |
