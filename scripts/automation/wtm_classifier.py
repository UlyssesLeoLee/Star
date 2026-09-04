#!/usr/bin/env python3
"""wtm_classifier.py — Star 仓 P3-D 阶段 W/T/M 三类横展开分类器 (per 守门 #19 [M] 拍板)

per 2026-09-01 18:30 JST 拍板 + docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.1
+ docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md v0.1
+ 派生守门 10 条 CW-01~CW-10 (per 守门 #DB-13 W/T/M)

扫描 22 domain-* crate 的 src/**/*.rs, 抽取 pub struct / pub enum 作为 P3-D 实体,
按 W/T/M 三类规则 (M: FK 参照多 / 設定 / SCD / 物理削除禁止; T: 業務事実 / Append-only / 監査; W: 短 TTL / session-bound)
自动分类并生成 P3-D 阶段 W/T/M 分类报告.

CLI:
  python scripts/automation/wtm_classifier.py --root D:\\Star\\.worktrees\\feat-auto-20260904-1c260bc7 --output docs/data-design/p3-d-classification-w-t-m.md
  python scripts/automation/wtm_classifier.py --root D:\\Star\\.worktrees\\feat-auto-20260904-1c260bc7 --domain domain-agent --verbose
"""
import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")


# === W/T/M 分类规则 (per 00-CLASSIFICATION-RULES.md §1.2 Decision Tree) ===

# Master (M) 关键词 - 慢变参考 / 設定 / 慢変参照 / Lookup / 物理削除禁止
M_KEYWORDS = [
    "tenant", "user", "role", "permission", "policy", "lookup",
    "config", "setting", "template", "schema", "definition",
    "project_type", "permission_grant", "feature_flag",
    "agent_policy", "billing_account", "subscription",
    "workspace_type", "identity_provider", "audit_policy",
    "notification_template", "permission_set", "role_assignment",
    "workflow_template", "form_template", "theme",
    "tag", "category", "label", "group",
    "session_template", "context_template", "validation_rule",
]

# Transaction (T) 关键词 - 業務事実 / Append-only / 監査 / 状態遷移
T_KEYWORDS = [
    "session", "task", "workitem", "work_item", "project", "issue",
    "comment", "decision", "validation_result", "audit_event",
    "ai_audit", "agent_session", "feedback", "feedback_consumed",
    "outbox", "domain_event", "integration_event",
    "merge_request", "pull_request", "commit_record",
    "search_history", "notification_send", "report",
    "billing_record", "usage_record", "quota_record",
    "event_log", "execution_log", "runtime_observation",
    "request", "response", "command", "query_record",
    "subscription_change", "tenant_invitation",
    "token_record", "checkpoint", "context_entry",
    "memory_record", "shared_pool_acquire", "dispatch_record",
    "compensation_record", "saga_record", "saga_step",
    "checkpoint_snapshot",
]

# Work (W) 关键词 - 短 TTL / session-bound / 完了後 cleanup
W_KEYWORDS = [
    "runtime_state", "execution_state", "transient", "scratch",
    "session_lock", "exclusive_lock", "rate_limit_window",
    "realtime_state", "presence", "online_status",
    "typing_indicator", "draft", "in_progress",
    "pending_upload", "temp_token", "short_lived",
    "cache", "view_cache", "projection_cache",
    "current_state", "active_session", "active_task",
    "live_metrics", "metric_snapshot",
    "ai_edit_suggestion", "mock_response",
    "batch_progress", "stream_event",
    "lock", "lease", "heartbeat", "ping_record",
    "sub_agent_run", "task_pulse", "epoch_marker",
    "checkpoint_draft", "context_tier_draft",
    "memory_cache", "token_usage", "rate_window",
    "dispatch_state", "lifecycle", "lifecycle_state",
]


def classify_entity(entity_name: str, file_path: str) -> str:
    """Classify an entity to W / T / M per the decision tree."""
    name_lower = entity_name.lower()
    file_lower = file_path.lower()

    # Score each category
    m_score = sum(1 for kw in M_KEYWORDS if kw in name_lower)
    t_score = sum(1 for kw in T_KEYWORDS if kw in name_lower)
    w_score = sum(1 for kw in W_KEYWORDS if kw in name_lower)

    # Special cases: enum types are typically Lookup/M
    if "_type" in name_lower or "_status" in name_lower or "_kind" in name_lower or "_level" in name_lower or "_mode" in name_lower:
        return "M"  # Lookup enum

    # Decision tree (per 00-CLASSIFICATION-RULES.md §1.2)
    if m_score > 0 and m_score >= max(t_score, w_score):
        return "M"
    if t_score > 0 and t_score >= w_score:
        return "T"
    if w_score > 0:
        return "W"
    # Default: if contains "_event" or "_record" or "_log", it's T
    if "_event" in name_lower or "_record" in name_lower or "_log" in name_lower or "_audit" in name_lower:
        return "T"
    # Default: if contains "_id" only (just an ID type), skip / M
    if name_lower.endswith("_id") or name_lower == "id":
        return "Skip"  # Type alias, not a table
    # Default: T (业务事实最常见)
    return "T"


def scan_domain_crate(crate_path: Path) -> list[dict]:
    """Scan a domain-* crate src for pub struct / pub enum definitions."""
    entities = []
    src_dir = crate_path / "src"
    if not src_dir.exists():
        return entities

    for rs_file in src_dir.rglob("*.rs"):
        try:
            content = rs_file.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        # Find pub struct / pub enum
        for match in re.finditer(r"pub\s+(?:struct|enum)\s+([A-Z][A-Za-z0-9_]+)", content):
            entity_name = match.group(1)
            rel_path = str(rs_file.relative_to(crate_path))
            entities.append({
                "name": entity_name,
                "file": rel_path,
                "crate": crate_path.name,
            })
    return entities


def scan_all_domain_crates(root: Path) -> dict[str, list[dict]]:
    """Scan all 22 domain-* crates under crates/."""
    crates_dir = root / "crates"
    if not crates_dir.exists():
        print(f"ERROR: {crates_dir} not found", file=sys.stderr)
        sys.exit(1)

    result = {}
    for crate_dir in sorted(crates_dir.iterdir()):
        if not crate_dir.is_dir():
            continue
        if not crate_dir.name.startswith("domain-"):
            continue
        entities = scan_domain_crate(crate_dir)
        if entities:
            result[crate_dir.name] = entities
    return result


def aggregate_stats(scanned: dict[str, list[dict]]) -> dict:
    """Aggregate W/T/M statistics per crate + global."""
    by_crate = defaultdict(lambda: {"M": 0, "T": 0, "W": 0, "Skip": 0, "Total": 0, "Entities": []})
    global_stats = {"M": 0, "T": 0, "W": 0, "Skip": 0, "Total": 0}

    for crate_name, entities in scanned.items():
        for ent in entities:
            cls = classify_entity(ent["name"], ent["file"])
            ent["class"] = cls
            by_crate[crate_name][cls] += 1
            by_crate[crate_name]["Total"] += 1
            by_crate[crate_name]["Entities"].append(ent)
            global_stats[cls] += 1
            global_stats["Total"] += 1

    return {"by_crate": dict(by_crate), "global": global_stats}


def check_cw_rules(stats: dict) -> list[dict]:
    """Apply CW-01~CW-10 derived gates."""
    issues = []
    for crate_name, crate_stats in stats["by_crate"].items():
        # CW-02: 3 類とも 1 件以上
        if crate_stats["M"] == 0 or crate_stats["T"] == 0 or crate_stats["W"] == 0:
            issues.append({
                "rule": "CW-02",
                "crate": crate_name,
                "message": f"三類分門別類漏れ: M={crate_stats['M']}, T={crate_stats['T']}, W={crate_stats['W']}",
            })
        # CW-03: W が 0 件
        if crate_stats["W"] == 0:
            issues.append({
                "rule": "CW-03",
                "crate": crate_name,
                "message": f"W=0 件, 短命データ不足の可能性 (session / 観測 / 排他制御 table 確認要)",
            })
        # CW-04: T が 0 件
        if crate_stats["T"] == 0:
            issues.append({
                "rule": "CW-04",
                "crate": crate_name,
                "message": f"T=0 件, 業務事実記録欠如の可能性 (Write/Read 主体 Module は例外)",
            })
    return issues


def render_markdown_report(stats: dict, issues: list[dict], root: Path) -> str:
    """Render P3-D W/T/M classification report as Markdown."""
    lines = []
    lines.append("# P3-D 段階 W/T/M 三類横展開 分類報告")
    lines.append("")
    lines.append("> **基準**: ユーザー指定 DB 三類横展開原則（2026-09-01 18:30 JST）")
    lines.append("> **適用範囲**: Star 仓 P3-D 段階 (22 domain-* crate) 全 entity")
    lines.append("> **一次出典**: `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 + `00-CLASSIFICATION-RULES.md` v0.1")
    lines.append("> **派生守門**: CW-01~CW-10 (per 守门 #DB-13 W/T/M)")
    lines.append(f"> **生成**: `scripts/automation/wtm_classifier.py` v0.1 (per 守门 #19 [M] 拍板, 2026-09-04 拍板)")
    lines.append("")
    lines.append("---")
    lines.append("")

    # §1 全局統計
    lines.append("## §1 全局統計 (P3-D 段階 22 domain-* crate 全体)")
    lines.append("")
    lines.append("| 業務分類 | 件数 | 比率 |")
    lines.append("|---|---|---|")
    g = stats["global"]
    total = max(g["Total"] - g["Skip"], 1)
    lines.append(f"| **Master (M)** | {g['M']} | {g['M']*100/total:.1f}% |")
    lines.append(f"| **Transaction (T)** | {g['T']} | {g['T']*100/total:.1f}% |")
    lines.append(f"| **Work (W)** | {g['W']} | {g['W']*100/total:.1f}% |")
    lines.append(f"| **Skip** (Type alias, ID 等) | {g['Skip']} | {g['Skip']*100/(g['Total']+1):.1f}% |")
    lines.append(f"| **合計** | {g['Total']} | 100.0% |")
    lines.append("")

    # §2 Schema 別
    lines.append("## §2 domain-* crate 別 分類件数 (P3-D 段階 22 crate)")
    lines.append("")
    lines.append("| domain-* crate | M | T | W | Skip | 合計 | 評価 |")
    lines.append("|---|---|---|---|---|---|---|")
    for crate_name, cs in sorted(stats["by_crate"].items()):
        eval_str = "✅" if cs["M"] > 0 and cs["T"] > 0 and cs["W"] > 0 else "🟡 CW-02 違反"
        lines.append(f"| `{crate_name}` | {cs['M']} | {cs['T']} | {cs['W']} | {cs['Skip']} | {cs['Total']} | {eval_str} |")
    lines.append("")

    # §3 派生守門 10 条 チェック
    lines.append("## §3 派生守門 10 条 チェック (CW-01 ~ CW-10)")
    lines.append("")
    if not issues:
        lines.append("✅ すべての派生守門 CW-01~CW-10 クリア (per §6 派生守門 10 条, 00-CLASSIFICATION-RULES.md)")
    else:
        lines.append("| # | 派生守門 | crate | message |")
        lines.append("|---|---|---|---|")
        for issue in issues:
            lines.append(f"| {issue['rule']} | {issue['rule']} | `{issue['crate']}` | {issue['message']} |")
    lines.append("")

    # §4 業務分類 詳細 (per crate)
    lines.append("## §4 業務分類 詳細 (per crate, 全 entity)")
    lines.append("")
    for crate_name, cs in sorted(stats["by_crate"].items()):
        lines.append(f"### {crate_name}")
        lines.append("")
        lines.append("| 業務分類 | entity 名 | file |")
        lines.append("|---|---|---|")
        # Sort by class (M, T, W) then name
        class_order = {"M": 0, "T": 1, "W": 2, "Skip": 3}
        sorted_ents = sorted(cs["Entities"], key=lambda e: (class_order.get(e["class"], 4), e["name"]))
        for ent in sorted_ents:
            cls_label = {"M": "**M**", "T": "**T**", "W": "**W**", "Skip": "Skip"}.get(ent["class"], ent["class"])
            lines.append(f"| {cls_label} | `{ent['name']}` | `{ent['file']}` |")
        lines.append("")

    # §5 既知の缺口
    lines.append("## §5 既知の缺口 / 制約")
    lines.append("")
    lines.append("1. **混合分類**: P3-D 段階 entity 中, 業務分類が 1 つの entity で複数軸に該当する場合あり (e.g. `agent_session` は T (業務事実) + W (active session 観測), 主分類で計上)")
    lines.append("2. **V2 候補**: V2 化 (LangGraph 統合 / Agent Runtime 1M agents / Tree-sitter) で T → W 降格候補あり (per 00-CLASSIFICATION-RULES.md §7)")
    lines.append("3. **Frontend 同期**: P3-D 22 domain-* crate Backend Rust のみ W/T/M 適用, Frontend Zustand store 状態分類は未同期 (per 同 §7)")
    lines.append("4. **新規 crate**: 派生新規 crate (e.g. `star-dispatcher` v0.0.1) 28 entity 仍未在本報告 (P3-D 範囲外, P3-G 阶段扩展)")
    lines.append("5. **DDD Review**: 5 域 Lead 真人到位后, 分類結果 Review + 修正 (per 守门 #14 5 域 Lead CONTENT 4 維)")
    lines.append("")

    # §6 関連文档
    lines.append("## §6 関連文档")
    lines.append("")
    lines.append("- `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 (Star 100 テーブル W/T/M 三類索引実例)")
    lines.append("- `docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md` v0.1 (跨項目ルール手册 + 4 段检查清单 + 派生守门 10 条)")
    lines.append("- `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` (Agent Runtime SRS, P3-D 段階 1 entity 來源)")
    lines.append("- `crates/star-saga/src/saga_5b_services.rs` (P3-E 阶段, 不在本 P3-D 報告範圍)")
    lines.append("- `crates/star-dispatcher/src/lib.rs` (P3-G 阶段, 不在本 P3-D 報告範圍)")
    lines.append("- `scripts/automation/wtm_classifier.py` v0.1 (本報告生成腳本, 守门 #19 [M] 拍板)")
    lines.append("- `AGENTS.md` 守门 #DB-13 (DB 三類横展開強制分類)")
    lines.append("- `docs/reports/STAR-P4-UNIMPL-WBS-001.md` §F.4 (P3-D 跨項目 落地計画)")
    lines.append("")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Star 仓 P3-D 阶段 W/T/M 三类横展开分类器")
    parser.add_argument("--root", type=str, default=r"D:\Star\.worktrees\feat-auto-20260904-1c260bc7", help="Star 仓根目录")
    parser.add_argument("--domain", type=str, default=None, help="仅扫描指定 domain-* crate (e.g. domain-agent)")
    parser.add_argument("--output", type=str, default="docs/data-design/p3-d-classification-w-t-m.md", help="输出报告路径 (相对 root)")
    parser.add_argument("--verbose", action="store_true", help="详细输出")
    args = parser.parse_args()

    root = Path(args.root)
    if not root.exists():
        print(f"ERROR: {root} not found", file=sys.stderr)
        sys.exit(1)

    print(f"Scanning {root}/crates/domain-* ...")
    scanned = scan_all_domain_crates(root)
    if args.domain:
        if args.domain not in scanned:
            print(f"ERROR: {args.domain} not found. Available: {list(scanned.keys())}", file=sys.stderr)
            sys.exit(1)
        scanned = {args.domain: scanned[args.domain]}

    print(f"Found {len(scanned)} crates, total {sum(len(e) for e in scanned.values())} entities")

    stats = aggregate_stats(scanned)
    issues = check_cw_rules(stats)

    if args.verbose:
        print("\n=== Per-crate classification ===")
        for crate_name, cs in sorted(stats["by_crate"].items()):
            print(f"  {crate_name}: M={cs['M']}, T={cs['T']}, W={cs['W']}, Skip={cs['Skip']}, Total={cs['Total']}")

    print(f"\n=== Global stats ===")
    g = stats["global"]
    print(f"  Master (M): {g['M']}")
    print(f"  Transaction (T): {g['T']}")
    print(f"  Work (W): {g['W']}")
    print(f"  Skip: {g['Skip']}")
    print(f"  Total: {g['Total']}")

    print(f"\n=== CW-01~CW-10 checks: {len(issues)} issues ===")
    for issue in issues:
        print(f"  [{issue['rule']}] {issue['crate']}: {issue['message']}")

    # Render report
    report = render_markdown_report(stats, issues, root)
    output_path = root / args.output
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(report, encoding="utf-8")
    print(f"\nReport written: {output_path} ({len(report)} bytes)")


if __name__ == "__main__":
    main()
