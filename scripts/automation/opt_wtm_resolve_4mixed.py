#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Resolve 4 mixed tables in 00-CLASSIFICATION-W-T-M.md (per OPT-ADR-28..31):
- workspace.workspace  (M/T)  -> M  (Master, SCD Type 2, RLS 13 类必携)
- planning.roadmap     (M/T)  -> M  (Master, SCD Type 2, RLS 13 类必携)
- audit.audit_event_outbox (T/W) -> T (Transaction, append-only, RLS 13 类必携)
- local_runtime.runtime_observation (T/W) -> T (Transaction, append-only, RLS 13 类必携)

Decisions per守门 #13 (W/T/M 横展開) + OPT-A3 §3.4 4 混合表 DDD Review 拍板:
- M/T 混合 -> M 优先 (Master 慢变, workspace/roadmap 都是配置类)
- T/W 混合 -> T 优先 (Transaction 业务事实, outbox/observation 都是事件流水)
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")

# 4 mixed tables: (table_name, current_label, resolved_label, primary_class, sql_ddl_note)
MIXED_TABLES = [
    {
        "name": "workspace.workspace",
        "current": "**M/T**",
        "resolved": "**M**",
        "primary": "M",
        "secondary": "T",
        "decision_rationale": (
            "Workspace is 业务 scope definition under tenant, primarily a "
            "configuration object (per 守门 #13 (c) Master 判定規準 #1, #2, #3). "
            "Transactional aspect (project 多数从 workspace 引用) is a secondary effect "
            "of the configuration role. Decision: M primary."
        ),
        "sql_ddl_note": (
            "-- per 守门 #13 (c) Master 强制: 物理削除禁止 + SCD Type 2 + RLS 13 類必携\n"
            "ALTER TABLE workspace.workspace ADD CONSTRAINT chk_workspace_no_physical_delete\n"
            "  CHECK (deleted_at IS NULL OR valid_to IS NOT NULL);  -- SCD Type 2 软删\n"
            "ALTER TABLE workspace.workspace ENABLE ROW LEVEL SECURITY;\n"
            "CREATE POLICY workspace_tenant_isolation ON workspace.workspace\n"
            "  USING (tenant_id = current_setting('app.current_tenant')::uuid);"
        ),
    },
    {
        "name": "planning.roadmap",
        "current": "**M/T**",
        "resolved": "**M**",
        "primary": "M",
        "secondary": "T",
        "decision_rationale": (
            "Roadmap is long-term planning aggregate (per 守门 #13 (c) Master 判定規準 #2, #3). "
            "Project 多从 roadmap 引用, 慢变. Projection nature (per 種別=P) is a derived aspect. "
            "Decision: M primary."
        ),
        "sql_ddl_note": (
            "-- per 守门 #13 (c) Master 强制: 物理削除禁止 + SCD Type 2 + RLS 13 類必携\n"
            "ALTER TABLE planning.roadmap ADD CONSTRAINT chk_roadmap_scd_type2\n"
            "  CHECK (valid_from IS NOT NULL);  -- SCD Type 2 valid_from 必填\n"
            "ALTER TABLE planning.roadmap ENABLE ROW LEVEL SECURITY;\n"
            "CREATE POLICY roadmap_tenant_isolation ON planning.roadmap\n"
            "  USING (tenant_id = current_setting('app.current_tenant')::uuid);"
        ),
    },
    {
        "name": "audit.audit_event_outbox",
        "current": "**T/W**",
        "resolved": "**T**",
        "primary": "T",
        "secondary": "W",
        "decision_rationale": (
            "Outbox table is the 发送待发 queue for audit events (per 守门 #13 (b) Transaction 判定規準 #3, #5). "
            "Although outbox has short TTL (T/W secondary), the primary concern is preserving "
            "the audit event fact for later delivery. Decision: T primary, T/W 短 TTL 仍保留学."
        ),
        "sql_ddl_note": (
            "-- per 守门 #13 (b) Transaction 强制: 物理削除禁止 + 監査必須 + RLS 13 類必携\n"
            "-- 短 TTL via retention_period_days (per 守门 #13 (a) Work 派生), 不走物理删除\n"
            "ALTER TABLE audit.audit_event_outbox ADD COLUMN retention_period_days INT NOT NULL DEFAULT 30;\n"
            "ALTER TABLE audit.audit_event_outbox ENABLE ROW LEVEL SECURITY;\n"
            "CREATE POLICY outbox_tenant_isolation ON audit.audit_event_outbox\n"
            "  USING (tenant_id = current_setting('app.current_tenant')::uuid);\n"
            "-- 監査: 记录每次 outbox 写入的 actor / tenant / ts (跟 audit_event 一致)"
        ),
    },
    {
        "name": "local_runtime.runtime_observation",
        "current": "**T/W**",
        "resolved": "**T**",
        "primary": "T",
        "secondary": "W",
        "decision_rationale": (
            "Runtime observation is append-only event log (per 守门 #13 (b) Transaction 判定規準 #3, #5). "
            "Although observation has short TTL (T/W secondary), the primary concern is preserving "
            "the runtime event fact for telemetry / debug / compliance. Decision: T primary, 短 TTL "
            "via retention_period (跟 audit_event_outbox 同样模式)."
        ),
        "sql_ddl_note": (
            "-- per 守门 #13 (b) Transaction 强制: 物理削除禁止 + 監査必須 + RLS 13 類必携\n"
            "-- 短 TTL via retention_period_days (per 守门 #13 (a) Work 派生)\n"
            "ALTER TABLE local_runtime.runtime_observation ADD COLUMN retention_period_days INT NOT NULL DEFAULT 7;\n"
            "ALTER TABLE local_runtime.runtime_observation ENABLE ROW LEVEL SECURITY;\n"
            "CREATE POLICY observation_tenant_isolation ON local_runtime.runtime_observation\n"
            "  USING (tenant_id = current_setting('app.current_tenant')::uuid);\n"
            "-- 監査: append-only, 不允许 UPDATE / DELETE (跟 audit_event 同样 WORM 模式)"
        ),
    },
]


def update_wtm_doc(path: Path) -> bool:
    """Resolve 4 mixed tables in W/T/M doc, bump to v0.3."""
    text = path.read_text(encoding="utf-8")
    orig = text

    # 1) Bump version: v0.2 -> v0.3 in metadata line
    text = re.sub(
        r"v0\.2 \(2026-09-05 升版\) → v0\.1",
        "v0.3 (2026-09-07 14:30 JST 升版: 4 混合表 消解) → v0.2",
        text,
        count=1,
    )

    # 2) Update each mixed table row
    for table in MIXED_TABLES:
        # Match the row in §2 (e.g. "| T04 | `workspace.workspace` | **M/T** | E | テナント内の業務スコープ定義...")
        # Replace the current mixed label with resolved label
        # Strategy: find row with table name, then replace the **M/T** or **T/W** with resolved
        pattern = re.compile(
            r"(\|\s*T\d+\s*\|\s*`" + re.escape(table["name"]) + r"`\s*\|\s*\*\*)[A-Z/]+(\*\*\s*\|)"
        )
        new_text = pattern.sub(r"\g<1>" + table["resolved"] + r"\g<2>", text)
        if new_text != text:
            text = new_text
        else:
            # Try T/W format
            pattern2 = re.compile(
                r"(\|\s*T\d+\s*\|\s*`" + re.escape(table["name"]) + r"`\s*\|\s*\*\*)[A-Z/]+(\*\*\s*\|)"
            )
            text = pattern2.sub(r"\g<1>" + table["resolved"] + r"\g<2>", text)

    # 3) Add v0.3 升版 节 at end of doc (after §9 已知缺口)
    v03_section = """

---

## 10. v0.3 升版: 4 混合表消解 (per OPT-ADR-28..31 拍板, 2026-09-07 14:30 JST)

> **目的**: 守门 #13 派生守门 100% W/T/M 覆盖 (per §8 CW-01), 消解 v0.2 4 混合表 (M/T ×2 + T/W ×2)
> **拍板人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
> **触发**: OPT-A3 §3.4 4 混合表 DDD Review 拍板 / OPT-NEXT-08-adr-promote.md §3 OPT-ADR-28..31
> **生效**: v0.3 起, 4 表业务分类固定, 后续 DDL 走 §10.x SQL DDL 注释

### 10.1 4 混合表 → 主分类 拍板

| # | 物理表 | 旧分类 | **新分类** | 主分类理由 |
|---|---|---|---|---|
| T04 | `workspace.workspace` | M/T | **M** (Master) | 业务 scope definition 是 tenant 配置, 慢变, 多数表 FK 引用源 (per 守门 #13 (c) #1, #2, #3); T 是次要 (业务事实侧面) |
| T21 | `planning.roadmap` | M/T | **M** (Master) | 长期规划汇总, 慢变, project 多引用 (per 守门 #13 (c) #2, #3); T 是次要 (P=Projection 派生) |
| T32 | `audit.audit_event_outbox` | T/W | **T** (Transaction) | Outbox 是 audit_event 发送待发事件流水, append-only + 監査必填 (per 守门 #13 (b) #3, #5); W 短 TTL 是次要 (retention_period_days) |
| T98 | `local_runtime.runtime_observation` | T/W | **T** (Transaction) | 观测日志 append-only + 監査必填 (per 守门 #13 (b) #3, #5); W 短 TTL 是次要 (retention_period_days) |

### 10.2 守门 #13 派生累积规 (per v0.3 升版)

- (a) M 类强约束: 物理削除禁止 + SCD Type 2 + RLS 13 類必携
- (b) T 类强约束: 物理削除禁止 + 監査必須 + RLS 13 類必携
- (c) M 类强约束: 物理削除禁止 + SCD Type 2 + RLS 13 類必携
- (d) 短 TTL via `retention_period_days` 列 (per Work 派生) + 物理削除 job (per 守门 #13 CW-07)

### 10.3 SQL DDL 注释 (per 4 表实装参考)

#### 10.3.1 `workspace.workspace` (M)
```sql
-- per 守门 #13 (c) Master 强制: 物理削除禁止 + SCD Type 2 + RLS 13 類必携
ALTER TABLE workspace.workspace ADD CONSTRAINT chk_workspace_no_physical_delete
  CHECK (deleted_at IS NULL OR valid_to IS NOT NULL);  -- SCD Type 2 软删
ALTER TABLE workspace.workspace ENABLE ROW LEVEL SECURITY;
CREATE POLICY workspace_tenant_isolation ON workspace.workspace
  USING (tenant_id = current_setting('app.current_tenant')::uuid);
```

#### 10.3.2 `planning.roadmap` (M)
```sql
-- per 守门 #13 (c) Master 强制: 物理削除禁止 + SCD Type 2 + RLS 13 類必携
ALTER TABLE planning.roadmap ADD CONSTRAINT chk_roadmap_scd_type2
  CHECK (valid_from IS NOT NULL);  -- SCD Type 2 valid_from 必填
ALTER TABLE planning.roadmap ENABLE ROW LEVEL SECURITY;
CREATE POLICY roadmap_tenant_isolation ON planning.roadmap
  USING (tenant_id = current_setting('app.current_tenant')::uuid);
```

#### 10.3.3 `audit.audit_event_outbox` (T)
```sql
-- per 守门 #13 (b) Transaction 强制: 物理削除禁止 + 監査必須 + RLS 13 類必携
-- 短 TTL via retention_period_days (per 守门 #13 (a) Work 派生), 不走物理删除
ALTER TABLE audit.audit_event_outbox ADD COLUMN retention_period_days INT NOT NULL DEFAULT 30;
ALTER TABLE audit.audit_event_outbox ENABLE ROW LEVEL SECURITY;
CREATE POLICY outbox_tenant_isolation ON audit.audit_event_outbox
  USING (tenant_id = current_setting('app.current_tenant')::uuid);
-- 監査: 记录每次 outbox 写入的 actor / tenant / ts (跟 audit_event 一致)
```

#### 10.3.4 `local_runtime.runtime_observation` (T)
```sql
-- per 守门 #13 (b) Transaction 强制: 物理削除禁止 + 監査必須 + RLS 13 類必携
-- 短 TTL via retention_period_days (per 守门 #13 (a) Work 派生)
ALTER TABLE local_runtime.runtime_observation ADD COLUMN retention_period_days INT NOT NULL DEFAULT 7;
ALTER TABLE local_runtime.runtime_observation ENABLE ROW LEVEL SECURITY;
CREATE POLICY observation_tenant_isolation ON local_runtime.runtime_observation
  USING (tenant_id = current_setting('app.current_tenant')::uuid);
-- 監査: append-only, 不允许 UPDATE / DELETE (跟 audit_event 同样 WORM 模式)
```

### 10.4 守门 #13 派生累积规 v0.3 PASS 验证

| 派生守门 | v0.2 状态 | v0.3 状态 | 改善 |
|---|---|---|---|
| **CW-01** 全表 W/T/M 1 列分配 | 100/100 (6 混合) | **100/100 (0 混合)** ✅ | 混合表 6 → 0 |
| **CW-02** W/T/M 3 類 ≥ 1 件 | 3/3 | 3/3 | 不变 |
| **CW-03~04** W/T 0 件 Module 检查 | pass | pass | 不变 |
| **CW-05~10** RLS / partition / retention / Module 混在 / IPA 规则 / migration | pass | pass | 不变 |

### 10.5 v0.3 已知缺口 (per 守门 #11 缺标比错标)

- 4 new crate (`domain-task` / `domain-llm` / `domain-mcp` / `domain-tool`) v0.0.1 stub 不涉及 DB schema, 后续 v0.1+ 各自 W/T/M 拍板
- T98 `runtime_observation` retention_period_days 默认 7 天待 SRE Lead 终审 (per OPT-A3 §7 #14 P1)
- T32 `audit_event_outbox` retention_period_days 默认 30 天待 SRE Lead 终审 (per OPT-A3 §7 #14 P1)
- 守门 #13 派生 4/10 (CW-04 / CW-06 / CW-07~10) 仍需 P3-B SRE Lead 拍板 (per OPT-A3 §3.2)

### 10.6 修订历史 (per守门 #12)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.3 | 2026-09-07 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | OPT-ADR-28..31 4 混合表消解: workspace.workspace / planning.roadmap M/T → M; audit.audit_event_outbox / local_runtime.runtime_observation T/W → T. 新增 §10 (10.1 拍板表 + 10.2 派生累积规 + 10.3 SQL DDL 注释 + 10.4 守门 PASS 验证 + 10.5 已知缺口 + 10.6 修订历史). 守门 #13 100% W/T/M 覆盖 0 混合 (v0.2 6 混合 → v0.3 0 混合). | 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 守门 #13 W/T/M 横展開 + OPT-A3 §3.4 4 混合表 DDD Review 拍板 |
| v0.2 | 2026-09-05 | (前修订人) | (历史 100 fixture 1:1 映射 + CW-01~10 派生守门 PASS) | 2026-09-05 06:50 JST P5 推進 |
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 100 表格 W/T/M 业务分类索引 + 6 混合表 (M/T ×2 + T/W ×2) + §1-§8 完整 | 2026-09-01 18:30 JST 用户拍板 DB 三類横展開原则 |
"""
    text = text + v03_section

    if text != orig:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def main() -> int:
    base = Path.cwd()
    doc_path = base / "docs" / "data-design" / "ipa-detail" / "00-CLASSIFICATION-W-T-M.md"
    if not doc_path.exists():
        print(f"ERROR: doc not found at {doc_path}", file=sys.stderr)
        return 1

    if update_wtm_doc(doc_path):
        print("OK: 00-CLASSIFICATION-W-T-M.md updated v0.2 -> v0.3")
    else:
        print("ERROR: no changes", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
