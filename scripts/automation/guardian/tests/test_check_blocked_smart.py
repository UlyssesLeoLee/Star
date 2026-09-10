"""v33 v0.2 智能 check_blocked 单元测试 (per 守门 v33 已知缺口 #3 + WBS-003 续做).

5+ TC, 覆盖:
- 解除规则 (Ulysses/architect/Mavis actor + parent_comment_id 指向 BLOCK + blocks=False)
- 非权威 actor (sub-agent) 不能解除
- 多条 BLOCK 留言部分解除
- 无解除留言 → 原行为 (返所有 BLOCK)
- 解除后 invoke 不抛
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
sys.path.insert(0, str(Path(__file__).resolve().parents[2].parent))

from dispatcher import SubagentDispatcher, BlockedByCommentError


@pytest.fixture
def dispatcher(tmp_path):
    briefs = tmp_path / "briefs"
    briefs.mkdir()
    audit = tmp_path / "audit.log"
    d = SubagentDispatcher(phase="v33_v02_test", briefs_dir=briefs, audit_log=audit)
    yield d


# ============ 解除规则 ============

def test_block_unresolved_returns_blocking(dispatcher):
    """无解除留言 → check_blocked 返原 BLOCK 留言 (v0.1 行为兼容)."""
    dispatcher.comment("T1", "BLOCK 留言", blocks=True, author="Ulysses", actor_role="Ulysses")
    blocking = dispatcher.check_blocked("T1")
    assert len(blocking) == 1
    assert blocking[0].body == "BLOCK 留言"


def test_block_resolved_by_ulysses_unblocks(dispatcher):
    """Ulysses actor 写 blocks=False + parent_comment_id → 解除."""
    c1 = dispatcher.comment("T2", "BLOCK 留言", blocks=True, author="Ulysses", actor_role="Ulysses")
    dispatcher.comment("T2", "已解决, 可以继续", blocks=False, author="Ulysses", actor_role="Ulysses", parent_comment_id=c1.id)
    blocking = dispatcher.check_blocked("T2")
    assert blocking == [], f"应已解除, 实际仍 {len(blocking)} 条 BLOCK"


def test_block_resolved_by_architect_unblocks(dispatcher):
    """architect actor 也能解除."""
    c1 = dispatcher.comment("T3", "BLOCK 留言", blocks=True, author="Mavis", actor_role="Mavis")
    dispatcher.comment("T3", "architect approve 解除", blocks=False, author="Ulysses", actor_role="architect", parent_comment_id=c1.id)
    blocking = dispatcher.check_blocked("T3")
    assert blocking == []


def test_block_resolved_by_mavis_unblocks(dispatcher):
    """Mavis actor 也能解除 (per 守门 #14 v3)."""
    c1 = dispatcher.comment("T4", "BLOCK 留言", blocks=True, author="Mavis", actor_role="Mavis")
    dispatcher.comment("T4", "Mavis 审核解除", blocks=False, author="Mavis", actor_role="Mavis", parent_comment_id=c1.id)
    blocking = dispatcher.check_blocked("T4")
    assert blocking == []


# ============ 非权威 actor 不能解除 ============

def test_block_not_resolved_by_subagent(dispatcher):
    """sub-agent actor 不能解除 BLOCK 留言 (per 守门 v33 §1.3 权威 actor 列表)."""
    c1 = dispatcher.comment("T5", "BLOCK 留言", blocks=True, author="Ulysses", actor_role="Ulysses")
    # sub-agent 写解除留言 → 不算权威
    dispatcher.comment("T5", "我已修好", blocks=False, author="worker-1", actor_role="sub-agent", parent_comment_id=c1.id)
    blocking = dispatcher.check_blocked("T5")
    assert len(blocking) == 1, "sub-agent 写的解除留言不权威, BLOCK 仍存在"


# ============ 多重 BLOCK 部分解除 ============

def test_multiple_blocks_partial_resolve(dispatcher):
    """多条 BLOCK 留言, 部分解除 → 只返未解除部分."""
    c1 = dispatcher.comment("T6", "BLOCK A", blocks=True, author="Ulysses", actor_role="Ulysses")
    c2 = dispatcher.comment("T6", "BLOCK B", blocks=True, author="Ulysses", actor_role="Ulysses")
    c3 = dispatcher.comment("T6", "BLOCK C", blocks=True, author="Ulysses", actor_role="Ulysses")
    # 只解除 c1 + c3
    dispatcher.comment("T6", "A 解除", blocks=False, author="Ulysses", actor_role="Ulysses", parent_comment_id=c1.id)
    dispatcher.comment("T6", "C 解除", blocks=False, author="Mavis", actor_role="Mavis", parent_comment_id=c3.id)
    blocking = dispatcher.check_blocked("T6")
    assert len(blocking) == 1
    assert blocking[0].id == c2.id  # 只有 c2 未解除


# ============ 解除留言不带 parent_comment_id → 不算解除 ============

def test_resolve_without_parent_id_does_not_unblock(dispatcher):
    """解除留言不带 parent_comment_id → 跟 BLOCK 留言无关联, 不解除."""
    c1 = dispatcher.comment("T7", "BLOCK 留言", blocks=True, author="Ulysses", actor_role="Ulysses")
    dispatcher.comment("T7", "随便说说", blocks=False, author="Ulysses", actor_role="Ulysses")  # 无 parent_comment_id
    blocking = dispatcher.check_blocked("T7")
    assert len(blocking) == 1


# ============ 解除后 invoke 不抛 ============

def test_invoke_after_resolve_proceeds(tmp_path, monkeypatch):
    """v0.1 test 已知缺口 #3 的解决: BLOCK 留言解除后 invoke 不抛."""
    from dispatcher import SubagentDispatcher

    briefs_dir = tmp_path / "briefs"
    briefs_dir.mkdir()
    audit_dir = tmp_path / "audit"
    audit_dir.mkdir()
    d = SubagentDispatcher(phase="v33_v02_inv", briefs_dir=briefs_dir, audit_log=audit_dir / "test.log")
    monkeypatch.delenv("STAR_PRE_TOOL_USE_GUARD", raising=False)

    brief_path = d.brief("P3-D.6-2-2", "## 2. DO\n- 加 Y 字段", agent="worker")
    c1 = d.comment("P3-D.6-2-2", "BLOCK: 等 P3-D.6-1 完", blocks=True, author="Ulysses", actor_role="Ulysses")
    # v0.1 时抛 BlockedByCommentError, v0.2 应该不抛
    d.comment("P3-D.6-2-2", "P3-D.6-1 已完成, 解除", blocks=False, author="Ulysses", actor_role="Ulysses", parent_comment_id=c1.id)
    # 不抛, 走 deferred 路径
    handle = d.invoke(brief_path, timeout=5)
    assert handle.status in ("deferred", "succeeded", "failed")
