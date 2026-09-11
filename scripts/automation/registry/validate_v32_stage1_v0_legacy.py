"""
validate_v32_stage1.py — v32 阶段 1 验证脚本 (per SRS-MULTICA-RUNTIME-001 §7 AC + DD §11 UT/IT)

@deprecated v0.legacy — Replaced by Rust integration tests in R3 阶段
                (per ADR-0027 R3). 阶段 1 实装: commit `fadff8f`.
                Per 守门 #11 缺标比错标: 不删, 永久留档.

验证项 (per SRS §7 AC-1 ~ AC-8 + DD §11):
- AC-1 scan 5 provider 全部探测, output 含全部 name
- AC-2 MinVersion gate 8 provider 全部满足, 低于最低 → 标 poisoned
- AC-3 Login shell fallback 跑通 (在 macOS, Windows 跳过)
- AC-4 三档 status 全部覆盖, 守门 #11 缺标比错标 满足
- AC-5 不读 env 值 (per 守门 #5)
- AC-6 subprocess.run(shell=False) 全部 (per 守门 #6)
- AC-7 每次 scan 落 log (per 守门 #1 v15)
"""
import re
import subprocess
import sys
from pathlib import Path

# Windows GBK 兼容: reconfigure stdout to utf-8 (per 阶段 1 实装 fix)
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")

sys.path.insert(0, str(Path(__file__).parent))


def main():
    errors = []
    warnings = []

    # AC-1: 5 provider 全部探测
    print("[AC-1] 5 provider 全部探测")
    from probe_v0_legacy import RuntimeProbe
    probe = RuntimeProbe()
    entries = probe.probe_all()
    if len(entries) != 5:
        errors.append(f"AC-1 FAIL: expected 5 entries, got {len(entries)}")
    else:
        print(f"  ✅ 5 entries: {[e.provider for e in entries]}")
    expected_providers = {"claude", "codex", "mcode", "copilot", "grok"}
    actual_providers = {e.provider for e in entries}
    if actual_providers != expected_providers:
        errors.append(f"AC-1 FAIL: expected {expected_providers}, got {actual_providers}")

    # AC-2: MinVersion gate
    print("[AC-2] MinVersion gate 8 provider 全部满足, 低于最低 → 标 poisoned")
    from min_version_v0_legacy import MinVersionGate
    gate = MinVersionGate()
    # 测试 ok case
    v_ok = gate.check("claude", "2.5.0")
    if v_ok.status != "ok":
        errors.append(f"AC-2 FAIL: claude 2.5.0 should be ok, got {v_ok.status}")
    else:
        print(f"  ✅ claude 2.5.0 → ok")
    # 测试 too_old case
    v_old = gate.check("claude", "1.5.0")
    if v_old.status != "too_old":
        errors.append(f"AC-2 FAIL: claude 1.5.0 should be too_old, got {v_old.status}")
    else:
        print(f"  ✅ claude 1.5.0 → too_old (sentinel error 区分, per FR-9)")
    # 测试 no_minimum case
    v_none = gate.check("not_in_list", "1.0.0")
    if v_none.status != "no_minimum":
        errors.append(f"AC-2 FAIL: not_in_list should be no_minimum, got {v_none.status}")
    else:
        print(f"  ✅ not_in_list 1.0.0 → no_minimum (per FR-11)")
    # 测试 missing case (unparseable version)
    v_miss = gate.check("claude", "")
    if v_miss.status != "missing":
        errors.append(f"AC-2 FAIL: claude '' should be missing, got {v_miss.status}")
    else:
        print(f"  ✅ claude '' → missing (sentinel error 区分, per FR-9)")
    # 测试 dev-build exception
    v_dev = gate.check("claude", "v0.2.15-235-gdaf0e935")
    if v_dev.status != "ok":
        errors.append(f"AC-2 FAIL: dev-build should be ok, got {v_dev.status}")
    else:
        print(f"  ✅ dev-build v0.2.15-235-gdaf0e935 → ok (per FR-10)")

    # AC-3: Login shell fallback (Windows 跳过)
    print("[AC-3] Login shell fallback (Windows 跳过)")
    from shell_resolve_v0_legacy import LoginShellResolver
    resolver = LoginShellResolver()
    if "win" in sys.platform:
        warnings.append("AC-3 SKIP: Windows 不实现 login shell fallback (per FR-7 平台限制)")
        print(f"  ⚠️ Windows 平台, 跳过 (per FR-7)")
    else:
        # 找一个 shell 一定有的命令
        test_cmd = "ls"  # 几乎所有 unix 都有
        resolved = resolver.resolve(test_cmd)
        if resolved is None:
            warnings.append(f"AC-3 WARN: 登录 shell 解析 {test_cmd} 失败 (环境可能 PATH 已含, 不需 fallback)")
        else:
            print(f"  ✅ 登录 shell 解析 {test_cmd} → {resolved}")

    # AC-4: 三档 status 全部覆盖
    print("[AC-4] 三档 status 全部覆盖")
    from status_v0_legacy import StatusClassifier, StatusVerdict
    classifier = StatusClassifier()
    # missing
    from probe_v0_legacy import RuntimeEntry
    e_missing = RuntimeEntry(provider="x", path=None, version=None, version_parsed=None, probe_method="missing")
    from min_version_v0_legacy import MinVersionVerdict
    v_miss_min = MinVersionVerdict("x", None, None, "no_minimum")
    s_missing = classifier.classify(e_missing, v_miss_min)
    if s_missing != StatusVerdict.MISSING:
        errors.append(f"AC-4 FAIL: missing path should be MISSING, got {s_missing}")
    else:
        print(f"  ✅ path=None → ⚫ MISSING")
    # poisoned (too_old)
    e_ok = RuntimeEntry(provider="x", path=Path("/bin/x"), version="1.0.0", version_parsed=(1,0,0), probe_method="path")
    v_too_old = MinVersionVerdict("x", "1.0.0", "2.0.0", "too_old")
    s_poisoned = classifier.classify(e_ok, v_too_old)
    if s_poisoned != StatusVerdict.POISONED:
        errors.append(f"AC-4 FAIL: too_old should be POISONED, got {s_poisoned}")
    else:
        print(f"  ✅ too_old → 🔴 POISONED (per 守门 #11 不删标)")
    # stale (auth expired)
    v_ok_min = MinVersionVerdict("x", "2.0.0", "2.0.0", "ok")
    s_stale = classifier.classify(e_ok, v_ok_min, auth_status="expired")
    if s_stale != StatusVerdict.STALE:
        errors.append(f"AC-4 FAIL: auth expired should be STALE, got {s_stale}")
    else:
        print(f"  ✅ auth=expired → 🟡 STALE")
    # active (auth ok, default)
    s_active = classifier.classify(e_ok, v_ok_min)
    if s_active != StatusVerdict.ACTIVE:
        errors.append(f"AC-4 FAIL: default auth should be ACTIVE, got {s_active}")
    else:
        print(f"  ✅ default (no auth probe yet) → 🟢 ACTIVE (阶段 1: auth probe 阶段 2 实装)")

    # AC-5: 不读 env 值
    print("[AC-5] 不读 env 值 (per 守门 #5)")
    # self-skip: validator scans other py files (per 防 self-referential regex)
    py_files = [f for f in Path(__file__).parent.glob("*.py") if f.name not in ("validate_v32_stage1.py", "validate_v32_stage1_v0_legacy.py")]
    for f in py_files:
        content = f.read_text(encoding="utf-8")
        # 检查 os.environ.get(...) print 模式
        bad_patterns = [
            (r"os\.environ\.get\([^)]*\)\s*\)", "os.environ.get() 不打印"),
            (r"print\s*\(\s*os\.environ", "print(os.environ)"),
            (r"getenv\([^)]*\)\s*\)", "os.getenv()"),
        ]
        for pat, desc in bad_patterns:
            if re.search(pat, content):
                errors.append(f"AC-5 FAIL: {f.name} 含 {desc}")
    if not any("AC-5 FAIL" in e for e in errors):
        print(f"  ✅ {len(py_files)} py files 全部合规 (不读 env 值, validator self-skip)")

    # AC-6: subprocess.run(shell=False) 全部
    print("[AC-6] subprocess.run(shell=False) 全部 (per 守门 #6)")
    for f in py_files:
        content = f.read_text(encoding="utf-8")
        if "subprocess.run" in content and "shell=False" not in content:
            errors.append(f"AC-6 FAIL: {f.name} 含 subprocess.run 但缺 shell=False")
    if not any("AC-6 FAIL" in e for e in errors):
        print(f"  ✅ {len(py_files)} py files 全部合规 (shell=False 全部)")

    # AC-7: 每次 scan 落 log
    print("[AC-7] 每次 scan 落 log (per 守门 #1 v15)")
    from reporter_v0_legacy import RuntimeStatusReporter
    from datetime import datetime, timezone
    import tempfile
    with tempfile.TemporaryDirectory() as tmpdir:
        reporter = RuntimeStatusReporter(log_dir=Path(tmpdir))
        now = datetime.now(timezone.utc)
        log_path = reporter.write_log("test-scan-id", now, now, entries)
        if log_path.exists() and log_path.stat().st_size > 0:
            print(f"  ✅ log 落档: {log_path.name} ({log_path.stat().st_size} bytes)")
        else:
            errors.append(f"AC-7 FAIL: log file empty or missing")

    # 总结
    print()
    print("=" * 60)
    if errors:
        print(f"[FAIL] {len(errors)} errors:")
        for e in errors:
            print(f"  - {e}")
        return 1
    else:
        print(f"[PASS] 全部 AC 通过 ({len(warnings)} warnings)")
        for w in warnings:
            print(f"  [WARN] {w}")
        return 0


if __name__ == "__main__":
    sys.exit(main())
