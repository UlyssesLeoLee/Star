#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/ai_log_mock.py — Log AI 分析 mock
(per docs/requirements/SRS-STAR-OPS-001.md §5.3 + docs/basic-design/OPS-BASIC-DESIGN-001.md §5.3)

per 守门 #23: 不开 OpenAI/Anthropic 第三方 API, 仅本地模板生成
per 守门 #19 v19: agent 跟外部交互的功能点强制走 scripts/automation/<purpose>.py 落地
per 守门 #24 v2: 调试控制台走 subprocess 替代 RPC, 可重放可观测

输入: log 文本 (stdin 或 --log)
输出: JSON {summary, anomalies: [...], suggestions: [...], confidence: 0.42, generated_by: "mock"}
       confidence 永远 < 0.5 提示用户手动 review (守门 #23 派生规)

3 条建议 (跟 ai_edit_mock.py 同模式):
- detect_errors: ERROR/WARN 计数 + 异常类型判断
- summarize_source: 按 source 聚合, 给 top N 来源
- suggest_action: 模板化建议 (add_healthcheck / rollback / review)

用法:
    echo "2026-09-08T07:30:00Z ERROR helm release 3 deploy failed" | python scripts/automation/ai_log_mock.py
    python scripts/automation/ai_log_mock.py --log "$(cat /var/log/star-mcp.log)"
"""
from __future__ import annotations

import argparse
import json
import re
import sys
import time
from dataclasses import dataclass, asdict
from typing import Optional


@dataclass
class Anomaly:
    """异常类型, per OPS-BASIC-DESIGN §3.2"""
    type: str  # spike / drop / pattern / unknown
    timestamp: str
    level: str
    message_excerpt: str


@dataclass
class Suggestion:
    """AI 建议, per OPS-BASIC-DESIGN §3.2"""
    id: str
    text: str
    confidence: float  # 永远 < 0.5 (守门 #23 派生规)


@dataclass
class MockOutput:
    """subprocess 输出 schema, per star-ops::ops_ai::mock::MockSubprocessOutput"""
    summary: str
    anomalies: list[Anomaly]
    suggestions: list[Suggestion]


# === 守门 #23 派生规: confidence 永远 < 0.5 ===
MOCK_CONFIDENCE = 0.42
MOCK_CHANNEL = "mock"


def detect_errors(log_text: str) -> tuple[int, int, list[Anomaly]]:
    """检测 ERROR/WARN 数量, 抽取前 3 条异常"""
    error_count = len(re.findall(r"\bERROR\b", log_text, re.IGNORECASE))
    warn_count = len(re.findall(r"\bWARN(?:ING)?\b", log_text, re.IGNORECASE))

    anomalies: list[Anomaly] = []
    # 抽取 ERROR 行
    for m in re.finditer(r"(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?)\s+(ERROR|WARN(?:ING)?)\s+(.{0,100})", log_text):
        ts, level, excerpt = m.group(1), m.group(2).upper().replace("WARNING", "WARN"), m.group(3).strip()
        anomalies.append(Anomaly(
            type="spike" if "ERROR" in level.upper() else "pattern",
            timestamp=ts,
            level=level,
            message_excerpt=excerpt,
        ))
        if len(anomalies) >= 3:
            break

    return error_count, warn_count, anomalies


def suggest_action(error_count: int, warn_count: int) -> list[Suggestion]:
    """基于错误数生成 1-2 条模板建议 (mock 永远 needs_review)"""
    suggestions: list[Suggestion] = []
    if error_count > 0:
        suggestions.append(Suggestion(
            id="s-healthcheck",
            text="检查 helm release / pod health check 配置 (mock 建议, 需人工 review)",
            confidence=MOCK_CONFIDENCE,
        ))
    if error_count >= 3 or warn_count >= 5:
        suggestions.append(Suggestion(
            id="s-rollback",
            text="ERROR 数过多时考虑 rollback 到上一个 stable revision (mock 建议, 需人工 review)",
            confidence=MOCK_CONFIDENCE * 0.8,  # 更低 confidence 反映更大不确定性
        ))
    if not suggestions:
        suggestions.append(Suggestion(
            id="s-ok",
            text="当前 log 无明显异常, 继续保持监控 (mock 建议)",
            confidence=MOCK_CONFIDENCE,
        ))
    return suggestions


def analyze_log(log_text: str) -> MockOutput:
    """主入口: 分析 log 文本, 返 MockOutput"""
    error_count, warn_count, anomalies = detect_errors(log_text)
    suggestions = suggest_action(error_count, warn_count)

    summary = (
        f"log 文本 {len(log_text)} 字符, "
        f"检测到 {error_count} ERROR / {warn_count} WARN, "
        f"mock 通道简化版 (subprocess 路径 [M] 子项实装, 当前仅正则模板)"
    )

    return MockOutput(
        summary=summary,
        anomalies=anomalies,
        suggestions=suggestions,
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Log AI 分析 mock (守门 #23 不开外部 API, 仅本地模板)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "--log",
        type=str,
        help="log 文本 (优先, 缺省读 stdin)",
    )
    parser.add_argument(
        "--confidence",
        type=float,
        default=MOCK_CONFIDENCE,
        help=f"confidence override (默认 {MOCK_CONFIDENCE}, 守门 #23 永远 < 0.5)",
    )
    args = parser.parse_args()

    if args.log is not None:
        log_text = args.log
    elif not sys.stdin.isatty():
        log_text = sys.stdin.read()
    else:
        print(json.dumps({
            "error": "no log input, use --log or pipe via stdin",
        }), file=sys.stderr)
        return 1

    started = time.time()
    output = analyze_log(log_text)
    elapsed_ms = int((time.time() - started) * 1000)

    # 顶层 JSON 包装 (per star-ops::ops_ai::mock::MockSubprocessOutput)
    result = {
        "summary": output.summary,
        "anomalies": [asdict(a) for a in output.anomalies],
        "suggestions": [asdict(s) for s in output.suggestions],
        "confidence": min(args.confidence, MOCK_CONFIDENCE),  # 守门 #23: 永远 <= 0.42
        "generated_by": MOCK_CHANNEL,
        "elapsed_ms": elapsed_ms,
        "needs_review": True,  # mock 永远 needs_review
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
