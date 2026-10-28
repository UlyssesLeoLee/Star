# PHASE-W16-PAYLOAD-SECURITY-DESIGN-20261028

> 状态：🟡 设计增量草案；不表示 W16 身份、KMS、加密保护或持久协作已实现。

## §0 目的

记录 W16 认证设备身份、持久密钥、协作载荷 envelope 与恢复设计，并把已完成的并行审计证据纳入本地 dev 的 WBS 路径。当前阶段仅形成要求与设计，不开放共享正文写入。

## §1 改动矩阵

| 文件/证据 | 改动 | 状态 |
|---|---|---|
| `SRS-W16-PAYLOAD-SECURITY-001.md` | 设备绑定/撤销、OAuth 防重放、envelope、安全恢复与有界资源验收 | 本 lane 草案 |
| `BD-W16-PAYLOAD-SECURITY-001.md` | owner 边界、W/T/M 逻辑记录、provider 与部署约束 | 本 lane 草案 |
| `DD-W16-PAYLOAD-SECURITY-001.md` | 身份链、AAD/envelope、撤销、轮换、恢复与错误流程 | 本 lane 草案 |
| `WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` | v0.69，记录三条并行审计结果与 W16-A..E 阶段门 | 本 lane 草案 |
| 三份 W16 审计/研究报告 | 身份、Rust 来源、provider 研究 | 已 rebase 并快进到本地 `dev` |

## §2 验证摘要

- 三条报告分支均在受检的单文件 manifest 范围内，`git diff --check` 通过、工作树干净；已分别 rebase 并 fast-forward 合入本地 `dev`。
- 本设计 lane 未运行 Cargo、数据库迁移、Neon、Multipass/K3s、浏览器、性能、KMS 或密码学测试。后续需在真实 provider 与持久数据库下完成独立验收。
- 对五份 manifest 文件运行 `rtk proxy git diff --cached --check`，成功且无 whitespace/error 输出；这只是差异静态检查，不是内容、密码学或运行验收。

## §3 已知缺口

- Access token 和 `AuthUser` 当前没有可信设备身份；Device 没有生产持久化/认证链。授权码与 refresh 消费有并发双成功风险。
- 没有生产 KMS provider、具体 `TranscriptBodyProtector` 或跨重启 decrypt path；`LocalMockKms` 不具备持久密钥能力。
- OpenBao Transit 是自托管候选而非已批准依赖；两节点 Raft 不具备故障容忍。AWS KMS 是托管服务，不满足纯自托管开源运行形态。
- 全传递依赖、镜像、chart、许可证公告与实际 SBOM 尚未审查；本地 Rust 源代码没有被复制。
- 主 SRS/BD/DD 路径受既有分支及 `D:\Star` 用户脏文档保护约束影响，本 lane 采用 W16 companion 文档；之后需择机无冲突地归并。

## §4 子代理失败接手清单

- 设备审计与 KMS 来源审计的子代理曾出现传输中断；已在原隔离 worktree 恢复并完成提交。协调者审阅后已 rebase/快进合入本地 `dev`。
- KMS provider 研究 lane 已完成并合入；研究报告中候选 provider 均保持未批准状态。
- 本设计 lane 没有派发子代理；测试设计同步作为下一独立 WBS lane，必须引用本报告所列 SRS/BD/DD 增量并维持 OPEN/NOT_RUN 状态。

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
| 评审主持 | 待并行测试设计 lane 完成 |
| PM | W16/WLD-7/PT-09/10 保持 OPEN/NOT_RUN |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 W16 载荷安全设计 companion、阶段门、并行审计证据与未验证边界；不表示实现通过。 | W16 payload readiness 审计与用户要求的安全/性能设计补全。 |
