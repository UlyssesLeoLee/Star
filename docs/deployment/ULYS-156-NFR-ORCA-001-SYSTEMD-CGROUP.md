# ULYS-156 / NFR-ORCA-001 — systemd cgroup 边界 (KillMode=mixed 风险)

> 配套 issue: [ULYS-156](https://app.multica.ai/issue/01a0bf6b-486c-71b8-99f4-92bde909c7f8)
> spec 锚点: `docs/ecosystem-survey/orca-design-survey.md` §2 NFR-ORCA-001(行 158-162)
> 父 issue: [ULYS-104](https://app.multica.ai/issue/01a0b920-28d5-7dc8-93c2-9cbc73ba9c4d)
> 状态: ✅ 落档(2026-09-23 JST, P0-A 续切片的文档产物)

## 1. 问题陈述

Orca spec §2 NFR-ORCA-001 指出:当 `domain-local-runtime` 以 systemd 单元运行时,父
runtime 升级/重启会向主进程发 SIGTERM;**默认 `KillMode=mixed` 会让主进程的所有
子进程(包括 CLI agent 子进程,如 `claude` / `codex`)继承 SIGTERM** → 用户 agent
会话被无差别强杀,违背 §A "升级/重启保持 Agent 会话"目标。

## 2. 风险面

| 风险 | 触发条件 | 后果 |
|---|---|---|
| 主进程 SIGTERM 串到子进程 | systemd 默认 `KillMode=mixed` + agent CLI spawn 未独立 cgroup | agent 会话被强杀,scrollback 丢失,违反 FR-ORCA-001 AC-1 |
| 主进程 SIGKILL 留下孤儿 | OOM kill / kernel panic / 主机重启 | child PID 仍在 OS 上跑,下次 attach 时 PID 已被回收复用 → 锁失效(FR-ORCA-004 直接命中) |
| 子进程 setpgid/setsid 失败 | 平台不支持或权限不足(Windows Job Object 缺失) | 跨平台一致性退化 |

## 3. systemd 单元推荐配置(单进程持久化层适配,§A 锁定路径)

> 本仓库采用**单进程**模型(per D-Boy 9/22 JST §A 决策)。systemd 单元配置按
> "主进程独立 cgroup + 子进程不继承 SIGTERM" 原则:

```ini
[Unit]
Description=Star Local Runtime
After=network-online.target
Wants=network-online.target

[Service]
Type=notify
ExecStart=/usr/local/bin/star-local-runtime
# 关键 1: 主进程收到 SIGTERM 时,只杀主进程,不串到子进程(子进程由
# graceful_shutdown 显式 cancel)
KillMode=process
# 关键 2: 给主进程 + 全部子进程分配独立 cgroup,便于 kill 边界控制
Delegate=yes
# 关键 3: 子进程归属主进程的 cgroup,但 settings 不让 cgroup 范围溢出 systemd 视图
CPUAccounting=yes
MemoryAccounting=yes
TasksAccounting=yes
# 关键 4: 关闭 OOM 串到子进程(子进程走 §A graceful shutdown 路径,不被 OOM kill 误伤)
OOMPolicy=continue
# 关键 5: 启动超时与重启策略(防止 crash-loop,NFR-ORCA-002 配套)
TimeoutStartSec=30s
TimeoutStopSec=10s
# NFR-ORCA-002 配合:启动频率限流,6 次 restart 才视为 failed
Restart=on-failure
RestartSec=10s
StartLimitIntervalSec=60s
StartLimitBurst=5
# 关键 6: 主进程 + 子进程的 cgroup 路径可观测,便于 ops 排错
Slice=star-local-runtime.slice
```

## 4. 平台适配矩阵

| 平台 | 进程组隔离 | 备注 |
|---|---|---|
| Linux | `setsid(2)` + `prctl(PR_SET_PDEATHSIG, ...)` | 本期由 `process_supervisor` 准备,真实 spawn 适配留 P1 (per 9/22 §A 决策已知 trade-off) |
| macOS | `setpgid(0, 0)` + `posix_spawn` attr | 与 Linux 同源,代码路径可复用 |
| Windows | Job Object (`CREATE_NEW_PROCESS_GROUP` + `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) | 本期留 P1 (per cli_spawn.rs 注释 "AC-1 子进程存活已在 PR #63 efa501a3 落地") |

## 5. 与 graceful_shutdown 模块的契约

| 触发源 | shutdown 路径 |
|---|---|
| systemd 发送 SIGTERM 到主进程 | `GracefulShutdown::shutdown_all` 串行 cancel 活跃 CliSession → release 全部 (session, pid) lock → 子进程 kill stub(本期 stub,真实 SIGTERM 留 P1) |
| 用户 Stop All / UI 触发 | 同上,通过 `register_handler` 走相同 shutdown 路径 |
| systemd OOM kill (OOMPolicy=continue) | 主进程被 SIGKILL,但 `KillMode=process` 保证子进程不被串杀;下次启动时由 `cli_session_registry` 恢复,`process_supervisor` history 清零(per INV-SUP-03 已知 trade-off) |
| systemd TimeoutStopSec 触发 | SIGKILL 到主进程;子进程仍因 KillMode=process 不被杀;但失去 orchestrator → 孤儿 → 下次启动由 `cli_session_lock::verify_pid_start_time` 校验(FR-ORCA-004) |

## 7. 验收(per v1.0 §16 落地检查表)

| ID | 状态 | 备注 |
|---|---|---|
| NFR-ORCA-001 AC-1:KillMode=mixed 风险已被识别 + 文档化 | ✅ | 本文件 |
| NFR-ORCA-001 AC-2:推荐 systemd 单元配置示例 | ✅ | §3 |
| NFR-ORCA-001 AC-3:跨平台适配矩阵 | ✅ | §4 |
| NFR-ORCA-001 AC-4:与 graceful_shutdown 契约 | ✅ | §5 |
| NFR-ORCA-001 AC-5:Windows Job Object P1 followup | 🟡 P1 | per cli_spawn.rs 注释 + 9/22 §A 决策 |
| NFR-ORCA-001 AC-6:实战压测(50 launches 验证 cgroup 边界) | ❌ P1 | 需 CI 集成 systemd-run / podman,与 §A MVP 边界外 |

## 8. P1 followup 提案

| 项 | 工时 | 触发条件 |
|---|---|---|
| Windows Job Object 真实 spawn 适配 | ~1 周 | 用户在 Windows 桌面端首次使用 ULYS-156 |
| systemd-run 实战压测 CI 集成 | ~0.5 周 | P1 集成测试阶段 |
| cgroup v2 unified hierarchy 迁移指南 | ~0.5 周 | 目标 OS 升级到 cgroup v2 only (Ubuntu 22+ / Debian 12+) |

---

> 落档于 2026-09-23 JST, ULYS-156 P0-A 续切片文档产物。
> 本文档不实装代码,与 `cli_session_lock.rs` / `process_supervisor.rs` /
> `graceful_shutdown.rs` / `health_self_test.rs` 同 PR #76 续 落地。