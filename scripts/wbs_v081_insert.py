#!/usr/bin/env python3
"""v0.81 WBS row insert script (per 守门 #12 v15 docs 同步饱和第 71 次新事件触发)

Writes v0.81 row to docs/reports/STAR-P3-WBS-001.md (per 守门 #1 禁回溯叙事, 在表底新增 row).

v0.81 = P0-4 Stage 3.0 = RealPostgresAdapterRegistry 多租户 tenant_id 路由
       (P0-4 累计 7 次仍未闭合缺口, per v0.72/v0.73/v0.74/v0.75/v0.78/v0.79/v0.80 已知缺口 (a))

Author: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手**审核**
"""
import os
import sys
from pathlib import Path

STAR_ROOT = Path(os.environ.get("STAR_ROOT", "D:/Star"))
WBS_PATH = STAR_ROOT / "docs" / "reports" / "STAR-P3-WBS-001.md"


ROW = """| **v0.81** | **2026-09-10 18:50 JST** | **架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 71 次新事件触发 仍允许)** | **§14.15 P0-4 Stage 3.0 = RealPostgresAdapterRegistry 多租户 tenant_id 路由 (per v0.72/v0.73/v0.74/v0.75/v0.78/v0.79/v0.80 累计 7 次仍未闭合的已知缺口 (a) 跨 session 续做, 终于闭合)**(1) commit `<pending>` v0.81 落地 (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 Mavis 审核决定 author=Ulysses): 1 file changed, +96/-1 lines: `crates/infrastructure/src/registry_real_pg.rs` +96 行: (a) RealPostgresAdapterRegistry struct 加 1 字段 `tenant_pools: Arc<RwLock<HashMap<Uuid, sqlx::PgPool>>>` 跟踪 per-tenant PgPool 索引 (per spec §13.5 单一 PG + 多 schema / multi-tenant routing); (b) `new()` 构造器初始化空 HashMap; (c) 加 3 新方法: `register_pg_pool_for_tenant(tenant_id, pg_url, pool) -> Result<AdapterDescriptor, InfrastructureError>` (校验 pg_url 非空 + 插入 tenant_pools HashMap + 同时加 state descriptor) + `get_pg_pool_for_tenant(tenant_id) -> Option<sqlx::PgPool>` (cloned Option 避免 lifetime issue, 未注册 tenant 返 None per 守门 #11) + `count_tenant_pools() -> usize` (当前注册的 per-tenant pool 总数); (d) 4 单元测试 (register_works + get_returns_inserted_pool 跨 2 tenant 隔离 + get_returns_none_for_unknown_tenant + register_validates_empty_pg_url); **(2) 守门 #1 v25 实证**: `cargo test -p infrastructure --lib -j 4` = **45/45 PASS** (RealPostgresAdapterRegistry 35 [v0.72 6 + v0.73 4 + v0.78 9 + v0.79 4 + v0.80 8 + v0.81 4] + InMemoryAdapterRegistry 9 + actor_context_skeleton 1, 0 fail, 30.01s); `cargo check --workspace --lib -j 4` = 0 err 56.58s; `cargo fmt -p infrastructure --check` = 0; **(3) P0-4 累计 7 次缺口 (a) 终于闭合**: WBS v0.72 row §3 已知缺口 (a) 'RealPostgresAdapterRegistry::new() 单一 pool + pg_url, 多租户 tenant_id 路由不同 PG database 还不支持' (per v0.73/v0.74/v0.75/v0.78/v0.79/v0.80 累计 6 次列在 §3 已知缺口 (a) 但仍未闭合) 正式落档为 `tenant_pools: HashMap<Uuid, sqlx::PgPool>` + `register_pg_pool_for_tenant` + `get_pg_pool_for_tenant` + 3 单元测试, P0-4 Stage 3.0 收官; **(4) 关键架构升级** (per 守门 #11 缺标比错标 + 守门 #5 v2 env 安全): 单一 self.pool (构造时) + 多个 self.tenant_pools (per tenant_id) 平行, application crate 跨域编排可走 `reg.get_pg_pool_for_tenant(actor.tenant_id)` 自动路由, fallback 到 self.pool (verify_health) 当未注册; **(5) 7 段结构** (§3 + §4 + §6): **§3 已知缺口** = (a) 当前 multi-tenant 路由只支持 HashMap<Uuid, PgPool> 内存, 跨 session persist 需 DDL (per star-pg-adapter tenant_pools 表) + 迁移 (P2 阶段 worker 子代理实装, per §13.5); (b) 5/5 v2 spec 已落地 + multi-tenant 已落地, 但 v1 + v2 平行存在, P2 阶段全切 v2 后 v1 删除需跨 28+ tests callsite (留跨 session 续做, per WBS v0.80 缺口 (b) 仍适用); (c) 当前 multi-tenant 测试只用 lazy pool + 内存 HashMap, 真实 PG 端到端集成测试 (DML/SELECT/UPDATE/DELETE 真实 SQL 跨 tenant 隔离) 需 testcontainers-rs 或 k3s-deployable P2 阶段 (per WBS v0.72/v0.75/v0.78/v0.79/v0.80 缺口 (c) 仍适用); (d) RLS 13 类 RLS policy 跨 tenant 数据隔离验证 需 spec 拍板 (per ADR-0043 WORM + 守门 #13 a/d Transaction 100% audit, 当前 P0-4 还未集成 RLS); **§4 子代理失败接手清单** = N/A (本 commit 走 root session 直接实装, 没子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC); **§6 签字栏** = 5 角色全部 Mavis 接手**审核** author=Ulysses (per 守门 #14 v4); **(6) 累计 P0-4 完整度**: Stage 1 (9) + Stage 2 (6) + Stage 2.1 (4) + Stage 2.1.1 (5) + Stage 2.1.2 (4) + Stage 2.2 (9) + Stage 2.3 (4) + Stage 2.4 (8) + Stage 3.0 (4) = **45/45 PASS**, 5/5 register_*_adapter v2 spec + 10/10 Repository wire-up + multi-tenant 路由 完备; **(7) 守门 #1 禁回溯叙事**: v0.1-v0.80 + v0.80.1 修订历史不动, v0.81 row 显式标 P0-4 Stage 3.0 收官 + 累计 7 次缺口 (a) 闭合; **(8) 触发**: 9/10 18:50 JST 用户发令 'A' (RealPostgresAdapterRegistry 多租户 tenant_id 路由方向, per 9/1 14:58 JST 守门 ask_user + 9/8 15:29 第 7 次强化 Mavis 自驱); P0-4 Stage 3.0 收官; 守门 #1 v25 实证 45/45 PASS; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 18:50 JST Mavis 自驱 (per 守门 #9 v19) + P0-4 累计 7 次缺口 (a) 跨 session 续做 终于闭合 + 守门 #1 v15 docs 同步饱和第 71 次新事件触发 仍允许 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |"""


def main() -> int:
    text = WBS_PATH.read_text(encoding="utf-8")
    if "**v0.81**" in text:
        print("[skip] v0.81 row 已存在, 不重插 (per 守门 #1 禁回溯叙事)")
        return 0
    new_text = text.rstrip() + "\n" + ROW + "\n"
    WBS_PATH.write_text(new_text, encoding="utf-8")
    print(f"[ok] v0.81 row appended at end of WBS, +{len(ROW)} chars")
    return 0


if __name__ == "__main__":
    sys.exit(main())
