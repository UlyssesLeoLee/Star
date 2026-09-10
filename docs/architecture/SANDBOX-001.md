# SANDBOX-001 — 子代理 subprocess 沙箱设计 (per plan-031 Phase C sandbox P0)

> **Status**: 🟢 v0.2 active (per 2026-09-10 22:18 JST Mavis 自驱)
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 Mavis 永久代签)
> **作者**: Mavis
> **创建**: 2026-09-10
> **关联 commit**: 待落档
> **关联守门**: #1 fail-open / #5 env 安全 / #6 PowerShell only / #9 子代理 / #11 缺标比错标 / #14 v3 Mavis 代签

---

## 0. 目的

子代理 dispatch (`scripts/automation/dispatcher.py invoke()`) 当前走 subprocess 直接 fork mavis CLI, **无资源限制** — 子代理可能:

1. 占用无限内存 (e.g. cargo build OOM 系统)
2. 占满 CPU (e.g. 跑 100% 永不退出)
3. 派生无限子进程 (fork bomb)
4. 父进程退出后子进程残留 (zombie process tree)

Phase C sandbox P0 给子代理 subprocess 加 **运行时安全护栏**, 跟 5 域 Lead 真人到位前 Mavis 临时代签决策 (per 守门 #14 v3) 配套 — 代签决策 + 沙箱隔离 = 双重安全网。

---

## 1. 设计目标 (Goals)

| # | 目标 | 优先级 | 状态 |
|---|---|---|---|
| G.1 | 内存工作集限制 (per 子代理 process) | P0 | 🟢 v0.2 |
| G.2 | CPU 速率限制 (跟 dispatcher 共享 CPU) | P0 | 🟢 v0.2 |
| G.3 | 进程数限制 (含子进程派生) | P1 | 🟢 v0.2 |
| G.4 | 父进程退出级联 kill (cascading) | P0 | 🟢 v0.2 |
| G.5 | 跨平台 (Windows + POSIX) | P2 | 🟡 v0.2 仅 Windows, v0.3 加 POSIX rlimit |
| G.6 | 网络隔离 | P2 | 🔴 不实装 (per 已知缺口 #1) |
| G.7 | 文件系统隔离 | P2 | 🔴 不实装 (per 已知缺口 #2) |
| G.8 | fail-open (沙箱自身错误不阻断 dispatch) | P0 | 🟢 v0.2 |

---

## 2. 选型 (Solution Choices)

### 2.1 Windows 候选: Job Objects (per 守门 #1 fail-open + 守门 #6 跨平台)

**Job Objects** (Windows API) 提供:
- 进程组隔离 (job 内的进程)
- 资源限制 (内存 + CPU + 进程数)
- 级联 kill (JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)
- 实时通知 (completion port)

**pywin32 绑定**: `win32job` module, 在 Windows + Python 3.x 100% 可用。

**优势**:
- OS 内核级, 子进程无法 bypass
- 跟 Windows 调度器集成, 资源限制硬实时
- 已落地 `pywin32 306+` 版本, 0 额外依赖

**劣势**:
- 仅 Windows (Linux 用 cgroups, macOS 用 sandbox-exec)
- 非 elevated 进程可能因 Access Denied 失败 (per 已知缺口 #4)

### 2.2 POSIX 候选: rlimit / cgroups (v0.3 跨平台, per 已知缺口 #3)

**rlimit** (POSIX resource limits):
- 内存限制: `RLIMIT_AS` (address space)
- CPU 时间: `RLIMIT_CPU` (per-process CPU seconds)
- 进程数: `RLIMIT_NPROC` (Linux only)

**cgroups** (Linux control groups):
- 跟 Job Objects 等价, Linux 内核级隔离
- 复杂 API, 需要 cgroups v1 / v2 配置

**v0.2 不实装**: 跨 session 续做 v0.3 (per 守门 #6 跨平台)。

### 2.3 替代方案: subprocess 限制 (降级, per 守门 #1 fail-open)

`subprocess.run` 自带 timeout, 但:
- 无内存限制
- 无 CPU 限制
- 无级联 kill (subprocess.run 只能 kill 直接子进程, 孙子进程残留)

**v0.2 降级路径**: 沙箱不可用时 → 走 `subprocess.run` 路径, 仅依赖 timeout。

---

## 3. 设计方案 (Solution)

### 3.1 SandboxLimits 数据类

```python
@dataclass
class SandboxLimits:
    memory_mb: int = 0           # 0 = 不限
    cpu_percent: int = 0         # 0 = 不限 (1-100)
    max_processes: int = 0       # 0 = 不限
    kill_on_parent_exit: bool = True
```

### 3.2 Windows Job Object 配置

| Sandbox 字段 | Job Object API | Job Object Flag | 默认值 |
|---|---|---|---|
| `memory_mb > 0` | `ProcessMemoryLimit` | `JOB_OBJECT_LIMIT_PROCESS_MEMORY` | 1024 MB |
| `cpu_percent > 0` | `JobObjectCpuRateControlInformation` | (Win 8+) | 50% |
| `max_processes > 0` | `ActiveProcessLimit` | `JOB_OBJECT_LIMIT_ACTIVE_PROCESS` | 100 |
| `kill_on_parent_exit` | (flag) | `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` | True |

### 3.3 子进程分配 (per 守门 v0.2 §3.3)

```python
import win32api
import win32job

h_process = win32api.OpenProcess(
    win32job.JOB_OBJECT_ASSIGN_PROCESS | win32job.JOB_OBJECT_TERMINATE,
    False,
    pid,
)
win32job.AssignProcessToJobObject(job, h_process)
```

**Access Denied (err 5) 处理** (per 守门 #1 fail-open): 非 elevated 进程可能无法 AssignProcessToJobObject, **不阻断 dispatch**, 仅 warn log。

### 3.4 run_with_sandbox 上下文管理器

```python
@contextmanager
def run_with_sandbox(cmd, *, timeout, cwd, limits):
    if not is_available():
        # 走 subprocess.run 路径 (per 守门 #1 fail-open)
        ...
    
    job = _create_windows_job(limits)
    try:
        proc = subprocess.Popen(cmd, ...)
        _assign_process_to_job(job, proc.pid)
        stdout, stderr = proc.communicate(timeout=timeout)
        yield _SandboxResult(proc.returncode, stdout, stderr)
    except subprocess.TimeoutExpired:
        # Job close 触发级联 kill
        proc.kill()
        raise
    finally:
        win32api.CloseHandle(job)  # 触发 KILL_ON_JOB_CLOSE
```

### 3.5 默认沙箱限制 (per 子代理 mavis task dispatch 30 min timeout 场景)

```python
def make_default_sandbox_limits() -> SandboxLimits:
    return SandboxLimits(
        memory_mb=1024,      # 1 GB, 适配 cargo build
        cpu_percent=50,      # 1 核的 1/2
        max_processes=100,    # cargo + rustc + test runner
        kill_on_parent_exit=True,
    )
```

---

## 4. 跟现有守门关系 (per 守门 #11 缺标比错标 显式列)

| 守门 | 跟 sandbox 联动 |
|---|---|
| **#1 fail-open** | 沙箱自身错误 (Access Denied / import 失败) → 走 subprocess.run 路径, 不阻断 |
| **#5 env 安全** | 沙箱不读 env, 0 泄露风险 |
| **#6 PowerShell only / 跨平台** | v0.2 仅 Windows (pywin32 + Job Objects), v0.3 加 POSIX rlimit |
| **#9 子代理 dispatch** | 沙箱是 dispatch 的运行时隔离层, 跟 brief 必先落档联动 |
| **#11 缺标比错标** | 5 已知缺口显式列 (network / fs / POSIX / mavis CLI / observability) |
| **#13 T append-only** | 沙箱不写 audit log (per 已知缺口 #5), 仅给 handle 加 `invoke_subprocess_sandboxed` action |
| **#14 v3 Mavis 永久代签** | 沙箱给代签决策加运行时安全护栏, 双重安全网 |
| **v35 ed25519 签名** | 沙箱不影响留言签名 (留言在 dispatcher 层, 沙箱在 subprocess 层) |
| **v36 audit log 索引** | 沙箱的 action="invoke_subprocess_sandboxed" 进 audit log, 走 v36 索引 |

---

## 5. 落地清单 (Deliverables)

| # | 文件 | 改动 | 估 LOC |
|---|---|---|---|
| 1 | `docs/architecture/SANDBOX-001.md` | 新增 (本文件) | 230 |
| 2 | `scripts/automation/guardian/sandbox.py` | 新增 (SandboxLimits + WindowsJobSandbox + run_with_sandbox + is_available + make_default_sandbox_limits) | 220 |
| 3 | `scripts/automation/dispatcher.py` | `invoke()` 集成 sandbox (use_sandbox=True 默认 + sandbox_limits 可选 + run_with_sandbox 替代 subprocess.run) | +30 |
| 4 | `scripts/automation/guardian/tests/test_sandbox.py` | 7 TC class / 9 TC | 200 |
| **总计** | **4 文件** | **1 commit 多文件** (per #1 v15) | **~680 LOC** |

**v0.2 实测**: 297 tests pass 0 回归 (288 旧 + 9 sandbox new, 1 skip 在非 elevated 进程 sandbox 不可用)

**估 token**: ~0.2M, ~20 min (实测 v0.2 落地)

---

## 6. 已知缺口 (per 缺标比错标)

| # | 缺口 | 严重度 | 缓解 |
|---|---|---|---|
| 1 | Network isolation: Windows Filtering Platform 复杂, v0.2 不实装 (0 跨网访问保护) | P2 | v0.3 走 WFP / Windows Firewall API |
| 2 | File system isolation: AppContainer / restricted token 复杂, v0.2 不实装 (走 chmod + W 工作目录) | P2 | v0.3 走 AppContainer |
| 3 | POSIX rlimit 兜底: v0.2 不实装, 跨平台 v0.3 (per 守门 #6 跨平台) | P2 | v0.3 macOS / Linux 兜底 |
| 4 | mavis CLI 尚未落地: dispatcher.invoke() 当前走 status="deferred" fallback (per 守门 #9 #2), sandbox 是 forward-looking 基础设施 | P1 | mavis CLI 落地后立刻接入 |
| 5 | 沙箱自身的 metric/observability: v0.2 不实装, v0.3 加 psutil + audit log 联动 (per 守门 #13) | P2 | v0.3 加 psutil / 沙箱 metric 写 audit log |
| 6 | 非 elevated 进程 Access Denied: 在某些 Windows 配置下, AssignProcessToJobObject 失败 (per 守门 #1 fail-open) | P1 | fail-open 仅 warn log; 跨 session 续做 UAC 提权或用 SeAssignPrimaryTokenPrivilege |

---

## 7. 修订履歴

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 22:18 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** (per 守门 #14 v4 反转 v0.62 + 9/8 15:19 第 6 次强化 Mavis 全权代理) | 初版落档, 7 段 (目的/选型/方案/守门/落地/缺口/修订), 4 落地文件 ~680 LOC, Windows Job Objects 隔离 + 4 维限制 (memory/cpu/procs/cascading kill) + fail-open 跨平台降级, 跟守门 #1+#6+#9+#11+#14 v3 联动, 6 已知缺口 (network/fs/posix/mavis/observability/elevated) | 2026-09-10 22:18 JST Mavis 自驱 (per plan-031 Phase C + 守门 #9 v19 + 守门 #14 v3 + 9/8 15:29 自驱强化) |
