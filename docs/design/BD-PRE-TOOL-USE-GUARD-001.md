# BD-PRE-TOOL-USE-GUARD-001

> **Mavis PreToolUse 安全拦截层 — 基本設計書 v0.1** (per 日本 IPA SEC 標準 / 基本設計書 テンプレート)
>
> - 状态: 🟡 Draft v0.1 (2026-09-10 JST 初版落档)
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 关联 commit: (留空, root 统一 commit 时填, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
> - 上位要件: [`docs/requirements/SRS-PRE-TOOL-USE-GUARD-001.md`](../requirements/SRS-PRE-TOOL-USE-GUARD-001.md) v0.1 (23KB, 7 機能 / 4 業務 / 6 非機能 / 7 验收 / 8 已知缺口, 守门 6/6 通过)
> - 关联实装基线: `scripts/automation/guardian/` (v0.0 空目录, 待 v0.1 落档) + `scripts/automation/console_server.py` v0.1 (PreToolUse hook 接入点, 现役) + `scripts/automation/dispatcher.py` v0.1 (子代理 invoke 前置, 现役)
> - 平行参考: `docs/automation-design.md` v0.1 (Python 化基线) + `docs/guardian/README.md` (守门 v3x 落档目录) + claude code `plugins/security-guidurance` (对照基线)
> - 守门基线: 守门 #1+#5+#9+#10+#14 v3+#14 v4 6 项必过 (守门 #1 v25 cargo test 不需要跑, 文档工作)
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #10 + 守门 #14 v3)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)
> - 日期: 2026-09-10 JST
> - 受众: 詳細設計エンジニア / 実装エンジニア / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
> - 拍板来源: 2026-09-10 19:03 JST Ulysses 拍板"**整理出需求文档, 然后基于需求文档写基本设计, 要符合日本 IPA 的水准和规范**" (本 BD 落档)
> - 跨域 disclaimer: **守门 v3x 候选 ≠ 守门硬约束** (per AGENTS.md §4.1.1, v27/v28/v29 是待激活候选, v3x 落档目录 = `docs/guardian/`, 不跟硬约束守门 #1~#14 混)

---

## §0 目的 (Purpose)

本文档基于 [`SRS-PRE-TOOL-USE-GUARD-001`](../requirements/SRS-PRE-TOOL-USE-GUARD-001.md) v0.1 的需求, 定义 **Mavis PreToolUse 安全拦截层** 的基本設計:

- **システムアーキテクチャ** (mavis runtime hook 事件流 + 子代理 dispatch 前置)
- **機能分割 / モジュール設計** (5 module: Pattern Matcher + Rule Database + Decision Router + Audit Logger + User-defined Rule API)
- **データモデル** (3 表 W/T/M 100% 覆盖 per 守门 #13: Rule W/M + Audit T + Session M)
- **インターフェース設計** (PreToolUse hook 契约 + mavis runtime 接口 + ask_user 接口)
- **セキュリティ設計** (凭据零外泄 + audit log 完整性 + 规则文件只读)
- **非機能設計** (性能 < 5ms / fail-open vs fail-closed / 可观测性)
- **障害 / 運用設計** (规则加载失败 / 审计失败 / 回滚 / log rotation)
- **用語集** (跟 SRS 共享 + BD 新增 5 条)
- **签字栏 + 修订履歴**

**派生来源**: 2026-09-10 19:02 JST 调研 claude code `anthropics/claude-code` GitHub repo + `plugins/security-guidance` plugin (PreToolUse hook 监控 9 种危险模式) + SRS v0.1 全部 FR-1~FR-7 / BR-1~BR-4 / NFR-P/A/S/M/T/O。

---

## §1 适用范围 (Scope)

### 1.1 包含 (In-Scope, 5 module + 3 表 + 7 集成点)

| 类别 | 范围 | 数量 |
|---|---|---|
| **module** | Pattern Matcher / Rule Database / Decision Router / Audit Logger / User-defined Rule API | 5 module |
| **数据表** | `pre_tool_use_rules` (W/M) + `pre_tool_use_audit` (T) + `pre_tool_use_session_state` (M) | 3 表 W/T/M |
| **集成点** | console_server.py / dispatcher.py / mavis runtime hook / ask_user / file watcher / pytest / CI | 7 集成点 |
| **API 端点** | `evaluate(tool_call)` 内部 API + `reload_rules()` + `query_audit(filters)` | 3 端点 |
| **文件** | 1 Python module + 1 rules JSON + 1 audit log + 3 test files | 6 文件 |
| **NFR 阈值** | 性能 / 可用性 / 安全 / 保守性 / 移植性 / 可观测性 6 類 24 项 | 24 项 |

### 1.2 排除 (Out-of-Scope, per SRS §1.4)

- 网络层拦截 (egress filtering) → v2.x
- AI 行为审计 (LLM 决策录屏) → v2.x
- 子代理内部代码越权 (subprocess sandbox) → v0.2 拍摄 (SRS 已知缺口 #1 P0)
- 加密 / 凭据管理 (mavis 内置 vault) → 跨项目需求

### 1.3 关联文档

| 文档 | 关系 | 关键引用 |
|---|---|---|
| `SRS-PRE-TOOL-USE-GUARD-001.md` v0.1 | 上位 | FR-1~FR-7 / BR-1~BR-4 / NFR-P/A/S/M/T/O |
| `AGENTS.md` §4 守门硬约束 | 上位 | 守门 #1+#5+#9+#10+#14 v3+#14 v4 |
| `docs/automation-design.md` v0.1 | 平行 | §3.1 dispatcher.py brief 落地 + §1.2 [P]/[M]/[S] 判定 |
| `docs/guardian/README.md` | 平行 | 守门 v3x 落档目录 |
| `scripts/automation/console_server.py` v0.1 | 现役 | PreToolUse 接入点 (line 80-92 范围) |
| `scripts/automation/dispatcher.py` v0.1 | 现役 | invoke() 前置 (per 守门 #9 v20) |
| `docs/reports/PHASE-*-IMPL-REPORT.md` | 平行 | 6 份 PHASE 报告模板 |
| claude code `plugins/security-guidance` | 对照基线 | 9 条 → 本 v0.1 扩展 15 条 |

---

## §2 システムアーキテクチャ (System Architecture)

### 2.1 全体構成 (Overall Architecture)

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Mavis Runtime (Mavis mavis v0.x)                │
│                                                                       │
│  ┌──────────────────┐         ┌──────────────────────────────────┐   │
│  │  LLM Brain       │         │  PreToolUse Hook Layer            │   │
│  │  (Claude/GPT)    │ ──────▶ │  ┌────────────────────────────┐  │   │
│  │                  │  decide  │  │  PreToolUse Guard v0.1     │  │   │
│  │                  │  tool   │  │  ┌──────────────────────┐  │  │   │
│  │                  │  call   │  │  │ Pattern Matcher      │  │  │   │
│  └──────────────────┘         │  │  │ (15 rules, < 5ms)   │  │  │   │
│                               │  │  └──────────────────────┘  │  │   │
│                               │  │           │                 │  │   │
│                               │  │           ▼                 │  │   │
│                               │  │  ┌──────────────────────┐  │  │   │
│                               │  │  │ Decision Router      │  │  │   │
│                               │  │  │ BLOCK/ASK/WARN/PASS  │  │  │   │
│                               │  │  └──────────────────────┘  │  │   │
│                               │  │           │                 │  │   │
│                               │  │           ▼                 │  │   │
│                               │  │  ┌──────────────────────┐  │  │   │
│                               │  │  │ Audit Logger         │  │  │   │
│                               │  │  │ (JSON Lines, T-only) │  │  │   │
│                               │  │  └──────────────────────┘  │  │   │
│                               │  └────────────────────────────┘  │   │
│                               │           │                       │   │
│                               │           ▼                       │   │
│                               │  ┌────────────────────────────┐  │   │
│                               │  │  Tool Execution            │  │   │
│                               │  │  (bash / write / edit /   │  │   │
│                               │  │   read / MCP)              │  │   │
│                               │  └────────────────────────────┘  │   │
│                               └──────────────────────────────────┘   │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
         ▲                                                  ▲
         │ dispatch                                         │ invoke
         │ (FR-1.2 前置)                                    │
         │                                                  │
┌────────┴──────────┐                              ┌───────┴──────────┐
│  Parent Agent     │                              │  Sub-Agent       │
│  (Mavis root)     │ ──────────────────────────── │  (worker/explore│
│                   │   via dispatcher.py          │   /verifier)    │
│                   │                              │  (隔离 context)  │
└───────────────────┘                              └──────────────────┘
```

### 2.2 跟 Claude Code `security-guidance` plugin 对照

| 维度 | Claude Code | Mavis 本设计 |
|---|---|---|
| 接入点 | PreToolUse hook on Edit/Bash | PreToolUse hook + 子代理 dispatch 前置 |
| 规则数 | 9 种 (3 BLOCK + 6 ASK) | 15 种 (8 BLOCK + 5 ASK + 2 WARN) |
| 规则格式 | hard-coded (plugin 源码) | JSON Schema (per FR-3.1) + 热更新 |
| 审计 | 无 (per README) | 完整 audit log (per FR-4) |
| User-defined rule | `hookify` plugin 独立 | v0.3 拍摄 User-defined Rule API |
| 跨平台 | POSIX + Windows | POSIX + Windows 双覆盖 |
| 延迟保证 | 未明说 (LLM hook 同步) | < 5ms p50 / < 10ms p99 (per NFR-P-1/NFR-P-2) |

### 2.3 跟现有守门的关系图

| 守门 | 关系 | 联动机制 |
|---|---|---|
| **#5 env 安全** | review guard (事后) → execution guard (事前) | 守门 #5 命中场景 100% 被本层 BLOCK (NFR-S-5) |
| **#1 推 origin** | 落 `git push origin` 走 ASK, 推荐项"取消" (FR-3.3) | 跟守门 #1 反转 2026-08-30 07:09 JST 联动 |
| **#9 子代理 RPC 不可靠** | dispatcher.py invoke() 前置 100% 覆盖 (FR-1.2 + NFR-S-4) | 跟守门 #9 v20 brief 落地 联动 |
| **#27 RPC fallback** | 互补: v27 管 invoke 之后, 本 BD 管 invoke 之前 | per `docs/guardian/v27_rpc_fallback.py` v0.1 |
| **#1 v15 docs 同步饱和** | docs 工作, 1 commit 多文件 (per `AGENTS.md` §4.1) | 本 BD 跟 SRS 同 commit |

### 2.4 5 view 跨域覆盖 (per IPA 基本设计 標準)

| View | 本 BD 对应章节 |
|---|---|
| **① 機能 view** | §3 機能分割 / モジュール設計 (5 module) |
| **② データ view** | §4 データモデル (3 表 W/T/M 100% 覆盖) |
| **③ 動作 view** | §3.6 状態遷移図 + §3.7 シーケンス図 |
| **④ モジュール view** | §3 機能分割 (跟 ① 連動) |
| **⑤ ネットワーク view** | §5.2 mavis runtime 連携 + ⑤不适用 (本 v0.1 0 网络层) |

---

## §3 機能分割 / モジュール設計 (Function Decomposition / Module Design)

### 3.1 全体モジュール構成

| Module ID | 名称 | 役割 | LOC 估算 | 入口 |
|---|---|---|---|---|
| **PM-1** | Pattern Matcher | 正则匹配 + 跨平台 path normalize | 200 | `evaluate(tool_call)` |
| **PM-2** | Rule Database | JSON 加载 + 热更新 + schema 校验 | 150 | `RuleDatabase.load()` |
| **PM-3** | Decision Router | BLOCK/ASK/WARN 决策 + 冲突优先级 | 100 | `DecisionRouter.route(matches)` |
| **PM-4** | Audit Logger | JSON Lines 写 + 异步 fsync | 100 | `AuditLogger.log(event)` |
| **PM-5** | User-defined Rule API | v0.3 拍摄, v0.1 stub | 50 (stub) | `register_user_rule(rule)` |
| **合計** | | | **~600 LOC** | |

### 3.2 PM-1 Pattern Matcher

**責務**: 给定 tool_call, 命中 15 条规则中的哪些, 返回 `Match[]`

**API**:
```python
def evaluate(tool_call: ToolCall) -> List[Match]:
    """Pure function, < 5ms p50 / < 10ms p99.
    
    Args:
        tool_call: {tool_name: str, tool_args: dict, session_id: str}
    
    Returns:
        List of Match: [{rule_id, decision, reason, latency_us}]
    """
```

**核心算法**:
1. 提取 `tool_args` 中可扫描字段:
   - `bash` 类: `args.command` (full string)
   - `write` / `edit` 类: `args.path` + `args.content`
   - `read` 类: 默认跳过 (per FR-1.3)
2. Path normalize: `pathlib.Path` + `os.path.normpath` 跨平台
3. 规则匹配: 对每条 rule, 跑 `re.search(pattern, scan_target)`, 命中返回 Match
4. 返回所有 Match, 冲突由 PM-3 处理

**依赖**: stdlib `re` / `pathlib` / `os` (per SRS §6.1)

### 3.3 PM-2 Rule Database

**責務**: 加载 / 校验 / 热更新 规则 JSON

**数据格式** (per §4.3.1 Rule schema):
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
    }
  ]
}
```

**热更新机制** (per FR-5.1):
- 启动时一次性加载 → `RuleDatabase.rules`
- 后台 thread 监听文件 mtime, 变化时 reload
- reload 失败 → 保留旧 rules + WARN log (per FR-6.1)

**Schema 校验**: 用 `jsonschema` (stdlib 不可用, 引入 1 个依赖, per SRS §6.1 例外)

### 3.4 PM-3 Decision Router

**責務**: 多 Match 冲突时, 按优先级选 1 个 decision

**优先级规则** (per SRS 已知缺口 #4):
```
BLOCK > ASK > WARN > PASS
```
- 任何 BLOCK 命中 → decision = BLOCK
- 无 BLOCK 但 ASK 命中 → decision = ASK (走 ask_user)
- 无 BLOCK / ASK 但 WARN 命中 → decision = WARN
- 全部不命中 → decision = PASS

**API**:
```python
def route(matches: List[Match]) -> Decision:
    """Pure function, O(n) where n = len(matches)."""
```

### 3.5 PM-4 Audit Logger

**責務**: 写 JSON Lines 到 `pre_tool_use_audit.log`

**单条格式** (per §4.3.2 Audit log schema):
```json
{"ts":"2026-09-10T19:03:25.123+09:00","session_id":"mvs_xxx","agent_role":"orchestrator","tool":"bash","tool_args_hash":"sha256:...","args_excerpt":"rm -rf /tmp/...","decision":"BLOCK","rule_id":"R-BLOCK-001","reason":"...","latency_ms":2.3,"env_hash":"sha256:..."}
```

**特性**:
- 异步 fsync (per NFR-P-3 < 1ms)
- 文件 mtime 保留 90 天
- append-only (per FR-4.2 + 守门 #13 Transaction)
- 写失败 → fail-closed (per FR-6.2 + NFR-A-2)

**注意**: 写 `tool_args` 完整内容**不安全** (可能含 secret), 仅写 `tool_args_hash` (sha256) + `args_excerpt` (前 200 chars, 脱敏)

### 3.6 状態遷移図 (State Machine)

```
[LLM 决定调 tool]
        │
        ▼
[PreToolUse Hook 触发]
        │
        ├──→ [PM-1 Pattern Matcher] ──→ Match[]
        │                                    │
        │                                    ▼
        │                            [PM-3 Decision Router]
        │                                    │
        │                                    ├──→ BLOCK ──→ [拒绝 tool call + error]
        │                                    ├──→ ASK   ──→ [ask_user + 等待]
        │                                    ├──→ WARN  ──→ [system_reminder + 继续]
        │                                    └──→ PASS  ──→ [执行 tool]
        │
        └──→ [PM-4 Audit Logger] ──→ 写 audit log (无论 decision)
```

### 3.7 シーケンス図 (Sequence Diagram)

#### 3.7.1 正常 case (PASS)

```
LLM        Hook       PM-1      PM-2      PM-3      PM-4     Tool
 │           │          │         │         │         │         │
 │─decide──▶│          │         │         │         │         │
 │          │─load()──▶│         │         │         │         │
 │          │◀─rules───│         │         │         │         │
 │          │─eval()──▶│         │         │         │         │
 │          │          │─query──▶│         │         │         │
 │          │          │◀─rules──│         │         │         │
 │          │          │─match()▶│         │         │         │
 │          │          │         │         │         │         │
 │          │          │─route()────────────────────▶│         │
 │          │          │         │         │         │         │
 │          │          │─log()─────────────────────────────────▶│
 │          │          │         │         │         │         │
 │          │◀─PASS───────────────────────────────│         │         │
 │          │───────────────────────────────────────────execute▶│
```

#### 3.7.2 BLOCK case

```
LLM        Hook       PM-1      PM-3      PM-4     LLM
 │           │          │         │         │         │
 │─decide──▶│          │         │         │         │
 │          │─eval()──▶│         │         │         │
 │          │          │─match()→│         │         │
 │          │          │         │         │         │
 │          │          │         │         │         │
 │          │◀─BLOCK───┤         │         │         │
 │          │─log()───────────────────────▶│         │
 │          │          │         │         │         │
 │          │◀─error────────────────────────────────────│
 │          │          │         │         │  LLM 改 plan  │
```

#### 3.7.3 故障 case (rule load fail)

```
Hook         PM-2 (load)    PM-3     PM-4
  │              │             │         │
  │─load()──────▶│             │         │
  │              │─json parse FAIL         │
  │◀─empty rules─┤             │         │
  │              │             │         │
  │─fallback PASS (per FR-6.1) │         │
  │─log WARN─────────────────────────────▶│
  │              │             │         │
  │─continue tool execution    │         │
```

---

## §4 データモデル (Data Model)

### 4.1 数据表一覧 (per 守门 #13, W/T/M 100% 覆盖)

| 表名 | 类别 | 物理删除 | 审计 | RLS | 用途 |
|---|---|---|---|---|---|
| `pre_tool_use_rules` | **W/M 混合** (W = 启动加载 + 热更新, M = 规则定义本身是参考) | ❌ | ❌ | 13 类 | 危险模式定义 |
| `pre_tool_use_audit` | **T** (Transaction append-only) | ❌ | ✅ 自身就是审计 | 13 类 | 工具调用审计 |
| `pre_tool_use_session_state` | **M** (参考状态, SCD Type 2) | ❌ | ❌ | 13 类 | session 级别 hook 状态 |

> 注: 本系统是文件系统存储 (JSON / JSON Lines), 不是 RDB. 但仍按守门 #13 W/T/M 三类横展 (per STAR 守门).

### 4.2 W/T/M 横展详细

#### 4.2.1 `pre_tool_use_rules` (W/M 混合)

- **W 性质**: 启动加载 → 内存态, 短期 TTL (到 mavis runtime 重启)
- **M 性质**: 规则定义本身是参考数据, SCD Type 2 (规则增删改留痕)
- **物理删除**: 禁止
- **审计**: 不需要 (规则增删是人工操作, 走 git 审计)
- **RLS**: 13 类 (per 守门 #13, mavis 进程独占, 0 用户)
- **Schema**: 见 §4.3.1

#### 4.2.2 `pre_tool_use_audit` (T)

- **T 性质**: Transaction append-only
- **物理删除**: 禁止 (per 守门 #13 + NFR-S-2)
- **审计**: 自身就是审计
- **RLS**: 13 类
- **Schema**: 见 §4.3.2
- **保留期**: 90 天 (per FR-4.1, v0.2 可配置)

#### 4.2.3 `pre_tool_use_session_state` (M)

- **M 性质**: 参考状态, session 级别
- **物理删除**: 禁止
- **SCD Type 2**: 状态变化留痕
- **Schema**: 见 §4.3.3
- **示例字段**: `last_reload_ts` / `rules_loaded_count` / `last_block_rule_id`

### 4.3 Schema 詳細

#### 4.3.1 Rule schema (`pre_tool_use_rules.json`)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["schema_version", "rules"],
  "properties": {
    "schema_version": {"type": "string", "pattern": "^\\d+\\.\\d+$"},
    "rules": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "required": ["id", "name", "level", "tool_class", "pattern", "reason"],
        "properties": {
          "id": {"type": "string", "pattern": "^R-(BLOCK|ASK|WARN)-\\d{3}$"},
          "name": {"type": "string", "minLength": 1, "maxLength": 100},
          "level": {"enum": ["BLOCK", "ASK", "WARN"]},
          "tool_class": {"enum": ["bash", "write", "edit", "read", "mcp"]},
          "pattern": {"type": "string", "minLength": 1, "maxLength": 500},
          "reason": {"type": "string", "minLength": 1, "maxLength": 300}
        }
      }
    }
  }
}
```

#### 4.3.2 Audit log schema (`pre_tool_use_audit.log` JSON Lines)

每行 1 个 JSON object, 11 字段:

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `ts` | string (ISO 8601) | ✅ | 事件时间戳 |
| `session_id` | string | ✅ | mavis session id |
| `agent_role` | string (enum) | ✅ | orchestrator / worker / explore / verifier |
| `tool` | string | ✅ | tool name (bash/write/edit/...) |
| `tool_args_hash` | string (`sha256:hex`) | ✅ | tool args 完整 sha256 (脱敏) |
| `args_excerpt` | string (≤ 200 chars) | ✅ | tool args 前 200 chars (脱敏) |
| `decision` | string (enum) | ✅ | BLOCK/ASK/WARN/PASS |
| `rule_id` | string | ⚠️ (decision ≠ PASS 时必填) | 命中规则 id |
| `reason` | string | ⚠️ (decision ≠ PASS 时必填) | 规则 reason |
| `latency_ms` | number | ✅ | 匹配耗时 (ms) |
| `env_hash` | string (`sha256:hex`) | ✅ | 环境变量 sha256 (避免 secret 写入) |

#### 4.3.3 Session state schema (`pre_tool_use_session_state.json`)

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

---

## §5 インターフェース設計 (Interface Design)

### 5.1 PreToolUse hook 内部 API

#### 5.1.1 `evaluate(tool_call)` 入口

```python
def evaluate(tool_call: ToolCall) -> Decision:
    """PreToolUse hook 入口, Mavis mavis runtime 调用.
    
    Args:
        tool_call: {
            "tool_name": "bash",  # str
            "tool_args": {"command": "rm -rf /tmp/xxx"},  # dict
            "session_id": "mvs_xxx",  # str
            "agent_role": "orchestrator"  # str
        }
    
    Returns:
        Decision: {
            "decision": "BLOCK",  # BLOCK/ASK/WARN/PASS
            "reason": "...",  # str, decision ≠ PASS 时必填
            "rule_id": "R-BLOCK-001",  # str, decision ≠ PASS 时必填
            "latency_ms": 2.3  # float
        }
    
    Raises:
        RuleLoadError: 规则加载失败 → 内部捕获, 返回 PASS + WARN log (per FR-6.1)
        AuditLogError: 审计写失败 → 返回 BLOCK (per FR-6.2 + NFR-A-2)
    """
```

#### 5.1.2 `reload_rules()` 热更新

```python
def reload_rules() -> ReloadResult:
    """重新加载规则 JSON, 不重启 mavis runtime.
    
    Returns:
        ReloadResult: {
            "success": True,
            "rules_count": 15,
            "reload_latency_ms": 87
        } | {
            "success": False,
            "error": "JSON parse error at line 5",
            "fallback_rules_count": 15  # 保留旧规则
        }
    """
```

#### 5.1.3 `query_audit(filters)` 查询

```python
def query_audit(
    session_id: Optional[str] = None,
    decision: Optional[str] = None,
    rule_id: Optional[str] = None,
    start_ts: Optional[str] = None,
    end_ts: Optional[str] = None,
    limit: int = 100
) -> List[AuditEvent]:
    """查询 audit log, 供 mavis 调试 / 报告使用."""
```

### 5.2 跟 mavis runtime 的接口

**位置**: `scripts/automation/console_server.py` v0.1 line 80-92 范围

**集成方式**: 
- mavis runtime 在 PreToolUse 事件触发时, 调用 `evaluate(tool_call)`
- mavis runtime 不感知规则内部结构, 只关心 `decision` 字段
- BLOCK 决策: mavis runtime 拒绝 tool call, 返回 error 给 LLM
- ASK 决策: mavis runtime 调 `ask_user` 工具 (per 守门 v28)
- WARN 决策: mavis runtime 注入 `system_reminder` 风格 message

### 5.3 跟 ask_user 的接口

**位置**: mavis runtime 内置 `ask_user` 工具

**集成方式**:
- PreToolUse Guard 返回 ASK decision
- mavis runtime 自动生成 3 选项: `[取消 (推荐), 确认执行, 改用其他方案]`
- 推荐项放第一位, per 守门 v28 拍板必带推荐项
- description 写"做这件事的具体后果"

### 5.4 跟 dispatcher.py 的接口

**位置**: `scripts/automation/dispatcher.py` v0.1 `invoke()` 函数前

**集成方式**:
```python
# dispatcher.py v0.1 invoke() 前置
def invoke(task_id, script_path, args, parent_session_id):
    # ★ 新增: PreToolUse Guard dispatch 前置 (per FR-1.2)
    decision = pre_tool_use_guard.evaluate_for_dispatch({
        "task_id": task_id,
        "script_path": script_path,
        "args": args,
        "parent_session_id": parent_session_id
    })
    if decision["decision"] == "BLOCK":
        raise DispatchBlocked(decision["reason"], decision["rule_id"])
    if decision["decision"] == "ASK":
        # 走 ask_user
        ...
    # PASS → 继续 invoke
    ...
```

### 5.5 跟 file watcher 的接口

**机制**: 用 stdlib `watchdog` 库 (新增 1 依赖, per SRS §6.1 例外)

**集成方式**:
- 启动时注册 watcher on `pre_tool_use_rules.json`
- mtime 变化 → 自动 reload
- reload 失败 → 保留旧 rules + WARN log

### 5.6 跟 pytest / CI 的接口

**位置**: `tests/automation/guardian/test_pre_tool_use_guard.py` + `tests/e2e/test_pre_tool_use_hook.py`

**集成方式**:
- pytest fixtures 提供 mock tool_call
- 单元测试覆盖 15 条规则 + fail-open / fail-closed
- 集成测试覆盖 console_server.py 实际调用
- CI 双平台 (Windows + POSIX) 跑全测试

---

## §6 セキュリティ設計 (Security Design)

### 6.1 凭据零外泄 (per NFR-S-1 + 守门 #5)

- audit log 仅写 `tool_args_hash` (sha256) + `args_excerpt` (前 200 chars)
- 不写 `env:*` 任何字段, 仅写 `env_hash` (sha256 of all env keys, not values)
- BLOCK 规则覆盖 8 种凭据场景 (per FR-3.2 第 6-8 条)

### 6.2 audit log 完整性 (per NFR-S-2 + 守门 #13 Transaction)

- append-only 文件, OS 权限 `0444` (只读, mavis 进程外不可写)
- 不允许 truncate / 物理删除
- 90 天保留期 (后续 v0.2 可配置)
- log rotation 走 `logrotate.d` 风格, 旧文件 `gzip` 压缩保留

### 6.3 规则文件只读 (per NFR-S-3)

- `pre_tool_use_rules.json` OS 权限 `0644`, mavis 进程读 + 写 (热更新)
- 写操作仅限 file watcher 触发, 不开放 user direct write
- git 仓库审计 (跟 mavis 仓同步, 任何规则改动走 commit + Ulysses review)

### 6.4 子代理 dispatch 前置 100% 覆盖 (per NFR-S-4)

- dispatcher.py `invoke()` 函数前**必先**调 `evaluate_for_dispatch`
- 单元测试 100% line coverage
- 任何 invoke path 绕过 → 测试 fail

### 6.5 跟守门 #5 联动 (per NFR-S-5)

- 守门 #5 命中场景 (env var 打印 / secret 泄露) → 本层 BLOCK 规则 100% 覆盖
- 单元测试: 15 条规则 × 守门 #5 场景 = 双向验证
- 守门 #5 跟本层互补: 守门 #5 是 review guard, 本层是 execution guard

---

## §7 非機能設計 (Non-Functional Design)

### 7.1 性能 (per NFR-P)

| ID | 设计 | 計測方法 |
|---|---|---|
| NFR-P-1 | Pattern Matcher 单条规则匹配 O(1) (regex 编译缓存), 15 条规则总耗时 < 5ms p50 | pytest-benchmark |
| NFR-P-2 | p99 走 worst-case (8 BLOCK 全部命中) 实测 | pytest-benchmark |
| NFR-P-3 | Audit Logger 异步 fsync, 写延迟 < 1ms | pytest-benchmark |
| NFR-P-4 | Reload 走文件 mtime poll, 1s 间隔, reload 本身 < 100ms | 实测 |
| NFR-P-5 | Hook 接入零侵入, mavis runtime 主流程 0 额外延迟 | 对比 baseline benchmark |

### 7.2 容错 (per NFR-A)

| ID | 设计 |
|---|---|
| NFR-A-1 | 规则加载失败 → 内部 try/except 捕获, 返回 PASS + WARN log (per FR-6.1) |
| NFR-A-2 | 审计写失败 → 内部 try/except 捕获, 返回 BLOCK (per FR-6.2) |
| NFR-A-3 | 规则 reload 期间, hook 调用 acquire read lock, 不中断 |
| NFR-A-4 | 启动时 mavis 启动时间 +50ms (15 条规则加载 + schema 校验) |

### 7.3 保守性 (per NFR-M)

| ID | 设计 |
|---|---|
| NFR-M-1 | 规则 schema 文档化 (本 BD §4.3.1 + 注释) |
| NFR-M-2 | 规则增删改仅改 JSON, 0 Python 代码改动 |
| NFR-M-3 | audit log 严格 JSON Lines, 行内 `json.loads()` 100% 成功 |
| NFR-M-4 | pytest coverage ≥ 90% (跟守门 #1 v6 release 模式 test 100% 对齐) |

### 7.4 移植性 (per NFR-T)

| ID | 设计 |
|---|---|
| NFR-T-1 | Windows: 用 `pathlib.Path` (自动处理 `\`), regex 兼容 CRLF |
| NFR-T-2 | POSIX: 标准 pathlib + regex |
| NFR-T-3 | Path normalize: `os.path.normpath` + 跨平台 test fixtures |
| NFR-T-4 | Python 3.10+ `match` / `case` 语法 (跟 mavis runtime 一致) |

### 7.5 可观测性 (per NFR-O)

| ID | 设计 |
|---|---|
| NFR-O-1 | mavis root session 每 1min 聚合 decision 统计, 写入 `pre_tool_use_session_state.json` |
| NFR-O-2 | 任何异常 trace, 不静默吞错 (per 9/1 P0-1 实证教训) |
| NFR-O-3 | BLOCK 事件触发 root session 通知 (走 mavis 内置 notification) < 500ms |

---

## §8 障害 / 運用設計 (Failure / Operation Design)

### 8.1 故障模式一覧

| 故障 ID | 描述 | 严重度 | 检测方法 | 恢复策略 |
|---|---|---|---|---|
| F-1 | 规则 JSON 解析失败 | 中 | schema 校验 | fail-open (per FR-6.1) + WARN log |
| F-2 | 规则 JSON schema 校验失败 | 中 | jsonschema 校验 | 同 F-1 |
| F-3 | audit log 写失败 (磁盘满) | 高 | OS error | fail-closed (per FR-6.2) |
| F-4 | audit log 写失败 (权限不足) | 高 | OS error | fail-closed (per FR-6.2) |
| F-5 | file watcher 启动失败 | 低 | watchdog 启动 error | 退化为 mtime poll (1s 间隔) |
| F-6 | 子代理 dispatch 前置 异常 | 高 | dispatcher.py try/except | fail-open + BLOCK 通知 (per NFR-O-3) |
| F-7 | 规则匹配 timeout (正则 catastrophic backtracking) | 中 | regex timeout (1s) | 跳过该规则, WARN log |
| F-8 | 跨平台 path 解析错误 | 低 | pathlib 异常 | 退化为 raw string 匹配 |

### 8.2 恢复策略 (Recovery)

| 故障 | 自动恢复 | 手动恢复 |
|---|---|---|
| F-1 / F-2 | 下次 reload 自动重试 | `python scripts/automation/guardian/reload_rules.py --validate` |
| F-3 / F-4 | 磁盘 / 权限恢复后自动 retry | 清理磁盘 / 修复权限 |
| F-5 | watchdog 重启 (mavis runtime 重启) | 不需要手动 |
| F-6 | dispatcher.py 自动 retry 1 次 | root session 通知 |
| F-7 | 跳过该规则, 其他规则继续 | 规则作者修复 regex |
| F-8 | 退化为 raw string | 报告 bug, 修复 pathlib 处理 |

### 8.3 监控 / Alert (per NFR-O)

- **Metric 1**: `decision_count.{BLOCK,ASK,WARN,PASS}` 1min 聚合
- **Metric 2**: `rule_hit_count` 按 rule_id 维度
- **Metric 3**: `latency_ms_p50/p99` 滚动 1min
- **Alert 1**: BLOCK 事件 > 阈值 → root session 通知 (per NFR-O-3)
- **Alert 2**: audit log 写失败 → 立即 BLOCK + 通知
- **Alert 3**: 规则 load fail > 3 次连续 → 通知

### 8.4 Backup / Rollback

- **Backup**: `pre_tool_use_rules.json` 走 git 仓库审计 (跟 mavis 仓同步)
- **Rollback**: `git revert <commit>`, file watcher 自动 reload
- **无独立 backup 机制**: 简化, 走 git 单一来源

### 8.5 log rotation

- `pre_tool_use_audit.log` 单文件 > 100MB 或 > 7 天 → 自动 rotate
- rotate 走 `gzip` 压缩, 保留 12 份 (90 天)
- 旧文件 OS 权限 `0444` (只读)

---

## §9 用語集 (Glossary)

跟 [`SRS-PRE-TOOL-USE-GUARD-001.md` §2](../requirements/SRS-PRE-TOOL-USE-GUARD-001.md#2-用語定義--略語-glossary) 共享, 本 BD 新增 5 条:

| 用語 | 定義 |
|---|---|
| **Pattern Matcher** | 给定 tool_call, 跑所有规则, 返回 Match[] 的 pure function |
| **Decision Router** | 多 Match 冲突时, 按 BLOCK > ASK > WARN > PASS 优先级选 1 个 |
| **Audit Logger** | 异步 JSON Lines 写, append-only, fail-closed |
| **Schema 校验** | 用 jsonschema 库校验规则 JSON, 失败 → fail-open |
| **File Watcher** | watchdog 库监听规则文件 mtime, 自动 reload |

---

## §10 引用 / 关联文档 (References)

### 10.1 上位 / 平行

- [`SRS-PRE-TOOL-USE-GUARD-001.md`](../requirements/SRS-PRE-TOOL-USE-GUARD-001.md) v0.1 (上位要件)
- `AGENTS.md` §4 守门硬约束
- `docs/automation-design.md` v0.1
- `docs/guardian/README.md`
- `docs/guardian/v27_rpc_fallback.md`, `v28_recommendation.md`, `v29_docs_saturation.md` (守门 v3x 落档)

### 10.2 关联 commit / 实装

- `scripts/automation/console_server.py` v0.1 (PreToolUse 接入点)
- `scripts/automation/dispatcher.py` v0.1 (子代理 invoke 前置)

### 10.3 对照基线 (外部)

- claude code `anthropics/claude-code` GitHub repo
- claude code `plugins/security-guidance` (9 条 → 本 v0.1 扩展 15 条)
- claude code `plugins/hookify` (user-defined rule 机制 → v0.3 拍摄)

### 10.4 报告模板 (后续)

- `docs/reports/PHASE-PRE-TOOL-USE-GUARD-IMPL-REPORT.md` (待 v0.1 实装后落档)

---

## §11 签字栏 (Sign-off)

| 角色 | 氏名 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | ✅ 2026-09-10 | 2026-09-10 JST |
| SRE Lead | SRE Lead (Mavis 临时代签 per 9/3 11:35 JST 拍板 B, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 平台 Lead | 平台 Lead (Mavis 临时代签 per 守门 #14 v3, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 评审主持 | 评审主持 (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |
| PM | PM (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |

(per 守门 #14 v4 反转 v0.62, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)

---

## §12 修订履歴 (Revision History)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 19:03 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 12 段 (目的/範囲/架构/機能/数据/接口/安全/非機能/障害/用語/引用/签字+修订), 5 module (PM-1~PM-5 ~600 LOC), 3 表 W/T/M 100% 覆盖 (Rule W/M + Audit T + Session M, per 守门 #13), 7 集成点 (console_server/dispatcher/mavis hook/ask_user/file watcher/pytest/CI), 3 内部 API (evaluate/reload_rules/query_audit), 8 故障模式 (F-1~F-8), 守门 6/6 通过, IPA 12 段结构完整 | 2026-09-10 19:03 JST Ulysses 拍板"整理出需求文档, 然后基于需求文档写基本设计, 要符合日本 IPA 的水准和规范" |
