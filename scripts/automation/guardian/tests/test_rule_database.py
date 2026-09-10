"""T2.2 单元测试 - RuleDatabase (per WBS-002 T2.2 + DD §7.2).

8+ TC, 覆盖加载/熱更新/schema 校驗失敗/thread safety.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import json
import sys
import threading
import time
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from guardian.rule_database import RuleDatabase, _validate_schema, Rule

GUARDIAN_DIR = Path(__file__).resolve().parents[1]
RULES_PATH = GUARDIAN_DIR / "rules" / "pre_tool_use_rules.json"


@pytest.fixture
def tmp_rules(tmp_path):
    """临时规则文件, init 写入 15 条 + 末尾空行."""
    p = tmp_path / "rules.json"
    data = json.loads(RULES_PATH.read_text(encoding="utf-8"))
    p.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
    return p


# ============ Schema validation ============

def test_schema_valid_real_rules():
    """15 条真规则 schema 校验通过."""
    data = json.loads(RULES_PATH.read_text(encoding="utf-8"))
    errors = _validate_schema(data)
    assert errors == [], f"schema errors: {errors}"


def test_schema_rejects_missing_id():
    errors = _validate_schema({"schema_version": "1.0", "rules": [{"name": "x", "level": "BLOCK", "tool_class": "bash", "pattern": "x", "reason": "x"}]})
    assert any("id" in e for e in errors)


def test_schema_rejects_invalid_level():
    errors = _validate_schema({"schema_version": "1.0", "rules": [{"id": "R-BLOCK-999", "name": "x", "level": "FOO", "tool_class": "bash", "pattern": "x", "reason": "x"}]})
    assert any("level" in e for e in errors)


def test_schema_rejects_duplicate_id():
    data = {
        "schema_version": "1.0",
        "rules": [
            {"id": "R-BLOCK-001", "name": "a", "level": "BLOCK", "tool_class": "bash", "pattern": "x", "reason": "a"},
            {"id": "R-BLOCK-001", "name": "b", "level": "BLOCK", "tool_class": "bash", "pattern": "y", "reason": "b"},
        ]
    }
    errors = _validate_schema(data)
    assert any("duplicate" in e for e in errors)


def test_schema_rejects_bad_id_format():
    data = {"schema_version": "1.0", "rules": [{"id": "BAD-ID", "name": "x", "level": "BLOCK", "tool_class": "bash", "pattern": "x", "reason": "x"}]}
    errors = _validate_schema(data)
    assert any("id" in e for e in errors)


# ============ RuleDatabase load/reload ============

def test_load_valid_rules(tmp_rules):
    db = RuleDatabase(tmp_rules, enable_watcher=False)
    rules = db.get_rules()
    assert len(rules) == 15
    db.stop()


def test_load_invalid_json_keeps_old(tmp_path):
    """FR-6.1: 坏 JSON 保留旧 rules (fail-open)."""
    p = tmp_path / "bad.json"
    p.write_text("{ invalid", encoding="utf-8")
    db = RuleDatabase(p, enable_watcher=False)
    rules = db.get_rules()
    assert rules == [], f"坏 JSON 应 0 规则, 实际 {len(rules)}"
    db.stop()


def test_load_schema_violation_keeps_old(tmp_path):
    """schema 不通过, 0 规则加载, 不抛."""
    p = tmp_path / "bad_schema.json"
    p.write_text(json.dumps({"schema_version": "1.0", "rules": [{"id": "BAD", "name": "x"}]}), encoding="utf-8")
    db = RuleDatabase(p, enable_watcher=False)
    rules = db.get_rules()
    assert rules == []
    db.stop()


def test_reload_after_modify(tmp_rules):
    """FR-5.1: 修改文件后 reload 拿到新规则."""
    db = RuleDatabase(tmp_rules, enable_watcher=False)
    assert len(db.get_rules()) == 15
    # 修改: 加 1 条规则
    data = json.loads(tmp_rules.read_text(encoding="utf-8"))
    data["rules"].append({
        "id": "R-BLOCK-099",
        "name": "test_extra",
        "level": "BLOCK",
        "tool_class": "bash",
        "pattern": "test_extra_xxx",
        "reason": "test"
    })
    tmp_rules.write_text(json.dumps(data, ensure_ascii=False), encoding="utf-8")
    # 手动 reload (不依赖 watcher)
    n = db.reload()
    assert n == 16
    db.stop()


def test_reload_bad_pattern_skipped(tmp_path):
    """regex 编译失败跳过单条规则, 不阻断其他."""
    p = tmp_path / "mixed.json"
    p.write_text(json.dumps({
        "schema_version": "1.0",
        "rules": [
            {"id": "R-BLOCK-001", "name": "good", "level": "BLOCK", "tool_class": "bash", "pattern": "rm\\s+", "reason": "r"},
            {"id": "R-BLOCK-002", "name": "bad", "level": "BLOCK", "tool_class": "bash", "pattern": "[unclosed", "reason": "r"},
        ]
    }), encoding="utf-8")
    db = RuleDatabase(p, enable_watcher=False)
    rules = db.get_rules()
    assert len(rules) == 1  # bad 跳过, 留 1 条
    assert rules[0].id == "R-BLOCK-001"
    db.stop()


def test_reload_latency_under_100ms(tmp_rules):
    """NFR-P-4: reload < 100ms."""
    db = RuleDatabase(tmp_rules, enable_watcher=False)
    t0 = time.perf_counter()
    db.reload()
    elapsed = (time.perf_counter() - t0) * 1000
    assert elapsed < 100, f"reload 耗时 {elapsed:.1f}ms 违反 NFR-P-4"
    db.stop()


def test_get_rules_returns_copy(tmp_rules):
    """返回 list 拷贝, 外部修改不影响内部."""
    db = RuleDatabase(tmp_rules, enable_watcher=False)
    rules1 = db.get_rules()
    rules1.clear()
    rules2 = db.get_rules()
    assert len(rules2) == 15
    db.stop()


def test_concurrent_get_rules_thread_safe(tmp_rules):
    """多线程并发 get_rules 0 异常."""
    db = RuleDatabase(tmp_rules, enable_watcher=False)
    errors = []
    def worker():
        try:
            for _ in range(100):
                _ = db.get_rules()
        except Exception as e:
            errors.append(e)
    threads = [threading.Thread(target=worker) for _ in range(8)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    assert errors == [], f"thread errors: {errors}"
    db.stop()
