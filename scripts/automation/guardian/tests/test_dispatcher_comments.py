"""任务卡留言机制单元测试 (per 守门 v33 §1 + WBS-003 + 守门 #9 v20 升级).

10+ TC, 覆盖 append-only / @mention / BLOCK / read-receipt / brief 拉留言.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
sys.path.insert(0, str(Path(__file__).resolve().parents[2].parent))

from dispatcher import (
    SubagentDispatcher, Comment, BlockedByCommentError, DispatchBlockedError,
)


@pytest.fixture
def dispatcher(tmp_path):
    """每个测试独立 briefs_dir + audit_log."""
    briefs = tmp_path / "briefs"
    briefs.mkdir()
    audit = tmp_path / "audit.log"
    d = SubagentDispatcher(phase="v33_test", briefs_dir=briefs, audit_log=audit)
    yield d


# ============ §1.1 append-only 物理存储 ============

def test_comment_appends_to_jsonl(dispatcher):
    """AC: 留言 append 到 .jsonl (per 守门 #13 T)."""
    c1 = dispatcher.comment("T1", "第一条留言")
    c2 = dispatcher.comment("T1", "第二条留言")
    assert c1.id == "comment_001"
    assert c2.id == "comment_002"
    # 物理文件: 2 行 JSON
    p = dispatcher._comments_path("T1")
    lines = [l for l in p.read_text(encoding="utf-8").splitlines() if l.strip()]
    assert len(lines) == 2
    # 每行可被 json.loads 解析
    for line in lines:
        d = json.loads(line)
        assert "id" in d and "body" in d


def test_comment_id_increments(dispatcher):
    """comment_id 单调递增."""
    ids = [dispatcher.comment("T2", f"msg{i}").id for i in range(5)]
    assert ids == ["comment_001", "comment_002", "comment_003", "comment_004", "comment_005"]


def test_missing_comments_file_returns_empty(dispatcher):
    """缺文件 → 0 留言 (per 缺标比错标 #11)."""
    assert dispatcher.list_comments("nonexistent") == []
    assert dispatcher.check_blocked("nonexistent") == []


# ============ §1.3 @mention 触发 ============

def test_mention_auto_added_to_task_id(dispatcher):
    """@mention 必含 task_id 自身 (per 守门 v33 §1.1)."""
    c = dispatcher.comment("T3", "请加 X 字段", mentions=["T4", "T5"])
    assert "T3" in c.mentions
    assert "T4" in c.mentions
    assert "T5" in c.mentions


def test_brief_pulls_mentions(dispatcher):
    """brief 落档时拉所有 @<task_id> 留言到 Read COMMENTS 段 (per 守门 v33 §1.4)."""
    # A 留言 @ T10
    dispatcher.comment("T10", "请包含 X 字段", mentions=["T10"])
    # 落 brief
    brief_path = dispatcher.brief("T10", "# Task body\n\n## 2. DO\n- 加 X", agent="worker")
    content = brief_path.read_text(encoding="utf-8")
    assert "## 6. Read COMMENTS" in content
    assert "请包含 X 字段" in content


# ============ §1.3 BLOCK 留言触发 ============

def test_block_comment_recorded(dispatcher):
    """blocks=True 留言被记录."""
    c = dispatcher.comment("T20", "BLOCK: 等 T19 完", blocks=True)
    assert c.blocks is True
    blocking = dispatcher.check_blocked("T20")
    assert len(blocking) == 1
    assert blocking[0].id == "comment_001"


def test_non_block_comments_excluded_from_check_blocked(dispatcher):
    """blocks=False 留言不出现在 check_blocked 结果."""
    dispatcher.comment("T21", "普通留言 1")
    dispatcher.comment("T21", "普通留言 2", blocks=False)
    dispatcher.comment("T21", "BLOCK 留言", blocks=True)
    blocking = dispatcher.check_blocked("T21")
    assert len(blocking) == 1
    assert "BLOCK" in blocking[0].body


# ============ §1.6 mark_read (read-receipt) ============

def test_mark_read_appends_receipt(dispatcher):
    """mark_read append 1 条 read-receipt 留言 (per 守门 v33 §1.6)."""
    c1 = dispatcher.comment("T30", "原留言")
    success = dispatcher.mark_read("T30", c1.id)
    assert success is True
    all_comments = dispatcher.list_comments("T30")
    assert len(all_comments) == 2
    # 第 2 条是 read-receipt
    assert all_comments[1].parent_comment_id == c1.id
    assert "read-receipt" in all_comments[1].tags


def test_mark_read_nonexistent_returns_false(dispatcher):
    """mark_read 不存在的 comment_id → False (不抛)."""
    assert dispatcher.mark_read("T31", "comment_999") is False


def test_mark_read_does_not_modify_original(dispatcher):
    """append-only: mark_read 不修改原留言 (per 守门 #13 T)."""
    c1 = dispatcher.comment("T32", "原留言")
    body_before = c1.body
    dispatcher.mark_read("T32", c1.id)
    # 重读原留言, body 不变
    c1_after = dispatcher.list_comments("T32")[0]
    assert c1_after.body == body_before


# ============ §1.5 brief 第 6 段渲染 ============

def test_brief_renders_read_comments_section_with_no_comments(dispatcher):
    """无留言时 brief 显示 (无任务卡留言)."""
    brief_path = dispatcher.brief("T40", "Task body", agent="worker")
    content = brief_path.read_text(encoding="utf-8")
    assert "## 6. Read COMMENTS" in content
    assert "(无任务卡留言)" in content


def test_brief_renders_read_comments_section_with_multiple(dispatcher):
    """多留言时 brief 按时间序列出, 标记 BLOCK."""
    dispatcher.comment("T41", "普通留言", author="Mavis", actor_role="orchestrator")
    dispatcher.comment("T41", "BLOCK: 等 P3-D.6-1-1 完", author="Ulysses", actor_role="Ulysses", blocks=True, tags=["urgent"])
    brief_path = dispatcher.brief("T41", "Task body", agent="worker")
    content = brief_path.read_text(encoding="utf-8")
    assert "[BLOCK]" in content
    assert "#urgent" in content
    assert "Ulysses" in content


# ============ §1.4 invoke 前 check_blocked 联动 ============

def test_invoke_raises_on_block_comment(tmp_path):
    """BLOCK 留言 → invoke 抛 BlockedByCommentError (per 守门 v33 §1.4)."""
    briefs = tmp_path / "briefs"
    briefs.mkdir()
    audit = tmp_path / "audit.log"
    d = SubagentDispatcher(phase="v33_test", briefs_dir=briefs, audit_log=audit)

    # 落 brief + 写 BLOCK 留言
    brief_path = d.brief("T50", "Task body", agent="worker")
    d.comment("T50", "BLOCK: 等 T49 完", blocks=True)

    # invoke 抛 BlockedByCommentError
    with pytest.raises(BlockedByCommentError) as exc_info:
        d.invoke(brief_path, timeout=5)
    # 错误信息含 comment id
    assert "T50" in str(exc_info.value)
    assert "comment_002" in str(exc_info.value) or "BLOCK" in str(exc_info.value)


def test_invoke_no_block_proceeds(tmp_path):
    """无 BLOCK 留言 → invoke 正常 (走 deferred 路径, 不会抛)."""
    briefs = tmp_path / "briefs"
    briefs.mkdir()
    audit = tmp_path / "audit.log"
    d = SubagentDispatcher(phase="v33_test", briefs_dir=briefs, audit_log=audit)
    brief_path = d.brief("T51", "Task body", agent="worker")
    d.comment("T51", "普通留言, 不阻断")
    # 无 BLOCK → invoke 走 subprocess 路径 (mavis CLI 不存在 → status="deferred", 不抛)
    handle = d.invoke(brief_path, timeout=5)
    assert handle.status in ("deferred", "succeeded", "failed")


# ============ §1.2 schema 完整性 ============

def test_comment_schema_fields(dispatcher):
    """Comment dataclass 含 9 字段 (per 守门 v33 §1.2)."""
    c = dispatcher.comment(
        "T60", "schema test",
        author="Mavis", actor_role="orchestrator",
        mentions=["T61"], blocks=False, tags=["requirement"],
        parent_comment_id=None, refs=["commit_abc123"],
    )
    assert c.id == "comment_001"
    assert c.author == "Mavis"
    assert c.actor_role == "orchestrator"
    assert c.body == "schema test"
    assert "T60" in c.mentions
    assert "T61" in c.mentions
    assert c.blocks is False
    assert "requirement" in c.tags
    assert c.parent_comment_id is None
    assert "commit_abc123" in c.refs


def test_comment_unicode_body(dispatcher):
    """unicode body (中文) 正确存储."""
    c = dispatcher.comment("T70", "请在 src/domain/lib.rs 加 字段 (per 守门 #5 派生)")
    assert "lib.rs" in c.body
    assert "守门" in c.body
    # 读回也能正确解析
    comments = dispatcher.list_comments("T70")
    assert comments[0].body == c.body


def test_many_comments_performance(dispatcher):
    """性能: 100 条留言读写 < 500ms (per 守门 #9 RPC 不可靠 派生)."""
    import time
    t0 = time.perf_counter()
    for i in range(100):
        dispatcher.comment("T80", f"msg {i}", tags=[f"tag{i}"])
    write_elapsed = (time.perf_counter() - t0) * 1000

    t0 = time.perf_counter()
    comments = dispatcher.list_comments("T80")
    read_elapsed = (time.perf_counter() - t0) * 1000

    assert len(comments) == 100
    assert write_elapsed < 500, f"100 写耗时 {write_elapsed:.1f}ms"
    assert read_elapsed < 100, f"100 读耗时 {read_elapsed:.1f}ms"
