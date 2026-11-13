# Brief: w16-collaboration-runtime-audit-20261025

**Agent**: explorer
**Phase**: WBS-PARALLEL-20261025-W16
**Created**: 2026-10-25 15:57:18

---

## 目标
依据当前 dev 的 WBS §1/W16 和 WLD-7 验收，审计现有实时协作路径，给出不会绕过 owner、授权和 durable delivery 边界的下一批实现拆解。

## 父协调器已核实的证据
- 父协调器本轮 CodeGraph MCP 返回 `Transport closed`；已定向检查下列源码：`frontend/src/lib/group/groupProjection.tsx` 在约 201 行实例化 `CanvasOutboxPoller`；`frontend/src/lib/group/worktreeGroupApi.ts` 的 `pollOnce` 读取 cursor 后调 `listWorktreeCanvasEvents`；`bff/src/collaboration/wss_hub.rs` 声明单实例 `tokio::broadcast`，文件注释仍标阶段占位/不订阅真实 broadcast；`crates/api/src/arg/sse_hub.rs` 是另一个单实例 bounded broadcast 范例。`frontend/src/app/worktree/[id]/group/page.tsx` 的 30s 定时读取是插件列表，不要误认为它就是 Canvas 轮询周期。
- WBS W16 明确禁止客户端常态固定轮询、1 秒 holdback、只靠进程内广播、不可恢复草稿或把 transport ACK 当 owner commit。冲突必须写入资源 owner 的 revision/draft/conflict/Audit/Outbox 事实；正常事件由 owner commit 唤醒 durable Outbox relay，cursor catch-up 仅用于启动/重连/gap/recovery。
- W16 依赖 W2/W4/W6/W8；不可用 mock、前端定时刷新或 BFF broadcast 冒充闭环。CodeGraph 不可用时以目标源码和测试设计为准。

## 任务
对当前代码、migration、API、测试与 WBS/协作详细设计做逐条证据矩阵，覆盖：资源级 auth+base-revision CAS；同字段冲突时双方 draft 的精确版本 seal/pin 与稳定 conflict_id；可读有界 base/current/mine/theirs diff 与持久未读提醒；owner-local Outbox 与 commit wake-up；跨实例 relay/fan-out、订阅/队列背压；cursor bootstrap/gap/recovery；撤权/RLS/retention；1s p99 与 workload/profile 的实际证据。每项标为 implemented / partial / missing / unknown，引用实际路径/行号，指出测试是设计、编译还是实际运行证据。

最后提出 1–3 个依赖顺序清晰的下一实现 lane：指出哪个 owner 与文件应负责、依赖哪个 WBS gate、完成的测试/运行验收是什么；若上游 owner/transport 未就绪，要明确先补的前置工作，禁止用临时轮询或内存队列替代持久协议。

## 严格交付范围
- 只新增并提交 `docs/reviews/WBS-W16-COLLABORATION-RUNTIME-AUDIT-20261025.md`。
- 不修改任何源码、schema、WBS 或设计基线，不创建替代 transport/mock，不执行外部服务，不运行测试。
- 文档中文；所有事实带路径/行号；区分代码存在、可运行、数据库验收和生产闭环；未知项写 unknown / NOT_RUN。报告必须能直接用于后续 MAGOS manifest 规划。


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
