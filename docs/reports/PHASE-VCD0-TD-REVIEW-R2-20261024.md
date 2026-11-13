# PHASE-VCD0-TD-REVIEW-R2-20261024

> - 阶段：VCD-0 测试设计独立复核
> - 状态：Review complete；VCD-0 **OPEN**；候选测试均 **NOT_RUN**
> - 范围：仅审阅 detail/test lane 中尚未提交的 T-049 增量；未修改候选测试设计或产品代码
> - 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> - 审批者：架构师（Mavis 接手 agent per DEC-008）
> - 日期：2026-10-24

## §0 目的

独立评审 `docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md` 新增的 T-049，检查其是否与当前 SRS v0.4、候选基本设计/详细设计、VCD WBS 与既有 VCD-0 测试设计兼容性审查一致。审核对象位于 `C:/Users/leo19/.codex/worktrees/vcd0-canonical-detail-test-20261024/Star`；本报告写入审核 lane，不改变被审阅文件。

**结论：T-049 不可作为已就绪的验收用例；VCD-0 继续 OPEN。** 增量补上了长连接授权、撤权、网关不可达和恢复等方向，但直连认证握手、远程文件浏览范围和长连接“可用”的实际验证标准仍不完整。没有运行测试、部署、浏览器会话或 Gateway。

## §1 改动与发现矩阵

| 项目 | 证据 | 判断 / 后续要求 |
|---|---|---|
| T-049 增量 | 候选 TD §4.7 第 161–167 行：新增 NOT_RUN 的长连接集成用例；AC-VC-005/006 与 FR-VC-008/013/016 追踪新增 T-049（§5.1 第 179–180 行、§5.2 第 200、205、208 行）。 | 需求 ID 均存在于 SRS v0.4。测试仍为 Draft/NOT_RUN；文本明确 VCD-0 OPEN，没有把设计或执行结果写成 PASS。 |
| **VCD-TD-R2-01 — P1：Gateway 直连的凭证传递与 Origin 边界未定义** | SRS FR-VC-008 第 161 行规定 access token 只在 HttpOnly cookie，由服务端代理读取并附加到上游，且不得进入浏览器 JS、URL、日志或终端。SRS FR-VC-013 第 176 行允许长连接直达执行面或经自有边缘代理。候选 BD §1.1 第 36–48 行只画出 Browser → 同源 BFF → Star API；BD §5.3 第 129–130 行允许直连 Star Gateway，并允许 chat SSE 走受限 Next.js route。候选 DD §3.1 第 66–70 行只定义 BFF/Star API 的认证授权，§7 第 160–161 行将 PTY、Canvas WSS、ARG WS 留给 Gateway。T-049 第 163–167 行要求浏览器从 Preview 连接 Gateway，但没有说明该跨源握手如何获得不暴露会话 token 的凭证，也没有明确 Gateway 的 Origin allowlist。 | 为每种传输明确使用既有的、由 Star 授权的握手方式及允许的 Preview Origin；不得把 actor/Run header 当成身份依据，也不得把可复用 bearer 放进 URL 或测试证据。若需短期连接凭证，应明确由现有授权边界签发、绑定身份/Run/能力/Origin 与连接、有限期和防重放，并记录撤权后如何使活动连接失效。证据采集须脱敏。当前未定义时，保持 NOT_RUN，不能推定握手安全。 |
| **VCD-TD-R2-02 — P1：remote file-browser WS 的授权范围与本地 Worktree 禁用边界未对齐** | SRS 第 35 行只把 remote file-browser WS 记录为现有仓库长连接事实；FR-VC-013 第 176 行没有为它定义验收要求。FR-VC-016 第 184 行要求 Vercel 形态 fail closed 禁用本地 Worktree 文件系统。候选 BD §1.1 第 28 行排除将本地 Worktree 文件系统迁入 Vercel，§5.3 第 129–130 行列出的 Gateway 能力没有 remote file-browser；DD §7 第 160 行同样未列它。T-049 第 167 行却要求该 WS 由 Star-owned Gateway 承载，并把它纳入成功标准。 | 先在 VCD-0 说明它是 Star 执行面上的远程、受限读操作，且不能开放 Vercel 本地 Worktree 文件系统；为其指定现有 Owner/授权能力和测试范围，并补齐 SRS/BD/DD/TD 追踪，或从 T-049 移除。当前证据只证明它是仓库现状清单的一项，不能据此授予功能或验收范围。 |
| **VCD-TD-R2-03 — P1：长连接“可用”缺少正向数据流与存续断言** | SRS FR-VC-013 第 176 行要求长连接在 Vercel 形态下实测仍可用。候选 T-049 第 167 行写“建立”五类连接并在撤权后“尝试继续收发”，但未要求撤权前的受保护测试帧/事件实际到达、SSE 事件被消费、双向通道完成一次往返，或连接按 Star Gateway 已声明的生命周期/心跳契约维持。重连成功也没有对应的数据面断言。 | 对每类连接分别定义一个无副作用的正向收发/事件断言、可关联的连接生命周期证据及按实际 Gateway 契约判断存续和重连的方法。不要在 SRS 未给阈值时自行编造延迟/持续时长门槛；若运行契约尚无可验证目标，应把此验收项标为未定义并继续 OPEN。 |
| **VCD-TD-R2-04 — P2：恢复期间“不重复命令/业务副作用”没有触发操作** | T-049 第 167 行先建立连接、测试撤权及 Gateway 不可达，再恢复授权并重连；没有安排一次可安全重试的命令/副作用或记录命令幂等身份，却要求证明恢复期间不重复命令或业务副作用。 | 将恢复验收限定为连接和传输恢复，或在现有 Owner 合同允许时增加可审计、幂等的安全操作及唯一回执断言。否则删除这项超出当前操作的结果承诺，不要把“未执行命令”写成“证明没有重复命令”。 |
| 既有兼容性审查 | `docs/reviews/VCD0-TD-COMPATIBILITY-20261024.md` §VCD-R3（第 25–31 行）要求区分通用 Functions WebSocket GA 与 Rust Runtime 未证实能力；§VCD-R5（第 41–47 行）要求 Preview 无生产副作用；§VCD-R6（第 49–51 行）保留 Multipass 两条记录上限。候选 TD §4.4 第 123–137 行明确将 Rust Runtime 探针隔离，§2 第 41–46 行保留 Preview/no-production 与 Multipass 两记录规则；T-049 把 Gateway 留在 Star 侧。 | 这些边界没有被 T-049 直接反转；但 T-049 不替代既有 VCD-0 追踪和执行门。旧审查报告中其它 finding 的闭环须由 canonical ADR/BD/DD/TD 与 WBS 单独确认。 |
| 当前 WBS 门 | 本 lane 当前 WBS `docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` §2 表第 56 行要求 canonical ADR/BD/DD/TD 对齐后才关闭 VCD-0，并说明候选文件不是 dev canonical baseline。 | T-049 是分支候选中的文档增量；本次审查不改变 WBS 状态。候选与本报告需经父任务复核和选择性集成，canonical 设计门仍未满足。 |

## §2 验证摘要

- 只读查看了指定候选 T-049 diff，并对照当前 dev 的 SRS、VCD WBS、既有测试设计兼容性审查，以及分离候选 lane 中的基本设计和详细设计。
- 核对了用例状态与需求追踪：T-049 为 **NOT_RUN**，全表仍以 **NOT_RUN** 开始，release criteria 将 **VCD-0 标为 OPEN**。
- 只对本报告执行 diff/whitespace 检查；未运行 Cargo、前端测试、浏览器、Gateway、账户或部署验证。
- 本报告未验证任何运行时行为、授权实现、真实身份来源或连接恢复效果。

## §3 已知缺口

1. T-049 尚未精确定义直连 Gateway 的安全凭证握手、Origin 校验和证据脱敏。
2. remote file-browser 的既有实现事实尚未转化为已批准的需求和 Owner/能力边界，且须与 FR-VC-016 的 Worktree 本地文件系统禁用要求相容。
3. 每种协议的正向数据流、存续判据及重连效果尚未可验证；具体目标必须取自实际 Star Gateway 契约。
4. “恢复期间不重复命令/业务副作用”尚无对应的测试操作与幂等回执证据。
5. VCD-0 的 canonical ADR/BD/DD/TD 集成与全部 SRS FR/NFR/GAP/AC 追踪仍由主任务按 WBS 处理；本报告不能关闭该门。

## §4 子代理失败接手清单

- 本 lane 收到的任务仅为独立只读审核，没有派发下游子代理。
- 没有子代理失败或未验证产物需要接手。

## §5 守门规则

1. 不改候选 TD 或产品代码；只在本审核 lane 写报告。
2. T-049 和所有部署/运行行为保持 NOT_RUN；VCD-0 保持 OPEN。
3. 不把通用 WebSocket GA 解释为 Rust Runtime 能力证明。
4. 不把 Vercel handler 描述为 Gateway、PTY、Canvas、Worktree 或 Agent 执行 Owner。
5. 不信任浏览器 actor/Run header；真实身份和 Run 授权须由 Star 当前授权边界判定。
6. 不在 URL、浏览器存储、日志、trace、测试附件或终端证据中保留 bearer、cookie 或可复用连接凭证。
7. Preview 不使用生产密钥/数据、不产生生产 Owner 或 Schedule 副作用；Multipass 仍受两条 inventory 记录上限及受控 launcher 约束。
8. 不以文档审查宣称测试、部署、性能或生产验收通过。

## §6 签字栏

| 角色 | 审核状态 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008）— 本次审核结论：T-049 未就绪，VCD-0 OPEN |
| SRE Lead | 本次范围未执行部署或运行验收；不作运行结论 |
| 平台 | 本次范围未执行目标平台/账户验证；不作能力结论 |
| 评审主持 | 本报告为独立文档审查记录 |
| PM | 后续按 WBS 由父任务复核并决定是否选择性集成 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次独立审核 T-049；记录 3 项 P1、1 项 P2；确认测试状态 NOT_RUN、VCD-0 OPEN；未修改候选文件。 | brief `WBS-VCD0-TD-REVIEW-R2-20261024` |
