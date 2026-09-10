#!/usr/bin/env python3
"""v0.72 WBS row insert script (per 守门 #12 v15 docs 同步饱和第 61 次新事件触发)

Writes v0.72 row to docs/reports/STAR-P3-WBS-001.md (per 守门 #1 禁回溯叙事, 在表底新增 row).
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
        "| **v0.72** | **2026-09-10 15:30 JST** | "
        "**架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 61 次新事件触发 仍允许)** | "
        "**§14.15 P0-4 Stage 2 = RealPostgresAdapterRegistry 骨架 (per v0.66 已知缺口 (c) PostgreSQL 真实 adapter (sqlx + star-pg-adapter) 跨 session 续做)**"
        "**(1) commit `8798727` 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 Mavis 审核决定 author=Ulysses + 守门 #1 v19 [P] docs 同步): "
        "5 files changed, 341 insertions(+), 3 deletions(-): "
        "(a) `crates/infrastructure/src/registry_real_pg.rs` 12.2KB (新文件, 跟 InMemoryAdapterRegistry 平行): "
        "RealPostgresAdapterRegistry struct { pool: sqlx::PgPool, pg_url: String, state: Arc<RwLock<HashMap<...>>> } + AdapterRegistry trait impl 5 register_*_adapter + AdapterQuery trait impl 1 list_registered_adapters + 3 helpers (count/list_by_kind/list_by_tenant) + 1 verify_health() 调 star_pg_adapter::healthcheck(&pool) + 6 单元测试; "
        "(b) `crates/infrastructure/src/lib.rs` +9 行: AdapterDescriptor 加 pg_url: Option<String> + registered_at: Option<DateTime<Utc>> 字段 (Option 兼容 InMemoryAdapterRegistry) + pub mod registry_real_pg 暴露; "
        "(c) `crates/infrastructure/src/registry.rs` +14/-3 行: InMemoryAdapterRegistry::register() 配合 AdapterDescriptor 新字段设 pg_url: None + registered_at: None (per backward compat 跟 v0.66 已 commit 不冲突) + v0.72 doc 注释追加 1 段 (5 行); "
        "(d) `crates/infrastructure/Cargo.toml` +7 行: 加 star-pg-adapter + sqlx + chrono + tracing 4 个 dep; "
        "(e) `Cargo.lock` +4 行: star-pg-adapter + sqlx feature 拉入 workspace lock; "
        "**(2) 守门 #1 v25 实证**: `cargo test -p infrastructure --lib -j 4` = **16/16 PASS** (RealPostgresAdapterRegistry 6 单元 + InMemoryAdapterRegistry 9 单元 + actor_context_skeleton 1, 0 fail, 30.01s); "
        "`cargo check --workspace --lib -j 4` = 0 err 0.33s (含 4 预存 unused import warning 不动 per 守门 #1 禁回溯叙事); "
        "`cargo fmt -p infrastructure --check` = 0; "
        "**(3) 跟 v0.66 已知缺口 (c) 闭合**: WBS v0.66 row §3 已知缺口 (c) 'PostgreSQL 真实 adapter (sqlx + star-pg-adapter) 在 v0.67 阶段继续实装' 正式落档为 RealPostgresAdapterRegistry 骨架, P0-4 Stage 2 启动; "
        "**(4) 关键架构差异 (per 守门 #11 缺标比错标)**: "
        "(a) InMemoryAdapterRegistry 永远不连真实 PG, descriptor.pg_url = None + descriptor.registered_at = None; "
        "(b) RealPostgresAdapterRegistry 持有 sqlx::PgPool, descriptor.pg_url = Some(self.pg_url.clone()) + descriptor.registered_at = Some(chrono::Utc::now()); "
        "(c) 单元测试用 PgPool::connect_lazy(\"postgres://invalid\") 不连真 PG (lazy pool 不实际连) + verify_health 走 Err 断言 (per 守门 #11 缺标比错标); "
        "**(5) 7 段结构** (§3 + §4 + §6): "
        "**§3 已知缺口** = "
        "(a) RealPostgresAdapterRegistry::verify_health() 在单元测试中 lazy pool 必返 Err, 真实 PG 健康检查需要 testcontainers-rs 或 k3s-deployable P2 阶段 跑 (per WBS v0.66 已知缺口 (b) InMemoryAdapterRegistry 不连真实 DB 同样适用); "
        "(b) AdapterRegistry trait cmd: () 还是 placeholder, 真实 RegisterPostgresAdapterCmd { pg_url, pool_size, ssl_mode } 需 spec 重构 (per WBS v0.66 已知缺口 (a)); "
        "(c) RealPostgresAdapterRegistry 跟 6 ops Repository (v0.67) 还没有 wire-up, application crate 跨域编排需要从 list_registered_adapters 拿 descriptor.pg_url 自行构造 pool 或 加 pg_pool: PgPool 字段共享; "
        "**§4 子代理失败接手清单** = N/A (本 commit 走 root session 直接实装, 没子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC); "
        "**§6 签字栏** = 5 角色 (架构师 / SRE Lead / 平台 / 评审主持 / PM) 全部 Mavis 接手**审核** author=Ulysses (per 守门 #14 v4 Mavis 审核决定 author=Ulysses); "
        "**(6) 累计 P0-4**: Stage 1 (InMemoryAdapterRegistry 9 单元) + Stage 2 (RealPostgresAdapterRegistry 6 单元) = 15 单元测试 + actor_context_skeleton 1 = **16/16 PASS**; "
        "**(7) 守门 #1 禁回溯叙事**: v0.1-v0.71 修订历史不动, v0.72 row 显式标 P0-4 Stage 2 启动 + v0.66 已知缺口 (c) 闭合; "
        "**(8) 触发**: 9/10 15:25 JST 用户发令 '继续推' (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 守门 #14 v4 Mavis 审核决定); "
        "P0-4 Stage 2 骨架落地; 守门 #1 v25 实证 16/16 PASS; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | "
        "2026-09-10 15:25 JST Mavis 自驱 (per 守门 #9 v19) + v0.66 已知缺口 (c) 跨 session 续做 + 守门 #1 v15 docs 同步饱和第 61 次新事件触发 仍允许 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |"
    )


def insert_row() -> None:
    text = WBS_PATH.read_text(encoding="utf-8")
    new_row = build_row()
    if "**v0.72**" in text:
        print("[skip] v0.72 row 已存在, 不重插 (per 守门 #1 禁回溯叙事)")
        return
    pattern = re.compile(r"(\n\|\s*\*\*v0\.71\*\*[^\n]*\n)")
    match = pattern.search(text)
    if not match:
        print("[fail] 没找到 v0.71 row anchor, 拒绝插入", file=sys.stderr)
        sys.exit(1)
    insert_at = match.end()
    new_text = text[:insert_at] + new_row + "\n" + text[insert_at:]
    WBS_PATH.write_text(new_text, encoding="utf-8")
    print(f"[ok] v0.72 row inserted after v0.71 anchor, +{len(new_row)} chars")


if __name__ == "__main__":
    insert_row()
