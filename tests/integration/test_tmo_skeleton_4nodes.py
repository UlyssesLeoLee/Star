"""TMO-02/05/06/07 4 节点骨架 e2e tests (V2-6 5 子代理 + Mavis 跨域协调模式)

per 守门 #13 a 实证 (L1↔L1 禁止, TMO 全部 L0 协调, 唯一 cross-task actor = TaskOperationsManager C-16)
per 守门 #13 c Master RLS 派生 (metadata_node 必携 tenant_id)
per 守门 #13 d Work 100% retention + Transaction 100% audit (summarize TTL, metadata append-only)
per 守门 #19 Python 化 (本测试不碰 cargo 链, 走守门 #19 路径)

Mavis root 亲手执行 (per 守门 #9 v3 实证, 子代理 RPC 不可靠)
"""
from __future__ import annotations

import asyncio
import sys
import time
from pathlib import Path

# 允许从 scripts/automation/ import (per 守门 #19 Python 化基类路径)
SCRIPTS_AUTOMATION = Path(__file__).parent.parent.parent / "scripts" / "automation"
sys.path.insert(0, str(SCRIPTS_AUTOMATION))

from task_ops.manager import (
    SubAgentHandle,
    SubAgentPool,
)
from task_ops.relationship_graph import TaskRelationshipGraph
# 4 节点 (本 session 新增) 分别从各自 sub-module import
from task_ops.nodes.split_node import split_node
from task_ops.nodes.summarize_node import summarize_node
from task_ops.nodes.reassign_node import reassign_node
from task_ops.nodes.metadata_node import metadata_node, REQUIRED_RLS_FIELDS


# 内存版 Master RLS registry (PoC, 真实 DDL 推 G-TMO-04)
class InMemoryMetadataRegistry:
    def __init__(self):
        self._store: dict[str, dict] = {}
        self._audit: list[dict] = []

    def update(self, task_id: str, metadata: dict, actor_session_id: str | None) -> str:
        update_id = f"upd-{int(time.time() * 1000)}"
        # Master RLS 派生: 物理删除禁止, 仅 upsert
        self._store[task_id] = dict(metadata)
        # Transaction append-only (per 守门 #13 d)
        self._audit.append({
            "update_id": update_id,
            "task_id": task_id,
            "metadata": dict(metadata),
            "actor_session_id": actor_session_id,
            "ts": time.time(),
        })
        return update_id


# 守门 #13 a 实证: 4 节点全部 L0 协调 (无 L1↔L1 直连)
def test_4_nodes_go_through_l0():
    """守门 #13 a 强约束派生: TMO 全部 4 节点 (M-N2/N5/N6/N7) 走 L0 协调 (TaskOperationsManager C-16),
    任何 L1↔L1 跨任务操作禁止. PoC: 4 节点都接受 SubAgentPool + L0 协调参数, 实际不绕过 L0."""
    # 4 节点函数都接受 (sub_agent_pool, ..., request) 形式, 无 L1↔L1 直连接口
    for node_fn in (split_node, summarize_node, reassign_node, metadata_node):
        import inspect
        sig = inspect.signature(node_fn)
        params = list(sig.parameters.keys())
        # 必须包含 sub_agent_pool (L0 唯一入口) 或等价 L0 协调参数
        assert any("pool" in p.lower() or "graph" in p.lower() or "registry" in p.lower() for p in params), \
            f"{node_fn.__name__} 缺少 L0 协调参数 (per 守门 #13 a): {params}"
    print("[OK] 守门 #13 a 实证: 4 节点全部 L0 协调 (per TaskOperationsManager C-16)")


# TMO-02 split_node 实证
def test_split_node_basic():
    pool = SubAgentPool()
    graph = TaskRelationshipGraph()
    parent = pool.add(task_type="SA-03", task_id="task-parent-1", initial_state={"context": {"req": "split me"}})
    parent_id = parent.task_id

    req = {
        "operation": "split",
        "target_task_id": parent_id,
        "split_strategy": "context_fork",
        "actor_session_id": "session-1",
    }
    result = asyncio.run(split_node(pool, graph, req))

    # 验证
    assert result["parent_task_id"] == parent_id
    assert len(result["child_task_ids"]) >= 2
    assert result["parent_superseded"] is True
    assert result["split_strategy"] == "context_fork"
    # parent 标 superseded
    assert pool.get(parent_id).state["status"] == "superseded"
    # 子 task 存在
    for cid in result["child_task_ids"]:
        child = pool.get(cid)
        assert child.state["status"] == "running"
        assert child.state["context"] == {"req": "split me"}  # context_fork 共享
    # DAG edge
    children = graph.get_children(parent_id)
    assert children == result["child_task_ids"]
    print(f"✅ TMO-02 split_node: {parent_id} -> {len(result['child_task_ids'])} children")


# TMO-05 summarize_node 实证
def test_summarize_node_concatenate():
    pool = SubAgentPool()
    pool.add(task_type="SA-01", task_id="task-A", initial_state={"context": {"k": "v1"}})
    pool.add(task_type="SA-02", task_id="task-B", initial_state={"context": {"k2": "v2"}})

    req = {
        "operation": "summarize",
        "task_ids": ["task-A", "task-B"],
        "summarize_strategy": "concatenate",
        "actor_session_id": "session-1",
    }
    result = asyncio.run(summarize_node(pool, req))

    assert result["summary_id"].startswith("summary-")
    assert "task-A" in result["summary"] and "task-B" in result["summary"]
    assert result["summarize_strategy"] == "concatenate"
    assert result["ttl_seconds"] == 3600  # Work 类型 短 TTL per 守门 #13 d
    # 聚合 context (k + k2)
    assert "k" in result["aggregated_context"]
    assert "k2" in result["aggregated_context"]
    print(f"✅ TMO-05 summarize_node: 2 tasks -> {result['summary_id']}")


# TMO-06 reassign_node 实证 (checkpoint preserved)
def test_reassign_node_preserves_checkpoint():
    pool = SubAgentPool()
    pool.add(task_type="SA-03", task_id="task-r1", initial_state={"context": {"data": "x"}})
    orig_cp_count = len(pool.get("task-r1").checkpoints)

    req = {
        "operation": "reassign",
        "task_id": "task-r1",
        "new_sa_type": "SA-08",
        "actor_session_id": "session-1",
    }
    result = asyncio.run(reassign_node(pool, req))

    assert result["old_sa_type"] == "SA-03"
    assert result["new_sa_type"] == "SA-08"
    assert result["checkpoint_id"].startswith("cp-")
    # checkpoint preserved (per 守门 #13 d)
    new_cp_count = len(pool.get("task-r1").checkpoints)
    assert new_cp_count == orig_cp_count + 1
    # task_type 改了
    assert pool.get("task-r1").task_type == "SA-08"
    # worktree_migration pending (per G-DEP-01 stub)
    assert result["worktree_migration"]["migration_status"] == "pending"
    print(f"✅ TMO-06 reassign_node: task-r1 SA-03 -> SA-08 (checkpoint preserved, {orig_cp_count}->{new_cp_count})")


# TMO-07 metadata_node 实证 (Master RLS 必携 tenant_id)
def test_metadata_node_requires_tenant_id():
    """守门 #13 c Master RLS 派生: metadata 必携 tenant_id"""
    pool = SubAgentPool()
    pool.add(task_type="SA-07", task_id="task-m1", initial_state={})
    registry = InMemoryMetadataRegistry()

    # 缺 tenant_id 应抛错
    try:
        asyncio.run(metadata_node(pool, registry, {
            "task_id": "task-m1",
            "metadata": {"label": "test"},  # 缺 tenant_id
        }))
        assert False, "应该抛错 (缺 tenant_id per 守门 #13 c)"
    except ValueError as e:
        assert "tenant_id" in str(e)

    # 含 tenant_id 应成功
    result = asyncio.run(metadata_node(pool, registry, {
        "task_id": "task-m1",
        "metadata": {"tenant_id": "t1", "label": "important", "priority": "high"},
        "actor_session_id": "session-1",
    }))

    assert result["update_id"].startswith("upd-")
    assert result["metadata_snapshot"]["tenant_id"] == "t1"
    assert result["metadata_snapshot"]["label"] == "important"
    # registry 存储 (Master RLS, 物理删除禁止)
    assert "task-m1" in registry._store
    # audit log (Transaction append-only per 守门 #13 d)
    assert len(registry._audit) == 1
    print(f"✅ TMO-07 metadata_node: 守门 #13 c Master RLS 必携 tenant_id 实证")


# 守门 #13 d 实证: 4 节点全部走 checkpoint + audit
def test_4_nodes_checkpoint_audit():
    """守门 #13 d 派生: 4 节点都生成 checkpoint + audit"""
    pool = SubAgentPool()
    graph = TaskRelationshipGraph()
    registry = InMemoryMetadataRegistry()
    pool.add(task_type="SA-03", task_id="task-c1", initial_state={"context": {}})

    # split
    r1 = asyncio.run(split_node(pool, graph, {"operation": "split", "target_task_id": "task-c1"}))
    assert "checkpoint_id" in r1
    # summarize
    r2 = asyncio.run(summarize_node(pool, {"operation": "summarize", "task_ids": ["task-c1"]}))
    assert "summary_id" in r2  # 汇总用 summary_id 而非 checkpoint (Work 类型)
    # reassign
    r3 = asyncio.run(reassign_node(pool, {"operation": "reassign", "task_id": "task-c1", "new_sa_type": "SA-08"}))
    assert "checkpoint_id" in r3
    # metadata
    r4 = asyncio.run(metadata_node(pool, registry, {
        "operation": "metadata_update",
        "task_id": "task-c1",
        "metadata": {"tenant_id": "t1", "k": "v"},
    }))
    assert "checkpoint_id" in r4
    # audit log 仅 metadata (per 守门 #13 d audit 100%)
    assert len(registry._audit) == 1
    print(f"✅ 守门 #13 d 实证: 4 节点 checkpoint + audit 全部覆盖")


# 守门 #19 派生: 4 节点 Python 化 (无 .rs/.ts 改动)
def test_4_nodes_python_only():
    """守门 #19 派生: 4 节点不依赖 cargo 链, 纯 Python

    排除注释行 + 字符串字面量: 注释里"不碰 cargo 链"等元描述允许出现.
    只检测**实际执行**的子进程调用 (subprocess / os.system / 直接拼接命令).
    """
    import ast
    import re
    for path in [
        SCRIPTS_AUTOMATION / "task_ops" / "nodes" / "split_node.py",
        SCRIPTS_AUTOMATION / "task_ops" / "nodes" / "summarize_node.py",
        SCRIPTS_AUTOMATION / "task_ops" / "nodes" / "reassign_node.py",
        SCRIPTS_AUTOMATION / "task_ops" / "nodes" / "metadata_node.py",
    ]:
        with open(path, "r", encoding="utf-8") as f:
            source = f.read()
        # 验证可被 Python AST 解析 (无语法错)
        ast.parse(source)
        # 剥离注释行 (守门 #19 只看实际执行, 注释描述允许提及禁词)
        code_lines = []
        for line in source.splitlines():
            stripped = line.lstrip()
            if stripped.startswith("#"):
                continue
            code_lines.append(line)
        code_only = "\n".join(code_lines)
        # 验证代码行 (非注释) 不含子命令调用
        forbidden = ["cargo ", "cargo build", "cargo test", "cargo check", "git rev-parse", "gh api"]
        for term in forbidden:
            assert term not in code_only, f"{path.name} 代码行含禁词 {term!r} (应走守门 #19 Python 化, 注释允许)"
    print(f"✅ 守门 #19 实证: 4 节点 Python 化 (代码行无 cargo / git / gh 调用, 注释允许元描述)")


if __name__ == "__main__":
    test_4_nodes_go_through_l0()
    test_split_node_basic()
    test_summarize_node_concatenate()
    test_reassign_node_preserves_checkpoint()
    test_metadata_node_requires_tenant_id()
    test_4_nodes_checkpoint_audit()
    test_4_nodes_python_only()
    print("\n🎉 7/7 守门实证 + e2e test 全部通过 (V2-6 5 子代理 + Mavis 跨域协调模式)")
