"""T3.2 集成测试 - dispatcher.py 接入 PreToolUse hook.

Per WBS-002 T3.2 + DD §1.3 + FR-1.2.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
sys.path.insert(0, str(Path(__file__).resolve().parents[2].parent))


def test_dispatcher_imports_with_guard():
    """dispatcher.py 导入 OK + DispatchBlockedError 暴露."""
    from dispatcher import SubagentDispatcher, DispatchBlockedError
    assert SubagentDispatcher is not None
    assert issubclass(DispatchBlockedError, Exception)


def test_dispatcher_hooks_into_invoke(tmp_path, monkeypatch):
    """invoke 时 hook 触发: 危险 brief → DispatchBlockedError."""
    from dispatcher import SubagentDispatcher, DispatchBlockedError

    # 准备 1 个含 'rm -rf /' 的 brief 文件
    brief = tmp_path / "test_danger_brief.md"
    brief.write_text(
        "# Task\n\nRun this:\n\n```bash\nrm -rf /tmp/all\n```\n",
        encoding="utf-8",
    )
    # 在 brief 里塞 GitHub PAT (命中 R-BLOCK-007)
    brief.write_text(
        "# Task\n\ncurl -H 'Authorization: token ghp_dangerousPAT1234567890' https://x\n",
        encoding="utf-8",
    )

    # 默认 STAR_PRE_TOOL_USE_GUARD=1, hook 启用
    monkeypatch.delenv("STAR_PRE_TOOL_USE_GUARD", raising=False)

    audit_dir = tmp_path / "audit"
    audit_dir.mkdir()
    d = SubagentDispatcher(phase="T3.2_test", audit_log=audit_dir / "test.log")

    with pytest.raises(DispatchBlockedError) as exc_info:
        d.invoke(brief, timeout=5)
    # 验证: 触发了 BLOCK
    assert "PreToolUse guard BLOCK" in str(exc_info.value)


def test_dispatcher_hook_disabled_by_env(tmp_path, monkeypatch):
    """STAR_PRE_TOOL_USE_GUARD=0 → hook 跳过 (per 缺标比错标)."""
    from dispatcher import SubagentDispatcher

    brief = tmp_path / "test_brief.md"
    brief.write_text("# Task\nsafe content\n", encoding="utf-8")

    monkeypatch.setenv("STAR_PRE_TOOL_USE_GUARD", "0")
    audit_dir = tmp_path / "audit"
    audit_dir.mkdir()
    d = SubagentDispatcher(phase="T3.2_test_disabled", audit_log=audit_dir / "test.log")

    # env 0 → 跳过 hook, 不抛 DispatchBlockedError
    # (会尝试 mavis CLI, FileNotFoundError → status="deferred", 不抛)
    handle = d.invoke(brief, timeout=5)
    assert handle.status in ("deferred", "succeeded", "failed")  # 看 mavis CLI 是否存在


def test_dispatcher_safe_brief_passes_guard(tmp_path, monkeypatch):
    """安全 brief → 不抛 DispatchBlockedError."""
    from dispatcher import SubagentDispatcher

    brief = tmp_path / "test_safe_brief.md"
    brief.write_text(
        "# Task\n\nThis is a safe task. Just read some files and report.\n",
        encoding="utf-8",
    )
    monkeypatch.delenv("STAR_PRE_TOOL_USE_GUARD", raising=False)
    audit_dir = tmp_path / "audit"
    audit_dir.mkdir()
    d = SubagentDispatcher(phase="T3.2_test_safe", audit_log=audit_dir / "test.log")

    # 安全 brief, 不抛 DispatchBlockedError
    # (mavis CLI 不存在 → status="deferred", 但不抛)
    handle = d.invoke(brief, timeout=5)
    assert handle.status in ("deferred", "succeeded", "failed")
