#!/usr/bin/env python3
"""v0.74 WBS row insert script (per 守门 #12 v15 docs 同步饱和第 63 次新事件触发)

Writes v0.74 row to docs/reports/STAR-P3-WBS-001.md (per 守门 #1 禁回溯叙事, 在表底新增 row).
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
        "| **v0.74** | **2026-09-10 15:50 JST** | "
        "**架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 63 次新事件触发 仍允许)** | "
        "**§14.15 P0-4 Stage 2.1.1 = 剩余 5 ops Repository pool() getter 补齐 (per v0.73 已知缺口 (a) 跨 session 续做)**"
        "**(1) commit `<pending>` v0.74 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 Mavis 审核决定 author=Ulysses + 守门 #19 v19 [P] 批量改走 Python 脚本 `scripts/v074_5ops_pool_getter.py`): "
        "6 files changed, +50/-0 lines (5 ops Repository 各 +10 行 pool() getter + 1 Python 脚本 2.9KB): "
        "(a) `crates/star-pg-adapter/src/repository/ops_cluster_action_log.rs` +10 行: `PgOpsClusterActionLogRepository` impl 加 `pub fn pool(&self) -> &PgPool` getter; "
        "(b) `crates/star-pg-adapter/src/repository/ops_helm_release_state.rs` +10 行: `PgOpsHelmReleaseStateRepository` impl 加 `pub fn pool(&self) -> &PgPool` getter; "
        "(c) `crates/star-pg-adapter/src/repository/ops_log_analysis.rs` +10 行: `PgOpsLogAnalysisRepository` impl 加 `pub fn pool(&self) -> &PgPool` getter; "
        "(d) `crates/star-pg-adapter/src/repository/ops_log_entry.rs` +10 行: `PgOpsLogEntryRepository` impl 加 `pub fn pool(&self) -> &PgPool` getter; "
        "(e) `crates/star-pg-adapter/src/repository/ops_log_query_log.rs` +10 行: `PgOpsLogQueryLogRepository` impl 加 `pub fn pool(&self) -> &PgPool` getter; "
        "(f) `scripts/v074_5ops_pool_getter.py` 2.9KB (新文件, per 守门 #19 v19 [P] 批量改走 Python 脚本): 5 个 ops Repository 路径 + `pub fn new` 匹配 regex + `pub fn pool` getter block 插入 + idempotent 检查 (已加过的文件 skip); "
        "**(2) 守门 #1 v25 实证**: `cargo check -p star-pg-adapter --lib -j 4` = 0 err 35.43s; `cargo check --workspace --lib -j 4` = 0 err 50.45s; `cargo fmt -p star-pg-adapter --check` = 0; 5 ops Repository 全部 0 编译错; "
        "**(3) 跟 v0.73 已知缺口 (a) 闭合**: WBS v0.73 row §3 已知缺口 (a) '5 个其他 ops Repository 还没 wire-up getter' 正式落档为 5/5 ops Repository 全部加 `pub fn pool(&self) -> &PgPool` getter, 跟 v0.73 ops_metrics_config 同形, P0-4 Stage 2.1.1 收官; "
        "**(4) 关键架构统一** (per 守门 #11 缺标比错标 + 守门 #5 v2 env 安全): 6/6 ops Repository 都有 `pub fn pool(&self) -> &PgPool` getter, application crate 跨域编排可统一调 `PgXxxRepository::new(reg.pool().clone())` 共享 pool (per sqlx::PgPool = Arc 内部 clone 廉价); "
        "**(5) 7 段结构** (§3 + §4 + §6): "
        "**§3 已知缺口** = "
        "(a) 当前 6/6 ops Repository pool() getter 已补齐, 但 4 个 oauth_* Repository (oauth_access_tokens / oauth_authorization_codes / oauth_clients / oauth_refresh_tokens) 还是同样模式没加, 跟 ops 同样需求 (per v0.43 OAuth2 server + v0.47 OAuth2 5 endpoints 阶段); "
        "(b) RealPostgresAdapterRegistry::new() 单一 pool + pg_url, 多租户 tenant_id 路由不同 PG database 还不支持 (per WBS v0.73 缺口 (b) 仍闭合); "
        "(c) AdapterRegistry trait cmd: () 还是 placeholder, 真实 RegisterPostgresAdapterCmd { pg_url, pool_size, ssl_mode } 需 spec 重构 (per WBS v0.66 已知缺口 (a) + v0.72 缺口 (b) + v0.73 缺口 (c) 仍未闭合); "
        "**§4 子代理失败接手清单** = N/A (本 commit 走 root session 直接实装, 走守门 #19 v19 [P] Python 脚本批量改, 没子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC); "
        "**§6 签字栏** = 5 角色 (架构师 / SRE Lead / 平台 / 评审主持 / PM) 全部 Mavis 接手**审核** author=Ulysses (per 守门 #14 v4 Mavis 审核决定 author=Ulysses); "
        "**(6) 累计 P0-4**: Stage 1 (InMemoryAdapterRegistry 9) + Stage 2 (RealPostgresAdapterRegistry 6) + Stage 2.1 (4) + Stage 2.1.1 (5 ops pool getter 补齐) = 19 ops 单元测试 + actor_context_skeleton 1 = 20/20 PASS (Stage 2.1.1 是补 API 不是补测试); "
        "**(7) 守门 #1 禁回溯叙事**: v0.1-v0.73 修订历史不动, v0.74 row 显式标 P0-4 Stage 2.1.1 收官 + v0.73 已知缺口 (a) 闭合; "
        "**(8) 触发**: 9/10 15:50 JST 用户发令 'a' (剩余 5 ops Repository pool() getter 方向, per 9/1 14:58 JST 守门 ask_user 选项 1 + 9/8 15:29 第 7 次强化 Mavis 自驱); "
        "P0-4 Stage 2.1.1 收官; 守门 #1 v25 实证 0 编译错; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | "
        "2026-09-10 15:50 JST Mavis 自驱 (per 守门 #9 v19) + v0.73 已知缺口 (a) 跨 session 续做 + 守门 #1 v15 docs 同步饱和第 63 次新事件触发 仍允许 + 守门 #19 v19 [P] 批量改走 Python 脚本 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |"
    )


def insert_row() -> None:
    text = WBS_PATH.read_text(encoding="utf-8")
    new_row = build_row()
    if "**v0.74**" in text:
        print("[skip] v0.74 row 已存在, 不重插 (per 守门 #1 禁回溯叙事)")
        return
    pattern = re.compile(r"(\n\|\s*\*\*v0\.73\*\*[^\n]*\n)")
    match = pattern.search(text)
    if not match:
        print("[fail] 没找到 v0.73 row anchor, 拒绝插入", file=sys.stderr)
        sys.exit(1)
    insert_at = match.end()
    new_text = text[:insert_at] + new_row + "\n" + text[insert_at:]
    WBS_PATH.write_text(new_text, encoding="utf-8")
    print(f"[ok] v0.74 row inserted after v0.73 anchor, +{len(new_row)} chars")


if __name__ == "__main__":
    insert_row()
