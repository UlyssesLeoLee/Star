# guard-pre-tool-use 实施 spec

> **状态**: Draft v0.1 (2026-09-10 JST 初版落档)
> **上游依赖**:
> - 《Requirements》[`docs/requirements/SRS-PRE-TOOL-USE-GUARD-001.md`](../requirements/SRS-PRE-TOOL-USE-GUARD-001.md) v0.1 (7 機能 / 4 業務 / 6 非機能 / 7 验收)
> - 《Basic Design》[`docs/design/BD-PRE-TOOL-USE-GUARD-001.md`](../design/BD-PRE-TOOL-USE-GUARD-001.md) v0.1 (5 module / 3 表 / 7 集成点)
> - 《Detailed Design》[`docs/detailed-design/DD-PRE-TOOL-USE-GUARD-001.md`](../detailed-design/DD-PRE-TOOL-USE-GUARD-001.md) v0.1 (10 段 / 10 文件 / 4 依赖)
> - 《WBS》[`docs/plans/WBS-002-pre-tool-use-guard.md`](../plans/WBS-002-pre-tool-use-guard.md) v0.1 (实施计划 + WBS 12 task)
> **下游交付**: Implementation team — Python 模块路径 `scripts/automation/guardian/`
> **核心语言**: Python 3.10+ (per SRS §6.1)
> **目标读者**: 实装工程师 (Mavis 自驱 / 子代理 worker)
> **最后审稿**: 2026-09-10 19:18 JST Mavis 自驱 (per 守门 #14 v3 Mavis 永久代签)
> **修订人**: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 守门 #10 + 守门 #14 v3)
> **审批**: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62)

---

## 1. 职责与边界

`guard-pre-tool-use` 承担三重职责 (§2.1):**PreToolUse hook 接入** + **危险模式匹配** + **审计 log 持久化**。对上承接 mavis runtime PreToolUse 事件, 对下接入 `console_server.py` / `dispatcher.py`, 对外暴露 3 个内部 API (evaluate / reload_rules / query_audit)。

**属于本模块的** (per SRS FR-1~FR-7, BD PM-1~PM-5):
- PreToolUse hook 接入点 (per FR-1.1)
- 子代理 dispatch 前置接入点 (per FR-1.2)
- 危险模式库 (15 条, BLOCK 8 + ASK 5 + WARN 2, per FR-3.2/3.3/3.4)
- 三级决策 (BLOCK/ASK/WARN, per FR-2.1/2.2/2.3)
- 审计 log (JSON Lines, append-only, per FR-4.1/4.2)
- 规则配置 (JSON Schema + 热更新, per FR-5.1)
- 旁路机制 (fail-open / fail-closed, per FR-6.1/6.2)

**不属于本模块的** (per SRS §1.4 + 已知缺口):
- ❌ 网络层拦截 (egress filtering) → v2.x
- ❌ AI 行为审计 (LLM 决策录屏) → v2.x
- ❌ 子代理内部代码越权 (subprocess sandbox) → v0.2 拍摄 (SRS 已知缺口 #1 P0 阻塞)
- ❌ 加密 / 凭据管理 (mavis 内置 vault) → 跨项目需求
- ❌ User-defined rule 实装 → v0.3 拍摄 (本 spec 仅 stub, per §2.4)

---

## 2. 关键实体

引用 [`DD-PRE-TOOL-USE-GUARD-001.md` §5 数据模型](../detailed-design/DD-PRE-TOOL-USE-GUARD-001.md#5-数据持久化-data-persistence-详细):

### 2.1 ToolCall (PreToolUse hook 入参)

```python
@dataclass(frozen=True)
class ToolCall:
    tool_name: str          # "bash" / "write" / "edit" / "read" / "mcp_*"
    tool_args: Dict[str, Any]
    session_id: str
    agent_role: str         # "orchestrator" / "worker" / "explore" / "verifier"
```

**约束**:
- `tool_name` 必填, 取值 5 种之一
- `tool_args` 必填 dict, 字段依 tool_name 而定
- `session_id` 必填, 来自 mavis runtime
- `agent_role` 必填, 5 种之一

### 2.2 Rule (per `pre_tool_use_rules.json`)

```python
@dataclass(frozen=True)
class Rule:
    id: str                 # 必填, 匹配 R-(BLOCK|ASK|WARN)-\d{3}
    name: str               # 必填, 1-100 chars
    level: str              # 必填, BLOCK|ASK|WARN
    tool_class: str         # 必填, bash|write|edit|read|mcp
    pattern: str            # 必填, 1-500 chars, regex
    reason: str             # 必填, 1-300 chars
```

**15 条规则** (per DD §5.1):
- BLOCK 8 条: R-BLOCK-001~008
- ASK 5 条: R-ASK-001~005
- WARN 2 条: R-WARN-001~002

### 2.3 AuditEvent (per `pre_tool_use_audit.log`)

```python
@dataclass(frozen=True)
class AuditEvent:
    ts: str                 # ISO 8601, 必填
    session_id: str         # 必填
    agent_role: str         # 必填
    tool: str               # 必填
    tool_args_hash: str     # sha256:hex, 必填 (脱敏)
    args_excerpt: str       # <= 200 chars, 必填 (脱敏)
    decision: Decision      # BLOCK|ASK|WARN|PASS, 必填
    rule_id: Optional[str]  # decision != PASS 时必填
    reason: Optional[str]   # decision != PASS 时必填
    latency_ms: float       # 必填
    env_hash: str           # sha256 of all env keys, 必填 (脱敏)
```

**关键约束** (per 守门 #5):
- `tool_args_hash` 仅写 sha256, 不写 tool_args 完整内容
- `env_hash` 仅写 env 键的 sha256, 不写 env 实际值
- `args_excerpt` 仅前 200 chars, 不含 secret

### 2.4 SessionState (per `pre_tool_use_session_state.json`)

```python
@dataclass
class SessionState:
    schema_version: str             # "1.0"
    session_id: str
    started_at: str
    last_reload_ts: str
    rules_loaded_count: int
    rules_load_fail_count: int
    last_block_rule_id: Optional[str]
    decision_count: Dict[str, int]  # BLOCK/ASK/WARN/PASS 计数
```

**更新时机**: 每 1 min 聚合 (per NFR-O-1) + 规则 reload 时 + 进程退出时

---

## 3. 公共 API 设计 (per DD §2 + BD §5.1)

### 3.1 `PreToolUseGuard.evaluate(tool_call) -> Decision`

**主入口**, mavis runtime PreToolUse hook 调用。

**契约** (per FR-1.1 + NFR-P-1/NFR-P-2 + FR-6.1/6.2):
- 延迟 p50 < 5ms, p99 < 10ms
- 规则加载失败 → fail-open (PASS + WARN log)
- audit 写失败 → fail-closed (BLOCK)
- 必写 audit log (无论 decision)

**Python 签名**:
```python
def evaluate(self, tool_call: ToolCall) -> Decision:
    """PreToolUse hook 入口."""
```

**调用示例**:
```python
from scripts.automation.guardian.pre_tool_use_guard import (
    PreToolUseGuard, ToolCall, Decision
)
from scripts.automation.guardian.rule_database import RuleDatabase
from scripts.automation.guardian.audit_logger import AuditLogger
from pathlib import Path

rule_db = RuleDatabase(Path("scripts/automation/guardian/rules/pre_tool_use_rules.json"))
audit = AuditLogger(Path("scripts/automation/guardian/logs/pre_tool_use_audit.log"))
guard = PreToolUseGuard(rule_db, audit)

tool_call = ToolCall(
    tool_name="bash",
    tool_args={"command": "rm -rf /tmp/xxx"},
    session_id="mvs_xxx",
    agent_role="orchestrator",
)
decision = guard.evaluate(tool_call)
# decision ∈ {Decision.BLOCK, Decision.ASK, Decision.WARN, Decision.PASS}
```

### 3.2 `PreToolUseGuard.evaluate_for_dispatch(...) -> Decision`

**子代理 dispatch 前置入口**, `dispatcher.py` invoke() 前调用。

**契约** (per FR-1.2 + NFR-S-4):
- 延迟必 < 5ms
- 100% line coverage (NFR-S-4 单元测试)
- BLOCK 决策 → dispatcher.py 抛 `DispatchBlocked` 异常

**Python 签名**:
```python
def evaluate_for_dispatch(
    self,
    task_id: str,
    script_path: str,
    args: Dict[str, Any],
    parent_session_id: str
) -> Decision:
    """子代理 dispatch 前置入口."""
```

**调用示例** (per DD §1.3 集成点):
```python
# 在 dispatcher.py invoke() 前插入
def invoke(task_id, script_path, args, parent_session_id):
    decision = guard.evaluate_for_dispatch(
        task_id=task_id,
        script_path=script_path,
        args=args,
        parent_session_id=parent_session_id,
    )
    if decision == Decision.BLOCK:
        raise DispatchBlocked(decision.reason, decision.rule_id)
    if decision == Decision.ASK:
        # 走 ask_user (per 守门 v28)
        ...
    # PASS → 继续 invoke
    ...
```

### 3.3 `RuleDatabase.reload_rules() -> ReloadResult`

**热更新入口**, 文件 mtime 变化时自动触发 (watchdog), 也可手动调用。

**契约** (per FR-5.1 + NFR-M-2):
- 延迟 < 100ms (15 条规则)
- 不中断 mavis runtime
- 失败 → 保留旧 rules + WARN log (per FR-6.1)

**Python 签名**:
```python
def reload_rules(self) -> ReloadResult:
    """重新加载规则 JSON, 不重启 mavis runtime."""
```

### 3.4 `AuditLogger.query(filters) -> List[AuditEvent]`

**查询入口**, 供调试 / 报告使用。

**契约** (per BD §5.1.3):
- 支持 session_id / decision / rule_id / start_ts / end_ts 过滤
- 限制 limit (默认 100)
- 失败 → 返回空 list + WARN log

**Python 签名**:
```python
def query(
    self,
    session_id: Optional[str] = None,
    decision: Optional[str] = None,
    rule_id: Optional[str] = None,
    start_ts: Optional[str] = None,
    end_ts: Optional[str] = None,
    limit: int = 100
) -> List[AuditEvent]:
    """查询 audit log."""
```

---

## 4. 实现要点 (Implementation Notes)

### 4.1 跨平台 path 处理 (per SRS §5 NFR-T-3 + DD §1.3)

**问题**: Windows `C:\Users\xxx\.ssh\id_rsa` vs POSIX `/home/xxx/.ssh/id_rsa`

**方案**:
```python
from pathlib import Path, PureWindowsPath, PurePosixPath

def normalize_path_for_match(path_str: str) -> str:
    """跨平台 path 归一化.
    
    1. 用 pathlib.Path 处理 (自动平台)
    2. 替换 \\ 为 / (regex 兼容)
    3. 展开 ~ 和 $HOME (per R-BLOCK-001)
    """
    if not path_str:
        return ""
    try:
        # 替换 \\ 为 / (regex 兼容)
        normalized = path_str.replace("\\", "/")
        # 展开 ~ 和 $HOME
        expanded = str(Path(normalized).expanduser())
        return expanded
    except (OSError, ValueError):
        return path_str  # 退化为 raw string (per F-8)
```

### 4.2 regex 编译缓存 (per NFR-P-1)

**问题**: 每次 evaluate 都 re.compile() 15 条规则 = 浪费

**方案**:
```python
import re
from functools import lru_cache

@lru_cache(maxsize=128)
def _compile_pattern(pattern: str) -> re.Pattern:
    """缓存编译后的 regex, per NFR-P-1 < 5ms p50."""
    return re.compile(pattern)
```

### 4.3 thread-safe 规则读取 (per DD §2.2 RLock)

**问题**: hook 可能在多线程调用, 规则加载和读取必须互斥

**方案**: `RuleDatabase` 用 `threading.RLock` 保护 `self._rules`, 读时拷贝避免外部修改 (per DD §2.2)

### 4.4 audit log 异步写 vs 同步写 (per NFR-P-3)

**问题**: 同步写阻塞主流程, 异步写复杂且难保证 fail-closed

**决策**: **同步写 + lazy fsync** (per DD §2.3)
- 不 await / 不 spawn thread
- `f.flush()` 不 `os.fsync()`, OS 会 lazy fsync (per NFR-P-3 < 1ms)
- 写失败立即 raise, 不会延迟到主流程之外

### 4.5 watchdog vs mtime poll (per FR-5.1 + F-5)

**方案**: watchdog 优先, mtime poll 兜底 (per DD §2.2)

```python
def _start_watcher(self):
    try:
        # 优先 watchdog
        self._observer = Observer()
        self._observer.schedule(...)
        self._observer.start()
    except Exception as e:
        # 退化为 mtime poll
        logger.warning("watchdog failed, fallback to mtime poll: %s", e)
        # TODO v0.2: 启动 mtime poll thread
```

### 4.6 守门 #5 联动 (per NFR-S-5)

**设计**: R-BLOCK-006 (env 打印) 必须在所有 bash tool call 上命中, 跟守门 #5 review guard 互补
- 守门 #5: 事后审计, 发现违规 → 报告
- PreToolUse: 事前阻断, 发现违规 → 直接拒绝 tool call

**测试**: 单元测试覆盖 R-BLOCK-006 15 个变种 (`Get-ChildItem env: | Format-Table` / `env | grep` / `printenv | head` / `set | grep`)

### 4.7 守门 v28 联动 (ASK 推荐项 "取消")

**设计**: ASK 决策时, mavis runtime 自动生成 ask_user 选项, **推荐项必放"取消"** (per 守门 v28)

**实现**: 不在本 spec 内, 由 mavis runtime PreToolUse 事件消费方实现。本 spec 仅返回 `Decision.ASK` + `rule_id` + `reason`。

---

## 5. 测试要求 (per DD §7 + SRS §7 AC-1~AC-7)

### 5.1 必含测试 (实装完成时, 100% pass)

| 测试类 | 文件 | 数量 | 覆盖率 |
|---|---|---|---|
| 单元测试 - Pattern Matcher | `tests/automation/guardian/test_pre_tool_use_guard.py` | 10+ | 100% line |
| 单元测试 - Rule Database | `tests/automation/guardian/test_rule_database.py` | 8+ | 100% line |
| 单元测试 - Audit Logger | `tests/automation/guardian/test_audit_logger.py` | 5+ | 100% line |
| 性能测试 - benchmark | `tests/automation/guardian/test_pre_tool_use_guard.py::test_p50_latency` | 2 | 阈值 |
| 集成测试 - console_server | `tests/e2e/test_pre_tool_use_hook.py::test_console_server_hook_real` | 3 | 100% path |
| 集成测试 - dispatcher | `tests/e2e/test_pre_tool_use_hook.py::test_dispatcher_hook_real` | 2 | 100% path |
| 渗透测试 | `tests/security/test_pre_tool_use_penetration.py` | 150 (15 规则 × 10 攻击) | 0 命中 |
| 跨平台测试 - Windows | CI workflow | 必跑 | 100% pass |
| 跨平台测试 - POSIX | CI workflow | 必跑 | 100% pass |

**总计**: 30+ test case + 150 渗透, pytest coverage ≥ 90% (per NFR-M-4)

### 5.2 必跑检查 (CI 门禁)

```bash
# 单 crate (本项目为 Python, 改 pytest)
pytest tests/automation/guardian/ -v --cov=scripts/automation/guardian --cov-report=term-missing
# 期望: 100% pass, coverage ≥ 90%

# 端到端 (含 subprocess 启动 console_server)
pytest tests/e2e/test_pre_tool_use_hook.py -v
# 期望: 100% pass

# 跨平台 (CI 必跑)
# Windows: pwsh -c "pytest ..."
# POSIX: bash -c "pytest ..."
```

### 5.3 报告要求 (per 守门 #12 docs 同步)

实装完成后, **必**写 `docs/reports/PHASE-PRE-TOOL-USE-GUARD-IMPL-REPORT.md` (跟现有 6 份 PHASE-*-IMPL-REPORT 一致), 含:
- §0 目的
- §1 任务完成矩阵 (per WBS-002 12 task)
- §2 验证摘要 (pytest 100% pass + coverage ≥ 90% + 跨平台 CI 全过)
- §3 已知缺口 (per 缺标比错标)
- §4 子代理失败接手 (如有)
- §5 守门规则 (15-17 项)
- §6 签字栏 (5 角色)
- §7 修订历史

---

## 6. 依赖 (per DD §1.2)

### 6.1 运行时依赖 (2 项)

```toml
# scripts/automation/pyproject.toml (新增)
[project]
name = "star-automation-guardian"
version = "0.1.0"
requires-python = ">=3.10"
dependencies = [
    "jsonschema>=4.20",   # 规则 schema 校验
    "watchdog>=3.0",      # 规则文件 mtime 监听
]
```

### 6.2 开发依赖 (2 项, dev 安装时)

```toml
[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "pytest-benchmark>=4.0",
]
```

### 6.3 显式不引入 (per SRS §6.1)

- ❌ `pyyaml` — JSON 够用
- ❌ `pydantic` — 过重, dataclass 即可
- ❌ `requests` — 0 网络调用
- ❌ `cryptography` — 跨项目需求
- ❌ `asyncio` — 同步实现够用, 不引入 async 复杂度

---

## 7. 交付清单 (Deliverables, 必含)

| # | 路径 | 类型 | LOC | 说明 |
|---|---|---|---|---|
| 1 | `scripts/automation/guardian/__init__.py` | Python | 5 | 空, 标识 package |
| 2 | `scripts/automation/guardian/pre_tool_use_guard.py` | Python | ~300 | PM-1+PM-3 主入口 |
| 3 | `scripts/automation/guardian/rule_database.py` | Python | ~150 | PM-2 规则加载/热更新 |
| 4 | `scripts/automation/guardian/audit_logger.py` | Python | ~120 | PM-4 JSON Lines 写入 |
| 5 | `scripts/automation/guardian/user_rule_api.py` | Python | ~50 | PM-5 v0.1 stub |
| 6 | `scripts/automation/guardian/rules/pre_tool_use_rules.json` | JSON | ~150 | 15 条规则 |
| 7 | `scripts/automation/guardian/hooks/console_server_hook.py` | Python | ~30 | console_server.py 集成 |
| 8 | `scripts/automation/guardian/hooks/dispatcher_hook.py` | Python | ~30 | dispatcher.py 集成 |
| 9 | `tests/automation/guardian/test_pre_tool_use_guard.py` | Python | ~250 | 10+ 单元测试 |
| 10 | `tests/automation/guardian/test_rule_database.py` | Python | ~150 | 8+ 单元测试 |
| 11 | `tests/automation/guardian/test_audit_logger.py` | Python | ~100 | 5+ 单元测试 |
| 12 | `tests/e2e/test_pre_tool_use_hook.py` | Python | ~200 | 5+ 集成测试 |
| 13 | `tests/security/test_pre_tool_use_penetration.py` | Python | ~300 | 150 渗透测试 |
| 14 | `scripts/automation/pyproject.toml` | TOML | ~30 | 依赖声明 |
| 15 | `docs/reports/PHASE-PRE-TOOL-USE-GUARD-IMPL-REPORT.md` | Markdown | ~300 | 实装报告 |

**总 LOC**: ~2100 行 (含注释 + tests + 配置 + 报告), 跟 DD §1.1 估算 ~1.5K 接近, 实装允许 ±20%

---

## 8. 关联文档 (References)

| 文档 | 关系 |
|---|---|
| [`SRS-PRE-TOOL-USE-GUARD-001.md`](../requirements/SRS-PRE-TOOL-USE-GUARD-001.md) v0.1 | 上位 (需求) |
| [`BD-PRE-TOOL-USE-GUARD-001.md`](../design/BD-PRE-TOOL-USE-GUARD-001.md) v0.1 | 上位 (基本设计) |
| [`DD-PRE-TOOL-USE-GUARD-001.md`](../detailed-design/DD-PRE-TOOL-USE-GUARD-001.md) v0.1 | 上位 (详细设计) |
| [`WBS-002-pre-tool-use-guard.md`](../plans/WBS-002-pre-tool-use-guard.md) v0.1 | 平行 (WBS + 实施计划) |
| `OPS-DETAILED-DESIGN-001.md` v0.1 | 模板 |
| `domain-agent-spec.md` v0.1 | 模板 (本 spec 格式) |
| `AGENTS.md` §4 守门硬约束 | 跨域 (守门 6 项必过) |
| `docs/automation-design.md` v0.1 | 跨域 (Python 化基线) |

---

## 9. 修订履歴 (Revision History)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 19:18 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 9 段 (职责边界/实体/API/实现要点/测试/依赖/交付/引用/修订), 4 关键实体 (ToolCall/Rule/AuditEvent/SessionState), 4 公共 API (evaluate/evaluate_for_dispatch/reload_rules/query_audit), 7 实现要点 (跨平台/regex缓存/thread-safe/同步写/watchdog/守门#5/守门v28), 9 测试类 30+ test case + 150 渗透, 15 交付文件 ~2100 LOC, 2 运行依赖 + 2 dev 依赖 | 2026-09-10 19:18 JST Ulysses 拍板"**根据详细设计制作spec, 实施计划, 并加入wbs**" |
