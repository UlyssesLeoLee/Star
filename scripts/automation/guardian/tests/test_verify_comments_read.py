"""v33 v0.3 verify 阶段 comments-read 硬约束单元测试 (per 守门 v33 已知缺口 #5).

6 TC, 覆盖 _verify_comments_read 方法 (软约束 warn 模式):
- 字段缺失 → 1 warn
- 字段存在但缺权威留言 → 1 warn
- 字段存在且全覆盖 → 0 warn
- 无权威留言 → 0 warn
- git log 失败 → False + warn
- 返回类型正确 (bool, list[str])

使用 mock subprocess.run 避免跨平台 git 复杂度。
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import sys
from pathlib import Path
from unittest.mock import patch, MagicMock

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
sys.path.insert(0, str(Path(__file__).resolve().parents[2].parent))

from dispatcher import SubagentDispatcher


@pytest.fixture
def dispatcher(tmp_path):
    briefs = tmp_path / "briefs"
    briefs.mkdir()
    audit = tmp_path / "audit.log"
    d = SubagentDispatcher(phase="v33_v03_test", briefs_dir=briefs, audit_log=audit)
    yield d


def _mock_git_log(commit_message, returncode=0):
    """构造 mock subprocess.run 返回值."""
    mock_result = MagicMock()
    mock_result.returncode = returncode
    mock_result.stdout = commit_message
    mock_result.stderr = "" if returncode == 0 else "fatal: bad commit"
    return mock_result


def test_no_comments_field_warns(dispatcher):
    """commit message 缺 `comments-read:` 字段 → 1 warn (软约束)."""
    with patch("dispatcher.subprocess.run", return_value=_mock_git_log("feat: test\n\nno field\n")):
        ok, warnings = dispatcher._verify_comments_read("task_1", "abc123")
    assert ok is False
    assert any("缺" in w and "comments-read" in w for w in warnings), f"warn 不符: {warnings}"


def test_field_present_full_coverage_passes(dispatcher):
    """commit 含 `comments-read: comment_001,comment_002` + 任务卡 2 条权威留言全覆盖 → 0 warn."""
    dispatcher.comment("task_2", "msg 1", author="Ulysses", actor_role="Ulysses")
    dispatcher.comment("task_2", "msg 2", author="Mavis", actor_role="Mavis")
    msg = "feat: test\n\ncomments-read: comment_001,comment_002\n"
    with patch("dispatcher.subprocess.run", return_value=_mock_git_log(msg)):
        ok, warnings = dispatcher._verify_comments_read("task_2", "abc123")
    assert ok is True, f"应通过, 实际 warn: {warnings}"
    assert warnings == []


def test_field_present_missing_authoritative_warns(dispatcher):
    """commit 含 comment_001 + 任务卡 2 条权威留言 (含 comment_002 缺漏) → 1 warn."""
    dispatcher.comment("task_3", "msg 1", author="Ulysses", actor_role="Ulysses")
    c2 = dispatcher.comment("task_3", "msg 2", author="Mavis", actor_role="Mavis")
    msg = "feat: test\n\ncomments-read: comment_001\n"
    with patch("dispatcher.subprocess.run", return_value=_mock_git_log(msg)):
        ok, warnings = dispatcher._verify_comments_read("task_3", "abc123")
    assert ok is False
    assert any(c2.id in w for w in warnings), f"应提示缺 {c2.id}: {warnings}"


def test_no_authoritative_comments_passes(dispatcher):
    """任务卡无权威留言 (仅 sub-agent) → 0 warn (空 check)."""
    dispatcher.comment("task_4", "msg", author="worker-1", actor_role="sub-agent")
    msg = "feat: test\n\ncomments-read: comment_001\n"
    with patch("dispatcher.subprocess.run", return_value=_mock_git_log(msg)):
        ok, warnings = dispatcher._verify_comments_read("task_4", "abc123")
    assert ok is True, f"无权威留言应通过, 实际 warn: {warnings}"
    assert warnings == []


def test_git_log_failure_returns_error(dispatcher):
    """git log 失败 (returncode != 0) → False + 1 warn."""
    with patch("dispatcher.subprocess.run", return_value=_mock_git_log("fatal", returncode=128)):
        ok, warnings = dispatcher._verify_comments_read("task_5", "abc123")
    assert ok is False
    assert any("exit=128" in w for w in warnings), f"warn 不符: {warnings}"


def test_subprocess_exception_returns_error(dispatcher):
    """subprocess.run 抛 FileNotFoundError → False + 1 warn."""
    with patch("dispatcher.subprocess.run", side_effect=FileNotFoundError("git not found")):
        ok, warnings = dispatcher._verify_comments_read("task_6", "abc123")
    assert ok is False
    assert any("git" in w for w in warnings), f"warn 不符: {warnings}"


def test_method_returns_correct_types(dispatcher):
    """返回类型: (bool, list[str])."""
    with patch("dispatcher.subprocess.run", return_value=_mock_git_log("no field")):
        ok, warnings = dispatcher._verify_comments_read("task_7", "abc123")
    assert isinstance(ok, bool)
    assert isinstance(warnings, list)
    assert all(isinstance(w, str) for w in warnings)
