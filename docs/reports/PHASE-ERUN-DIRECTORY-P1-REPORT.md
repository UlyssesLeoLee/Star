# PHASE-ERUN-DIRECTORY-P1-REPORT

> v0.1 · 2026-10-02 · Ulysses（一人公司12角色 per DEC-008）— Mavis 接手审核
> 状态：目录基础实现；Phase 1 / 商业闭环未完成。自动化：`scripts/automation/engineering_run_directory.py`，`automation-design.md §4.36`。

## §0 目的

将 Project → Cloud Branch → Engineering Run → Worktree 从目标设计推进为持久身份、三层授权目录与独立 owner/focus context；同步需求/基本/详细设计和综合交付指引。旧 Task/Canvas/Plugin 所有权与真实执行不由本切片宣称完成。

## §1 改动矩阵

| 范围 | 交付 | owner |
|---|---|---|
| 数据 | 八表3T/5M、复合FK、current grants、SCD2、自动审计、目录keyset索引 | schema worker独立worktree，root审阅并cherry-pick |
| Rust API | 五个认证GET、分页/无path/context/focus、修正Project grant future条件 | root |
| 前端 | Run session adapter、严校验client、懒树、canonical context shell、Sidebar/Provider装配 | nav worker独立worktree，root整合 |
| 设计/自动化 | WTG-021、AC-ERUN-003、RunContext/focus分离、方向指引与实施计划§6.67、runner/registry | root |

## §2 验证摘要

- rebase到最新dev后，`cargo check -p star-api-rest --all-targets -j 4` 再次在临时依赖解析条件下通过，存在既有warning，未执行tests。默认`--locked`拒绝现有lock更新；原Cargo.lock已恢复，不能声称committed lock可复现。
- PostgreSQL18 disposable loopback DDL：新migration两次应用成功，catalog确认八表启用并FORCE RLS，集群成功停止并清理新自有目录。旧Worktree migration依赖未随链提供的`audit_trigger_func`，sandbox使用显式stand-in；这不是旧Audit或真实RLS角色行为验证。
- 整合标题/Project hint及15秒deadline修正后，再次运行 `scripts/automation/engineering_run_directory.py --frontend`，`npm run typecheck` exit 0。没有浏览器/桌面运行、视觉、真实性能或产品E2E证据。
- 精确源码独立review发现owner context随checkout变化和分页索引不匹配两项，均已修正。CodeRabbit Windows/Ubuntu前置执行Permission denied，不能声称其review通过。

## §3 已知缺口

1. 宿主真实会话/生产登录或Rust session bridge、token issuer/IdP与同源反代未装配；不使用旧nil-user OAuth、localStorage/public API key替代。
2. 可信SCM Repository→Branch ingest、initial Branch/Run grants、Run kickoff与Worktree binding owner command未接；空目录不从seed/backfill构造。
3. 目标DB/runtime grants、真实角色RLS与授权撤销/并发/写审计行为、EXPLAIN与负载验收未完成。八表tenant RLS不等于完整13类验收。
4. Run-owned Task/Canvas/Inbox/Workflow/Plugin、卡内CLI admission/真实Runtime/验证写回、chat、durable事件投递仍缺；context能力明确false。
5. Advanced Hooks page当前尚不消费Run shell深链的scope query；打开标签页不代表已自动定位该Run生效策略。
6. 新目录缓存/并发/DOM上限和覆盖排队/token/body的15秒deadline有实现；客户端超时能释放请求槽，但不能终止忽略取消的底层host操作，生产provider仍须支持cancellation/drain。桌面RSS、并行峰值、p95、cleanup与长运行基准尚未实测。
7. 仓库Cargo.lock存在workspace解析缺口；未将本轮生成的大量无关依赖改动提交。
8. 首次脚本遇后台pipe继承挂起，root停止自有集群后修复；后续尝试遇旧DDL外部Audit依赖并补显式sandbox stand-in。首次停止后的临时目录删除被自动审批以`blocked by policy`拒绝，保留 `.cache/engineering-run-directory/postgres-6f8ec270-ce2e-4fc6-8e5b-00f4ea267ac6`，未绕过重试。

## §4 子代理失败接手清单

| agent | 实际证据 | 接手/限制 |
|---|---|---|
| schema reviewer/writer | readonly审计后独立checkout源提交`da4d3cfb`；rebase后root整合提交`bfbb9c10` | source与最终migration文件零diff；root检查DDL/catalog，无其数据库/运行成功主张 |
| nav reviewer/writer | readonly追踪auth装配，源提交`93eee374`/`eed71685`/`335b0c32`；rebase后root整合提交`bba4ce72`/`745826bb`/`c3624af8` | worker最终源码与root对应目录/client/shell/Sidebar/providers/route零diff；typecheck通过，deadline独立源码review无阻断，无tests/build代报 |
| API reviewer | 两项源码发现，root按建议修正 | 图/index工具不可用，Tier2源码fallback；没有graph coverage结论 |

`dispatcher.brief`真实落地；native collaboration调用具有可见状态/产出。既有dispatcher invoke/verify stub未用来证明此协作成功。

集成以最新`origin/dev`的`bd97197f`为rebase基线，保留新桌面PoC/文档提交和候选分支既有成果；没有force push。上表整合提交是rebase后的候选链；dev远端/本地最终指针须以push后Git确认记录为准。

## §5 守门规则

| # | 本阶段规则与状态 |
|---|---|
| 1 | Ulysses授权author；不编造历史 |
| 2 | root integration串行，schema/nav写checkout分离 |
| 3 | 原有next-env生成修改保留并排除提交 |
| 4 | 无secret值输出，无公开环境key作会话 |
| 5 | 0新增unsafe |
| 6 | Cargo -j4；compile与test区分 |
| 7 | locked解析缺口明示 |
| 8 | 三层current grant，撤销后再解析拒绝为目标运行门 |
| 9 | owner/focus版本分离 |
| 10 | no-path/有界分页，cursor parent绑定 |
| 11 | 三类八表完整分类；无W则不虚构TTL |
| 12 | T immutable、M SCD2，审计与投递区分 |
| 13 | 租户RLS与完整ACL运行验收区分 |
| 14 | Native Hook保留高级设置tab |
| 15 | 代码改进同步三层设计；无纯文档饱和循环 |
| 16 | sandbox路径验证、停止成功后cleanup；拒绝后不绕过 |
| 17 | 未实测项目不标完成；Phase 1和全目标仍开放 |

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | 架构师（Mavis接手agent per DEC-008）；目录基础通过，生产门未闭 |
| SRE Lead | Mavis接手审核；仅disposable DDL，目标部署未验收 |
| 平台 | Mavis接手审核；身份/Runtime能力缺失保持阻断 |
| 评审主持 | Mavis接手审核；源码第二意见已处理，CodeRabbit不可用 |
| PM | Mavis接手审核；先首个真实CLI闭环，再扩展Run Apps/BI/插件 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容/触发 |
|---|---|---|---|
| v0.1 | 2026-10-02 | Ulysses（一人公司12角色 per DEC-008）— Mavis接手审核 | canonical目录基础、owner/focus改进、精确验证边界与综合方向指引；承接Project/Branch/Run用户反馈 |
