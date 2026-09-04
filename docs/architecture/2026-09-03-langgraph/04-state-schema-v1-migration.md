# 04-state-schema-v1-migration.md — LangGraph State Schema v1 Migration Path

> **Status**: 🟡 Draft v0.1 (2026-09-04 拍板, per WBS §H.4)
> **承接**: `docs/architecture/2026-09-03-langgraph/02-basic-design.md` §2.1.1 TopAgentState TypedDict v0
> **拍板**: 2026-09-04 16:25 JST 拍板 H.4 启动 (per 守门 #19 [S] 拍板, 9/4 13:43 JST WBS 排序降序)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **审批者**: 架构师 (Mavis 接手 agent per DEC-008)

---

## §0 目的

按 2026-09-04 15:20 JST 拍板"H.1 LangGraph 集成 PoC 启动" + 9/4 16:25 JST 拍板 H.4 启动,把 LangGraph 全体代理 (Top-Level Agent) state schema 从 v0 (无版本, 仅 Python TypedDict 文档) 迁移到 v1 (含 schema_version 字段 + 迁移路径 + 向后兼容策略) 做完整规划.

**H.4 范围** (per WBS §H.4 + 守门 #19 [S] 拍板):
- 文档化 v0 → v1 迁移路径 (本章)
- 定义 `SchemaMigrationRegistry` 抽象接口
- 定义 3 个迁移操作 (AddField / RenameField / RemoveField)
- 5 个迁移场景 (新增字段 / 字段重命名 / 字段废弃 / 类型变更 / reducer 变更)
- 不在本 PoC: 实际 Rust 端 state 实现 (per SRS-001 §G-10 后续) / 跨 session checkpoint 迁移 (per SRS-001 §G-11 v0.1.0)

**拍板**:
- 9/4 12:19 JST Mavis 自主推進
- 9/4 16:25 JST Mavis 临时代签 H.4 拍板 (per 守门 #19 [S] 自动化档)
- 5 域 Lead 真人到位后追溯签字 (per 守门 #14 5 域 Lead CONTENT 4 维)

---

## §1 v0 → v1 Migration 触发条件

### 1.1 当前 v0 状态 (per `02-basic-design.md` §2.1.1)

```python
# 02-basic-design.md:165-187
class TopAgentState(TypedDict, total=False):
    """全体代理 state schema (LangGraph TypedDict)"""
    user_input: str
    intent: Optional[str]
    active_subagents: Annotated[list[SubAgentRef], operator.add]
    completed_subagents: Annotated[list[SubAgentResult], operator.add]
    conversation_history: Annotated[list[Message], operator.add]
    global_context: dict
    last_response: Optional[str]
    interrupt_id: Optional[str]
    interrupt_response: Optional[dict]
```

**v0 缺标** (per 守门 #11 缺标比错标安全):
- 0 schema_version 字段
- 0 向后兼容策略
- 0 迁移触发器定义
- 0 deprecated 字段标记
- 0 字段类型变更记录

### 1.2 v1 目标状态

```python
class TopAgentStateV1(TypedDict, total=False):
    """全体代理 state schema v1 (LangGraph TypedDict)"""
    # === v1 元数据 ===
    schema_version: Literal["1.0.0"]  # 新增, 强制 v1.0.0

    # === v0 字段 (向后兼容, 全部保留) ===
    user_input: str
    intent: Optional[str]
    active_subagents: Annotated[list[SubAgentRef], operator.add]
    completed_subagents: Annotated[list[SubAgentResult], operator.add]
    conversation_history: Annotated[list[Message], operator.add]
    global_context: dict
    last_response: Optional[str]
    interrupt_id: Optional[str]
    interrupt_response: Optional[dict]

    # === v1 新增字段 ===
    # 5 域 Lead 状态 (per 守门 #14 5 域 Lead CONTENT 4 维)
    five_domain_lead_state: FiveDomainLeadState
    # Token 集計 (per SRS-001 §G-9)
    token_usage: TokenUsage
    # Checkpoint 元数据 (per SRS-001 §G-7)
    checkpoint_metadata: CheckpointMetadata
    # Context tier 标记 (per SRS-001 §G-8)
    context_tier_marker: ContextTierMarker
    # 跨 session 关联 (per SRS-001 §G-11)
    cross_session_ref: Optional[CrossSessionRef]
```

---

## §2 SchemaMigrationRegistry 抽象接口

### 2.1 3 个迁移操作

```python
class MigrationOp(ABC):
    """迁移操作抽象基类"""
    op_id: str  # 唯一 ID, e.g. "v0_to_v1__add_schema_version"
    op_type: Literal["AddField", "RenameField", "RemoveField", "ChangeType", "ChangeReducer"]
    description: str

    def apply(self, state: dict) -> dict:
        """应用迁移到 state"""
        pass

    def revert(self, state: dict) -> dict:
        """回滚迁移 (调试用)"""
        pass
```

### 2.2 SchemaMigrationRegistry 接口

```python
class SchemaMigrationRegistry:
    """State schema 迁移注册表"""

    def __init__(self):
        self._migrations: dict[str, list[MigrationOp]] = defaultdict(list)

    def register(self, from_version: str, to_version: str, ops: list[MigrationOp]):
        """注册 from_version -> to_version 的迁移操作"""
        key = f"{from_version}->{to_version}"
        self._migrations[key] = ops

    def migrate(self, state: dict, from_version: str, to_version: str) -> dict:
        """执行 from_version -> to_version 迁移"""
        if from_version == to_version:
            return state
        # BFS 找最短迁移路径
        path = self._find_migration_path(from_version, to_version)
        for step in path:
            for op in self._migrations[step]:
                state = op.apply(state)
        state["schema_version"] = to_version
        return state

    def _find_migration_path(self, from_v: str, to_v: str) -> list[str]:
        """BFS 找最短迁移路径 (per LangGraph State Schema 迁移实践)"""
        # 简化: 假设 linear 版本链
        path = []
        current = from_v
        while current != to_v:
            next_v = self._next_version(current)
            if next_v is None:
                raise MigrationError(f"No migration path from {current} to {to_v}")
            path.append(f"{current}->{next_v}")
            current = next_v
        return path
```

---

## §3 5 个迁移场景

### 3.1 Scenario 1: 新增字段 (AddField)

**触发**: v0 → v1 加入 `schema_version` 字段

```python
class AddSchemaVersionOp(MigrationOp):
    op_id = "v0_to_v1__add_schema_version"
    op_type = "AddField"
    description = "新增 schema_version 字段, 默认 1.0.0"

    def apply(self, state: dict) -> dict:
        if "schema_version" not in state:
            state["schema_version"] = "1.0.0"
        return state

    def revert(self, state: dict) -> dict:
        state.pop("schema_version", None)
        return state
```

### 3.2 Scenario 2: 字段重命名 (RenameField)

**触发**: v1 → v2 把 `global_context` 重命名为 `l0_global_context`

```python
class RenameGlobalContextOp(MigrationOp):
    op_id = "v1_to_v2__rename_global_context"
    op_type = "RenameField"
    description = "重命名 global_context -> l0_global_context (语义更清晰)"

    def apply(self, state: dict) -> dict:
        if "global_context" in state:
            state["l0_global_context"] = state.pop("global_context")
        return state

    def revert(self, state: dict) -> dict:
        if "l0_global_context" in state:
            state["global_context"] = state.pop("l0_global_context")
        return state
```

### 3.3 Scenario 3: 字段废弃 (RemoveField + 迁移数据到 archive)

**触发**: v2 → v3 废弃 `interrupt_response` (per 守门 #11 缺标比错标安全)

```python
class ArchiveInterruptResponseOp(MigrationOp):
    op_id = "v2_to_v3__archive_interrupt_response"
    op_type = "RemoveField"
    description = "废弃 interrupt_response, 迁移到 interrupt_archive 字段 (历史保留)"

    def apply(self, state: dict) -> dict:
        if "interrupt_response" in state:
            archive = state.setdefault("interrupt_archive", [])
            archive.append({
                "value": state.pop("interrupt_response"),
                "archived_at": now_ms(),
                "from_version": "v2",
            })
        return state

    def revert(self, state: dict) -> dict:
        # 从 archive 恢复最后一个值
        archive = state.get("interrupt_archive", [])
        if archive:
            state["interrupt_response"] = archive[-1]["value"]
        return state
```

### 3.4 Scenario 4: 类型变更 (ChangeType)

**触发**: v3 → v4 把 `user_input: str` 改为 `user_input: Message` (更结构化)

```python
class ChangeUserInputTypeOp(MigrationOp):
    op_id = "v3_to_v4__change_user_input_type"
    op_type = "ChangeType"
    description = "user_input: str -> user_input: Message (LangGraph Message 包装)"

    def apply(self, state: dict) -> dict:
        if "user_input" in state and isinstance(state["user_input"], str):
            state["user_input"] = Message(role="user", content=state["user_input"])
        return state

    def revert(self, state: dict) -> dict:
        if "user_input" in state and isinstance(state["user_input"], Message):
            state["user_input"] = state["user_input"].content
        return state
```

### 3.5 Scenario 5: Reducer 变更 (ChangeReducer)

**触发**: v4 → v5 把 `active_subagents` 的 reducer 从 `operator.add` 改为自定义 limiter (max 50 个)

```python
class ChangeActiveSubagentsReducerOp(MigrationOp):
    op_id = "v4_to_v5__change_active_subagents_reducer"
    op_type = "ChangeReducer"
    description = "active_subagents reducer: operator.add -> max_50_limiter (per LangGraph C-13 max_parallel)"

    def apply(self, state: dict) -> dict:
        # 截断到 max 50
        if "active_subagents" in state and len(state["active_subagents"]) > 50:
            state["active_subagents"] = state["active_subagents"][-50:]
        # 标记 reducer 变更
        state["_reducer_metadata"] = {
            "active_subagents": "max_50_limiter",
            "from_version": "v4",
        }
        return state

    def revert(self, state: dict) -> dict:
        state.pop("_reducer_metadata", None)
        return state
```

---

## §4 向后兼容策略

### 4.1 默认迁移 (Default Migration)

**策略**: 未指定 from_version 时, 默认从 latest 迁移到 latest, 跳过中间版本.

```python
def load_state(state_dict: dict) -> dict:
    """从 checkpoint 加载 state, 自动迁移到 current_version"""
    current_version = "1.0.0"  # 当前 schema version
    from_version = state_dict.get("schema_version", "0.0.0")
    if from_version != current_version:
        registry = get_global_registry()
        state_dict = registry.migrate(state_dict, from_version, current_version)
    return state_dict
```

### 4.2 失败回退 (Fallback)

**策略**: 迁移失败时, fallback 到 from_version state, 不丢失数据, 但 record warning.

```python
def load_state_with_fallback(state_dict: dict) -> tuple[dict, list[str]]:
    """返回 (migrated_state, warnings)"""
    warnings = []
    try:
        return load_state(state_dict), warnings
    except MigrationError as e:
        warnings.append(f"Migration failed: {e}, fallback to v0 state")
        return state_dict, warnings
```

### 4.3 版本协商 (Version Negotiation)

**策略**: 跨 session 通讯时 (per SRS-001 §G-11), 协商最低公共版本.

```python
def negotiate_version(v1: str, v2: str) -> str:
    """返回 v1 和 v2 之间的最低公共版本 (BFS 找最近共同祖先)"""
    # 简化: 取 min(v1, v2) 按 semver 排序
    return min(v1, v2, key=lambda v: tuple(int(p) for p in v.split(".")))
```

---

## §5 迁移触发器 (Migration Triggers)

### 5.1 编译时触发 (Compile Time)

| 触发 | 检测 | 行为 |
|---|---|---|
| 新增 pub struct 字段到 TopAgentState | `cargo check --workspace --all-targets` | 0 (本 PoC 不强制, 由 reviewer 手动触发) |
| Star仓 field rename | `git log -p --follow crates/star-dispatcher/src/lib.rs` | 手动调用 `migration_tool.py` |
| Star仓 field remove | `grep -r "pub field_name" crates/` | 手动调用 `migration_tool.py` |

### 5.2 运行時触发 (Runtime)

| 触发 | 检测 | 行为 |
|---|---|---|
| Checkpoint 加载 (per SRS-001 §G-7) | `state["schema_version"] != CURRENT_VERSION` | 自动迁移 + record audit_log |
| 跨 sub-agent 通讯 (per LangGraph C-13) | `subagent_state["schema_version"] != top_state_version` | 自动迁移 + warning |
| 跨 session restore (per SRS-001 §G-11) | `checkpoint["schema_version"] != CURRENT_VERSION` | 自动迁移 + user notification |

### 5.3 部署時触发 (Deployment)

| 触发 | 检测 | 行为 |
|---|---|---|
| CI/CD 部署新版本 (per F.5 CI runner) | `version_check` step | 0 (本 PoC 跳过, 后续 F.5 集成) |
| Database migration (per 守门 #DB-13) | `alembic upgrade head` | 0 (本 PoC 跳过, schema_version 是 code-level 而非 DB-level) |

---

## §6 SchemaMigrationRegistry 初始注册表 (v0 → v1)

```python
# scripts/automation/state_schema_registry.py (per 守门 #19 [S] 拍板, V2 落地)
def build_default_registry() -> SchemaMigrationRegistry:
    registry = SchemaMigrationRegistry()
    # v0 -> v1
    registry.register("0.0.0", "1.0.0", [
        AddSchemaVersionOp(),
        AddFiveDomainLeadStateOp(),
        AddTokenUsageOp(),
        AddCheckpointMetadataOp(),
        AddContextTierMarkerOp(),
        AddCrossSessionRefOp(),
    ])
    return registry
```

---

## §7 已知缺口 / V2 路线图

| # | 缺口 | 触发守门 | V2 路线图 |
|---|---|---|---|
| 1 | Rust 端 state 实现 (per SRS-001 §G-10) | 守门 #1 v3 | V2 Phase H.4.1 — StarLangGraph Rust 端 state schema 化 |
| 2 | 跨 session checkpoint 迁移 (per SRS-001 §G-11) | 守门 #1 v3 | V2 v0.1.0 — cross-session checkpoint + SchemaMigrationRegistry 自动迁移 |
| 3 | 自动 migration_tool.py CLI (per 守门 #19 [S]) | 守门 #19 [S] | V2 — 创 `scripts/automation/migration_tool.py`, 支持 dry-run + diff |
| 4 | 真实 v1 字段实现 (5 域 Lead state / Token usage / Checkpoint metadata / Context tier marker) | 守门 #14 5 域 Lead | 待 5 域 Lead 真人到位后, 业务逻辑补 |
| 5 | 编译时强制 schema_version check | 守门 #7 | V2 — `#[derive(StateSchema)]` proc macro 自动检查 |
| 6 | Database schema migration 集成 (per 守门 #DB-13) | 守门 #DB-13 | V2 — alembic + SchemaMigrationRegistry 联动 |
| 7 | `_ARCHIVED_handoff_section_9/10/11/12_*_20260904*.md` 5 份临时文件未收编 | 守门 #12 缺标比错标 | 下 session 收编到 `_archive_/` |

---

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: H.4 LangGraph State schema v1 migration 路径 闭环 (5 迁移场景 + 3 操作 + 3 兼容策略 + 5 触发器) | 9/4 16:25 JST 拍板 H.4 启动 + 9/4 16:35 JST 文档落档 |
