# DD-PRE-TOOL-USE-GUARD-001

> **Mavis PreToolUse 安全拦截层 — 詳細設計書 v0.1** (per 日本 IPA SEC 標準 / 詳細設計書 テンプレート + STAR 仓 OPS-DETAILED-DESIGN-001 模板)
>
> - 状态: 🟡 Draft v0.1 (2026-09-10 JST 初版落档)
> - 上游: [`docs/requirements/SRS-PRE-TOOL-USE-GUARD-001.md`](../requirements/SRS-PRE-TOOL-USE-GUARD-001.md) v0.1 (23KB, 7 機能 / 4 業務 / 6 非機能 / 7 验收) + [`docs/design/BD-PRE-TOOL-USE-GUARD-001.md`](../design/BD-PRE-TOOL-USE-GUARD-001.md) v0.1 (36KB, 5 module / 3 表 / 7 集成点)
> - 下游: 实装代码 + 测试 + 报告
> - 核心语言: Python 3.10+ (per SRS §6.1 制約)
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #10 + 守门 #14 v3)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST)
> - 日期: 2026-09-10 JST

---

## §0 目的 (Purpose)

本詳細設計書は `BD-PRE-TOOL-USE-GUARD-001.md` v0.1 で定めた基本設計を実装可能なレベルまで展開する。MVP-骨架段階の Python ファイル + JSON 設定 + pytest テストの物理形状と 100% 一致させ、実装者が追加設計判断をせずに済む粒度で仕様を提供する。

**核心スコープ**:

- **5 維** 設計: モジュール / クラス / 時序 / 状態遷移 / テスト
- **W/T/M 3 表 100% カバー** (守門 #13) — `pre_tool_use_rules` (W/M) + `pre_tool_use_audit` (T) + `pre_tool_use_session_state` (M)
- **15 条危险模式** (BLOCK 8 + ASK 5 + WARN 2) — 跟 Claude Code `security-guidance` 9 条对照 + 扩展
- **三级决策** (BLOCK/ASK/WARN) — 跟守门 v28 拍板必带推荐项联动
- **fail-open / fail-closed** — 規則加載失敗 vs 審計失敗 二分
- **Framework 選型** (per 既存 mavis 仓 实证): stdlib `re` / `json` / `pathlib` / `logging` + `jsonschema` (1 依赖) + `watchdog` (1 依赖)

**不做什么** (per SRS-001 §1.4):
- 网络层拦截 (egress filtering) → v2.x
- AI 行为审计 (LLM 决策录屏) → v2.x
- 子代理内部代码越权 (subprocess sandbox) → v0.2 拍摄 (SRS 已知缺口 #1 P0)
- 加密 / 凭据管理 (mavis 内置 vault) → 跨项目需求

---

## §1 モジュール設計 (Module Design)

### 1.1 物理ファイル構成 (target 6 文件 + 3 テスト + 1 fixture = 10 文件)

```
scripts/automation/guardian/
├── __init__.py                     # 空, 標識 Python package
├── pre_tool_use_guard.py           # PM-1+PM-3 (Pattern Matcher + Decision Router) 核心, ~300 LOC
├── rule_database.py                # PM-2 (Rule Database) 規則加載/熱更新/schema 校驗, ~150 LOC
├── audit_logger.py                 # PM-4 (Audit Logger) JSON Lines 寫入, ~120 LOC
├── user_rule_api.py                # PM-5 (User-defined Rule API) v0.1 stub, ~50 LOC
├── rules/
│   └── pre_tool_use_rules.json     # 15 条規則, ~150 行
├── logs/
│   └── pre_tool_use_audit.log      # runtime 生成, JSON Lines
├── state/
│   └── pre_tool_use_session_state.json  # runtime 生成, session 級
└── hooks/
    ├── console_server_hook.py      # 接入 console_server.py v0.1, ~30 LOC
    └── dispatcher_hook.py          # 接入 dispatcher.py v0.1 invoke() 前, ~30 LOC

tests/automation/guardian/
├── __init__.py
├── test_pre_tool_use_guard.py      # 15 條規則 + fail-open/closed, ~250 LOC
├── test_rule_database.py           # 規則加載/熱更新/schema 校驗, ~150 LOC
├── test_audit_logger.py            # 寫入/异步 fsync/fail-closed, ~100 LOC
└── fixtures/
    └── test_rules.json             # 測試用規則, ~50 條 (含 5 條故意錯誤)

tests/e2e/
└── test_pre_tool_use_hook.py       # 端到端, console_server.py + dispatcher.py, ~200 LOC
```

**总行数**: 10 文件, ~1.5K 行 (含注释 + tests), MVP 阶段 0 dead code。

### 1.2 依存関係 (per SRS §6.1, 最小化)

```toml
# pyproject.toml (新增, 或 scripts/automation/pyproject.toml)
[project]
requires-python = ">=3.10"
dependencies = [
    "jsonschema>=4.20",   # 1 依赖, 規則 schema 校驗
    "watchdog>=3.0",      # 1 依赖, 規則文件 mtime 監聽
]
# dev-deps (2 项)
[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "pytest-benchmark>=4.0",
]
```

**運行時依賴 2 项 + dev 依赖 2 项 = 4 总依赖**, MVP 阶段最小化。
**显式不引入** (per SRS §6.1):
- ❌ `pyyaml` — JSON 夠用
- ❌ `pydantic` — 過重, dataclass 即可
- ❌ `requests` — 0 網絡調用
- ❌ `cryptography` — 跨項目需求, 不在本專題

### 1.3 既存 mavis 仓 集成点

| 集成点 | 現有代碼 | 集成方式 |
|---|---|---|
| `console_server.py` v0.1 | `scripts/automation/console_server.py` line 80-92 | 在 tool 調用前插入 `pre_tool_use_guard.evaluate(tool_call)` |
| `dispatcher.py` v0.1 | `scripts/automation/dispatcher.py` `invoke()` 前 | 插入 `pre_tool_use_guard.evaluate_for_dispatch({...})` |
| mavis runtime hook 事件流 | mavis runtime 內建 | 通過 console_server.py 集成, 不開新通道 |

---

## §2 クラス設計 (Class Design)

### 2.1 `pre_tool_use_guard.py` 核心类

```python
# scripts/automation/guardian/pre_tool_use_guard.py
"""PreToolUse 安全拦截层 — Pattern Matcher + Decision Router."""
from __future__ import annotations

import re
import time
import logging
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import List, Optional, Dict, Any

from .rule_database import RuleDatabase, Rule
from .audit_logger import AuditLogger

logger = logging.getLogger(__name__)


class Decision(str, Enum):
    """三级决策 + 1 个 PASS."""
    BLOCK = "BLOCK"
    ASK = "ASK"
    WARN = "WARN"
    PASS = "PASS"


@dataclass(frozen=True)
class ToolCall:
    """PreToolUse hook 入参."""
    tool_name: str          # "bash" / "write" / "edit" / "read" / "mcp_*"
    tool_args: Dict[str, Any]
    session_id: str
    agent_role: str         # "orchestrator" / "worker" / "explore" / "verifier"


@dataclass(frozen=True)
class Match:
    """单条规则匹配结果."""
    rule_id: str
    level: Decision
    reason: str
    latency_us: int         # microseconds


@dataclass(frozen=True)
class AuditEvent:
    """audit log 单条事件."""
    ts: str
    session_id: str
    agent_role: str
    tool: str
    tool_args_hash: str      # sha256:hex
    args_excerpt: str        # <= 200 chars
    decision: Decision
    rule_id: Optional[str]   # decision != PASS 时必填
    reason: Optional[str]    # decision != PASS 时必填
    latency_ms: float
    env_hash: str            # sha256 of all env keys (not values)


class PatternMatcher:
    """PM-1: pure function, 给定 tool_call 返回 Match[]."""
    
    # 提取可扫描字段, 跨平台
    @staticmethod
    def extract_scan_target(tool_call: ToolCall) -> str:
        """从 tool_args 提取可扫描字符串.
        
        Returns:
            bash  → args.command
            write → args.path + "\n" + args.content (前 500 chars)
            edit  → args.path + "\n" + args.new_string
            read  → "" (per FR-1.3, 不掃)
            mcp_* → json.dumps(args)
        """
        if tool_call.tool_name == "bash":
            return tool_call.tool_args.get("command", "")
        if tool_call.tool_name in ("write", "edit"):
            path = tool_call.tool_args.get("path", "")
            content = tool_call.tool_args.get("content", "") or tool_call.tool_args.get("new_string", "")
            return f"{path}\n{content[:500]}"
        if tool_call.tool_name == "read":
            return ""  # per FR-1.3
        # MCP / 未知 tool
        import json
        return json.dumps(tool_call.tool_args, ensure_ascii=False)
    
    @classmethod
    def match_all(
        cls, 
        tool_call: ToolCall, 
        rules: List[Rule]
    ) -> List[Match]:
        """跑所有规则, 返回 Match[].
        
        Latency: p50 < 5ms, p99 < 10ms (per NFR-P-1/NFR-P-2).
        """
        target = cls.extract_scan_target(tool_call)
        if not target:
            return []
        
        matches: List[Match] = []
        for rule in rules:
            t0 = time.perf_counter_ns()
            try:
                if re.search(rule.pattern, target, re.MULTILINE | re.DOTALL):
                    latency_us = (time.perf_counter_ns() - t0) // 1000
                    matches.append(Match(
                        rule_id=rule.id,
                        level=Decision(rule.level),
                        reason=rule.reason,
                        latency_us=latency_us,
                    ))
            except re.error as e:
                # 规则 regex 错误, 跳过 + warn log (per F-7)
                logger.warning("Rule %s regex error: %s", rule.id, e)
                continue
        return matches


class DecisionRouter:
    """PM-3: 多 Match 冲突时, 按优先级选 1 个."""
    
    PRIORITY = {
        Decision.BLOCK: 4,
        Decision.ASK:   3,
        Decision.WARN:  2,
        Decision.PASS:  1,
    }
    
    @classmethod
    def route(cls, matches: List[Match]) -> Decision:
        """返回优先级最高的 decision."""
        if not matches:
            return Decision.PASS
        return max(matches, key=lambda m: cls.PRIORITY[m.level]).level


class PreToolUseGuard:
    """主入口, 整合 PM-1 + PM-3 + audit."""
    
    def __init__(self, rule_db: RuleDatabase, audit: AuditLogger):
        self._rules = rule_db
        self._audit = audit
    
    def evaluate(self, tool_call: ToolCall) -> Decision:
        """PreToolUse hook 入口.
        
        Per FR-1.1 + NFR-P-1/NFR-P-2:
        - p50 < 5ms
        - p99 < 10ms
        - audit log 必写 (无论 decision)
        """
        t0 = time.perf_counter()
        try:
            rules = self._rules.get_rules()  # hot-reload safe
            matches = PatternMatcher.match_all(tool_call, rules)
            decision = DecisionRouter.route(matches)
            rule_id = matches[0].rule_id if matches else None
            reason = matches[0].reason if matches else None
        except Exception as e:
            # per FR-6.1 fail-open: 規則加載失敗 → PASS + WARN log
            logger.error("Rule evaluation failed, fail-open: %s", e)
            decision = Decision.PASS
            rule_id = None
            reason = f"fail-open: {e}"
        
        latency_ms = (time.perf_counter() - t0) * 1000
        
        # audit log 必写 (per FR-4.1)
        try:
            self._audit.log(AuditEvent(
                ts=time.strftime("%Y-%m-%dT%H:%M:%S"),
                session_id=tool_call.session_id,
                agent_role=tool_call.agent_role,
                tool=tool_call.tool_name,
                tool_args_hash=hashlib.sha256(json.dumps(tool_call.tool_args, sort_keys=True).encode()).hexdigest(),
                args_excerpt=PatternMatcher.extract_scan_target(tool_call)[:200],
                decision=decision,
                rule_id=rule_id,
                reason=reason,
                latency_ms=latency_ms,
                env_hash=hashlib.sha256(",".join(sorted(os.environ.keys())).encode()).hexdigest(),
            ))
        except Exception as e:
            # per FR-6.2 fail-closed: 審計失敗 → BLOCK
            logger.error("Audit log write failed, fail-closed: %s", e)
            return Decision.BLOCK
        
        return decision
    
    def evaluate_for_dispatch(
        self, 
        task_id: str, 
        script_path: str, 
        args: Dict[str, Any],
        parent_session_id: str
    ) -> Decision:
        """子代理 dispatch 前置入口, per FR-1.2."""
        # 把 dispatch 参數包裝成 ToolCall
        tool_call = ToolCall(
            tool_name="mcp_dispatch",
            tool_args={"task_id": task_id, "script_path": script_path, "args": args},
            session_id=parent_session_id,
            agent_role="orchestrator",  # 派發方視為 orchestrator
        )
        return self.evaluate(tool_call)
```

**关键设计点**:
1. **`PatternMatcher.extract_scan_target` 跨平台**: 統一提取字符串, 後續所有規則只對這個字符串跑 regex
2. **`DecisionRouter.PRIORITY` 字典**: BLOCK > ASK > WARN > PASS, 簡單明確
3. **fail-open vs fail-closed 二分**: 規則錯誤 → fail-open (不阻斷主流程), 審計錯誤 → fail-closed (凭据零外泄)

### 2.2 `rule_database.py` Rule Database

```python
# scripts/automation/guardian/rule_database.py
"""規則加載 + 熱更新 + schema 校驗."""
from __future__ import annotations

import json
import logging
import threading
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional

import jsonschema
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

logger = logging.getLogger(__name__)

# 規則 schema (per BD §4.3.1)
RULE_SCHEMA = {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "type": "object",
    "required": ["schema_version", "rules"],
    "properties": {
        "schema_version": {"type": "string", "pattern": r"^\d+\.\d+$"},
        "rules": {
            "type": "array",
            "minItems": 1,
            "items": {
                "type": "object",
                "required": ["id", "name", "level", "tool_class", "pattern", "reason"],
                "properties": {
                    "id": {"type": "string", "pattern": r"^R-(BLOCK|ASK|WARN)-\d{3}$"},
                    "name": {"type": "string", "minLength": 1, "maxLength": 100},
                    "level": {"enum": ["BLOCK", "ASK", "WARN"]},
                    "tool_class": {"enum": ["bash", "write", "edit", "read", "mcp"]},
                    "pattern": {"type": "string", "minLength": 1, "maxLength": 500},
                    "reason": {"type": "string", "minLength": 1, "maxLength": 300},
                },
            },
        },
    },
}


@dataclass(frozen=True)
class Rule:
    id: str
    name: str
    level: str          # BLOCK / ASK / WARN
    tool_class: str
    pattern: str
    reason: str


class RuleDatabase:
    """PM-2: 規則加載/熱更新/schema 校驗."""
    
    def __init__(self, rules_path: Path):
        self._path = rules_path
        self._rules: List[Rule] = []
        self._lock = threading.RLock()
        self._observer: Optional[Observer] = None
        self._load_rules()  # 啟動時一次性加載
        self._start_watcher()
    
    def get_rules(self) -> List[Rule]:
        """線程安全讀取 (RLock)."""
        with self._lock:
            return list(self._rules)  # 拷貝避免外部修改
    
    def _load_rules(self) -> None:
        """加載規則, 失敗 → 保留舊 rules + WARN log (per FR-6.1)."""
        try:
            with self._path.open("r", encoding="utf-8") as f:
                data = json.load(f)
            jsonschema.validate(data, RULE_SCHEMA)  # schema 校驗
            new_rules = [
                Rule(
                    id=r["id"],
                    name=r["name"],
                    level=r["level"],
                    tool_class=r["tool_class"],
                    pattern=r["pattern"],
                    reason=r["reason"],
                )
                for r in data["rules"]
            ]
            with self._lock:
                self._rules = new_rules
            logger.info("Loaded %d rules from %s", len(new_rules), self._path)
        except (json.JSONDecodeError, jsonschema.ValidationError, OSError) as e:
            logger.warning("Rule load failed, keeping old rules: %s", e)
            # per FR-6.1: 不抛異常, 保留舊 rules
    
    def _start_watcher(self) -> None:
        """監聽規則文件 mtime, 自動 reload (per FR-5.1)."""
        if self._observer is not None:
            return
        
        class Handler(FileSystemEventHandler):
            def __init__(self, db: RuleDatabase):
                self._db = db
            
            def on_modified(self, event):
                if Path(event.src_path) == self._db._path:
                    logger.info("Rules file modified, reloading...")
                    self._db._load_rules()
        
        try:
            self._observer = Observer()
            self._observer.schedule(Handler(self), str(self._path.parent), recursive=False)
            self._observer.start()
        except Exception as e:
            # per F-5: watchdog 啟動失敗 → 退化为 mtime poll
            logger.warning("File watcher failed, falling back to mtime poll: %s", e)
            # TODO v0.2: 啟動 mtime poll thread (1s 間隔)
    
    def stop(self) -> None:
        if self._observer is not None:
            self._observer.stop()
            self._observer.join()
```

**关键设计点**:
1. **`RLock` 線程安全**: hook 可能在多線程調用, 規則加載和讀取必須互斥
2. **加載失敗保留舊規則**: per FR-6.1, 不破壞現有 mavis runtime 流程
3. **`watchdog` 監聽 mtime**: 自動 reload, 0 人工介入
4. **`jsonschema` 校驗**: 防 JSON 寫錯導致運行時崩潰

### 2.3 `audit_logger.py` Audit Logger

```python
# scripts/automation/guardian/audit_logger.py
"""JSON Lines 寫入, 異步 fsync, fail-closed."""
from __future__ import annotations

import asyncio
import json
import logging
import os
import time
from dataclasses import asdict
from pathlib import Path
from typing import Optional

from .pre_tool_use_guard import AuditEvent

logger = logging.getLogger(__name__)


class AuditLogger:
    """PM-4: 異步寫 audit log, append-only, fail-closed."""
    
    def __init__(self, log_path: Path):
        self._path = log_path
        self._path.parent.mkdir(parents=True, exist_ok=True)
        self._lock = asyncio.Lock()
        # 設置 OS 權限 0444 (per NFR-S-2 audit log 完整性)
        # 注意: mavis 進程仍可寫, 因為是 root 或 owner
        # 其他進程讀 + 不可寫
        try:
            os.chmod(self._path, 0o644)  # 創建時 644, mavis 寫
        except OSError:
            pass
    
    def log(self, event: AuditEvent) -> None:
        """同步寫入 (per NFR-P-3 < 1ms)."""
        line = json.dumps(asdict(event), ensure_ascii=False) + "\n"
        try:
            with self._path.open("a", encoding="utf-8", errors="replace") as f:
                f.write(line)
                f.flush()  # 不 fsync (per NFR-P-3 < 1ms), OS 會 lazy fsync
        except (OSError, IOError) as e:
            # per FR-6.2 + NFR-A-2: 寫失敗 → fail-closed
            logger.error("Audit log write failed: %s", e)
            raise AuditLogError(f"Audit log write failed: {e}") from e
    
    def query(
        self,
        session_id: Optional[str] = None,
        decision: Optional[str] = None,
        rule_id: Optional[str] = None,
        limit: int = 100,
    ) -> list[AuditEvent]:
        """查詢 audit log, 供調試/報告 (per BD §5.1.3)."""
        events = []
        try:
            with self._path.open("r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        event = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    if session_id and event.get("session_id") != session_id:
                        continue
                    if decision and event.get("decision") != decision:
                        continue
                    if rule_id and event.get("rule_id") != rule_id:
                        continue
                    events.append(AuditEvent(**event))
                    if len(events) >= limit:
                        break
        except (OSError, IOError) as e:
            logger.warning("Audit log read failed: %s", e)
        return events


class AuditLogError(Exception):
    """Audit log 寫入失敗, 用於 fail-closed 判斷."""
    pass
```

**关键设计点**:
1. **同步寫入 + lazy fsync**: NFR-P-3 < 1ms, 不阻塞主流程
2. **寫失敗 → raise**: 讓 PreToolUseGuard 捕獲後決定 BLOCK (per FR-6.2)
3. **`errors="replace"`**: 防止編碼錯誤導致崩潰
4. **`os.chmod 0644`**: 創建時設權限, mavis 進程可寫, 其他進程只讀

### 2.4 `user_rule_api.py` User-defined Rule API (v0.1 stub)

```python
# scripts/automation/guardian/user_rule_api.py
"""User-defined rule API, v0.1 stub, v0.3 实装."""
from __future__ import annotations

import logging
from pathlib import Path
from typing import List, Optional

logger = logging.getLogger(__name__)


def register_user_rule(rule_yaml: str, user_rules_dir: Path) -> bool:
    """v0.1 stub: 拋 NotImplementedError.
    
    v0.3 實裝 (per Claude hookify plugin 對照):
    - 接受 YAML / JSON
    - 寫入 user_rules_dir/{rule_id}.json
    - RuleDatabase 監聽並加載
    """
    raise NotImplementedError(
        "User-defined rule API is v0.3 計劃項, v0.1 僅提供 stub. "
        "見 SRS-PRE-TOOL-USE-GUARD-001.md §1.4 + 已知缺口 #4."
    )


def list_user_rules(user_rules_dir: Path) -> List[str]:
    """列出用戶自定義規則 ID, v0.1 stub 返回空 list."""
    if not user_rules_dir.exists():
        return []
    return [p.stem for p in user_rules_dir.glob("R-*.json")]
```

---

## §3 状態遷移 (State Machine, 詳細)

### 3.1 規則狀態機 (per Rule)

```
[未加載] ──load()──> [已加載] ──modify file──> [reload 排隊] ──reload()──> [已重載]
                        │                            │
                        │ schema fail                │ 失敗
                        ▼                            ▼
                   [加載失敗, 保留舊規則]      [重載失敗, 保留舊規則]
```

**狀態轉換條件**:

| From | To | 條件 |
|---|---|---|
| 未加載 | 已加載 | 啟動時 `_load_rules()` 成功 |
| 已加載 | 排隊 | watchdog 監測到 mtime 變化 |
| 排隊 | 已重載 | `_load_rules()` 成功 |
| 排隊 | 失敗 | JSON parse / schema 校驗 / OS error |
| 已加載 / 已重載 | 排隊 | 文件 mtime 變化 (持續監聽) |
| 任意 | 任意 | 進程退出 |

### 3.2 audit log 狀態機 (per AuditEvent)

```
[創建] ──log()──> [寫入中] ──flush──> [已寫入]
                       │
                       │ 失敗
                       ▼
                  [寫入失敗] ──raise AuditLogError──> [PreToolUseGuard fail-closed → BLOCK]
```

**重要**: 寫入失敗**不丟失事件**, 而是 fail-closed (BLOCK tool call), 强制人工介入。

### 3.3 session state 狀態機 (per SessionState)

```
[未啟動] ──首次 evaluate──> [活躍] ──規則 reload──> [活躍, 規則 v2]
                                          │
                                          ▼
                                    [活躍, 規則 N 次 reload]
                                          │
                                          ▼
                                    [進程退出] → [已結束, 寫最終狀態]
```

**更新時機**: 每 1 min 聚合 decision 統計 (per NFR-O-1) + 規則 reload 時 (per NFR-M-2) + 進程退出時。

---

## §4 シーケンス図 (Sequence Diagram, 詳細)

### 4.1 完整 lifecycle (per BR-1 + BR-2 + FR-1.1 + FR-4.1)

```
LLM            mavis runtime      console_server.py     PreToolUseGuard    RuleDatabase    AuditLogger    OS
 │                  │                    │                    │                │              │            │
 │─decide call───▶│                    │                    │                │              │            │
 │  bash: rm -rf /tmp/xxx                │                    │                │              │            │
 │                  │                    │                    │                │              │            │
 │                  │─PreToolUse event──▶│                    │                │              │            │
 │                  │                    │                    │                │              │            │
 │                  │                    │─evaluate()───────▶│                │              │            │
 │                  │                    │                    │─get_rules()───▶│              │            │
 │                  │                    │                    │                │              │            │
 │                  │                    │                    │◀─[Rule]×15────│              │            │
 │                  │                    │                    │                │              │            │
 │                  │                    │                    │─PatternMatcher.match_all()      │            │
 │                  │                    │                    │  提取 scan_target (rm -rf /tmp/xxx)             │
 │                  │                    │                    │  跑 15 條 regex                       │            │
 │                  │                    │                    │  命中 R-ASK-002 sudo 類? 否           │            │
 │                  │                    │                    │  命中 R-WARN-001? 否                  │            │
 │                  │                    │                    │                │              │            │
 │                  │                    │                    │  Match[] = []                        │            │
 │                  │                    │                    │                │              │            │
 │                  │                    │                    │─DecisionRouter.route([]) = PASS     │            │
 │                  │                    │                    │                │              │            │
 │                  │                    │                    │─audit.log()────────────────────▶│            │
 │                  │                    │                    │                │              │─open 'a'──▶│
 │                  │                    │                    │                │              │─write line─▶│
 │                  │                    │                    │                │              │─flush──────▶│
 │                  │                    │                    │                │              │            │
 │                  │                    │◀─PASS──────────────│                │              │            │
 │                  │                    │                    │                │              │            │
 │                  │◀─continue──────────│                    │                │              │            │
 │                  │                    │                    │                │              │            │
 │                  │─execute bash───────────────────────▶│                │              │            │
 │                  │                    │                    │                │              │            │
 │                  │◀─stdout/stderr────│                    │                │              │            │
```

### 4.2 BLOCK 場景 (per BR-3 + FR-2.1)

```
LLM            mavis runtime      PreToolUseGuard     AuditLogger
 │                  │                    │                │
 │─decide call───▶│                    │                │
 │  bash: rm -rf /                       │                │
 │                  │                    │                │
 │                  │─PreToolUse────────▶│                │
 │                  │                    │─match_all()    │
 │                  │                    │  命中 R-BLOCK-001 (rm -rf /)
 │                  │                    │                │
 │                  │                    │─route() = BLOCK
 │                  │                    │                │
 │                  │                    │─audit.log()──▶│
 │                  │                    │                │
 │                  │◀─BLOCK────────────│                │
 │                  │                    │                │
 │                  │─reject tool call──▶│ (LLM 收到 error)
 │                  │                    │                │
 │                  │─LLM 改 plan─────▶│                │
```

### 4.3 ASK 場景 (per BR-3 + FR-2.2 + 守门 v28)

```
LLM            mavis runtime      PreToolUseGuard     ask_user     User
 │                  │                    │                │            │
 │─decide call───▶│                    │                │            │
 │  bash: sudo apt install xxx           │                │            │
 │                  │                    │                │            │
 │                  │─PreToolUse────────▶│                │            │
 │                  │                    │─match_all()    │            │
 │                  │                    │  命中 R-ASK-002 (sudo)
 │                  │                    │                │            │
 │                  │                    │─route() = ASK  │            │
 │                  │                    │                │            │
 │                  │                    │─audit.log()──▶│            │
 │                  │                    │                │            │
 │                  │◀─ASK──────────────│                │            │
 │                  │                    │                │            │
 │                  │─ask_user─────────────────────────────────────▶│
 │                  │  options = [取消(推薦), 確認執行, 改用其他]   │
 │                  │                    │                │            │
 │                  │                    │                │◀─user reply─┤
 │                  │                    │                │  選了"取消" │
 │                  │◀─rejected────────────────────────────────────│
 │                  │                    │                │            │
 │                  │─LLM 改 plan─────▶│                │            │
```

### 4.4 規則 reload 場景 (per FR-5.1)

```
User (edit JSON)    watchdog       RuleDatabase     PreToolUseGuard (running)
       │               │                │                       │
       │─save─────────▶│                │                       │
       │               │─on_modified()─▶│                       │
       │               │                │                       │
       │               │                │─_load_rules()         │
       │               │                │  schema 校驗 OK       │
       │               │                │─swap _rules (RLock)   │
       │               │                │                       │
       │               │                │                       │─get_rules() 拿到新規則
       │               │                │                       │  (線程安全, 0 中斷)
```

### 4.5 audit log 寫失敗 → fail-closed 場景 (per FR-6.2)

```
LLM            PreToolUseGuard     AuditLogger       OS
 │                  │                │                │
 │─evaluate()─────▶│                │                │
 │                  │─match()        │                │
 │                  │─route() = PASS │                │
 │                  │                │                │
 │                  │─audit.log()──▶│                │
 │                  │                │─open 'a'──────▶│
 │                  │                │◀─ENOSPC────────│  (磁盤滿)
 │                  │                │                │
 │                  │                │─raise AuditLogError
 │                  │◀─raise─────────│                │
 │                  │                │                │
 │                  │ (per FR-6.2)   │                │
 │                  │ return BLOCK   │                │
 │                  │                │                │
 │                  │ (LLM 收到 error, 改 plan, 同時 audit 寫失敗事件)
```

---

## §5 データ永続化 (Data Persistence, 詳細)

### 5.1 `pre_tool_use_rules.json` (W/M 混合)

**位置**: `scripts/automation/guardian/rules/pre_tool_use_rules.json`

**文件格式** (per BD §4.3.1, 实际内容):

```json
{
  "schema_version": "1.0",
  "rules": [
    {
      "id": "R-BLOCK-001",
      "name": "rm -rf 命中危险目录",
      "level": "BLOCK",
      "tool_class": "bash",
      "pattern": "rm\\s+(-[a-z]*f[a-z]*\\s+)*(-[a-z]*r[a-z]*\\s+)*(/|~|\\\\$HOME|\\\\$\\{?HOME\\}?\\b)",
      "reason": "rm -rf 不可逆, 命中 / ~ $HOME 即阻断"
    },
    {
      "id": "R-BLOCK-002",
      "name": "dd if= 无 of= 限速",
      "level": "BLOCK",
      "tool_class": "bash",
      "pattern": "dd\\s+.*of=/dev/(sd|nvme|hd)",
      "reason": "dd 写入 /dev 设备, 不可逆"
    },
    {
      "id": "R-BLOCK-003",
      "name": "mkfs / fdisk",
      "level": "BLOCK",
      "tool_class": "bash",
      "pattern": "(mkfs(\\.[a-z0-9]+)?\\s+|fdisk\\s+).*/dev/",
      "reason": "mkfs / fdisk 格式化磁盘, 不可逆"
    },
    {
      "id": "R-BLOCK-004",
      "name": "chmod 777 / 根目录",
      "level": "BLOCK",
      "tool_class": "bash",
      "pattern": "chmod\\s+[0-7]*[0-7]7[0-7]\\s+/(\\s|$)",
      "reason": "chmod 777 根目录, 严重安全风险"
    },
    {
      "id": "R-BLOCK-005",
      "name": "写 /etc /boot /sys /proc",
      "level": "BLOCK",
      "tool_class": "bash",
      "pattern": ">\\s*/(etc|boot|sys|proc)/",
      "reason": "重定向写入系统关键目录, 不可逆"
    },
    {
      "id": "R-BLOCK-006",
      "name": "打印 env 内容 (守门 #5)",
      "level": "BLOCK",
      "tool_class": "bash",
      "pattern": "(Get-ChildItem\\s+env:|env\\s*$|printenv|set\\s*$).*\\|.*Format-",
      "reason": "env 内容可能含 secret, 禁止打印 (per 守门 #5)"
    },
    {
      "id": "R-BLOCK-007",
      "name": "GitHub PAT / OpenAI key 出现在参数",
      "level": "BLOCK",
      "tool_class": "bash",
      "pattern": "(ghp_|gho_|ghs_|github_pat_|xox[abp]-|sk-[a-zA-Z0-9]{20,})",
      "reason": "凭据出现在参数, 禁止 (per 守门 #5 扩展)"
    },
    {
      "id": "R-BLOCK-008",
      "name": "SSH 私钥读取",
      "level": "BLOCK",
      "tool_class": "bash",
      "pattern": "cat\\s+.*\\.ssh/(id_rsa|id_ed25519|.*_rsa)\\b",
      "reason": "SSH 私钥读取, 严重凭据泄露风险"
    },
    {
      "id": "R-ASK-001",
      "name": "curl | bash / wget | sh",
      "level": "ASK",
      "tool_class": "bash",
      "pattern": "(curl|wget)\\s+.*\\|\\s*(bash|sh|zsh)\\b",
      "reason": "远程脚本直接执行, 不可逆且供应链风险"
    },
    {
      "id": "R-ASK-002",
      "name": "sudo 任意调用",
      "level": "ASK",
      "tool_class": "bash",
      "pattern": "^sudo\\s+",
      "reason": "sudo 提权, 需要用户确认"
    },
    {
      "id": "R-ASK-003",
      "name": "git push --force",
      "level": "ASK",
      "tool_class": "bash",
      "pattern": "git\\s+push\\s+.*(-f|--force)\\b",
      "reason": "force push 覆盖远程历史, 不可逆"
    },
    {
      "id": "R-ASK-004",
      "name": "写 ~/.ssh/ 下文件",
      "level": "ASK",
      "tool_class": "write",
      "pattern": ".*\\.ssh/.*",
      "reason": "写 SSH 目录, 需要用户确认"
    },
    {
      "id": "R-ASK-005",
      "name": "改 ~/.gitconfig 全局",
      "level": "ASK",
      "tool_class": "write",
      "pattern": ".*\\.gitconfig.*",
      "reason": "改全局 git 配置, 影响所有仓库"
    },
    {
      "id": "R-WARN-001",
      "name": "cat | > /dev/null (静默丢弃)",
      "level": "WARN",
      "tool_class": "bash",
      "pattern": "cat\\s+.*\\|\\s*.*>\\s*/dev/null",
      "reason": "可能静默丢弃输出, 难以排查"
    },
    {
      "id": "R-WARN-002",
      "name": "全局 npm install / pip install",
      "level": "WARN",
      "tool_class": "bash",
      "pattern": "(npm\\s+install\\s+-g|pip\\s+install\\s+)(?!.*--user)",
      "reason": "全局安装, 可能污染系统 Python/Node"
    }
  ]
}
```

**15 条規則**: BLOCK 8 + ASK 5 + WARN 2, 跟 SRS FR-3.2/3.3/3.4 1:1。

### 5.2 `pre_tool_use_audit.log` (T)

**位置**: `scripts/automation/guardian/logs/pre_tool_use_audit.log`

**格式**: JSON Lines, 每行 1 个 AuditEvent (per BD §4.3.2 schema)

**示例** (3 条):

```json
{"ts":"2026-09-10T19:03:25.123+09:00","session_id":"mvs_xxx","agent_role":"orchestrator","tool":"bash","tool_args_hash":"sha256:abc123...","args_excerpt":"rm -rf /tmp/xxx","decision":"PASS","rule_id":null,"reason":null,"latency_ms":2.3,"env_hash":"sha256:def456..."}
{"ts":"2026-09-10T19:04:10.456+09:00","session_id":"mvs_xxx","agent_role":"orchestrator","tool":"bash","tool_args_hash":"sha256:789ghi...","args_excerpt":"rm -rf /","decision":"BLOCK","rule_id":"R-BLOCK-001","reason":"rm -rf 不可逆, 命中 / ~ $HOME 即阻断","latency_ms":2.8,"env_hash":"sha256:def456..."}
{"ts":"2026-09-10T19:05:00.789+09:00","session_id":"mvs_xxx","agent_role":"orchestrator","tool":"bash","tool_args_hash":"sha256:jkl012...","args_excerpt":"sudo apt install nginx","decision":"ASK","rule_id":"R-ASK-002","reason":"sudo 提权, 需要用户确认","latency_ms":2.1,"env_hash":"sha256:def456..."}
```

**容量估算**: 假设高频 tool call 100/h, 24h = 2400 条/天, 90 天 = 216000 条, ~50MB JSON Lines (单条 ~250 bytes), 远低于 100MB rotate 阈值。

### 5.3 `pre_tool_use_session_state.json` (M)

**位置**: `scripts/automation/guardian/state/pre_tool_use_session_state.json`

**示例**:

```json
{
  "schema_version": "1.0",
  "session_id": "mvs_22895d7a09ff40be87e24dbeff523fab",
  "started_at": "2026-09-10T19:03:25.123+09:00",
  "last_reload_ts": "2026-09-10T19:03:30.456+09:00",
  "rules_loaded_count": 15,
  "rules_load_fail_count": 0,
  "last_block_rule_id": "R-BLOCK-001",
  "decision_count": {
    "BLOCK": 3,
    "ASK": 0,
    "WARN": 12,
    "PASS": 1234
  }
}
```

**更新時機**: 每 1 min 聚合 (per NFR-O-1) + 規則 reload 時 + 進程退出時。

---

## §6 エラーハンドリング (Error Handling)

### 6.1 錯誤碼一覧

| 錯誤類型 | 觸發 | 處理 | HTTP status (如適用) |
|---|---|---|---|
| `RuleLoadError` | JSON parse fail / schema 校驗 fail | fail-open (PASS + WARN log) | N/A |
| `RuleRegexError` | regex catastrophic backtracking / pattern invalid | 跳過該規則, 其他繼續, WARN log | N/A |
| `AuditLogError` | 磁盤滿 / 權限不足 / 編碼錯誤 | fail-closed (BLOCK) | N/A |
| `ToolCallFormatError` | tool_call 缺必填字段 | 返回 ERROR decision + 詳細 log | N/A |
| `WatcherError` | watchdog 啟動失敗 | 退化为 mtime poll (TODO v0.2) | N/A |
| `PathNormalizeError` | pathlib 處理 UNC / 8.3 短名失敗 | 退化为 raw string 匹配 | N/A |

### 6.2 例外處理原則

- **永不靜默吞錯** (per 9/1 P0-1 实证教訓)
- **fail-open vs fail-closed 明確二分**:
  - 規則錯誤 (邏輯) → fail-open, 不阻斷主流程
  - 審計錯誤 (凭据) → fail-closed, 阻斷 tool call
- **log 必含 trace**: traceback.format_exc() + 時間戳 + session_id

### 6.3 例外伝播鏈

```
RuleDatabase._load_rules() 
  → except → logger.warning, 保留舊規則 → 不傳播
  
PreToolUseGuard.evaluate()
  → except RuleLoadError → fail-open PASS + WARN
  → except AuditLogError → return BLOCK (per FR-6.2)
  → except 其他 → fail-open PASS + ERROR log
```

---

## §7 テスト設計 (Test Design)

### 7.1 測試矩陣 (per SRS §7 AC-1~AC-7)

| 測試 ID | 測試名 | 關聯 AC | 輸入 | 預期 |
|---|---|---|---|---|
| TC-1.1 | R-BLOCK-001 命中 `rm -rf /` | AC-1 | bash `rm -rf /` | BLOCK, R-BLOCK-001 |
| TC-1.2 | R-BLOCK-001 命中 `rm -rf ~` | AC-1 | bash `rm -rf ~` | BLOCK, R-BLOCK-001 |
| TC-1.3 | R-BLOCK-001 不命中 `rm file.txt` | AC-1 | bash `rm file.txt` | PASS |
| TC-1.4 | R-BLOCK-002 命中 `dd if=/dev/zero of=/dev/sda` | AC-1 | bash dd | BLOCK |
| TC-1.5 | R-BLOCK-006 命中 `Get-ChildItem env: \| Format-Table` | AC-1 + NFR-S-5 | PowerShell | BLOCK |
| TC-1.6 | R-BLOCK-007 命中 `ghp_abc...` 参数 | AC-1 + NFR-S-1 | bash 含 token | BLOCK |
| TC-1.7 | R-ASK-002 命中 `sudo apt install` | AC-1 | bash sudo | ASK |
| TC-1.8 | R-ASK-003 命中 `git push --force` | AC-1 | bash force push | ASK |
| TC-1.9 | R-WARN-001 命中 `cat file \| grep x > /dev/null` | AC-1 | bash | WARN |
| TC-1.10 | 15 條規則全部跑通 | AC-1 | 15 個 fixture | 100% 命中預期 |
| TC-2.1 | p50 < 5ms (100 次平均) | AC-2 | benchmark | < 5ms |
| TC-2.2 | p99 < 10ms (1000 次) | AC-2 | benchmark | < 10ms |
| TC-3.1 | 規則 JSON 故意壞掉 → fail-open | AC-3 | 壞 JSON | PASS + WARN log |
| TC-3.2 | audit log 故意 read-only → fail-closed | AC-3 | chmod 0444 | BLOCK + ERROR log |
| TC-4.1 | audit log append-only | AC-4 | append 100 條 | 100 條全在, 無 truncate |
| TC-4.2 | audit log OS 權限 | AC-4 | chmod 0444 | 其他進程不可寫 |
| TC-5.1 | Windows PowerShell 兼容 | AC-5 | Windows CI | 100% pass |
| TC-5.2 | POSIX bash 兼容 | AC-5 | POSIX CI | 100% pass |
| TC-6.1 | 規則修改後 < 100ms reload | AC-6 | modify JSON | < 100ms + 0 中斷 |
| TC-7.1 | 15 條規則 × 10 攻擊場景 = 150 渗透測試 | AC-7 | 攻擊 fixture | 0 命中 |

**總計**: 20+ 個 test case, 覆蓋 AC-1~AC-7 全部, +150 渗透測試。

### 7.2 單元測試樣板 (test_pre_tool_use_guard.py)

```python
# tests/automation/guardian/test_pre_tool_use_guard.py
import pytest
from pathlib import Path
import tempfile

from scripts.automation.guardian.pre_tool_use_guard import (
    ToolCall, PatternMatcher, DecisionRouter, PreToolUseGuard, Decision
)
from scripts.automation.guardian.rule_database import RuleDatabase
from scripts.automation.guardian.audit_logger import AuditLogger


@pytest.fixture
def rule_db_with_15_rules():
    """15 條規則 fixture, per §5.1."""
    with tempfile.TemporaryDirectory() as tmp:
        rules_path = Path(tmp) / "rules.json"
        # 寫 15 條規則
        rules_path.write_text(RULES_15_JSON, encoding="utf-8")
        db = RuleDatabase(rules_path)
        yield db
        db.stop()


@pytest.fixture
def audit_log():
    with tempfile.TemporaryDirectory() as tmp:
        log_path = Path(tmp) / "audit.log"
        yield AuditLogger(log_path)


def test_r_block_001_命中_root(rule_db_with_15_rules, audit_log):
    """TC-1.1: rm -rf / 必須 BLOCK."""
    guard = PreToolUseGuard(rule_db_with_15_rules, audit_log)
    tool_call = ToolCall(
        tool_name="bash",
        tool_args={"command": "rm -rf /"},
        session_id="test",
        agent_role="orchestrator",
    )
    decision = guard.evaluate(tool_call)
    assert decision == Decision.BLOCK


def test_r_block_006_命中_env_print(rule_db_with_15_rules, audit_log):
    """TC-1.5: env 打印必須 BLOCK (per 守门 #5)."""
    guard = PreToolUseGuard(rule_db_with_15_rules, audit_log)
    tool_call = ToolCall(
        tool_name="bash",
        tool_args={"command": "Get-ChildItem env: | Format-Table"},
        session_id="test",
        agent_role="orchestrator",
    )
    decision = guard.evaluate(tool_call)
    assert decision == Decision.BLOCK


def test_p50_latency(rule_db_with_15_rules, audit_log):
    """TC-2.1: p50 < 5ms."""
    guard = PreToolUseGuard(rule_db_with_15_rules, audit_log)
    tool_call = ToolCall(
        tool_name="bash",
        tool_args={"command": "ls -la"},
        session_id="test",
        agent_role="orchestrator",
    )
    # 預熱
    for _ in range(10):
        guard.evaluate(tool_call)
    # 測量
    import time
    latencies = []
    for _ in range(100):
        t0 = time.perf_counter()
        guard.evaluate(tool_call)
        latencies.append((time.perf_counter() - t0) * 1000)
    p50 = sorted(latencies)[50]
    assert p50 < 5, f"p50 = {p50}ms, 期望 < 5ms"


def test_rule_load_fail_open(audit_log):
    """TC-3.1: 規則壞掉 → fail-open PASS."""
    with tempfile.TemporaryDirectory() as tmp:
        bad_path = Path(tmp) / "bad.json"
        bad_path.write_text("{ invalid json", encoding="utf-8")
        db = RuleDatabase(bad_path)  # 不拋, 保留空 rules
        guard = PreToolUseGuard(db, audit_log)
        tool_call = ToolCall(
            tool_name="bash",
            tool_args={"command": "rm -rf /"},
            session_id="test",
            agent_role="orchestrator",
        )
        decision = guard.evaluate(tool_call)
        # fail-open: 無規則 → PASS
        assert decision == Decision.PASS


def test_audit_log_write_fail_closed(rule_db_with_15_rules):
    """TC-3.2: audit 寫失敗 → fail-closed BLOCK."""
    with tempfile.TemporaryDirectory() as tmp:
        log_path = Path(tmp) / "readonly.log"
        log_path.write_text("")  # 創建
        # 設為只讀, 寫會失敗
        import os, stat
        os.chmod(log_path, stat.S_IRUSR | stat.S_IRGRP | stat.S_IROTH)
        audit = AuditLogger(log_path)
        guard = PreToolUseGuard(rule_db_with_15_rules, audit)
        tool_call = ToolCall(
            tool_name="bash",
            tool_args={"command": "ls"},
            session_id="test",
            agent_role="orchestrator",
        )
        decision = guard.evaluate(tool_call)
        # fail-closed: 審計失敗 → BLOCK
        assert decision == Decision.BLOCK
        # 還原權限 (cleanup)
        os.chmod(log_path, stat.S_IRUSR | stat.S_IWUSR)
```

### 7.3 整合測試樣板 (test_pre_tool_use_hook.py)

```python
# tests/e2e/test_pre_tool_use_hook.py
import subprocess
import time
import requests
import pytest


def test_console_server_hook_real():
    """E2E: 啟動 console_server.py, 模擬 tool call, 驗證 hook 觸發."""
    # 啟動 console_server.py
    proc = subprocess.Popen(
        ["python", "scripts/automation/console_server.py"],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    try:
        time.sleep(2)  # 等啟動
        # 模擬 LLM 調用 bash, 期望 hook 攔截
        response = requests.post(
            "http://localhost:8080/api/tool_call",
            json={
                "tool_name": "bash",
                "tool_args": {"command": "rm -rf /"},
                "session_id": "e2e_test",
                "agent_role": "orchestrator",
            },
            timeout=5,
        )
        assert response.status_code == 403  # BLOCK
        data = response.json()
        assert data["decision"] == "BLOCK"
        assert data["rule_id"] == "R-BLOCK-001"
    finally:
        proc.terminate()
        proc.wait(timeout=5)


def test_dispatcher_hook_real():
    """E2E: 模擬 dispatcher.py invoke, 驗證前置 hook 攔截."""
    # 調 dispatcher.py invoke, args 含 rm -rf /
    result = subprocess.run(
        ["python", "scripts/automation/dispatcher.py", "invoke",
         "--task-id", "test_bad",
         "--script-path", "scripts/automation/some_script.py",
         "--args", '{"command": "rm -rf /"}',
         "--parent-session-id", "e2e_test"],
        capture_output=True, text=True, timeout=10,
    )
    # 期望 exit code != 0, 因 BLOCK
    assert result.returncode != 0
    assert "BLOCK" in result.stderr or "R-BLOCK" in result.stderr
```

---

## §8 運用 Runbook (Operation Runbook)

### 8.1 啟動 / 停止

```bash
# 啟動 (mavis runtime 啟動時自動)
# 不需要手動啟動, mavis 進程載入 pre_tool_use_guard

# 規則 reload (熱更新, 不重啟 mavis)
$ python -c "from scripts.automation.guardian.rule_database import RuleDatabase; \
  db = RuleDatabase(Path('scripts/automation/guardian/rules/pre_tool_use_rules.json')); \
  print(f'Loaded {len(db.get_rules())} rules')"

# 查詢 audit log
$ python -c "from scripts.automation.guardian.audit_logger import AuditLogger; \
  a = AuditLogger(Path('scripts/automation/guardian/logs/pre_tool_use_audit.log')); \
  print(f'Last 5 BLOCK events:'); \
  [print(e) for e in a.query(decision='BLOCK', limit=5)]"
```

### 8.2 故障排查

| 故障 | 排查命令 | 解決 |
|---|---|---|
| 規則不生效 | `python -c "...db.get_rules()..."` 看是否 15 條 | 檢查 JSON 格式 |
| 誤攔截正常命令 | `cat pre_tool_use_audit.log \| grep BLOCK` | 修 regex |
| audit log 寫失敗 | `ls -la pre_tool_use_audit.log` 看權限 | `chmod 644` |
| 啟動時間變慢 | `time python -c "...create db..."` | 應該 < 50ms |

### 8.3 log rotation

由外部 logrotate 配置 (per BD §8.5):
```
/path/to/pre_tool_use_audit.log {
    daily
    rotate 90
    compress
    missingok
    notifempty
    copytruncate
}
```

---

## §9 关联文档 / 引用 (References)

| 文档 | 关系 |
|---|---|
| `SRS-PRE-TOOL-USE-GUARD-001.md` v0.1 | 上位 (FR/NBR/AC 全部) |
| `BD-PRE-TOOL-USE-GUARD-001.md` v0.1 | 上位 (module/接口/安全/障害) |
| `guard-pre-tool-use-spec.md` (本仓) | 平行 (实装 spec) |
| `WBS-002-pre-tool-use-guard.md` (本仓) | 平行 (实施 WBS) |
| `OPS-DETAILED-DESIGN-001.md` v0.1 | 模板 (本 BD 格式参考) |
| `AGENTS.md` §4 守门硬约束 | 跨域 (守门 6 项) |
| `docs/automation-design.md` v0.1 | 跨域 (Python 化基线) |

---

## §10 修订履歴 (Revision History)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 19:18 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 10 段 (目的/模块/类/状态/时序/数据/错误/测试/运用/引用+修订), 5 module 物理実装 (pre_tool_use_guard + rule_database + audit_logger + user_rule_api + 2 hook), 10 文件 (~1.5K 行), 4 依赖 (jsonschema + watchdog + pytest + pytest-benchmark), 3 表 W/T/M 100% 覆盖 (Rule W/M + Audit T + Session M, per 守门 #13), 15 条规则 1:1 落地 (BLOCK 8 + ASK 5 + WARN 2), 20+ 單元測試 + 5 整合測試 + 150 渗透測試, IPA 詳細設計書 模板 + 5 維 (模块/类/状态/时序/测试) 覆盖 | 2026-09-10 19:03 JST Ulysses 拍板"整理出需求文档, 然后基于需求文档写基本设计, 要符合日本 IPA 的水准和规范" + 19:18 JST Ulysses 拍板"**根据详细设计制作spec, 实施计划, 并加入wbs**" |
