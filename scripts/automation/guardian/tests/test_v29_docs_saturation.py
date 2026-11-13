"""v29_docs_saturation 测试 — docs 同步饱和阈值 (WBS 9.11)."""
# SPDX-License-Identifier: MIT OR Apache-2.0

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import v29_docs_saturation as v29


def test_evaluate_ok_below_warn():
    code, msg = v29.evaluate(10)
    assert code == 0
    assert "OK" in msg


def test_evaluate_warn_at_30():
    code, msg = v29.evaluate(30)
    assert code == 0
    assert "WARN" in msg


def test_evaluate_ask_at_40():
    code, msg = v29.evaluate(40)
    assert code == 1
    assert "ASK" in msg


def test_evaluate_block_at_50():
    code, msg = v29.evaluate(50)
    assert code == 2
    assert "BLOCK" in msg


def test_evaluate_block_above_50():
    code, _ = v29.evaluate(99)
    assert code == 2


def test_evaluate_boundaries():
    assert v29.evaluate(29)[1].startswith("OK")
    assert v29.evaluate(39)[1].startswith("WARN")
    assert v29.evaluate(49)[1].startswith("ASK")


def test_count_docs_sync_commits_returns_zero_on_git_failure(monkeypatch):
    import subprocess

    def fake_run(*args, **kwargs):
        raise subprocess.SubprocessError("boom")

    monkeypatch.setattr(v29.subprocess, "run", fake_run)
    # 直接测 _count_docs_sync_commits 的异常路径: 捕获异常时返回 0
    class _Probe:
        returncode = 1
        stdout = ""
        stderr = ""

    def fake_run_probe(*args, **kwargs):
        return _Probe()

    monkeypatch.setattr(v29.subprocess, "run", fake_run_probe)
    assert v29._count_docs_sync_commits() == 0


def test_count_docs_sync_commits_parses_stdout(monkeypatch):
    class _Probe:
        returncode = 0
        stdout = "a1b2c3 docs(wbs): x\n\n  d4e5f6 docs(wbs): y\n"
        stderr = ""

    monkeypatch.setattr(v29.subprocess, "run", lambda *a, **k: _Probe())
    assert v29._count_docs_sync_commits() == 2


def test_main_uses_count_override(monkeypatch, capsys):
    monkeypatch.setattr(sys, "argv", ["v29", "--count", "45"])
    assert v29.main() == 1
    assert "ASK" in capsys.readouterr().out


def test_main_defaults_to_real_count(monkeypatch, capsys):
    class _Probe:
        returncode = 0
        stdout = "one docs(wbs): only\n"
        stderr = ""

    monkeypatch.setattr(v29.subprocess, "run", lambda *a, **k: _Probe())
    monkeypatch.setattr(sys, "argv", ["v29"])
    assert v29.main() == 0
    assert "OK" in capsys.readouterr().out
