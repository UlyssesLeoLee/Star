# PHASE-VCD0-TD-REVIEW-R3-20261024

> - 阶段：VCD-0 测试设计第二轮独立复核
> - 状态：**BLOCKED for canonical integration/acceptance**；VCD-0 **OPEN**；全部测试 **NOT_RUN**
> - 审核对象：候选 commit `c7065825a987090be1801eff6120c3dc8ec616b1`
> - 范围：T-049 修订及其与当前 SRS、候选 BD/DD、WBS 和 R2 报告的对照
> - 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> - 审批者：架构师（Mavis 接手 agent per DEC-008）
> - 日期：2026-10-24

## §0 目的

复核 T-049 针对 R2 审查意见所作的提交，确认已解决项、残留项和能否进入 canonical VCD-0 设计基线。候选文件位于 `C:/Users/leo19/.codex/worktrees/vcd0-canonical-detail-test-20261024/Star`；本报告只写入本 review lane。

**结论：集成仍被阻塞。** R3 正确把未知的 Gateway 握手和 Origin 策略设为 fail-closed 前置条件，但当前 SRS/BD/DD 没有落下实际认证握手契约；同时 T-049 遗漏 SRS/BD/DD 明确列出的 noVNC 通道。可以保留这份修订作为 Draft，但在上述契约和覆盖范围对齐前，不得把它接纳为已就绪的 canonical 验收基线。没有访问 Gateway、浏览器、账户或部署环境。

## §1 发现与修复矩阵

| 严重度 / 项目 | 第二轮证据 | 结论 / 后续门 |
|---|---|---|
| **P1 — Gateway 握手前置未满足，阻塞 canonical 集成** | T-049 第 163 行明确说浏览器凭证握手和允许的 Preview Origin 尚未定义，要求 canonical SRS/BD/DD 记录真实握手、Origin 校验、当前授权、撤权与数据/生命周期契约；缺少任一项则 NOT_RUN/OPEN。SRS FR-VC-008 第 161 行只定义 HttpOnly cookie 由服务端代理读取，禁止凭证进入浏览器 JS/URL/日志/终端；FR-VC-013 第 176 行允许连接留在执行面或通过自有边缘代理。候选 BD §3.1 第 78–82 行仅定义同源 BFF→Star API；§5.3 第 137–138 行仅称页面可直连“已验证”Gateway，未定义握手与 Origin。候选 DD §3.1 第 66–70 行定义浏览器会话、BFF 和 Owner 授权；§7 第 160 行仅指定 Gateway 归属，未定义浏览器到 Gateway 的凭证传递、Origin 策略或撤权信号。 | **明确回答：是，握手前置仍使 canonical 集成阻塞。** R3 已避免猜测协议，并使缺契约时不能执行；这修复了 R2 中测试步骤无安全前置的缺陷，但不等于契约已经存在。须先在 canonical SRS/BD/DD 指明真实 Star-authorized 握手和允许 Origin，并闭合当前 Run/capability 授权及撤权行为；再决定并执行对应正负验证。不能把该缺口仅通过 T-049 的条件句视为已满足。候选 TD 可作为条件式 Draft 留存，不能作为 READY 的 canonical acceptance baseline。 |
| **P1 — SRS 长连接覆盖缺少 noVNC** | SRS FR-VC-013 第 176 行明确列有 terminal WS/PTy/noVNC、Canvas WSS、ARG WS，并要求拓扑标明归属且实际可用；候选 BD §5.3 第 137 行和 DD §7 第 160 行均列出 PTY/noVNC。T-049 第 167 行现在列 terminal PTY/session WS、ARG WS、Canvas WS、chat SSE 四种连接，但没有 noVNC；候选将 FR-VC-013 映射到 T-049。 | 在 T-049 中把 noVNC 作为独立通道加入正向、授权/撤权及断连恢复覆盖，或在 SRS/BD/DD 先有依据地澄清它已由另一项测试覆盖并更新追踪。现有文字不能证明 FR-VC-013 的全部既定通道均已覆盖。 |
| **P2 — disallowed-Origin 拒绝仍可被跳过** | T-049 第 163 行把 Origin 校验列为前置契约；第 167 行却写“只有实际 Gateway contract 可验证时才测试 disallowed Origin”。 | 契约门满足后，应明确验证一个获准 Preview Origin 成功和一个未获准 Origin 被拒；若现有 Gateway 没有可验证的 Origin 限制，必须由 canonical 设计记录这个决策及替代的跨站请求防护，不应默默跳过负向用例。 |
| **P2 — TD 页眉版本未随修订历史升版** | 候选 TD 第 3 行仍标 v0.1；§Revision history 第 282–284 行已有 v0.1、v0.2、v0.3。 | canonical 集成前将页眉版本与当前修订历史对齐。 |
| R2-02：remote file-browser 范围 | R2 报告 §1 对应项要求移除未定义的 remote file-browser，避免扩大 Vercel Worktree 文件系统能力。T-049 第 163 行明确排除此通道并指向 T-018；第 167 行仅列四类连接；FR 追踪不再把 T-049 关联 FR-VC-016。 | **已解决。** 当前用例没有把 remote file-browser 或 Vercel 本地 Worktree 文件系统纳入正向能力。 |
| R2-03：长连接的正向功能与生命周期证据 | T-049 第 167 行逐类要求经授权的无副作用 fixture event/frame/heartbeat；双向 WebSocket 做协议定义的安全 round trip；SSE 消费事件；使用 Gateway 已记录的生命周期目标，未定义则 NOT_RUN，不增加延迟/时长阈值。 | **基本解决，但受 noVNC 覆盖缺口限制。** 已列四类路径具备可观察的正向断言且不虚构阈值；FR-VC-013 的通道集合仍不完整。 |
| R2-04：无重复副作用断言 | T-049 第 167 行声明本用例不触发业务命令/副作用，因此不对“无重复副作用”作结论。 | **已解决。** 结果不再超出实际测试操作。 |
| 测试状态与 WBS 门 | TD 第 5 行仍标 Draft；第 77 行说明所有用例当前均 NOT_RUN；T-049 第 163/167 行保持 NOT_RUN；第 261 行写明 VCD-0 OPEN。当前 dev WBS §2 第 56 行要求 canonical ADR/BD/DD/TD 对齐后才关闭 VCD-0，并说明候选文件不属于当前 dev canonical 基线。 | 状态诚实且没有 runtime/deployment PASS 声明；WBS 门仍未满足。 |

## §2 验证摘要

- 只读检查候选提交 `c7065825a987090be1801eff6120c3dc8ec616b1` 的 T-049 改动，并对照 R2 报告、当前 dev SRS/WBS 和候选 BD/DD。
- 确认：移除了 remote file-browser；为列出的连接补充正向 fixture 与按现有契约判断生命周期的断言；明确凭证脱敏；不再宣称未测试的副作用保证。
- 确认：认证握手和 Origin 契约在候选 SRS/BD/DD 中尚未具体化；noVNC 尚未进入 T-049 测试矩阵。
- 确认所有用例仍为 NOT_RUN、测试设计为 Draft、VCD-0 为 OPEN；没有运行测试、Gateway、浏览器、账户或部署验证，也没有 runtime 证据。
- 仅检查本报告的 diff/whitespace；未 rebase、merge、push 或删除 worktree。

## §3 已知缺口

1. **Canonical 集成阻塞项：** 必须先记录实际 Gateway credential handshake、允许 Origin、授权检查和撤权处理；R3 的条件门本身不是契约实现或设计完成证据。
2. **P1 覆盖项：** T-049 缺 noVNC，无法覆盖 SRS FR-VC-013 的完整通道集合。
3. **P2 负向测试项：** 契约完成后需强制执行获准/未获准 Origin 的结果验证，避免条件句被用于跳过。
4. **P2 文档项：** TD 页眉仍是 v0.1，修订历史已到 v0.3。
5. 本审查不关闭 VCD-0 的其它 canonical 设计、完整追踪或部署/运行验收门。

## §4 子代理失败接手清单

- 本次仅执行父任务派发的独立第二轮审查，没有派发下游子代理。
- 无失败或未验证的子代理产物需要接手。

## §5 守门规则

1. 不修改候选 TD、SRS、BD、DD 或产品代码；仅写本审核报告。
2. 握手/Origin 契约缺失时不执行连接测试、不推断身份，也不把条件门当作实现证明。
3. T-049 与所有其它测试保持 NOT_RUN；VCD-0 保持 OPEN。
4. 不将通用 WebSocket 能力推断为 Vercel Rust Runtime 能力。
5. Preview 不使用生产凭证/数据，不产生生产 Owner/Schedule 副作用；证据必须脱敏。
6. 不声称 Gateway、账户、部署、性能或生产行为已经验证。

## §6 签字栏

| 角色 | 审核状态 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008）— 当前 disposition：canonical 集成阻塞，VCD-0 OPEN |
| SRE Lead | 未执行运行/部署验证，不作运行结论 |
| 平台 | 未访问目标账号或 Gateway，不作平台能力结论 |
| 评审主持 | 本报告为指定候选提交的第二轮独立审查 |
| PM | 后续由父任务按 WBS 决定整合顺序；不得提前关闭 VCD-0 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 复核 T-049 修订；确认 R2 文件范围、正向数据流和副作用断言问题已处理；列明握手/Origin 契约及 noVNC 遗留门；决定 canonical 集成仍 BLOCKED，保留 Draft/NOT_RUN/OPEN。 | brief `WBS-VCD0-TD-REVIEW-R3-20261024`，候选 commit `c7065825` |
