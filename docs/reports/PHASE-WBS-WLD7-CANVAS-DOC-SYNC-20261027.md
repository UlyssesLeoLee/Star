# PHASE-WBS-WLD7-CANVAS-DOC-SYNC-20261027

## 0. 目的

按已登记的 WLD-7 文档 lane，将当前协作 owner 与验收设计同步到 Canvas 需求、基本设计、详细设计和 WBS。范围是设计文本与追踪关系；本报告不宣称协作能力已经实现或通过运行验收。

## 1. 改动矩阵

| 文件 | 版本 | 本轮内容 | 状态 |
|---|---|---|---|
| `docs/requirements/SRS-CANVAS-AGENT-001.md` | v1.4 | 增加 Project → Engineering Run → Worktree → local Branch、hostless Run、同级 Canvas/Task Card、owner-local 协作和 AC-COLLAB-09/测试映射；明确历史候选不构成当前实现。 | 已编辑，待独立审阅 |
| `docs/design/BD-CANVAS-AGENT-001.md` | v0.2 | 增加 Run-scoped owner、授权、资源和协作语义的基本设计；标明仍待实现与验收。 | 已编辑，待独立审阅 |
| `docs/design/DD-CANVAS-AGENT-001.md` | v0.2 | 增加 owner-local revision/audit/outbox、草稿/冲突、Resolve/CAS、授权撤销及有界资源的详细设计契约。 | 已编辑，待独立审阅 |
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` | v0.66 | 新增 §23.6，保留 WLD-7、PT-09、W16 OPEN/NOT_RUN，并列出有依赖的后续实现与验收顺序。 | 已编辑 |
| `docs/reports/PHASE-WBS-WLD7-CANVAS-DOC-SYNC-20261027.md` | v0.1 | 记录本阶段范围、静态检查和已知缺口。 | 当前报告 |

当前依据为协作 spec v1.2、owner-local data design v0.4、Canvas test design v1.7。候选审计中仅采用 Keep/Adapt 项，没有整份复制旧提案。

## 2. 验证摘要

- 完成对登记范围内文档及候选审计的静态一致性检查；逐项核对层级、Run/Worktree 归属、授权、owner-local 事实、冲突恢复和测试映射。
- `rtk git diff --check`：通过，退出码 0。
- 对本轮需求、设计和 WBS 文件执行了指定产品品牌词扫描：无匹配。
- 未运行测试、构建、Cargo、数据库/RLS、Multipass/K3s、浏览器、桌面或性能检查。
- lane 当前 HEAD：`345c048774535dfb9d7ccbf35aa2d7a68be5772f`；本阶段未提交。

时间口径已分别记录：完整的第二份 proposal 在 owner 侧 ≤1000 ms 内抵达是测试刺激条件；PT-09 的 p99 ≤1 s 从 conflict commit 起算，需在真实多实例负载下验证。前者不是冲突窗口、服务端承诺或持有期。

## 3. 已知缺口

- 本阶段仅同步设计，不包含 API/schema/migration/RLS、持久 Outbox/事件推送、双方草稿、冲突检测、Resolve/CAS 或恢复功能实现。
- 目标数据库、权限撤销竞态、多实例事件恢复和端到端行为尚未验证。
- 连接、帧、队列、草稿在途写入、缓存、限频及恢复的容量值均待测试；本文不将候选值作为性能承诺。
- WLD-7、PT-09、W16 均保持 **OPEN / NOT_RUN**。静态文档检查不会关闭上述阶段。

## 4. 子代理失败接手清单

- 本阶段由协调者派发 1 个文档 worker；无子代理失败或待接手产物。
- 独立文档审阅由协调者在集成前完成；不得以本报告替代该复核。

## 5. 守门规则

1. 导航保持 Project → Engineering Run → Worktree → local Branch；Run 可无宿主 Branch，远端 provider/repository/branch 只能是可选元数据。
2. Canvas 与 Task Card/work item 是所选 Run/Worktree 下的同级 App；所有变更按当前资源范围逐操作校验授权。
3. 事实 owner 只在自己的事务中写 revision、append-only audit、draft/conflict 事实和 Outbox；通知负载不得泄漏草稿正文。
4. Resolve 的 OT 预览无副作用；用户确认后使用 owner 当前 revision 做 CAS。取消不写入；CAS 过期须保留双方草稿与冲突。
5. ≤1000 ms 只约束测试刺激到达；PT-09 p99 ≤1 s 的计时起点是 conflict commit。不得互换或表述为已达成。
6. 旧端点、schema、传输、CRDT/LWW 与固定容量数值仅作历史、非规范资料；运行与性能能力没有证据时不得标记完成。
7. 本 lane 不修改清单外文件，不提交、不推送、不更改其他分支或工作树。

## 6. 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) — 本阶段静态审阅记录，不代表实现验收 |
| SRE Lead | 未执行运行环境、可用性或性能验证 |
| 平台 | 未执行 API、数据库、migration、RLS 或基础设施验证 |
| 评审主持 | 当前文档 lane 自检完成，等待协调者独立复核 |
| PM | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次记录 WLD-7 Canvas 需求与设计同步、静态检查范围、验证缺口和后续依赖顺序。 | WBS §23.6 文档 lane 完成。 |
