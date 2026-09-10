# WBS-002-pre-tool-use-guard — PreToolUse 安全拦截层 实施计划 + WBS

> **Status**: 🟡 Draft v0.1 (2026-09-10 JST 初版落档, 待 Ulysses review)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）
> **Pair with**:
> - [`docs/requirements/SRS-PRE-TOOL-USE-GUARD-001.md`](../requirements/SRS-PRE-TOOL-USE-GUARD-001.md) v0.1 (需求)
> - [`docs/design/BD-PRE-TOOL-USE-GUARD-001.md`](../design/BD-PRE-TOOL-USE-GUARD-001.md) v0.1 (基本设计)
> - [`docs/detailed-design/DD-PRE-TOOL-USE-GUARD-001.md`](../detailed-design/DD-PRE-TOOL-USE-GUARD-001.md) v0.1 (详细设计)
> - [`docs/specs/guard-pre-tool-use-spec.md`](../specs/guard-pre-tool-use-spec.md) v0.1 (实施 spec)
> **修订人**: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #10 + 守门 #14 v3)
> **审批**: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST)

---

## 0. 实施目标 + 总览表

### 0.1 实施目标 (Goals)

1. **G-1**: 落地 `scripts/automation/guardian/` 目录 + 5 module + 15 条规则 (per BD §1.1)
2. **G-2**: 接入 `console_server.py` v0.1 + `dispatcher.py` v0.1 的 PreToolUse hook 接入点 (per DD §1.3)
3. **G-3**: 单元测试覆盖率 ≥ 90% (per NFR-M-4) + 跨平台 CI 100% pass (per AC-5)
4. **G-4**: 150 渗透测试 0 命中 (per AC-7 + 守门 #5 联动)
5. **G-5**: 实装报告 `PHASE-PRE-TOOL-USE-GUARD-IMPL-REPORT.md` 落档 (per SRS §7 + 守门 #12 docs 同步)

### 0.2 非目标 (Non-Goals)

1. ❌ 网络层拦截 (egress filtering) → v2.x
2. ❌ AI 行为审计 (LLM 决策录屏) → v2.x
3. ❌ 子代理内部代码越权 (subprocess sandbox) → v0.2 拍摄 (SRS 已知缺口 #1 P0)
4. ❌ User-defined rule 实装 → v0.3 拍摄 (本 spec 仅 stub, per guard-pre-tool-use-spec §2.4)
5. ❌ log rotation 实装 → v0.2 拍摄 (本 v0.1 走外部 logrotate 配置)

### 0.3 总览表 (12 task, T1-T3 档位)

| # | 任务 | 档位 | 估 token | 依赖 | 同步设计文档 | 状态 |
|---|---|---|---|---|---|---|
| **T1.1** | 创建 guardian 目录结构 + `__init__.py` + `pyproject.toml` | T1 | 0.05M | 无 | 无 (脚手架) | ⚪ 未开始 |
| **T1.2** | 编写 15 条规则 JSON (`pre_tool_use_rules.json`) | T1 | 0.1M | T1.1 | 无 (per DD §5.1) | ⚪ 未开始 |
| **T1.3** | `rule_database.py` (PM-2 规则加载/热更新/schema 校验) | T1 | 0.2M | T1.1 + T1.2 | 无 (per DD §2.2) | ⚪ 未开始 |
| **T1.4** | `audit_logger.py` (PM-4 JSON Lines 写入/fail-closed) | T1 | 0.15M | T1.1 | 无 (per DD §2.3) | ⚪ 未开始 |
| **T1.5** | `pre_tool_use_guard.py` (PM-1+PM-3 Pattern Matcher + Decision Router) | T1 | 0.3M | T1.3 + T1.4 | 无 (per DD §2.1) | ⚪ 未开始 |
| **T1.6** | `user_rule_api.py` (PM-5 v0.1 stub) + `hooks/console_server_hook.py` + `hooks/dispatcher_hook.py` | T1 | 0.1M | T1.5 | 无 (per DD §2.4 + §1.3) | ⚪ 未开始 |
| **T2.1** | 单元测试 - `test_pre_tool_use_guard.py` (10+ TC, 覆盖 15 规则 + fail-open/closed + p50/p99 benchmark) | T2 | 0.4M | T1.5 | 无 (per DD §7.2) | ⚪ 未开始 |
| **T2.2** | 单元测试 - `test_rule_database.py` (8+ TC, 加载/热更新/schema 校验失败) | T2 | 0.2M | T1.3 | 无 | ⚪ 未开始 |
| **T2.3** | 单元测试 - `test_audit_logger.py` (5+ TC, 写入/异步 fsync/fail-closed) | T2 | 0.15M | T1.4 | 无 | ⚪ 未开始 |
| **T2.4** | 集成测试 - `tests/e2e/test_pre_tool_use_hook.py` (5+ TC, console_server + dispatcher) | T2 | 0.3M | T1.6 | 无 (per DD §7.3) | ⚪ 未开始 |
| **T2.5** | 渗透测试 - `tests/security/test_pre_tool_use_penetration.py` (150 TC, 15 规则 × 10 攻击) | T2 | 0.4M | T1.5 | 无 (per AC-7 + 守门 #5) | ⚪ 未开始 |
| **T3.1** | CI 跨平台测试 (Windows + POSIX GitHub Actions) | T3 | 0.15M | T2.1~T2.5 | 必做: `.github/workflows/pre-tool-use-guard.yml` (新建) | ⚪ 未开始 |
| **T3.2** | 接入 console_server.py + dispatcher.py 实装 (per DD §1.3) | T3 | 0.2M | T3.1 | 必做: `console_server.py` v0.2 升级 + `dispatcher.py` v0.2 升级 (2 commit) | ⚪ 未开始 |
| **T3.3** | 实装报告 `PHASE-PRE-TOOL-USE-GUARD-IMPL-REPORT.md` 落档 | T3 | 0.2M | T3.2 | 必做: `docs/reports/PHASE-PRE-TOOL-USE-GUARD-IMPL-REPORT.md` (新建, 跟现有 6 份 PHASE-*-IMPL-REPORT 一致) | ⚪ 未开始 |
| **小计** | | | **~2.9M** | | | **0/14 完成** |

> **档位说明** (per `docs/refactor/RF-001-spec.md`):
> - **T1**: 单文件, 0 跨文件依赖, < 0.3M token
> - **T2**: 多文件, 跨模块依赖, 0.15-0.4M token
> - **T3**: 跨 crate / 跨项目, 0.15-0.2M token

> **token 估算方法** (per `STAR-OLU-001.md` 换算基线):
> - 1 SRE·周 ≈ 1.2M tokens
> - 1 人·天 ≈ 100-300K tokens
> - 本 WBS 总计 ~2.9M tokens ≈ 2.4 SRE·周 ≈ 12-29 人·天

---

## 1. 实施路线图 (Roadmap)

### 1.1 阶段划分

| 阶段 | 任务 | 持续时间 | 守门验证 |
|---|---|---|---|
| **Phase 1: 脚手架 + 数据** | T1.1, T1.2 | 1 day | 目录创建 + JSON 校验通过 |
| **Phase 2: 核心模块** | T1.3, T1.4, T1.5, T1.6 | 3 days | 5 module 编译通过 + watchdog 启动 |
| **Phase 3: 单元测试** | T2.1, T2.2, T2.3 | 2 days | 23+ TC 100% pass + coverage ≥ 90% |
| **Phase 4: 集成 + 渗透** | T2.4, T2.5 | 2 days | 5+ e2e + 150 渗透 0 命中 |
| **Phase 5: CI + 集成** | T3.1, T3.2, T3.3 | 2 days | 跨平台 CI + console_server/dispatcher 接入 + 报告落档 |
| **总计** | 14 task | **~10 days (2 SRE·周)** | |

### 1.2 建议执行顺序

```
T1.1 ──▶ T1.2 ──▶ T1.3 ──┐
                          ├──▶ T1.5 ──▶ T1.6 ──┐
              T1.4 ────────┘                     │
                                                ├──▶ T2.1 ──▶ T2.4 ──▶ T3.1 ──▶ T3.2 ──▶ T3.3
                            T1.3 ──▶ T2.2 ──────┤
                            T1.4 ──▶ T2.3 ──────┘
                                                │
                                  T1.5 ──▶ T2.5 ┘
```

**关键路径**: T1.1 → T1.2 → T1.3 → T1.5 → T2.1 → T3.1 → T3.2 → T3.3 (8 task, ~6-7 days)

---

## 2. T1 详情 (脚手架 + 核心模块, 4 days)

### T1.1 创建 guardian 目录结构

**目标**: 落地 `scripts/automation/guardian/` 物理目录, 含 `__init__.py` + `pyproject.toml`

**步骤**:
1. `mkdir -p scripts/automation/guardian/{rules,logs,state,hooks}`
2. `touch scripts/automation/guardian/__init__.py` (空文件, 标识 package)
3. 创建 `scripts/automation/guardian/pyproject.toml` (per DD §1.2)
4. 创建 `scripts/automation/guardian/README.md` (简述, 引用 SRS / BD / DD)
5. `git add` + `git commit -m "feat(guardian): 创建 PreToolUse 安全拦截层脚手架"` (author=Ulysses)

**Acceptance**:
- 目录存在 + 4 个子目录
- `pyproject.toml` 含 2 运行依赖 + 2 dev 依赖
- `python -c "from scripts.automation.guardian import __init__"` 不报错

**风险**:
- 0 (纯脚手架)

### T1.2 编写 15 条规则 JSON

**目标**: 落地 `scripts/automation/guardian/rules/pre_tool_use_rules.json`, 含 15 条规则 (per DD §5.1)

**步骤**:
1. 复制 DD §5.1 的 JSON 模板到 `pre_tool_use_rules.json`
2. `python -c "import json, jsonschema; ...; jsonschema.validate(...)"` 验证 schema
3. 验证 15 条规则 id 唯一性 + 格式 (`R-(BLOCK|ASK|WARN)-\d{3}`)
4. 验证每条规则的 pattern 至少能匹配 1 个 fixture (后续 T2.1 用)
5. `git add` + `git commit -m "feat(guardian): 15 条危险模式规则初版 (BLOCK 8 + ASK 5 + WARN 2)"` (author=Ulysses)

**Acceptance**:
- 15 条规则 + 全部 schema 校验通过
- 每条规则至少 1 个测试 fixture 命中

**风险**:
- regex 误匹配 false positive → 后续 T2.1 测试发现, 修
- regex catastrophic backtracking → 测试时 benchmark, 加 timeout (per F-7)

### T1.3 `rule_database.py` (PM-2)

**目标**: 实现 PM-2 规则加载/热更新/schema 校验 (per DD §2.2)

**步骤**:
1. 创建 `rule_database.py` 含 `Rule` dataclass + `RuleDatabase` class + `RULE_SCHEMA` constant
2. 实现 `_load_rules()` 走 `jsonschema.validate` + 错误捕获 (per FR-6.1)
3. 实现 `_start_watcher()` 走 watchdog, 失败 fallback mtime poll (per F-5)
4. 实现 `get_rules()` 用 `threading.RLock` 保护 (per DD §2.2)
5. 手动测试: 修改 JSON 看是否自动 reload
6. `git add` + `git commit -m "feat(guardian): RuleDatabase PM-2 规则加载/热更新/schema 校验"` (author=Ulysses)

**Acceptance**:
- 加载 15 条规则 0 错
- 修改 JSON 后 < 100ms 自动 reload (per NFR-P-4)
- 错误 JSON 不抛异常, 保留旧规则 (per FR-6.1)

**风险**:
- watchdog 跨平台兼容性 → Windows / POSIX 双测 (后续 T3.1)
- threading 死锁 → 用 RLock (可重入) 避免

### T1.4 `audit_logger.py` (PM-4)

**目标**: 实现 PM-4 JSON Lines 写入, 同步写 + lazy fsync (per DD §2.3)

**步骤**:
1. 创建 `audit_logger.py` 含 `AuditEvent` dataclass + `AuditLogger` class + `AuditLogError` exception
2. 实现 `log()` 同步写, 失败 raise (per FR-6.2)
3. 实现 `query()` 读 + 过滤 (per BD §5.1.3)
4. 实现 `mkdir(parents=True, exist_ok=True)` + `os.chmod 0644`
5. 手动测试: 写 100 条, 用 `cat` 看是否 100 行 JSON Lines
6. `git add` + `git commit -m "feat(guardian): AuditLogger PM-4 JSON Lines 写入/fail-closed"` (author=Ulysses)

**Acceptance**:
- 写 100 条, 文件 100 行
- 写失败 (chmod 0444) 立即 raise
- query 过滤正确 (per session_id / decision / rule_id)

**风险**:
- 磁盘满 → 立即 raise, PreToolUseGuard 捕获后 BLOCK (per FR-6.2)
- 编码错误 (含非 UTF-8 字符) → `errors="replace"` 处理

### T1.5 `pre_tool_use_guard.py` (PM-1 + PM-3)

**目标**: 实现 PM-1 Pattern Matcher + PM-3 Decision Router + 主入口 (per DD §2.1)

**步骤**:
1. 创建 `pre_tool_use_guard.py` 含 `Decision` enum + `ToolCall` / `Match` / `AuditEvent` dataclass
2. 实现 `PatternMatcher.extract_scan_target()` (per DD §2.1, 跨平台)
3. 实现 `PatternMatcher.match_all()` 走 `re.search` + regex 缓存 (`@lru_cache`, per §4.2)
4. 实现 `DecisionRouter.route()` 按 BLOCK > ASK > WARN > PASS 优先级 (per DD §2.1)
5. 实现 `PreToolUseGuard.evaluate()` 主入口, 含 fail-open / fail-closed 二分 (per §4.6)
6. 实现 `PreToolUseGuard.evaluate_for_dispatch()` 子代理 dispatch 前置 (per §3.2)
7. `git add` + `git commit -m "feat(guardian): PreToolUseGuard PM-1+PM-3 Pattern Matcher + Decision Router"` (author=Ulysses)

**Acceptance**:
- 15 条规则全部跑通 (后续 T2.1 验证)
- fail-open / fail-closed 二分正确
- 延迟 p50 < 5ms (后续 T2.1 benchmark 验证)

**风险**:
- regex catastrophic backtracking → 后续 T2.1 benchmark 发现, 加 timeout (per F-7)
- cross-platform path 处理 bug → 跨平台 CI (T3.1) 验证

### T1.6 hooks + user_rule_api stub

**目标**: 实现 `console_server_hook.py` + `dispatcher_hook.py` + `user_rule_api.py` (per DD §2.4 + §1.3)

**步骤**:
1. 创建 `hooks/console_server_hook.py`: 在 console_server.py v0.1 line 80-92 插入 hook 调用样板 (注释, 不实装 console_server.py 修改)
2. 创建 `hooks/dispatcher_hook.py`: 在 dispatcher.py v0.1 invoke() 前插入 hook 调用样板 (注释)
3. 创建 `user_rule_api.py`: 仅 `NotImplementedError` stub (per DD §2.4)
4. `git add` + `git commit -m "feat(guardian): hooks 接入点 + user_rule_api stub"` (author=Ulysses)

**Acceptance**:
- 3 文件存在
- `console_server_hook.py` / `dispatcher_hook.py` 注释清晰, 实装者可复制
- `user_rule_api.py` 调用时立即 raise NotImplementedError

**风险**:
- 0 (样板 + stub)

---

## 3. T2 详情 (测试, 4 days)

### T2.1 单元测试 - pre_tool_use_guard.py

**目标**: `test_pre_tool_use_guard.py` 含 10+ TC, 覆盖 15 条规则 + fail-open/closed + p50/p99 (per DD §7.2)

**步骤**:
1. 创建 `tests/automation/guardian/__init__.py` (空)
2. 创建 `test_pre_tool_use_guard.py`, 含:
   - `test_r_block_001_命中_root` (TC-1.1)
   - `test_r_block_006_命中_env_print` (TC-1.5)
   - `test_r_ask_002_命中_sudo` (TC-1.7)
   - `test_p50_latency` (TC-2.1, pytest-benchmark)
   - `test_p99_latency` (TC-2.2, pytest-benchmark)
   - `test_rule_load_fail_open` (TC-3.1)
   - `test_audit_log_write_fail_closed` (TC-3.2)
   - 3+ 其他 TC (TC-1.2/1.3/1.4/1.6/1.8/1.9/1.10 中选 3)
3. `pytest tests/automation/guardian/test_pre_tool_use_guard.py -v --cov --cov-report=term-missing`
4. 验证 100% pass + coverage ≥ 90%
5. `git add` + `git commit -m "test(guardian): test_pre_tool_use_guard 10+ TC 覆盖"` (author=Ulysses)

**Acceptance**:
- 10+ TC 100% pass
- coverage ≥ 90% (per NFR-M-4)
- p50 < 5ms + p99 < 10ms benchmark 通过

**风险**:
- 跨平台 timing 差异 → CI 多平台跑, 取最差值
- 误匹配 false positive → 修 regex

### T2.2 单元测试 - rule_database.py

**目标**: `test_rule_database.py` 含 8+ TC, 覆盖加载/热更新/schema 校验失败

**步骤**:
1. 创建 `test_rule_database.py`, 含:
   - `test_load_valid_rules`
   - `test_load_invalid_json_keeps_old` (per FR-6.1)
   - `test_load_schema_violation_keeps_old`
   - `test_reload_after_modify` (per FR-5.1)
   - `test_reload_with_bad_pattern_keeps_old`
   - `test_concurrent_get_rules` (thread safety)
   - `test_reload_latency_under_100ms` (per NFR-P-4)
   - 1+ 其他
2. `pytest ... --cov --cov-report=term-missing`
3. 验证 100% pass
4. `git add` + `git commit -m "test(guardian): test_rule_database 8+ TC"` (author=Ulysses)

**Acceptance**:
- 8+ TC 100% pass
- coverage ≥ 90% (rule_database.py 自身)

**风险**:
- mtime 修改在不同文件系统精度不同 → 用 os.utime() 强制 mtime

### T2.3 单元测试 - audit_logger.py

**目标**: `test_audit_logger.py` 含 5+ TC, 覆盖写入/查询/fail-closed

**步骤**:
1. 创建 `test_audit_logger.py`, 含:
   - `test_log_single_event`
   - `test_log_100_events`
   - `test_log_readonly_raises` (per FR-6.2)
   - `test_query_by_session_id`
   - `test_query_by_decision`
   - 0+ 其他
2. `pytest ... --cov --cov-report=term-missing`
3. 验证 100% pass
4. `git add` + `git commit -m "test(guardian): test_audit_logger 5+ TC"` (author=Ulysses)

**Acceptance**:
- 5+ TC 100% pass
- coverage ≥ 90% (audit_logger.py 自身)

**风险**:
- chmod 在 Windows 上行为不同 → 跨平台测试 (T3.1) 验证

### T2.4 集成测试 - console_server + dispatcher

**目标**: `tests/e2e/test_pre_tool_use_hook.py` 含 5+ TC, 端到端验证 (per DD §7.3)

**步骤**:
1. 创建 `tests/e2e/test_pre_tool_use_hook.py`, 含:
   - `test_console_server_hook_real` (TC-E2E-1)
   - `test_console_server_block_rm_rf` (TC-E2E-2)
   - `test_dispatcher_hook_real` (TC-E2E-3)
   - `test_dispatcher_block_bad_dispatch` (TC-E2E-4)
   - 1+ 其他 (PASS case)
2. 用 `subprocess.Popen` 启 console_server.py, `requests` 调 API
3. 用 `subprocess.run` 调 dispatcher.py, 验证 exit code / stderr
4. 验证 100% pass
5. `git add` + `git commit -m "test(guardian): e2e 5+ TC"` (author=Ulysses)

**Acceptance**:
- 5+ TC 100% pass
- console_server 真实启动 + 调用
- dispatcher 真实 invoke + 前置 hook 拦截

**风险**:
- 端口冲突 → 用临时端口 / free_port fixture
- 启动慢 → 加 timeout + retry

### T2.5 渗透测试 - 150 TC

**目标**: `tests/security/test_pre_tool_use_penetration.py` 含 150 TC, 15 规则 × 10 攻击场景 (per AC-7 + 守门 #5)

**步骤**:
1. 创建 `tests/security/test_pre_tool_use_penetration.py`
2. 为每条规则 (15 条) 写 10 个攻击 fixture, e.g.
   - R-BLOCK-001 (rm -rf): `rm -rf /` / `rm -rf ~` / `rm -rf /*` / `rm -fr /` / `rm -rf $HOME` / `rm -rf /tmp/../` / 等等
   - R-BLOCK-007 (GitHub PAT): `curl -H "Authorization: token ghp_xxx"` / `git push https://ghp_xxx@github.com/...` / 等等
3. 每个攻击 fixture 验证 decision ∈ {BLOCK, ASK}
4. 验证 150 TC 100% pass
5. `git add` + `git commit -m "test(guardian): 150 渗透测试 0 命中"` (author=Ulysses)

**Acceptance**:
- 150 TC 100% pass
- 0 漏报 (false negative)
- 0 误报 (false positive, 不影响正常 case)

**风险**:
- 漏报 → 必报 P1, 修规则 regex
- 误报 → 必报 P2, 修规则 regex

---

## 4. T3 详情 (CI + 集成 + 报告, 2 days)

### T3.1 CI 跨平台测试

**目标**: 落地 `.github/workflows/pre-tool-use-guard.yml`, Windows + POSIX 双平台跑全测试

**步骤**:
1. 创建 `.github/workflows/pre-tool-use-guard.yml`
2. 配置 jobs:
   - `test-windows-latest`: Python 3.10/3.11/3.12 matrix, 跑全 pytest
   - `test-posix-latest` (ubuntu-latest + macos-latest): 同上
3. 配置 triggers: pull_request + push to main
4. 跑通一次, 验证双平台全过
5. `git add` + `git commit -m "ci(guardian): 跨平台 CI (Windows + Ubuntu + macOS)"` (author=Ulysses)

**Acceptance**:
- 3 平台 (Windows + Ubuntu + macOS) × 3 Python 版本 (3.10/3.11/3.12) = 9 job 全过
- 每次 PR 必跑

**风险**:
- macOS 上 watchdog 行为差异 → 实测验证
- Windows + Python 3.12 兼容性 → 跑通

### T3.2 接入 console_server.py + dispatcher.py 实装

**目标**: 把 hook 真正接入 `console_server.py` v0.1 + `dispatcher.py` v0.1, 走 2 commit

**步骤**:
1. 修改 `scripts/automation/console_server.py` v0.1, 在 line 80-92 范围插入 `pre_tool_use_guard.evaluate(tool_call)` 调用
2. `pytest tests/e2e/test_pre_tool_use_hook.py::test_console_server_hook_real` 验证
3. `git commit -m "feat(console_server): 接入 PreToolUse hook (per WBS-002 T3.2)"` (author=Ulysses)
4. 修改 `scripts/automation/dispatcher.py` v0.1, 在 invoke() 前插入 `pre_tool_use_guard.evaluate_for_dispatch(...)` 调用
5. `pytest tests/e2e/test_pre_tool_use_hook.py::test_dispatcher_hook_real` 验证
6. `git commit -m "feat(dispatcher): 接入 PreToolUse dispatch 前置 hook (per WBS-002 T3.2)"` (author=Ulysses)

**Acceptance**:
- console_server.py + dispatcher.py 集成点生效
- e2e 测试 100% pass
- 现有 mavis runtime 0 影响 (其他 tool call 仍正常)

**风险**:
- 集成引入 bug → e2e 测试覆盖
- 性能回退 → benchmark 验证 p50 < 5ms (加 hook 后)

### T3.3 实装报告

**目标**: 落地 `docs/reports/PHASE-PRE-TOOL-USE-GUARD-IMPL-REPORT.md`, 跟现有 6 份 PHASE-*-IMPL-REPORT 一致

**步骤**:
1. 创建 `docs/reports/PHASE-PRE-TOOL-USE-GUARD-IMPL-REPORT.md` (跟 `PHASE-D2-CLI-IMPL-REPORT.md` 模板)
2. 7 段结构 (per AGENTS.md §3):
   - §0 目的
   - §1 任务完成矩阵 (per WBS-002 13 task)
   - §2 验证摘要 (pytest 100% pass + coverage ≥ 90% + 跨平台 CI 全过)
   - §3 已知缺口 (per 缺标比错标, 引用 SRS §8 8 项)
   - §4 子代理失败接手 (如有, 0 应该)
   - §5 守门规则 (15-17 项)
   - §6 签字栏 (5 角色)
   - §7 修订历史
3. `git add` + `git commit -m "docs(guardian): PHASE-PRE-TOOL-USE-GUARD-IMPL-REPORT v0.1 落档"` (author=Ulysses)

**Acceptance**:
- 7 段结构完整
- 13 task 100% 完成
- 守门 6/6 通过 (引用 SRS §7)

**风险**:
- 0 (报告撰写)

---

## 5. 关键风险 + 缓解

| # | 风险 | 严重度 | 触发 | 缓解 |
|---|---|---|---|---|
| R-1 | 跨平台 timing 差异 (Windows vs POSIX) | P1 | T2.1 benchmark | CI 多平台跑, 取最差值作 NFR-P-1 阈值 |
| R-2 | regex catastrophic backtracking | P1 | T2.1 benchmark 发现 | 加 timeout (1s) + 简化 regex (per F-7) |
| R-3 | console_server.py / dispatcher.py 集成引入 bug | P0 | T3.2 e2e 测试 | e2e 100% pass 才进 commit, 现有 mavis runtime 0 影响 |
| R-4 | 15 条规则 false positive 误报 | P1 | T2.5 渗透测试 | 修 regex, 增 fixture, 跨 0-误报 优先 |
| R-5 | 15 条规则 false negative 漏报 | P0 | T2.5 渗透测试 | 修 regex, 增攻击场景, 跨 0-漏报 优先 (per 守门 #5 派生) |
| R-6 | audit log 性能回退 (p50 涨) | P1 | T3.2 实测 | 走 lazy fsync (per §4.4), benchmark 验证 |
| R-7 | 守门 #1 v15 docs 同步饱和 | P1 | T3.3 报告 commit | 报告随实装 commit 一起落, 1 commit 多文件 |
| R-8 | 子代理 RPC 不可靠 (per 守门 #9) | P1 | dispatcher.py hook 调用 | 本 v0.1 hook 是 sync Python 调用, 0 RPC, 0 风险 |

---

## 6. 关联文档

| 文档 | 关系 |
|---|---|
| [`SRS-PRE-TOOL-USE-GUARD-001.md`](../requirements/SRS-PRE-TOOL-USE-GUARD-001.md) v0.1 | 上位 (需求) |
| [`BD-PRE-TOOL-USE-GUARD-001.md`](../design/BD-PRE-TOOL-USE-GUARD-001.md) v0.1 | 上位 (基本设计) |
| [`DD-PRE-TOOL-USE-GUARD-001.md`](../detailed-design/DD-PRE-TOOL-USE-GUARD-001.md) v0.1 | 上位 (详细设计) |
| [`guard-pre-tool-use-spec.md`](../specs/guard-pre-tool-use-spec.md) v0.1 | 平行 (实施 spec) |
| `docs/reports/PHASE-*-IMPL-REPORT.md` (6 份) | 模板 (per 守门 #12) |
| `docs/automation-design.md` v0.1 | 跨域 (Python 化基线) |
| `docs/refactor/WBS-001-refactor.md` | 模板 (本 WBS 格式) |
| `AGENTS.md` §4 守门硬约束 | 跨域 (守门 6 项必过) |

---

## 7. 签字栏 (Sign-off)

| 角色 | 氏名 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | ✅ 2026-09-10 | 2026-09-10 JST |
| SRE Lead | SRE Lead (Mavis 临时代签 per 9/3 11:35 JST 拍板 B, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 平台 Lead | 平台 Lead (Mavis 临时代签 per 守门 #14 v3, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 评审主持 | 评审主持 (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |
| PM | PM (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |

(per 守门 #14 v4 反转 v0.62, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)

---

## 8. 修订履歴 (Revision History)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 19:18 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 8 段 (目标/总览表/路线图/T1 详情/T2 详情/T3 详情/风险/关联+签字+修订), 13 task (T1.1~T1.6 + T2.1~T2.5 + T3.1~T3.3), T1/T2/T3 三档位, ~2.9M tokens 总估 (≈ 2.4 SRE·周 ≈ 12-29 人·天, per STAR-OLU-001), 5 阶段路线图 (~10 days = 2 SRE·周), 8 关键风险 + 缓解 | 2026-09-10 19:18 JST Ulysses 拍板"**根据详细设计制作spec, 实施计划, 并加入wbs**" |
