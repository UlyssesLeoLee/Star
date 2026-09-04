#!/usr/bin/env python3
# _generate_100_fixtures.py - 100 table W/T/M fixture generator
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-05 06:50 JST user 拍板 "推進" + P5 DB W/T/M 100% 表覆蓋 (推荐)
# 守门: #11 缺标比错标 + #13 派生累積規 CW-01~CW-10
#
# 用途: 一次性 generator 脚本, 从 docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.2 §2
# 提取 100 表 + 业务分类, 生成 100 份 mock fixture (33 M / 47 T / 14 W + 6 mixed = 100)
#
# 不变量: 已存在的 12 份 fixture 保留 (tenant, session_cache, audit_event, etc.),
# generator 跳过已存在文件 (idempotent).

from __future__ import annotations

import json
from pathlib import Path
from typing import Literal

REPO_ROOT = Path(__file__).resolve().parents[3]
MOCK_DATA = REPO_ROOT / "tools" / "star-flash-mock" / "mock_data" / "db-wtm"

# 100 表 W/T/M 分类 (per 00-CLASSIFICATION-W-T-M.md v0.2 §2)
TABLES: list[dict] = [
    # tenant (3) - 3 M
    {"id": "T01", "schema": "tenant", "table": "tenant", "class": "M", "op": "GET", "scenario": "tenant-lookup", "rationale": "テナント分離の源流"},
    {"id": "T02", "schema": "tenant", "table": "tenant_policy", "class": "M", "op": "GET", "scenario": "tenant-policy-lookup", "rationale": "構成情報,慢変"},
    {"id": "T03", "schema": "tenant", "table": "provider_data_boundary", "class": "M", "op": "GET", "scenario": "provider-data-boundary", "rationale": "データ境界定義"},
    # workspace (1) - 1 M/T (主分类 M)
    {"id": "T04", "schema": "workspace", "table": "workspace", "class": "M", "op": "GET", "scenario": "workspace-lookup", "rationale": "M/T 混合 (主分类 M)"},
    # project (3) - 2 M + 1 T
    {"id": "T05", "schema": "project", "table": "project", "class": "T", "op": "GET", "scenario": "project-lookup", "rationale": "業務事実,状態遷移"},
    {"id": "T06", "schema": "project", "table": "project_policy", "class": "M", "op": "GET", "scenario": "project-policy-lookup", "rationale": "構成情報"},
    {"id": "T07", "schema": "project", "table": "project_template", "class": "M", "op": "GET", "scenario": "project-template-lookup", "rationale": "テンプレート,慢変"},
    # work_item (5) - 2 M + 3 T
    {"id": "T08", "schema": "work_item", "table": "work_item", "class": "T", "op": "GET", "scenario": "work-item-lookup", "rationale": "業務核心,状態遷移"},
    {"id": "T09", "schema": "work_item", "table": "requirement", "class": "T", "op": "GET", "scenario": "requirement-lookup", "rationale": "業務要件,弱実体"},
    {"id": "T10", "schema": "work_item", "table": "acceptance_criterion", "class": "T", "op": "GET", "scenario": "acceptance-criterion-lookup", "rationale": "受入基準"},
    {"id": "T11", "schema": "work_item", "table": "business_goal", "class": "M", "op": "GET", "scenario": "business-goal-lookup", "rationale": "業務目標,慢変"},
    {"id": "T12", "schema": "work_item", "table": "work_item_status", "class": "M", "op": "GET", "scenario": "work-item-status-lookup", "rationale": "Lookup enum"},
    # workflow (3) - 3 M
    {"id": "T13", "schema": "workflow", "table": "workflow_definition", "class": "M", "op": "GET", "scenario": "workflow-definition-lookup", "rationale": "構成情報"},
    {"id": "T14", "schema": "workflow", "table": "workflow_state", "class": "M", "op": "GET", "scenario": "workflow-state-lookup", "rationale": "状態定義"},
    {"id": "T15", "schema": "workflow", "table": "workflow_transition", "class": "M", "op": "GET", "scenario": "workflow-transition-lookup", "rationale": "遷移ルール"},
    # board (3) - 3 T
    {"id": "T16", "schema": "board", "table": "board", "class": "T", "op": "GET", "scenario": "board-lookup", "rationale": "kanban board,業務事実"},
    {"id": "T17", "schema": "board", "table": "board_column", "class": "T", "op": "GET", "scenario": "board-column-lookup", "rationale": "Board カラム"},
    {"id": "T18", "schema": "board", "table": "board_swimlane", "class": "T", "op": "GET", "scenario": "board-swimlane-lookup", "rationale": "スイムレーン"},
    # planning (4) - 1 M + 2 T + 1 M/T
    {"id": "T19", "schema": "planning", "table": "sprint", "class": "T", "op": "GET", "scenario": "sprint-lookup", "rationale": "スプリント,状態遷移"},
    {"id": "T20", "schema": "planning", "table": "backlog", "class": "T", "op": "GET", "scenario": "backlog-lookup", "rationale": "バックログ,排序変更"},
    {"id": "T21", "schema": "planning", "table": "roadmap", "class": "M", "op": "GET", "scenario": "roadmap-lookup", "rationale": "M/T 混合 (主分类 M)"},
    {"id": "T22", "schema": "planning", "table": "sprint_state", "class": "M", "op": "GET", "scenario": "sprint-state-lookup", "rationale": "Lookup enum"},
    # relation (2) - 2 T
    {"id": "T23", "schema": "relation", "table": "relation", "class": "T", "op": "GET", "scenario": "relation-lookup", "rationale": "業務関連"},
    {"id": "T24", "schema": "relation", "table": "dependency", "class": "T", "op": "GET", "scenario": "dependency-lookup", "rationale": "依存関係"},
    # comment (4) - 1 M + 3 T
    {"id": "T25", "schema": "comment", "table": "comment", "class": "T", "op": "GET", "scenario": "comment-lookup", "rationale": "業務事実"},
    {"id": "T26", "schema": "comment", "table": "mention", "class": "T", "op": "GET", "scenario": "mention-lookup", "rationale": "メンション"},
    {"id": "T27", "schema": "comment", "table": "attachment", "class": "T", "op": "GET", "scenario": "attachment-lookup", "rationale": "添付"},
    {"id": "T28", "schema": "comment", "table": "comment_visibility", "class": "M", "op": "GET", "scenario": "comment-visibility-lookup", "rationale": "Lookup enum"},
    # search (1) - 1 W
    {"id": "T29", "schema": "search", "table": "search_index", "class": "W", "op": "GET", "scenario": "search-index-lookup", "rationale": "派生,再構築可能"},
    # audit (3) - 3 T (含 1 T/W 主分类 T)
    {"id": "T30", "schema": "audit", "table": "audit_event", "class": "T", "op": "POST", "scenario": "audit-event-append", "rationale": "Append-only 監査"},
    {"id": "T31", "schema": "audit", "table": "ai_audit_metadata", "class": "T", "op": "POST", "scenario": "ai-audit-metadata-append", "rationale": "AI 監査メタ"},
    {"id": "T32", "schema": "audit", "table": "audit_event_outbox", "class": "T", "op": "POST", "scenario": "audit-event-outbox-append", "rationale": "Outbox, T/W 混合"},
    # integration (3) - 2 M + 1 T
    {"id": "T33", "schema": "integration", "table": "integration", "class": "M", "op": "GET", "scenario": "integration-lookup", "rationale": "外部統合設定"},
    {"id": "T34", "schema": "integration", "table": "integration_sync_state", "class": "T", "op": "GET", "scenario": "integration-sync-state", "rationale": "同期状態"},
    {"id": "T35", "schema": "integration", "table": "integration_status", "class": "M", "op": "GET", "scenario": "integration-status-lookup", "rationale": "Lookup enum"},
    # automation (4) - 4 M
    {"id": "T36", "schema": "automation", "table": "automation_rule", "class": "M", "op": "GET", "scenario": "automation-rule-lookup", "rationale": "ルール定義"},
    {"id": "T37", "schema": "automation", "table": "automation_trigger", "class": "M", "op": "GET", "scenario": "automation-trigger-lookup", "rationale": "トリガ定義"},
    {"id": "T38", "schema": "automation", "table": "automation_action", "class": "M", "op": "GET", "scenario": "automation-action-lookup", "rationale": "アクション定義"},
    {"id": "T39", "schema": "automation", "table": "rule_status", "class": "M", "op": "GET", "scenario": "rule-status-lookup", "rationale": "Lookup enum"},
    # identity (5) - 3 M + 1 T + 1 W
    {"id": "T40", "schema": "identity", "table": "user", "class": "M", "op": "GET", "scenario": "user-lookup", "rationale": "構成情報,慢変"},
    {"id": "T41", "schema": "identity", "table": "device", "class": "M", "op": "GET", "scenario": "device-lookup", "rationale": "デバイス登録"},
    {"id": "T42", "schema": "identity", "table": "device_binding", "class": "M", "op": "GET", "scenario": "device-binding-lookup", "rationale": "三重バインディング"},
    {"id": "T43", "schema": "identity", "table": "credential", "class": "T", "op": "GET", "scenario": "credential-lookup", "rationale": "資格情報"},
    {"id": "T44", "schema": "identity", "table": "user_session", "class": "W", "op": "GET", "scenario": "user-session-lookup", "rationale": "短 TTL session-bound"},
    # notification (4) - 3 M + 1 T
    {"id": "T45", "schema": "notification", "table": "notification_channel", "class": "M", "op": "GET", "scenario": "notification-channel-lookup", "rationale": "チャネル設定"},
    {"id": "T46", "schema": "notification", "table": "notification_template", "class": "M", "op": "GET", "scenario": "notification-template-lookup", "rationale": "テンプレート"},
    {"id": "T47", "schema": "notification", "table": "notification", "class": "T", "op": "GET", "scenario": "notification-lookup", "rationale": "送信済み通知"},
    {"id": "T48", "schema": "notification", "table": "notification_status", "class": "M", "op": "GET", "scenario": "notification-status-lookup", "rationale": "Lookup enum"},
    # permission (4) - 4 M
    {"id": "T49", "schema": "permission", "table": "role", "class": "M", "op": "GET", "scenario": "role-lookup", "rationale": "ロール,構成情報"},
    {"id": "T50", "schema": "permission", "table": "permission", "class": "M", "op": "GET", "scenario": "permission-lookup", "rationale": "Lookup enum"},
    {"id": "T51", "schema": "permission", "table": "permission_scheme", "class": "M", "op": "GET", "scenario": "permission-scheme-lookup", "rationale": "スキーム"},
    {"id": "T52", "schema": "permission", "table": "security_policy", "class": "M", "op": "GET", "scenario": "security-policy-lookup", "rationale": "セキュリティポリシー"},
    # collaboration (2) - 2 W
    {"id": "T53", "schema": "collaboration", "table": "presence", "class": "W", "op": "GET", "scenario": "presence-lookup", "rationale": "短 TTL session-bound"},
    {"id": "T54", "schema": "collaboration", "table": "realtime_subscription", "class": "W", "op": "GET", "scenario": "realtime-subscription-lookup", "rationale": "リアルタイム購読"},
    # scm (8) - 2 M + 6 T + 1 W (per §2.18)
    {"id": "T55", "schema": "scm", "table": "repository", "class": "M", "op": "GET", "scenario": "repository-lookup", "rationale": "リポジトリ登録"},
    {"id": "T56", "schema": "scm", "table": "branch", "class": "T", "op": "GET", "scenario": "branch-lookup", "rationale": "ブランチ"},
    {"id": "T57", "schema": "scm", "table": "commit", "class": "T", "op": "GET", "scenario": "commit-lookup", "rationale": "コミット,Append-only"},
    {"id": "T58", "schema": "scm", "table": "pull_request", "class": "T", "op": "GET", "scenario": "pull-request-lookup", "rationale": "PR,状態遷移"},
    {"id": "T59", "schema": "scm", "table": "review", "class": "T", "op": "GET", "scenario": "review-lookup", "rationale": "レビュー"},
    {"id": "T60", "schema": "scm", "table": "pipeline", "class": "T", "op": "GET", "scenario": "pipeline-lookup", "rationale": "CI パイプライン"},
    {"id": "T61", "schema": "scm", "table": "webhook_event", "class": "W", "op": "POST", "scenario": "webhook-event-receive", "rationale": "Webhook 受信,短 TTL"},
    {"id": "T62", "schema": "scm", "table": "pull_request_status", "class": "M", "op": "GET", "scenario": "pull-request-status-lookup", "rationale": "Lookup enum"},
    # development (9) - 6 T + 3 W
    {"id": "T63", "schema": "development", "table": "development_execution", "class": "T", "op": "GET", "scenario": "development-execution-lookup", "rationale": "開発実行"},
    {"id": "T64", "schema": "development", "table": "change_set", "class": "T", "op": "GET", "scenario": "change-set-lookup", "rationale": "変更セット"},
    {"id": "T65", "schema": "development", "table": "file_change", "class": "T", "op": "GET", "scenario": "file-change-lookup", "rationale": "ファイル変更"},
    {"id": "T66", "schema": "development", "table": "symbol_change", "class": "T", "op": "GET", "scenario": "symbol-change-lookup", "rationale": "シンボル変更"},
    {"id": "T67", "schema": "development", "table": "risk_signal", "class": "T", "op": "GET", "scenario": "risk-signal-lookup", "rationale": "リスクシグナル"},
    {"id": "T68", "schema": "development", "table": "change_set_link", "class": "T", "op": "GET", "scenario": "change-set-link-lookup", "rationale": "リンク"},
    {"id": "T69", "schema": "development", "table": "symbol_index", "class": "W", "op": "GET", "scenario": "symbol-index-lookup", "rationale": "派生,再構築可能"},
    {"id": "T70", "schema": "development", "table": "repository_context", "class": "W", "op": "GET", "scenario": "repository-context-lookup", "rationale": "派生"},
    {"id": "T71", "schema": "development", "table": "development_context", "class": "W", "op": "GET", "scenario": "development-context-lookup", "rationale": "派生"},
    # worktree (5) - 1 M + 2 T + 2 W
    {"id": "T72", "schema": "worktree", "table": "worktree", "class": "T", "op": "GET", "scenario": "worktree-lookup", "rationale": "業務核心"},
    {"id": "T73", "schema": "worktree", "table": "worktree_status_observed", "class": "W", "op": "GET", "scenario": "worktree-status-observed-lookup", "rationale": "観測派生"},
    {"id": "T74", "schema": "worktree", "table": "worktree_conflict", "class": "T", "op": "GET", "scenario": "worktree-conflict-lookup", "rationale": "衝突,業務事実"},
    {"id": "T75", "schema": "worktree", "table": "worktree_heatmap", "class": "W", "op": "GET", "scenario": "worktree-heatmap-lookup", "rationale": "派生 MV"},
    {"id": "T76", "schema": "worktree", "table": "worktree_status", "class": "M", "op": "GET", "scenario": "worktree-status-lookup", "rationale": "Lookup enum"},
    # agent (5) - 3 M + 2 T
    {"id": "T77", "schema": "agent", "table": "agent", "class": "M", "op": "GET", "scenario": "agent-lookup", "rationale": "エージェント登録"},
    {"id": "T78", "schema": "agent", "table": "agent_session", "class": "T", "op": "GET", "scenario": "agent-session-lookup", "rationale": "セッション,状態遷移"},
    {"id": "T79", "schema": "agent", "table": "agent_session_event", "class": "T", "op": "POST", "scenario": "agent-session-event-append", "rationale": "Append-only"},
    {"id": "T80", "schema": "agent", "table": "agent_policy", "class": "M", "op": "GET", "scenario": "agent-policy-lookup", "rationale": "ポリシー,構成情報"},
    {"id": "T81", "schema": "agent", "table": "agent_session_status", "class": "M", "op": "GET", "scenario": "agent-session-status-lookup", "rationale": "Lookup enum"},
    # feedback (4) - 1 M + 2 T + 1 W
    {"id": "T82", "schema": "feedback", "table": "feedback", "class": "T", "op": "GET", "scenario": "feedback-lookup", "rationale": "業務事実"},
    {"id": "T83", "schema": "feedback", "table": "feedback_consumed_event", "class": "T", "op": "POST", "scenario": "feedback-consumed-event-append", "rationale": "Append-only"},
    {"id": "T84", "schema": "feedback", "table": "feedback_inbox_item", "class": "W", "op": "GET", "scenario": "feedback-inbox-item-lookup", "rationale": "派生 MV"},
    {"id": "T85", "schema": "feedback", "table": "feedback_status", "class": "M", "op": "GET", "scenario": "feedback-status-lookup", "rationale": "Lookup enum"},
    # context (4) - 1 M + 3 T
    {"id": "T86", "schema": "context", "table": "context_packet", "class": "T", "op": "GET", "scenario": "context-packet-lookup", "rationale": "業務事実"},
    {"id": "T87", "schema": "context", "table": "provenance_entry", "class": "T", "op": "GET", "scenario": "provenance-entry-lookup", "rationale": "系統記録"},
    {"id": "T88", "schema": "context", "table": "decision", "class": "T", "op": "GET", "scenario": "decision-lookup", "rationale": "意思決定"},
    {"id": "T89", "schema": "context", "table": "decision_status", "class": "M", "op": "GET", "scenario": "decision-status-lookup", "rationale": "Lookup enum"},
    # validation (6) - 2 M + 3 T + 1 W
    {"id": "T90", "schema": "validation", "table": "validation_result", "class": "T", "op": "GET", "scenario": "validation-result-lookup", "rationale": "検証結果"},
    {"id": "T91", "schema": "validation", "table": "validation_evidence", "class": "T", "op": "GET", "scenario": "validation-evidence-lookup", "rationale": "検証証拠"},
    {"id": "T92", "schema": "validation", "table": "acceptance_coverage", "class": "T", "op": "GET", "scenario": "acceptance-coverage-lookup", "rationale": "受入カバレッジ"},
    {"id": "T93", "schema": "validation", "table": "validation_policy", "class": "M", "op": "GET", "scenario": "validation-policy-lookup", "rationale": "ポリシー,構成情報"},
    {"id": "T94", "schema": "validation", "table": "acceptance_coverage_report", "class": "W", "op": "GET", "scenario": "acceptance-coverage-report-lookup", "rationale": "派生 MV"},
    {"id": "T95", "schema": "validation", "table": "validation_status", "class": "M", "op": "GET", "scenario": "validation-status-lookup", "rationale": "Lookup enum"},
    # local_runtime (5) - 2 M + 3 T (含 1 T/W 主分类 T)
    {"id": "T96", "schema": "local_runtime", "table": "runtime", "class": "M", "op": "GET", "scenario": "runtime-lookup", "rationale": "ランタイム登録"},
    {"id": "T97", "schema": "local_runtime", "table": "runtime_command", "class": "T", "op": "GET", "scenario": "runtime-command-lookup", "rationale": "コマンド履歴"},
    {"id": "T98", "schema": "local_runtime", "table": "runtime_observation", "class": "T", "op": "POST", "scenario": "runtime-observation-append", "rationale": "Append-only 短 TTL, T/W 混合"},
    {"id": "T99", "schema": "local_runtime", "table": "reconciliation_report", "class": "T", "op": "GET", "scenario": "reconciliation-report-lookup", "rationale": "調整レポート"},
    {"id": "T100", "schema": "local_runtime", "table": "runtime_status", "class": "M", "op": "GET", "scenario": "runtime-status-lookup", "rationale": "Lookup enum"},
]


def build_fixture(table: dict) -> dict:
    """根据 W/T/M 分类生成 fixture。"""
    cls: Literal["M", "T", "W"] = table["class"]
    base = {
        "fixture_version": "v1",
        "module": "db-wtm",
        "class": {"M": "Master", "T": "Transaction", "W": "Work"}[cls],
        "table": table["table"],
        "schema": table["schema"],
        "id": table["id"],
        "method": table["op"],
        "scenario": table["scenario"],
        "description": f"DB {cls} 类: {table['schema']}.{table['table']} (per docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.2 §2 {table['id']})",
        "rationale": table["rationale"],
        "守门": ["#13 a/b/c/d W=物理删除+T=物理删除禁止+M=物理删除禁止+SCD"],
        "classification_locked": True,
        "migration_required_if_changed": True,
    }
    if cls == "M":
        base.update({
            "request": {"table": table["table"], "id": f"{table['schema']}-{table['table']}-001"},
            "response_200": {
                "id": f"{table['schema']}-{table['table']}-001",
                "scd_version": 1,
                "valid_from": "2026-01-01T00:00:00Z",
                "valid_to": None,
                "active": True,
                "rls_13_classes_attached": True,
                "physical_delete_forbidden": True,
            },
            "fixture_assertion": {"wtm_class": "Master", "scd_type": 2, "rls_13_classes": True, "physical_delete_forbidden": True},
        })
    elif cls == "T":
        base.update({
            "request": {"table": table["table"], "id": f"{table['schema']}-{table['table']}-001", "actor": "session-2026-09-05-001"},
            "response_200_201": {
                "id": f"{table['schema']}-{table['table']}-001",
                "append_only_id": 12345,
                "created_at": "2026-09-05T07:00:00Z",
                "actor_session_id": "session-2026-09-05-001",
                "rls_13_classes_attached": True,
                "physical_delete_blocked": True,
                "partition_strategy": "RANGE(created_at) monthly" if table["table"] in {"audit_event", "ai_audit_metadata", "audit_event_outbox", "agent_session_event", "feedback_consumed_event", "runtime_observation"} else None,
            },
            "fixture_assertion": {"wtm_class": "Transaction", "append_only": True, "physical_delete_blocked": True, "rls_13_classes": True},
        })
    elif cls == "W":
        base.update({
            "request": {"table": table["table"], "id": f"{table['schema']}-{table['table']}-001"},
            "response_200": {
                "id": f"{table['schema']}-{table['table']}-001",
                "retention_period_days": 7,
                "ttl_expires_at": "2026-09-12T07:00:00Z",
                "physical_delete_on_expiry": True,
                "rls_13_classes_attached": True,
            },
            "fixture_assertion": {"wtm_class": "Work", "retention_period_days": 7, "physical_delete_on_expiry": True},
        })
    return base


def main():
    """主入口: 100 表生成, 跳过已存在文件 (idempotent)。"""
    M = MOCK_DATA / "master"
    T = MOCK_DATA / "transaction"
    W = MOCK_DATA / "work"
    for d in (M, T, W):
        d.mkdir(parents=True, exist_ok=True)

    created = 0
    skipped = 0
    for table in TABLES:
        cls = {"M": "master", "T": "transaction", "W": "work"}[table["class"]]
        sub = MOCK_DATA / cls
        filename = f"v1--db-wtm--{cls}--{table['table'].replace('_', '-')}--{table['op']}.json"
        path = sub / filename
        if path.exists():
            skipped += 1
            continue
        path.write_text(json.dumps(build_fixture(table), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        created += 1
    print(f"created: {created}, skipped (idempotent): {skipped}, total: {len(TABLES)}")
    print(f"  Master: {sum(1 for t in TABLES if t['class'] == 'M')}")
    print(f"  Transaction: {sum(1 for t in TABLES if t['class'] == 'T')}")
    print(f"  Work: {sum(1 for t in TABLES if t['class'] == 'W')}")


if __name__ == "__main__":
    main()
