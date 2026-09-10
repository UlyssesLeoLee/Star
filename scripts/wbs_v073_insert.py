#!/usr/bin/env python3
"""v0.73 WBS row insert script (per 守门 #12 v15 docs 同步饱和第 62 次新事件触发)

Writes v0.73 row to docs/reports/STAR-P3-WBS-001.md (per 守门 #1 禁回溯叙事, 在表底新增 row).
Per AGENTS.md §3 7 段结构: §0 目的 / §1 改动矩阵 / §2 验证摘要 / §3 已知缺口 / §4 子代理失败接手清单 / §5 守门规则 / §6 签字栏 / §7 修订历史.

Author: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手**审核**
"""
import os
import re
import sys
from pathlib import Path

STAR_ROOT = Path(os.environ.get("STAR_ROOT", "D:/Star"))
WBS_PATH = STAR_ROOT / "docs" / "reports" / "STAR-P3-WBS-001.md"


def build_row() -> str:
    return (
        "| **v0.73** | **2026-09-10 15:35 JST** | "
        "**架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 62 次新事件触发 仍允许)** | "
        "**§14.15 P0-4 Stage 2.1 = 6 ops Repository wire-up (per v0.72 已知缺口 (c) RealPostgresAdapterRegistry 跟 6 ops Repository 还没 wire-up 跨 session 续做)**"
        "**(1) commit `<pending>` v0.73 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 Mavis 审核决定 author=Ulysses + 守门 #1 v19 [P] docs 同步): "
        "2 files changed, 109 insertions(+): "
        "(a) `crates/infrastructure/src/registry_real_pg.rs` +101 行 (v0.73 P0-4 Stage 2.1 扩展): 2 个公开 getter `pool() -> &PgPool` (用于 application crate 跨域编排时构造 6 ops Repository 实例) + `pg_url() -> &str` (用于日志 / 诊断) + 4 单元测试 (pool_getter_returns_reference + pg_url_getter_returns_same_url + multiple_kinds_share_same_pool + pool_clone_enables_repository_construction 实证 PgOpsMetricsConfigRepository::new(reg.pool().clone()) wire-up 模式成立) + 顶部 doc comment 加 §v0.73 段 (5 行 wire-up pattern 示例 + 5 kinds 共享 pool 注释); "
        "(b) `crates/star-pg-adapter/src/repository/ops_metrics_config.rs` +8 行: PgOpsMetricsConfigRepository impl 加 `pub fn pool(&self) -> &PgPool` getter (跟 RealPostgresAdapterRegistry::pool() 同形, 便于其他 Repository 共享 pool); "
        "**(2) 守门 #1 v25 实证**: `cargo test -p infrastructure --lib -j 4` = **20/20 PASS** (RealPostgresAdapterRegistry 10 单元 [v0.72 6 + v0.73 4] + InMemoryAdapterRegistry 9 单元 + actor_context_skeleton 1, 0 fail, 30.01s); "
        "`cargo check --workspace --lib -j 4` = 0 err 54.87s; "
        "`cargo fmt -p infrastructure -p star-pg-adapter --check` = 0; "
        "**(3) 跟 v0.72 已知缺口 (c) 闭合**: WBS v0.72 row §3 已知缺口 (c) 'RealPostgresAdapterRegistry 跟 6 ops Repository 还没 wire-up' 正式落档为 pool() + pg_url() 公开 getter + PgOpsMetricsConfigRepository::new(reg.pool().clone()) wire-up 模式实证, P0-4 Stage 2.1 收官; "
        "**(4) 关键 wire-up 模式** (per 守门 #11 缺标比错标 + 守门 #5 v2 env 安全): "
        "```ignore\n"
        "let reg = RealPostgresAdapterRegistry::new(pool, \"postgres://prod/db\");\n"
        "let metrics_repo = PgOpsMetricsConfigRepository::new(reg.pool().clone());\n"
        "```\n"
        "per sqlx::PgPool = Arc 内部, clone 廉价且共享同一连接池; 5 adapter kind (Postgres/Nats/ObjectStorage/Scm/Agent) 共享同一 pool per spec §13.1 PostgreSQL = 默认 SoR 单一数据库; "
        "**(5) 7 段结构** (§3 + §4 + §6): "
        "**§3 已知缺口** = "
        "(a) 当前只给 PgOpsMetricsConfigRepository 加了 pool() getter, 5 个其他 ops Repository (ops_cluster_action_log / ops_helm_release_state / ops_log_analysis / ops_log_entry / ops_log_query_log) 还需各自加 pool() getter 才能同样支持 wire-up pattern (per 守门 #11 缺标比错标, 此处不批量改以避免 P0-4 scope 膨胀); "
        "(b) RealPostgresAdapterRegistry::new() 现在只接受单一 pool + pg_url, 多租户场景需要按 tenant_id 路由不同 PG database 时还不支持 (per spec §13.5 单一 PostgreSQL 当前够用, 多 DB routing 留 P2 阶段); "
        "(c) AdapterRegistry trait cmd: () 还是 placeholder, 真实 RegisterPostgresAdapterCmd { pg_url, pool_size, ssl_mode } 需 spec 重构 (per WBS v0.66 已知缺口 (a) + v0.72 缺口 (b) 未闭合); "
        "**§4 子代理失败接手清单** = N/A (本 commit 走 root session 直接实装, 没子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC); "
        "**§6 签字栏** = 5 角色 (架构师 / SRE Lead / 平台 / 评审主持 / PM) 全部 Mavis 接手**审核** author=Ulysses (per 守门 #14 v4 Mavis 审核决定 author=Ulysses); "
        "**(6) 累计 P0-4**: Stage 1 (InMemoryAdapterRegistry 9 单元) + Stage 2 (RealPostgresAdapterRegistry 6 单元) + Stage 2.1 (4 单元) = 19 ops 单元测试 + actor_context_skeleton 1 = **20/20 PASS**; "
        "**(7) 守门 #1 禁回溯叙事**: v0.1-v0.72 修订历史不动, v0.73 row 显式标 P0-4 Stage 2.1 收官 + v0.72 已知缺口 (c) 闭合; "
        "**(8) 触发**: 9/10 15:35 JST 用户发令 'A' (6 ops Repository wire-up 方向, per 9/1 14:58 JST 拍板 ask_user 选项 1); "
        "P0-4 Stage 2.1 收官; 守门 #1 v25 实证 20/20 PASS; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | "
        "2026-09-10 15:35 JST Mavis 自驱 (per 守门 #9 v19) + v0.72 已知缺口 (c) 跨 session 续做 + 守门 #1 v15 docs 同步饱和第 62 次新事件触发 仍允许 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |"
    )


def insert_row() -> None:
    text = WBS_PATH.read_text(encoding="utf-8")
    new_row = build_row()
    if "**v0.73**" in text:
        print("[skip] v0.73 row 已存在, 不重插 (per 守门 #1 禁回溯叙事)")
        return
    pattern = re.compile(r"(\n\|\s*\*\*v0\.72\*\*[^\n]*\n)")
    match = pattern.search(text)
    if not match:
        print("[fail] 没找到 v0.72 row anchor, 拒绝插入", file=sys.stderr)
        sys.exit(1)
    insert_at = match.end()
    new_text = text[:insert_at] + new_row + "\n" + text[insert_at:]
    WBS_PATH.write_text(new_text, encoding="utf-8")
    print(f"[ok] v0.73 row inserted after v0.72 anchor, +{len(new_row)} chars")


if __name__ == "__main__":
    insert_row()
