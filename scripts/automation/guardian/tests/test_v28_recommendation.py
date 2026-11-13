"""v28_recommendation 测试 — ask_user 推荐项格式校验 (WBS 9.11)."""
# SPDX-License-Identifier: MIT OR Apache-2.0

import io
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import v28_recommendation as v28


def _opt(label: str, description: str = "desc") -> dict:
    return {"label": label, "description": description}


def test_score_ok_full_compliance():
    score, issues = v28._score([_opt("A (推荐)"), _opt("B"), _opt("C")])
    assert score == 2
    assert issues == []


def test_score_rejects_empty_options():
    score, issues = v28._score([])
    assert score == 0
    assert any("option count" in i for i in issues)


def test_score_rejects_single_option():
    score, issues = v28._score([_opt("A (推荐)")])
    assert score == 0
    assert any("not in [2,4]" in i for i in issues)


def test_score_rejects_more_than_4():
    score, _ = v28._score([_opt("A (推荐)"), _opt("B"), _opt("C"), _opt("D"), _opt("E")])
    assert score == 0


def test_score_rejects_missing_recommended_tag():
    score, issues = v28._score([_opt("A"), _opt("B")])
    assert score == 0
    assert any("missing" in i for i in issues)


def test_score_rejects_recommended_not_first():
    score, issues = v28._score([_opt("A"), _opt("B (推荐)"), _opt("C")])
    assert score == 0
    assert any("first option" in i for i in issues)


def test_score_rejects_empty_description():
    score, issues = v28._score([_opt("A (推荐)"), _opt("B", "")])
    assert score == 0
    assert any("empty description" in i for i in issues)


def test_score_accepts_fullwidth_and_english_tags():
    assert v28._score([_opt("A （推荐）"), _opt("B")])[0] == 2
    assert v28._score([_opt("A (recommended)"), _opt("B")])[0] == 2


def test_validate_prints_ok_on_pass(capsys):
    assert v28.validate([_opt("A (推荐)"), _opt("B")]) == 0
    assert "OK" in capsys.readouterr().out


def test_validate_prints_fail_on_violation(capsys):
    assert v28.validate([_opt("A"), _opt("B")]) == 1
    assert "FAIL" in capsys.readouterr().out


def test_lint_accepts_payload_from_stdin(monkeypatch, capsys):
    monkeypatch.setattr(sys, "stdin", io.StringIO('{"options": [{"label": "A (推荐)", "description": "x"}, {"label": "B", "description": "y"}]}'))
    assert v28.lint() == 0


def test_lint_uses_choices_key(monkeypatch):
    monkeypatch.setattr(sys, "stdin", io.StringIO('{"choices": [{"label": "A (推荐)", "description": "x"}, {"label": "B", "description": "y"}]}'))
    assert v28.lint() == 0


def test_lint_rejects_invalid_json(monkeypatch, capsys):
    monkeypatch.setattr(sys, "stdin", io.StringIO("{not json"))
    assert v28.lint() == 1
    assert "invalid JSON" in capsys.readouterr().out
