# 渡口架构与交付方向

> v1.0 · 2026-10-02 · Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
>
> 对照：requirements §50、basic-design §16、DD-WORKTREE-GROUP-001、WORKTREE-GROUP-IMPL-PLAN-001。本指引安排交付顺序，不替代既有 Phase 完成门。源码、编译、数据库与产品运行证据分别记录。

## 1. 产品主线

渡口以多 Agent 开发的 Worktree 控制面为核心。用户要能看清每个 checkout 属于哪个项目、远端分支和协作 Run，由谁负责、哪些 Agent 在执行，以及哪些成果可以验证、合并和安全清理。任务管理、Canvas 和 BI 共同服务这条开发链。

```text
Project                         项目、成员、仓库、聚合质量与设置
└─ Cloud Branch                 可信远端身份、保护规则、合并目标与进度
   └─ Engineering Run           协作工作区、成员、计划、预算与跨 App 数据归属
      └─ Worktree               本地 checkout、Runtime/Agent 绑定与执行焦点
```

点击 Worktree 叶节点后，右侧打开该 Run 的同级 Apps：Inbox、Work Items/Multica/Jira、Task Card、Canvas、Workflow/LangGraph、BI/Benchmark、Plugins。Task Card 内打开 CLI；Canvas 与任务卡通过 canonical EntityRef 和 owner API 互动。展开 Project/Branch/Run 只加载下一层，不自动选择首个 checkout。Project Worktree Index 保留为跨 Run 比较与管理入口。

同一 Run 切换 Worktree，只切换执行焦点。Run 身份、授权版本与 App 数据归属保持一致；checkout 的 workspace、绑定版本和生命周期版本置于独立 WorktreeFocus。切换 Run 或登录会话后取消旧请求，清除旧焦点与终端附件，重新解析授权。

## 2. 功能层级与所有权

| 层级 | 功能 | 所有者与边界 |
|---|---|---|
| 全局 | 跨项目搜索、通知汇总、GLOBAL chat、个人凭证 | 聚合视图按目标逐一授权；不得以全局界面获得跨项目写权限 |
| Project | 成员、Repository、Worktree Index、跨 Run Quality & Improvement | Project 管理授权和聚合，不拥有各 Run 的 Task/Canvas |
| Cloud Branch | 远端 ref、保护/同步、PR/合并队列 | SCM 持有可信 Branch 身份；本地 branch 字符串仅是观测值 |
| Engineering Run | Inbox、Work Items/Task Card、Canvas、Workflow、BI、Benchmark、Plugin Apps | 各 App 可独立挂载，事实由相应 Domain 持有并以 Run 标识归属 |
| WorktreeFocus | checkout、Agent/Runtime/session、Git 状态、任务 CLI 目标 | Worktree/Runtime 持有生命周期；一个有效 checkout 同时仅绑定一个 Run |
| Advanced Settings | Skills、Hooks、MCP、Plugins 同级标签页 | Hooks 固定 `/settings/advanced/hooks`；树只展示生效策略与结果、提供深链 |

`EngineeringRun` 是协作容器；`TaskExecutionRun` 是一次任务执行尝试，具有独立 ID、状态与证据。两者不得复用身份或数据库表。

## 3. 独立 App 与后端拆分

先建立可独立发布的 App manifest、版本化 API 和明确 Domain owner，在 Rust modular monolith 内保持低进程成本。只有负载、故障隔离或发布频率提供实际证据时，再将某个 owner 提升为独立服务。

| App/模块 | canonical 事实 | 通信方式 |
|---|---|---|
| SCM/Branch/Merge | Repository、Branch、PR、merge receipt | 可信 provider + owner API；远端事实不从 UI seed 构造 |
| Engineering Run | Run 身份/版本、成员、Worktree 关系 | 授权目录与 RunContext；不直接改 Task/Canvas 表 |
| Work Item/Multica/Jira | 任务、计划、关系、生命周期 | 同域存储过程/事务；Canvas 或 Workflow 调 owner command |
| Canvas | Document、Element、EntityRef、视觉连线 | 自有 API/CAS；关系事实由 Work Item owner 处理 |
| Agent/Runtime | 执行尝试、PTY、资源租约、结果/验证证据 | Rust admission + CLI adapter；流输出有界、可取消、可恢复 |
| Workflow/Loop | 计划、触发、checkpoint、循环预算 | 幂等 command；恢复时重新授权并复验租约 |
| BI/Benchmark | 指标定义、版本、cohort、证据/回放 receipt | 消费事件生成 read model；不替代业务事务 |
| Plugin Registry/Gateway | manifest、trust、capability、安装/撤权状态 | 可视化配置 + 版本契约 + 隔离执行；热卸载先拒绝新准入再 drain |
| Native Hook | 原生强约束、版本化策略和决策证据 | 每次执行/写入/合并入口强制校验；App 卸载不得移除强约束 |

同域事务/存储过程只修改该 owner 的事实，同时写 Audit 与 Outbox。跨域同步 command 通过 API，异步事实通过 Outbox→传输→consumer Inbox 去重→投影；重试、失败与 DLQ 都可审计。SQL 跨域 JOIN 可用于受控只读投影，不授权一个服务直接写其他 owner 的表。目录的不可变审计事件尚不能称为已完成 Outbox 投递。

当前保留 PostgreSQL SoR + NATS JetStream 传输基线。JetStream 提供持久化/回放与至少一次投递，仍需 consumer 幂等与 pending/队列上限。[NATS 官方文档](https://docs.nats.io/reference/2.12/jetstream)、[consumer flow control](https://github.com/nats-io/nats.docs/blob/master/nats-concepts/jetstream/consumers.md)。

Kafka 和 Fluvio 进入相同负载的分析流 PoC。Kafka 的日志、consumer 与事务模型，Fluvio 的 Rust/Wasm SmartModule 变换分别是评估依据；语言不能证明实际 RSS、可靠性或运维成本。尚无 Star 同负载实测前，不为桌面客户端追加 broker，也不提前宣布胜者。[Kafka 官方设计](https://kafka.apache.org/design/)、[Fluvio 官方 SmartModules](https://www.fluvio.io/docs/latest/smartmodules/overview/)。

## 4. 原生 Agent 与一期 CLI

借鉴 pi agent 的核心职责分离：可取消的 agent loop、消息/工具事件流、上下文变换和独立会话 backend；其工具 preflight 与执行有明确事件顺序，也支持并行工具执行。[pi 官方 Agent core](https://github.com/earendil-works/pi/blob/main/packages/agent/README.md)。据此，渡口采用 Rust core，并把强制 Hook 设为工具派发前的原生阻断门，扩展回调不得绕过它。采用 CLI adapter 的一期也必须保留 task/run/worktree 绑定、超时与预算、原生 Hook、验证与审计。

Memory、Skill、Context、Validation 均预留版本/摘要/来源和 API 边界。一期允许外部 CLI 提供能力，Run 启动固定所采用的 profile 与依赖快照；恢复或新尝试须重新验证当前授权与可用版本。Independent Validation 与生成过程分开记录，CLI 自报成功不能单独满足任务完成门。

CLI adapter 必须声明其工具事件、取消、进程回收和文件权限等能力。外层原生 admission、预算、工作区隔离、文件 claim 与结果验证必须生效；无法观测或拦截的 CLI 内部工具调用不能标为已受 Native Hook 逐工具约束。强制 policy 所要求的能力缺失时拒绝启动，并给出原因。可信宿主也须实现 cancellation/drain；客户端 Promise 超时退出和释放请求槽，不证明忽略取消的底层操作已经终止。

Schedule Loop 同时覆盖定时、事件和 agent 自主循环。每个 loop 有唯一运行身份、幂等触发、租约、checkpoint、时间/token/费用/迭代预算、取消与暂停状态；worker 重启不重复执行已提交副作用。调度器按 Project/Run 公平分配资源，队列有界，预算不足进入可见等待。BI 反馈形成有来源和版本的策略建议；仅按已生效 Native Hook policy 决定允许的自动动作。

Hook 编辑以高级设置的可视化表单/规则构造器为主，展示作用域、触发点、条件、阻断/提示动作、预算、版本与生效范围，提供 dry-run 与解释。发布后生成可审计 policy version；Run 和 Worktree 显示当前生效版本及最近决策。失效、缺失或不可解释的强制策略拒绝执行。

## 5. Rust 桌面与并行能力

- Core、调度器和 Hook 路径由 Rust 管理；CLI/Plugin 隔离进程按活动任务按需启动，并有资源租约。避免为每个 App/Agent 启动常驻完整服务或 WebView。
- 目录逐层分页懒加载，缓存、请求并发、展开行 DOM 都有硬上限；收起与会话切换取消请求。不把全部项目/Run/Canvas/终端日志一次装入内存。
- 终端输出有界 ring buffer 与落盘策略；Canvas 使用 viewport 裁剪和增量投影；跨 IPC 发送批量增量，控制序列化/复制和在途请求。
- Runtime 使用共享调度与有界 Tokio 通道、背压、fair admission、deadline/cancel/drain。父进程统一跟踪子进程、租约与文件 claim；异常退出不得留下无主执行。
- 按相同环境测空闲 RSS、1/2/4/8/16 Agent 的峰值 RSS、启动/任务/取消 p95、队列等待、长运行漂移、IPC bytes 与缓存命中。发布前冻结设备、数据集、负载与预算；没有实测不填写“低内存已达标”。前端类型通过、Rust 编译通过、桌面打包和真实性能是不同证据。

## 6. 交付顺序与完成门

下列批次对应既有 Phase，不重置其历史状态；先完成能在真实身份和真实 checkout 上使用的一条开发链，再扩展体验与分析。

| 批次 | 对应工作 | 完成门 |
|---|---|---|
| A 导航身份基础 | Phase 1/2：Branch/Run registry、三层授权、RunContext、主树 | 可信 SCM ingest、grants provisioning、宿主会话与目标 DB 到位；真实 actor 能逐层打开自己的 Run，越界拒绝 |
| B 首个商业使用闭环 | Phase 2/4/8/9：Run kickoff、Worktree create/import、自动分配 Agent、卡内 CLI | 创建任务→分配→实际执行/流输出→取消/恢复→独立验证→结果写回→PR/合并/安全清理；失败有可理解状态与证据 |
| C Run App 归属迁移 | Phase 3/5/6：Task/Canvas/Inbox/Workflow/Plugin 的 Run owner | 同 Run 切换 Worktree 保留 Task/Canvas；跨 Run/租户引用拒绝；旧 Group resolver 不猜身份；Canvas 双向任务联动 |
| D 原生约束与循环 | Native Hooks、profile/context、Schedule Loop、BI feedback | Hook 强制生效并可视化；loop 有界可恢复；BI 建议有版本/来源且不能越过授权 |
| E 协作与扩展 | durable Outbox/Inbox/realtime、LangGraph、Plugin 热插拔、Benchmark | 断线/重复事件/撤权/插件崩溃不破坏事实；重放与指标可解释、版本可追踪 |
| F 发布与性能 | 跨 App 验收、Rust desktop baseline、目标部署/运维 | 真实认证/RLS/Runtime、打包、故障恢复、并行性能、资源释放均有证据；锁文件与 release 构建可复现 |

当前 A 批次已有 canonical schema/API/导航实现工作；上线闭环仍受宿主会话、可信 SCM 与授权写入、目标库部署及 Run App/执行接线限制。现有旧 Worktree Apps 和单元/类型结果不能替代上述完成门。

## 7. 实施与文档对账

每个批次先记录修改的 owner/API/存储与验收条件，完成后对比需求、基本设计、详细设计和实施计划。代码改善了设计时，同步修订所有受影响层；仅有代码时标“实现”，有编译时标“编译”，数据库/部署/真实产品闭环分别追加证据。归属迁移、授权、资源上限与 cleanup 不允许用 mock 或 UI 标签代替。

本轮改进：RunContext 与 WorktreeFocus 的版本分开；目录分页增加与 keyset 顺序一致的索引；强制策略与可卸载 App 边界分开。完整工程进度见实施计划 §6.67，不宣称所有 Phase 已完成。
