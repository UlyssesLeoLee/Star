# STAR Stable W8 Multipass/K3s Capacity Review

## 审查范围
复核 W8 lane 在 dev@3726c977 的 W8 phase report、W8 brief 与 dispatcher JSONL。检查范围限于本机 Multipass version/list 输出及仓库脚本和文档静态内容。

## 证据核对
| 项目 | 结论 | 限定 |
|---|---|---|
| Multipass daemon/CLI | 1.16.4+win | 未检查 driver |
| 实例 inventory | 1 条 Stopped Ubuntu 26.04 LTS 记录 | 没有访问 guest 或 K3s |
| 测试上限 | 脚本常量为 2，并在 inventory 与 launch 周围持有独占文件句柄 | 仅约束遵循该脚本的同一用户进程 |
| Fail-closed | 已识别形状的解析/字段错误会失败；`{}` 未识别空对象会返回 0 条 | 可误当空库存继续 launch；缺少专门自动化测试 |
| 状态计数 | 根据全部返回行计数；不按状态过滤，Deleted 同名实例拒绝且不 purge | 本轮只观察到一个 Stopped 记录 |
| K3s/Star infra | 无两实例集群、升级/恢复、owner reservation 或 sandbox 运行证据 | W8/Release 保持 OPEN/NO-GO |

## 发现
1. 当前 Multipass 1.16.4+win 低于脚本最低版本 1.17.0；README 仍保留 RC1 安装/测试文案。
2. 当前唯一 inventory 条目是停止的 Ubuntu 26.04；脚本新建目标使用 Ubuntu 24.04，未证明两者中的任一已运行 K3s。
3. 静态实现对脚本内并发提供独占文件锁，但 parser 对 `{}` 等未识别空对象返回零条，可能作为空库存继续 launch；因此 inventory 并非所有不明形状都 fail closed。该锁也不会限制直接 Multipass CLI 或其他用户。
4. 缺少 cap parser、Stopped/Deleted 计数、锁争用与两实例容量的自动化测试。
5. 没有 K3s 健康、工作负载、升级恢复、image/BOM/许可或 Star 原生基础设施执行证据。

## 验证
- 本机只读 multipass version 与 multipass list --format json 查询成功。
- 静态检查了 multipass_k3s_gitops.ps1 的版本/driver门、inventory解析、exclusive file lock、计数与不自动 purge分支。
- 文档 diff check、报告 §0–§7 结构和 dispatcher JSONL 解析通过；未运行 VM 生命周期操作或产品测试。

## 结论
ACCEPT WITH LIMITATIONS 作为 W8 只读审计报告；W8 仍 OPEN，稳定版仍 NO-GO。不得把脚本静态门禁或单个停止 VM 记录作为两实例 K3s 验收。
