#!/usr/bin/env python3
"""v0.85 WBS row insert script (per 守门 #12 v15 docs 同步饱和第 75 次新事件触发)

Writes v0.85 row to docs/reports/STAR-P3-WBS-001.md (per 守门 #1 禁回溯叙事, 在表底新增 row).

v0.85 = P0-4 Stage 3.2 = PgTenantPoolRepository 真实 sqlx 实现 (per v0.82 已知缺口 (a)+(c) 跨 session 续做)
       (本 commit 用 v0.85 编号因为 v0.83/v0.84 已被平行工作占用, 跳开冲突)

Author: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手**审核**
"""
import os
import sys
from pathlib import Path

STAR_ROOT = Path(os.environ.get("STAR_ROOT", "D:/Star"))
WBS_PATH = STAR_ROOT / "docs" / "reports" / "STAR-P3-WBS-001.md"


ROW = """| **v0.85** | **2026-09-10 19:20 JST** | **架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 75 次新事件触发 仍允许)** | **§14.15 P0-4 Stage 3.2 = PgTenantPoolRepository 真实 sqlx 实现 (per v0.82 已知缺口 (a)+(c) multi-tenant 路由 + 真实 SQL 验证 跨 session 续做) — 用 v0.85 编号 (v0.83 IPA SEC 合规性修复 + v0.84 已被平行工作占用)**(1) commit `<pending>` v0.85 落地 (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 Mavis 审核决定 author=Ulysses + 守门 #19 v19 复用现有 11 Repository 模式): 1 file changed, +105/-30 lines: `crates/star-pg-adapter/src/repository/tenant_pool.rs` +105/-30 行: (a) `row_to_tenant_pool(row: &PgRow) -> Result<TenantPool, sqlx::Error>` helper 函数 (把 sqlx::PgRow 转 TenantPool, 12 字段对应 db/migrations/2026-09-10-tenant-pools.sql 列顺序); (b) PgTenantPoolRepository 5 method 全部从 placeholder Err 替换为真实 sqlx::query (dynamic, 不用 DATABASE_URL 编译, 走 P2 阶段 worker 子代理实测 testcontainers): `create` 走 `INSERT INTO tenant_pools (id, tenant_id, pg_url, pool_size, ssl_mode, schema_name, health_status, pgpool_version, created_at, created_by, updated_at, updated_by) VALUES ($1..$12)` + SCD Type 2 pgpool_version=1 强制; `find_current` 走 `SELECT * FROM v_tenant_pools_current WHERE tenant_id = $1` (DISTINCT ON 视图); `list_all` 走 `SELECT * FROM v_tenant_pools_current` (per 守门 #11 缺标比错标: P0-4 阶段 RLS 13 类未启用, P2 阶段补); `update` 走 MAX(pgpool_version)+1 + 新 version INSERT (旧 version 保留 per 守门 #13 c SCD Type 2) + 查旧 created_at/created_by 保留; `soft_delete` 走 SCD Type 2 health_status='Deleted' 标记 (守门 #13 b 物理删除禁止); (c) 4 单元测试 (create_runs_query_path + find_current_runs_query_path + list_all_runs_query_path + soft_delete_runs_query_path) 全部验证 sqlx::query 路径已走, lazy pool 必返 Err (per 守门 #11 缺标比错标); **(2) 守门 #1 v25 实证**: `cargo test -p star-pg-adapter --lib -j 4` = **35/35 PASS** (31 预存 + 4 v0.85 新增, 0 fail, 30.02s); `cargo check --workspace --lib -j 4` = 0 err 51.40s; `cargo fmt -p star-pg-adapter --check` = 0; **(3) 跟 v0.82 已知缺口 (a)+(c) 部分闭合**: WBS v0.82 row §3 已知缺口 (a) 'PgTenantPoolRepository 5 method placeholder Err 走 P2 阶段 + v0.83 testcontainers' 正式落档为真实 sqlx::query 实现 (走 P2 阶段 worker 子代理实测, P0-4 阶段用 lazy pool 验证 query 路径已走), 真实 PG 验证走 v0.83/v0.85.1+ testcontainers 端到端, P0-4 Stage 3.2 收官; **(4) 关键架构 — sqlx::query (dynamic) vs sqlx::query! (compile-time)** (per 守门 #19 v19 复用现有模式): v0.85 用 sqlx::query 动态 query 不需 DATABASE_URL 编译, P0-4 阶段单 crate 模式可编译可测; sqlx::query_as! (compile-time) 需 DATABASE_URL 或 offline cache 留给 P2 阶段 worker 子代理; **(5) 7 段结构** (§3 + §4 + §6): **§3 已知缺口** = (a) PgTenantPoolRepository 5 method 用 sqlx::query 动态, 真实 PG 端到端验证需 testcontainers-rs 或 k3s-deployable (P2 阶段 worker 子代理实装, per 守门 #11); (b) tenant_pools 表 RLS policy 13 类尚未启用 (DDL 已留占位, P0-4 阶段 RLS 未集成, per spec §6.1 待 ADR-0043/§6.1 拍板后落地); (c) 5/5 v2 spec + multi-tenant + DDL + PgTenantPoolRepository sqlx 实现 已落地, v1 + v2 平行存在, P2 阶段全切 v2 后 v1 删除需跨 28+ tests callsite (per WBS v0.80+ 缺口 (b) 仍适用); (d) PgTenantPoolRepository::update 没走 SCD Type 2 跨 session 测试 (lazy pool 限制), 需 testcontainers 实测; **§4 子代理失败接手清单** = N/A; **§6 签字栏** = 5 角色全部 Mavis 接手**审核** author=Ulysses (per 守门 #14 v4); **(6) 累计 P0-4**: 11 star-pg-adapter Repository (6 ops + 4 oauth + 1 tenant_pool) + 5/5 register_*_adapter v2 spec + multi-tenant 路由 + DDL 持久化 + PgTenantPoolRepository 真实 sqlx 实现 完备; **(7) 守门 #1 禁回溯叙事**: v0.1-v0.82 + v0.82.1 修订历史不动, v0.85 row 显式标 P0-4 Stage 3.2 收官 + 用 v0.85 编号 (v0.83 IPA SEC 合规性修复 + v0.84 已被平行工作占用); **(8) 触发**: 9/10 19:20 JST 用户发令 'E.自驱' (Mavis 自驱选方向, per 守门 #9 v19 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 守门 #14 v4 Mavis 审核决定 author=Ulysses); P0-4 Stage 3.2 收官; 守门 #1 v25 实证 35/35 PASS; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 19:20 JST Mavis 自驱 (per 守门 #9 v19) + v0.82 已知缺口 (a)+(c) 跨 session 续做 + 守门 #1 v15 docs 同步饱和第 75 次新事件触发 仍允许 + 守门 #19 v19 复用现有 11 Repository 模式 + 守门 #14 v4 Mavis 审核决定 author=Ulysses + 用 v0.85 编号 (v0.83 + v0.84 已被平行工作占用) |"""


def main() -> int:
    text = WBS_PATH.read_text(encoding="utf-8")
    if "**v0.85**" in text:
        print("[skip] v0.85 row 已存在, 不重插 (per 守门 #1 禁回溯叙事)")
        return 0
    new_text = text.rstrip() + "\n" + ROW + "\n"
    WBS_PATH.write_text(new_text, encoding="utf-8")
    print(f"[ok] v0.85 row appended at end of WBS, +{len(ROW)} chars")
    return 0


if __name__ == "__main__":
    sys.exit(main())
