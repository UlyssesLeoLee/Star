# PHASE-W16-PAYLOAD-SECURITY-DESIGN-20261028

> 状态：🟡 设计增量与后续实现状态同步；不表示 W16 身份、生产 KMS、持久协作或可共享正文闭环已实现。

## §0 目的

记录 W16 认证设备身份、持久密钥、协作载荷 envelope 与恢复设计，并把已完成的并行审计证据纳入本地 dev 的 WBS 路径。当前阶段仅形成要求与设计，不开放共享正文写入。

## §1 改动矩阵

| 文件/证据 | 改动 | 状态 |
|---|---|---|
| `SRS-W16-PAYLOAD-SECURITY-001.md` | 设备绑定/撤销、OAuth 防重放、envelope、安全恢复与有界资源验收 | 本 lane 草案 |
| `BD-W16-PAYLOAD-SECURITY-001.md` | owner 边界、W/T/M 逻辑记录、provider 与部署约束 | 本 lane 草案 |
| `DD-W16-PAYLOAD-SECURITY-001.md` | 身份链、AAD/envelope、撤销、轮换、恢复与错误流程 | 本 lane 草案 |
| `TEST-DESIGN-WORKTREE-CANVAS-001.md` | v1.8，新增 W16 UT/IT/E2E-COLLAB-12..19/PT-10 目标验收设计 | 已审阅并同步；均未实现/执行 |
| `W16-PAYLOAD-SECURITY-TEST-DESIGN-20261028.md` | 测试设计审阅结论、依赖与未验证边界 | 已审阅；仅设计级 |
| `WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` | v0.70，记录三条并行审计结果、W16-A..E 阶段门及测试设计 lane 集成 | 本轮文档同步 |
| `PHASE-W16-C-ENVELOPE-CORE-20261028.md` | 后续 W16-C 阶段报告：Rust envelope core、focused 单测、AAD 版本增量和生产缺口 | 已新增；见其独立验证结果 |
| 三份 W16 审计/研究报告 | 身份、Rust 来源、provider 研究 | 已 rebase 并快进到本地 `dev` |

## §2 验证摘要

- 三条报告分支均在受检的单文件 manifest 范围内，`git diff --check` 通过、工作树干净；已分别 rebase 并 fast-forward 合入本地 `dev`。
- 本设计 lane 未运行 Cargo、数据库迁移、Neon、Multipass/K3s、浏览器、性能、KMS 或密码学测试。后续需在真实 provider 与持久数据库下完成独立验收。
- 独立测试设计 lane 经 `rtk proxy git diff --check` 与协调者文档审阅后 rebase/快进到本地 `dev`；该 lane 也未运行测试或构建，所有新增测试仍为设计状态。
- 对五份 manifest 文件运行 `rtk proxy git diff --cached --check`，成功且无 whitespace/error 输出；这只是差异静态检查，不是内容、密码学或运行验收。
- 后续 W16-C 实施 lane 的详细证据记录于独立报告：`rtk proxy cargo fmt --package domain-kms` 成功；首轮测试 19/21 是 AAD 长度期望值错误，随后一次共享 Cargo target 遇到 `LNK1104`；修正并串行重跑后为 21/21，再以 lane 专属 `CARGO_TARGET_DIR=E:\DevCache\cargo\target-w16-c-envelope-core` 重跑也为 21/21。最终结果 `21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s`。该 core 单测不覆盖本设计阶段列出的真实 KMS/DB/RLS/恢复/性能门。

## §3 已知缺口

- Access token 和 `AuthUser` 当前没有可信设备身份；Device 没有生产持久化/认证链。授权码与 refresh 消费有并发双成功风险。
- 没有生产 KMS provider、具体 `TranscriptBodyProtector` 或跨重启 decrypt path；`LocalMockKms` 不具备持久密钥能力。
- 后续 W16-C branch 有 `TranscriptEnvelopeProtector` provider-neutral core，但没有生产 profile resolver、owner/REST/RLS 接入、持久化或 `TranscriptBodyProtector` adapter；wrapped key 轮换、跨重启恢复、可靠 zeroization 与性能目标仍未验收。AAD 的 optional Worktree 是相对旧 DD 的显式增量，放在 Run 之后，tag 0/1 固定编码；最大长度为 absent 201 / present 217 bytes。
- OpenBao Transit 是自托管候选而非已批准依赖；两节点 Raft 不具备故障容忍。AWS KMS 是托管服务，不满足纯自托管开源运行形态。
- 全传递依赖、镜像、chart、许可证公告与实际 SBOM 尚未审查；本地 Rust 源代码没有被复制。
- 主 SRS/BD/DD 路径受既有分支及 `D:\Star` 用户脏文档保护约束影响，本 lane 采用 W16 companion 文档；之后需择机无冲突地归并。

## §4 子代理失败接手清单

- 设备审计与 KMS 来源审计的子代理曾出现传输中断；已在原隔离 worktree 恢复并完成提交。协调者审阅后已 rebase/快进合入本地 `dev`。
- KMS provider 研究 lane 已完成并合入；研究报告中候选 provider 均保持未批准状态。
- 独立测试设计子代理 lane `codex/w16-payload-security-test-design-20261028` 基于本 lane commit `42948810` 完成；协调者审阅后 rebase 并快进合入本地 `dev`。其加入 TEST-DESIGN-WORKTREE-CANVAS-001 v1.8、对应 review、dispatcher brief/audit，并同步本 WBS 与本阶段报告；UT/IT/E2E-COLLAB-12..19/PT-10 均未实现或执行。

## §5 守门规则

- 保留 `D:\Star` 中所有 dirty/untracked 内容；未触碰其用户文件、旧 WLD-7 worktree 或 `main`。
- 只接纳有来源与 hash 证据的审计结果；未完成权属/依赖闭包的来源不复制、不宣称许可放行。
- 安全身份、provider、RLS、持久化与恢复任何一门未通过，协作正文保持 fail closed；不得由 mock、进程内状态或单进程事件总线替代。
- 测试环境遵守最多 2 个 Multipass inventory 实例；不需要 Docker Desktop，不声称两节点 HA。
- W16 companion 文档是目标设计增量，不代表代码实现或商业发行验证。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 / 本报告审阅 | 架构师（Mavis 接手审核）；仅审阅本阶段范围与证据，不批准 provider 或关闭 W16 |
| SRE Lead | 待目标 provider/恢复/资源预算验收 |
| 平台 Lead | 待身份与 KMS Rust adapter 实现 review |
| 评审主持 | 测试设计 v1.8 已审阅并同步到本地 `dev`；实现与运行验收仍开放 |
| PM | W16/WLD-7/PT-09/10 保持 OPEN/NOT_RUN |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 W16 载荷安全设计 companion、阶段门、并行审计证据与未验证边界；不表示实现通过。 | W16 payload readiness 审计与用户要求的安全/性能设计补全。 |
| v0.2 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 W16 测试设计 v1.8 的协调者审阅与本地 dev 集成，更新改动矩阵、阶段状态与签字栏；所有安全、数据库、provider 与性能验收仍 OPEN/NOT_RUN。 | 独立测试设计 lane 完成并合入本地 dev。 |
| v0.3 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 链接后续 W16-C envelope-core 阶段报告并同步单元测试结果、AAD optional Worktree 编码与生产缺口；不将 core 单测视为 W16 闭环验收。 | W16-C envelope core 与 SRS/BD/DD/WBS 状态同步。 |
