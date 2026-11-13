"""v27_rpc_fallback 测试 — 子代理 dispatch 实证后备 (WBS 9.11)."""
# SPDX-License-Identifier: MIT OR Apache-2.0

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import v27_rpc_fallback as v27


def test_constants():
    assert v27.MAX_RETRIES == 2
    assert v27.VERIFY_TIMEOUT_SEC == 30
    assert v27.DISPATCHER_PY.name == "dispatcher.py"


def test_utc_now_iso_format():
    out = v27._utc_now_iso()
    assert out.endswith("Z") or "+00:00" in out
    assert "T" in out


def test_write_and_read_status_roundtrip(tmp_path, monkeypatch):
    monkeypatch.setattr(v27, "STATUS_DIR", tmp_path)
    payload = {"task_id": "t1", "phase": "invoke", "retry_count": 1}
    p = v27._write_status("t1", payload)
    assert p.exists()
    read = v27._read_status("t1")
    assert read["retry_count"] == 1
    assert read["phase"] == "invoke"


def test_read_status_missing_returns_empty(tmp_path, monkeypatch):
    monkeypatch.setattr(v27, "STATUS_DIR", tmp_path)
    assert v27._read_status("nope") == {}


def test_run_dispatcher_timeout(monkeypatch):
    import subprocess

    def fake_run(*args, **kwargs):
        raise subprocess.TimeoutExpired(cmd=args[0] if args else "x", timeout=1)

    monkeypatch.setattr(v27.subprocess, "run", fake_run)
    result = v27._run_dispatcher("invoke", "t", "b", "w")
    assert result.get("_timeout") is True


def test_run_dispatcher_nonzero_exit(monkeypatch):
    class _Probe:
        returncode = 3
        stdout = ""
        stderr = "err-detail"

    monkeypatch.setattr(v27.subprocess, "run", lambda *a, **k: _Probe())
    result = v27._run_dispatcher("invoke", "t", "b", "w")
    assert result.get("_exit") == 3
    assert result.get("_error") == "err-detail"


def test_run_dispatcher_parses_json_stdout(monkeypatch):
    class _Probe:
        returncode = 0
        stdout = '{"ok": true}'
        stderr = ""

    monkeypatch.setattr(v27.subprocess, "run", lambda *a, **k: _Probe())
    result = v27._run_dispatcher("invoke", "t", "b", "w")
    assert result == {"ok": True}


def test_run_dispatcher_non_json_stdout(monkeypatch):
    class _Probe:
        returncode = 0
        stdout = "not json at all"
        stderr = ""

    monkeypatch.setattr(v27.subprocess, "run", lambda *a, **k: _Probe())
    result = v27._run_dispatcher("invoke", "t", "b", "w")
    assert result.get("_raw") == "not json at all"


def test_run_dispatcher_empty_stdout(monkeypatch):
    class _Probe:
        returncode = 0
        stdout = ""
        stderr = ""

    monkeypatch.setattr(v27.subprocess, "run", lambda *a, **k: _Probe())
    assert v27._run_dispatcher("invoke", "t", "b", "w") == {}


def test_invoke_writes_status_and_calls_dispatcher(tmp_path, monkeypatch):
    monkeypatch.setattr(v27, "STATUS_DIR", tmp_path)
    monkeypatch.setattr(v27, "OUTPUT_DIR", tmp_path)

    calls = []

    def fake_run(*args, **kwargs):
        calls.append(args)
        return {"dispatched": True}

    monkeypatch.setattr(v27, "_run_dispatcher", fake_run)
    status = v27.invoke("task-9", "docs/briefs/b9.md", "D:/Star-worktrees/wt9")
    assert status["phase"] == "invoke"
    assert status["retry_count"] == 0
    assert len(calls) == 1
    assert calls[0][0] == "invoke"
    # status 文件落盘, history 记录了 invoke 结果
    on_disk = json.loads((tmp_path / "task-9.status.json").read_text(encoding="utf-8"))
    assert len(on_disk["history"]) == 1
    assert on_disk["history"][0]["result"] == {"dispatched": True}

def test_verify_returns_ok_on_status_ok(monkeypatch):
    monkeypatch.setattr(v27, "_run_dispatcher", lambda *a, **k: {"status": "ok"})
    assert v27.verify("t1") == {"status": "ok"}


def test_verify_returns_fail_status(monkeypatch):
    monkeypatch.setattr(v27, "_run_dispatcher", lambda *a, **k: {"status": "fail"})
    assert v27.verify("t1") == {"status": "fail"}


def test_verify_times_out_after_window(monkeypatch):
    monkeypatch.setattr(v27, "VERIFY_TIMEOUT_SEC", 0)
    result = v27.verify("t1")
    assert result.get("_timeout") is True


def test_collect_output_bumps_retry_and_writes_stub(tmp_path, monkeypatch):
    monkeypatch.setattr(v27, "STATUS_DIR", tmp_path)
    monkeypatch.setattr(v27, "OUTPUT_DIR", tmp_path)
    monkeypatch.setattr(v27, "_run_dispatcher", lambda *a, **k: {"output": "real"})
    payload = v27.collect_output("task-9", 1)
    assert payload["retry_count"] == 1
    assert payload["phase"] == "collect_output"
    assert payload["collected_output"] == {"output": "real"}
    stub = tmp_path / "task-9.output.md"
    assert stub.exists()
    assert "Fallback Output" in stub.read_text(encoding="utf-8")


def test_collect_output_seed_from_empty_status(tmp_path, monkeypatch):
    monkeypatch.setattr(v27, "STATUS_DIR", tmp_path)
    monkeypatch.setattr(v27, "OUTPUT_DIR", tmp_path)
    monkeypatch.setattr(v27, "_run_dispatcher", lambda *a, **k: {})
    payload = v27.collect_output("ghost", 2)
    assert payload["task_id"] == "ghost"
    assert payload["retry_count"] == 2


def test_main_skip_verify_path(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(v27, "STATUS_DIR", tmp_path)
    monkeypatch.setattr(v27, "OUTPUT_DIR", tmp_path)
    monkeypatch.setattr(v27, "_run_dispatcher", lambda *a, **k: {})
    monkeypatch.setattr(sys, "argv", ["v27", "--task-id", "t", "--brief", "b.md", "--worktree", "w", "--skip-verify"])
    assert v27.main() == 0
    assert "skipped_verify" in capsys.readouterr().out


def test_main_verified_ok_path(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(v27, "STATUS_DIR", tmp_path)
    monkeypatch.setattr(v27, "OUTPUT_DIR", tmp_path)
    monkeypatch.setattr(v27, "_run_dispatcher", lambda *a, **k: {"status": "ok"})
    monkeypatch.setattr(sys, "argv", ["v27", "--task-id", "t", "--brief", "b.md", "--worktree", "w"])
    assert v27.main() == 0
    assert '"phase": "verified"' in capsys.readouterr().out


def test_main_verify_fail_retry_path(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(v27, "STATUS_DIR", tmp_path)
    monkeypatch.setattr(v27, "OUTPUT_DIR", tmp_path)

    def fake_run(*a, **k):
        return {"status": "fail"} if a[0] == "verify" else {}

    monkeypatch.setattr(v27, "_run_dispatcher", fake_run)
    monkeypatch.setattr(sys, "argv", ["v27", "--task-id", "t", "--brief", "b.md", "--worktree", "w"])
    assert v27.main() == 1
    assert "retry_required" in capsys.readouterr().out


def test_main_verify_fail_fallback_root_session(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(v27, "STATUS_DIR", tmp_path)
    monkeypatch.setattr(v27, "OUTPUT_DIR", tmp_path)

    def fake_run(*a, **k):
        return {"status": "fail"} if a[0] == "verify" else {}

    monkeypatch.setattr(v27, "_run_dispatcher", fake_run)
    monkeypatch.setattr(v27, "MAX_RETRIES", 1)
    monkeypatch.setattr(sys, "argv", ["v27", "--task-id", "t", "--brief", "b.md", "--worktree", "w"])
    assert v27.main() == 2
    assert "fallback_root_session" in capsys.readouterr().out

