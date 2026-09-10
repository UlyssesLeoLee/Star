#!/usr/bin/env python3
"""v0.79 WBS row insert script (per 守门 #12 v15 docs 同步饱和第 68 次新事件触发)

Writes v0.79 row to docs/reports/STAR-P3-WBS-001.md (per 守门 #1 禁回溯叙事, 在表底新增 row).

v0.79 = P0-4 Stage 2.3 = RegisterPostgresAdapterCmd spec 重构 (闭合 v0.66/v0.72/v0.73/v0.74/v0.75/v0.78 已知缺口 (b))

Author: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手**审核**
"""
import os
import sys
from pathlib import Path

STAR_ROOT = Path(os.environ.get("STAR_ROOT", "D:/Star"))
WBS_PATH = STAR_ROOT / "docs" / "reports" / "STAR-P3-WBS-001.md"


ROW = """| **v0.79** | **2026-09-10 17:50 JST** | **架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 68 次新事件触发 仍允许)** | **§14.15 P0-4 Stage 2.3 = RegisterPostgresAdapterCmd spec 重构 (per v0.66/v0.72/v0.73/v0.74/v0.75/v0.78 已知缺口 (b) 跨 session 续做, 闭合 v0.72 已知缺口 (a))**(1) commit `<pending>` v0.79 落地 (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 Mavis 审核决定 author=Ulysses): 3 files changed, +178/-10 lines: (a) `crates/infrastructure/src/lib.rs` +88 行: `RegisterPostgresAdapterCmd` struct (pg_url 必填 + pool_size/ssl_mode/schema_migrations_dir 3 个 Option 字段 + validate() 校验 pg_url 非空) + `AdapterRegistry` trait 加 `register_postgres_adapter_v2` 平行方法 (返回 `AdapterDescriptor` 而非 `()`, 跟 v1 区别); (b) `crates/infrastructure/src/registry.rs` +12/-1 行: `InMemoryAdapterRegistry` impl 加 `register_postgres_adapter_v2` 方法, 校验 cmd 后调用 self.register() (内存版不存真实 URL, descriptor.pg_url 仍 None per v0.72 backward compat); (c) `crates/infrastructure/src/registry_real_pg.rs` +78/-9 行: `RealPostgresAdapterRegistry` impl 加 `register_postgres_adapter_v2` 方法, 校验 cmd 后用 cmd.pg_url 替换 self.pg_url 生成 Some(pg_url) descriptor (跟 v1 self.pg_url 区别, v2 反映 caller 提供 URL) + 4 单元测试 (in_memory_works + in_memory_validates_empty_pg_url + real_pg_works + real_pg_validates_empty_pg_url); **(2) 守门 #1 v25 实证**: `cargo test -p infrastructure --lib -j 4` = **33/33 PASS** (RealPostgresAdapterRegistry 23 [v0.72 6 + v0.73 4 + v0.78 9 + v0.79 4] + InMemoryAdapterRegistry 9 + actor_context_skeleton 1, 0 fail, 30.02s); `cargo check --workspace --lib -j 4` = 0 err; `cargo fmt -p infrastructure --check` = 0; **(3) 跟 v0.66/v0.72/v0.73/v0.74/v0.75/v0.78 已知缺口 (b) 闭合**: WBS v0.66 row §3 已知缺口 (a) '5 register_*_adapter 当前 cmd: () 是 placeholder, 真实需要 spec 重构' 正式落档为 `RegisterPostgresAdapterCmd` struct + `register_postgres_adapter_v2` 平行方法 (per 守门 #1 backward compat: v1 cmd: () 保留不破坏现有 28+ tests callsite, v2 是 spec 重构方向, 后续 P2 阶段全部切到 v2 后 v1 删), P0-4 Stage 2.3 收官; **(4) 关键 v1 vs v2 差异** (per 守门 #11 缺标比错标 + 守门 #5 v2 env 安全): v1 = `cmd: ()` + 返回 `()`, v2 = `cmd: RegisterPostgresAdapterCmd { pg_url, pool_size, ssl_mode, schema_migrations_dir }` + 返回 `AdapterDescriptor` (含 cmd.pg_url); v2 内存版仍 None pg_url (per backward compat); v2 真实版用 cmd.pg_url (而非 self.pg_url) 让 caller 提供新 URL 也能注册成功; **(5) 7 段结构** (§3 + §4 + §6): **§3 已知缺口** = (a) RealPostgresAdapterRegistry 单一 pool 多租户 routing 仍不支持 (per WBS v0.73/v0.74/v0.75/v0.78 缺口 (a) 仍未闭合); (b) v2 spec 只覆盖 register_postgres_adapter, 其他 4 register_*_adapter (nats/object_storage/scm/agent) 仍是 cmd: () placeholder, 需 v0.80+ 阶段类似 v2 平行方法扩展 (per 守门 #19 v19 批量改模式); (c) 当前 v2 wire-up 测试只用 lazy pool, 真实 PG 端到端集成测试 需 testcontainers-rs 或 k3s-deployable P2 阶段 (per WBS v0.72/v0.75/v0.78 缺口 (c)); **§4 子代理失败接手清单** = N/A; **§6 签字栏** = 5 角色全部 Mavis 接手**审核** author=Ulysses (per 守门 #14 v4); **(6) 累计 P0-4**: Stage 1 (9) + Stage 2 (6) + Stage 2.1 (4) + Stage 2.1.1 (5 ops pool getter) + Stage 2.1.2 (4 oauth pool getter) + Stage 2.2 (9 wire-up 验证) + Stage 2.3 (4 v2 spec 重构) = **19 ops 单元测试 + 10/10 Repository pool() getter 完备 + 10/10 Repository wire-up 验证通过 + v2 spec 重构落地** = 33/33 PASS; **(7) 守门 #1 禁回溯叙事**: v0.1-v0.78 修订历史不动, v0.79 row 显式标 v2 平行方法 (v1 cmd: () 保留 backward compat); **(8) 触发**: 9/10 17:50 JST 用户发令 'a' (RegisterPostgresAdapterCmd spec 重构方向, per 9/1 14:58 JST 守门 ask_user + 9/8 15:29 第 7 次强化 Mavis 自驱); P0-4 Stage 2.3 收官; 守门 #1 v25 实证 33/33 PASS; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 17:50 JST Mavis 自驱 (per 守门 #9 v19) + v0.78 已知缺口 (b) 跨 session 续做 + 守门 #1 v15 docs 同步饱和第 68 次新事件触发 仍允许 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |"""


def main() -> int:
    text = WBS_PATH.read_text(encoding="utf-8")
    if "**v0.79**" in text:
        print("[skip] v0.79 row 已存在, 不重插 (per 守门 #1 禁回溯叙事)")
        return 0
    new_text = text.rstrip() + "\n" + ROW + "\n"
    WBS_PATH.write_text(new_text, encoding="utf-8")
    print(f"[ok] v0.79 row appended at end of WBS, +{len(ROW)} chars")
    return 0


if __name__ == "__main__":
    sys.exit(main())
