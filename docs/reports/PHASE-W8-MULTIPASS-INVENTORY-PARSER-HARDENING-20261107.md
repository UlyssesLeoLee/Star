# PHASE-W8 Multipass Inventory Parser Hardening

**版本**：v0.1
**状态**：实现候选完成，等待协调者独立审查；W8 仍 OPEN，稳定版仍 NO-GO
**工作基线**：本地 dev@996ae3ae（创建 lane 时）
**分支**：`codex/wbs-w8-multipass-inventory-parser-20261107`
**修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

阻止无效或含糊的 Multipass JSON inventory 被解释成零个实例，进而绕过测试阶段最多两个实例的磁盘保护。该切片只加固既有 PowerShell parser 与本地回归测试，不运行 Multipass 或任何 VM/K3s 操作。

## §1 改动矩阵

| 范围 | 改动 | 结果 |
|---|---|---|
| Inventory parser | 仅显式 `{"list":[]}` 可表示零实例；拒绝空根对象、null、额外 list 顶层字段、错误容器、错误记录、空名称/状态及重复名称 | 已实现并通过合成 parser 测试 |
| 既有 map 形态 | 保留非空的实例名到单条/多条状态记录形式；空记录、错误记录或缺失状态失败关闭 | 已实现；尚无当前 Multipass 主机响应的实测证据 |
| 创建路径 | 保留原有二实例上限与跨进程锁；测试通过本地函数替身确认 parser 异常在 `multipass launch` 前返回失败 | 已实现并通过 31 项合成断言 |
| 测试 | 从部署脚本 AST 只载入 parser 与创建函数；整个脚本入口和真实 CLI 均不执行 | 已实现 |
| Graph headers | 更新 parser 类型关系；新增测试的 AST/测试边；刷新两个 PowerShell 文件 source hash | CypherGraph Guardian 通过 |
| WBS 主文件 | 不在登记清单内，本 lane 未修改 | W8 仍 OPEN；需协调者在独立同步中记录本切片 |

## §2 验证摘要

验证运行于本机 PowerShell 7.6.9；本测试脚本不依赖 Pester。

| 命令 | 结果 |
|---|---|
| `rtk proxy pwsh -NoProfile -File 'D:\orcaWork\Star\wbs-w8-multipass-inventory-parser-20261107\scripts\automation\test_multipass_k3s_gitops.ps1'` | PASS，`PASS: 31 parser and fail-closed create-path assertions.` |
| `rtk proxy python 'C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py' --root 'D:\orcaWork\Star\wbs-w8-multipass-inventory-parser-20261107' --stamp scripts/automation/multipass_k3s_gitops.ps1 scripts/automation/test_multipass_k3s_gitops.ps1` | PASS，`CypherGraph Guardian: PASS (2 source file(s) checked)` |
| `rtk proxy python 'C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py' --root 'D:\orcaWork\Star\wbs-w8-multipass-inventory-parser-20261107' scripts/automation/multipass_k3s_gitops.ps1 scripts/automation/test_multipass_k3s_gitops.ps1` | PASS，2 个 PowerShell source files checked |
| `rtk proxy git -C 'D:\orcaWork\Star\wbs-w8-multipass-inventory-parser-20261107' diff --check` | PASS，exit 0，无空白错误 |

测试覆盖显式空 list、list/非空 map 有效记录、空根对象、JSON null、非对象根、null/非数组 list、额外 list 字段、null/字段缺失/类型错误/名称或状态为空/重复名称的 list 记录、空或 null map 记录、空白 map 名称、map 状态缺失/非字符串/空值以及错误 JSON。创建路径测试以替身注入 `{}` 解析失败，并断言结果失败、记录拒绝原因且 Multipass 命令计数仍为 0。

以上均为**合成输入的 parser-only 测试**，不证明某个当前 Multipass 版本的实际 JSON 响应、实例库存、跨进程竞争、VM 创建、K3s 就绪或磁盘占用行为。未运行 Cargo、Pester 或 Multipass。

## §3 已知缺口

1. 尚未在用户允许的 Multipass 1.17.0/HCS 环境采集 `multipass list --format json` 原始输出；`{"list":[]}` 空库存表示及兼容 map 形态仍需真实版本验证。
2. 双实例容量、Stopped/Deleted 计数、并发锁争用、启动与恢复、K3s readiness/upgrade/rollback 均未验证。
3. 此 lane 不更新共享 Stable Release WBS；协调者需在集成状态同步中补充当前 lane 提交、测试范围与仍开放的 W8 门禁。
4. 当前 `dev` 的远端 tracking refs 未刷新，不能据本地 ahead 数推断远端状态；此 lane 不改 `main`、不 push。

## §4 子代理失败接手清单

本 lane 由一个 worker 在独立登记 worktree 执行，未派生子代理；无 RPC、超时或子代理提交待接手项。提交完成不等于独立审查或集成通过，协调者仍需独立检查差异与提交链。

## §5 守门规则（17 项）

1. 仅修改 MAGOS 登记的六个路径。
2. 写入只发生在本 lane 独立 worktree。
3. 不修改共享 WBS 或其他 lane 文件。
4. 不接触 `D:\Star` 根工作树的用户改动。
5. 不执行 `multipass` 命令。
6. 不读取、创建、启动、停止或清理 VM。
7. 不改测试阶段二实例上限。
8. 不改跨进程独占锁。
9. 只有显式数组 `{"list":[]}` 可作为零条实例 inventory。
10. 未知/空根 schema、非数组 list 和未知 list 顶层字段失败关闭。
11. 每条记录必须具备非空字符串名称和状态。
12. map 形态必须至少含一条有效记录。
13. 重复名称、无效 JSON 与错误记录必须失败。
14. 创建路径拒绝测试必须证明确实到达 parser 且未调用命令替身。
15. 合成 parser 测试不得冒充真实 Multipass/K3s 验证。
16. source graph 更新后必须通过 CypherGraph Guardian。
17. 集成前必须由协调者再次审查与验证；不得改 `main`、远端、tag 或发布状态。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | Mavis 接手审核：通过本 lane 的 parser fail-closed 切片自审；不代表真实 Multipass schema 或发布门通过 |
| SRE Lead | Mavis 接手审核：确认未运行 VM/集群操作；双实例与恢复门仍开放 |
| 平台 | Mavis 接手审核：确认锁与容量逻辑未改，关注点限于解析边界 |
| 评审主持 | Mavis 接手审核：本文件为作者自审记录，协调者独立审查后才可集成 |
| PM | Mavis 接手审核：W8 OPEN，稳定版 NO-GO |

## §7 修订历史

| 版本 | 日期 | 修订内容 | 触发与证据 |
|---|---|---|---|
| v0.1 | 2026-11-07 | 加固 Multipass inventory parser；新增 29 项无真实基础设施副作用的 PowerShell 断言；更新 Cypher graph headers 与阶段报告 | W8 容量复核记录 `{}` 被误判为空库存及 parser 专项测试缺失；本报告 §2 列明实测命令 |
| v0.2 | 2026-11-07 | 补充空白 map 实例名与非字符串 map 状态拒绝断言；当前 suite 共 31 项，无生产代码变化 | 独立规格审查指出原 29 项未覆盖这两个 map 拒绝分支；重新执行聚焦测试与 source graph 校验 |
