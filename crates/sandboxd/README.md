# sandboxd — Sandbox-as-a-Service 独立 daemon (WBS 6.5.2)

Sandbox-as-a-Service daemon: 独立进程 + gRPC IPC (4 RPC) + 4 维隔离 (resource / network / fs / capability) + session 池化 + 跨 client 复用。

- 要件定义: `docs/requirements/SRS-SANDBOX-002.md` v0.1.2 (决策点 D-1~D-10 已拍板)
- 基本设计: `docs/basic-design/SANDBOX-BASIC-DESIGN-002.md` v0.1.2
- 详细设计: `docs/detailed-design/DD-SANDBOX-002.md` v0.1.1
- 架构视图: `docs/architecture/SANDBOX-002.md` v0.1.1

## 快速开始

```bash
cargo run -p sandboxd                     # 默认配置 (gRPC 50051, HTTP /metrics+/healthz 50052)
cargo run -p sandboxd -- --config config/sandboxd.yaml
cargo test -p sandboxd
```

`/healthz`: `curl http://127.0.0.1:50052/healthz`
`/metrics`: `curl http://127.0.0.1:50052/metrics` (Prometheus scrape)

## 4 RPC (proto/sandboxd.proto)

- `CreateSession(client_id, task_id, parent_session_id, policy, timeout_sec)` → session_id
- `RunCommand(session_id, command, timeout_sec)` → returncode/stdout/stderr/latency_ms/hit_violations
- `DestroySession(session_id, reason)` → Destroyed/NotFound
- `StreamLogs(session_id, from_offset)` → stream LogEntry (1000 条环形缓冲)

## 4 维隔离 (backend/)

| 平台 | resource | network | fs | capability |
|---|---|---|---|---|
| Windows | Job Objects (win32job safe) | policy+audit (WFP v0.2) | policy+audit (AppContainer v0.2) | spawn 拦截 + 隐式 drop |
| Linux | cgroups v2 | netns + iptables | mount ns (unshare) | capsh drop |
| macOS | policy+audit | sandbox-exec network filter | sandbox-exec file filter | implicit drop |

## fail-open / fail-closed

- sandboxd 不可用 → client `run_with_failover` 走 SANDBOX-001 v0.2 subprocess 降级 (FR-7.1)
- audit log 写失败 → BLOCK session 创建 (FR-7.2)

## 数据模型 (migrations/, 守门 #13 W/T/M)

2 T (sandbox_session / sandbox_audit) + 2 M (sandbox_policy / sandbox_capability), 0 W, RLS 13 類必携。
