# PHASE — W8 Multipass/K3s 容量复核

## §0 目的
按稳定版 WBS-W8 只读复核本机 Multipass 版本与实例清单，并静态检查仓库启动脚本的两实例 fail-closed 门禁。此阶段不执行 VM 生命周期操作；检查结果只描述当前宿主与代码，不等同于 K3s 集群或 Star 原生基础设施运行验收。

## §1 改动矩阵
| 文件 | 改动 | 验收 |
|---|---|---|
| docs/reports/PHASE-STAR-STABLE-W8-MULTIPASS-CAPACITY-RECHECK-20261101.md | 记录本机版本、库存和脚本静态证据 | 清楚区分 host、guest、K3s 与 Star owner |
| docs/reviews/STAR-STABLE-W8-MULTIPASS-CAPACITY-RECHECK-20261101.md | 独立复核证据范围和 W8 结论 | 不把静态实现或单实例库存算作两实例验收 |
| docs/reports/WBS-DISPATCH-STAR-STABLE-W8-MULTIPASS-CAPACITY-RECHECK-20261101.jsonl | 追加取证、报告、复核和验证记录 | JSONL 每行可解析 |
| shared WBS | 未修改 | 由协调者统一同步 |

## §2 验证摘要
### 本机 Multipass 快照
- multipass version 与 multipassd version 都返回 1.16.4+win。
- multipass list --format json 返回 1 条实例记录：state=Stopped，release=Ubuntu 26.04 LTS；名称不记录。
- 本机库存当前占用 1/2 个测试配额名额，尚未观察到第二个实例或 Deleted 记录。
- 本轮只执行版本与列表查询。没有读取 local.driver，没有启动/停止/创建/删除实例，也没有访问 VM 或查询 K3s 节点。

### 仓库静态门禁
- scripts/automation/multipass_k3s_gitops.ps1 将 MaxTestPhaseInstances 设为 2；Stage 0 要求 Multipass >=1.17.0、driver=hcs，并检查磁盘余量。
- 实例创建前先获取 %LOCALAPPDATA%\Star\multipass-instance-cap.lock 的独占文件句柄，再在同一临界区读取 inventory、检查目标名称/总数并调用 launch；句柄在 finally 中释放。
- Inventory 命令失败、JSON 无法解析、已识别形状中的条目缺少名称/状态、字段空值或重复名称都会失败关闭。解析器支持 Multipass list 数组和 name→state map 两种形状；但当前 `{}` 等未识别的空对象会返回零条，可能被误当作空库存并继续 launch，这是 fail-closed 缺口。
- 上限判断不按运行状态筛选，对 Multipass 返回的所有条目计数；Stopped、Suspended 等状态不会释放名额。目标名称已有 Deleted 记录时直接拒绝，不自动恢复或 purge。
- 该锁只协调同一用户配置目录下使用此脚本的并发执行；它不能阻止用户或其他工具绕过脚本直接调用 Multipass。
- 没有找到专门覆盖两实例上限、Malformed/unknown inventory、Stopped/Deleted 计数和并发锁的自动化测试。脚本有静态门控代码不等于并发测试通过。

### 文档与真实部署
- deploy/multipass/README.md 仍有 1.17.0-rc1 的安装/验证记录和旧测试快照；脚本当前要求稳定版本号至少 1.17.0 与 HCS，文档需统一到已核实的正式版与受支持 host 配置。
- 脚本实际 launch 目标为 Ubuntu 24.04；当前清单中唯一记录为 Ubuntu 26.04 且处于 Stopped。没有验证该 guest 的 K3s、containerd、chart/add-on 或 cloud-init 状态。
- 没有两实例 K3s 创建、容量、升级、恢复日志；没有实际 image/guest BOM 与许可证闭包；没有证据证明 Star Host Infrastructure Manager、双 owner reservations、Run binding、Agent sandbox 或 workload runtime 已通过。

## §3 已知缺口
1. 本机 Multipass 1.16.4+win 低于仓库启动脚本要求的 1.17.0；本轮未查询 driver，无法证明 HCS 配置已就绪。
2. 当前库存只有一个停止的 Ubuntu 26.04 guest；它不是已验证的运行集群，也不构成两实例容量验收。
3. Inventory parser 对 `{}` 等未识别的空对象返回零条；它可能被误当作有效空库存并继续 launch，属于 fail-closed 缺口。该路径也没有专门自动化测试。
4. 没有 cap parser/lock 的自动化并发与失效测试；当前静态判断不能证明跨进程压力下的真实行为。
5. 同一用户的脚本锁不保护直接 CLI 或其他用户的 Multipass 操作。
6. README 的 RC 安装说明和宿主/guest 测试快照未与当前正式版及当前 inventory 同步。
7. 没有两实例 K3s health、升级/回滚、节点恢复、工作负载与资源预算数据；W8 和稳定版仍 OPEN/NO-GO。

## §4 子代理失败接手清单
本 lane 由单一审计 worker 执行，没有派生子代理或待接手结果。未接管 D2 owner 的 Multipass guest，也未读取或修改其他 worktree 的内容。

## §5 守门规则
1. 本机版本证据只来自 Multipass version 输出。
2. 实例数量只来自 multipass list --format json 的本轮响应。
3. 当前返回 1 条记录、状态 Stopped；本轮没有发现第二条或 Deleted 记录。
4. 本轮未启动、停止、创建、删除或 purge 任一实例。
5. 不把本机库存等同于 K3s node/cluster readiness。
6. 不把 cloud-init 脚本存在等同于成功 bootstrap。
7. cap 的静态值为 2，不等同于通过两实例并发测试。
8. 独占文件句柄覆盖脚本内的 inventory 检查与 launch。
9. 锁只协调遵循同一路径锁的脚本进程，不约束直接 CLI 或其他 Windows 用户。
10. Inventory command 非零退出时脚本返回失败，不降级为零实例。
11. 已识别 JSON 形状中的字段、空值、重复名称异常会失败关闭；但 `{}` 当前返回零条，可能作为空库存继续 launch，fail-closed 尚不完整。
12. cap 对所有返回条目计数，不按 Running 状态过滤。
13. 目标实例为 Deleted 状态时拒绝并保留，不自动 recover/purge。
14. 当前 1.16.4+win 小于脚本最低版本 1.17.0；Stage 0 应拒绝启动流程。
15. README 的 RC 说明仍是文档差异，不能作为当前正式版验收。
16. 不把 Ubuntu 26.04 Stopped guest 与脚本要求 Ubuntu 24.04 的新建目标混为同一测试配置。
17. W8 保持 OPEN；未满足两实例 K3s 配置、升级、恢复和容量证据前不得放行稳定版。

## §6 签字栏
| 角色 | 结论 | 责任边界 |
|---|---|---|
| 架构 | Mavis 接手审核：接受静态门禁描述 | 只审查 host/guest/Run 基础设施身份区分 |
| SRE Lead | OPEN | 无两实例 K3s health、升级/恢复及容量数据 |
| 平台 | OPEN | 本机版本低于脚本要求，driver 未复核 |
| 评审主持 | Mavis 接手审核 | 只复核本 lane 证据与范围 |
| PM / Release owner | NO-GO | W8 和 v1.0.6 发布门未通过 |

## §7 修订历史
| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 只读记录 Multipass 版本/库存并静态复核两实例脚本门禁；W8 保持 OPEN。 |
