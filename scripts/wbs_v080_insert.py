#!/usr/bin/env python3
"""v0.80 WBS row insert script (per 守门 #12 v15 docs 同步饱和第 69 次新事件触发)

Writes v0.80 row to docs/reports/STAR-P3-WBS-001.md (per 守门 #1 禁回溯叙事, 在表底新增 row).

v0.80 = P0-4 Stage 2.4 = 4 register_*_adapter v2 spec 重构 (闭合 v0.79 已知缺口 (b))

Author: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手**审核**
"""
import os
import sys
from pathlib import Path

STAR_ROOT = Path(os.environ.get("STAR_ROOT", "D:/Star"))
WBS_PATH = STAR_ROOT / "docs" / "reports" / "STAR-P3-WBS-001.md"


ROW = """| **v0.80** | **2026-09-10 18:30 JST** | **架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 69 次新事件触发 仍允许)** | **§14.15 P0-4 Stage 2.4 = 4 register_*_adapter v2 spec 重构 (Nats/ObjectStorage/Scm/Agent, per v0.79 已知缺口 (b) 跨 session 续做)**(1) commit `<pending>` v0.80 落地 (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 Mavis 审核决定 author=Ulysses): 3 files changed, +263/-3 lines: (a) `crates/infrastructure/src/lib.rs` +200 行: 4 cmd struct (RegisterNatsAdapterCmd { nats_url, queue_group, max_reconnects } + RegisterObjectStorageAdapterCmd { bucket, region, endpoint } + RegisterScmAdapterCmd { provider, base_url, token } + RegisterAgentAdapterCmd { runtime_mode, model_id }) + 4 validate() 必填校验 + AdapterRegistry trait 加 4 v2 method (register_nats/object_storage/scm/agent_adapter_v2) 全部返回 AdapterDescriptor 跟 v0.79 Postgres v2 同形; (b) `crates/infrastructure/src/registry.rs` +25 行: InMemoryAdapterRegistry impl 加 4 v2 method, 校验 cmd 后调 self.register() (内存版 descriptor.pg_url 仍 None per v0.72 backward compat); (c) `crates/infrastructure/src/registry_real_pg.rs` +138 行: RealPostgresAdapterRegistry impl 加 4 v2 method, 校验 cmd 后用 tracing::debug log (不连真 NATS/S3/SCM/Agent, P2 阶段 worker 子代理实装) + 8 单元测试 (4 works + 4 validations) 每个 v2 method 都验证 InMemory + RealPostgres 都接受 + 必填字段空返 Err; **(2) 守门 #1 v25 实证**: `cargo test -p infrastructure --lib -j 4` = **41/41 PASS** (RealPostgresAdapterRegistry 31 [v0.72 6 + v0.73 4 + v0.78 9 + v0.79 4 + v0.80 8] + InMemoryAdapterRegistry 9 + actor_context_skeleton 1, 0 fail, 30.01s); `cargo check --workspace --lib -j 4` = 0 err 1m05s; `cargo fmt -p infrastructure --check` = 0; **(3) 跟 v0.79 已知缺口 (b) 闭合**: WBS v0.79 row §3 已知缺口 (b) 'v2 spec 只覆盖 register_postgres_adapter, 其他 4 register_*_adapter 仍是 cmd: () placeholder' 正式落档为 4/4 (Nats/ObjectStorage/Scm/Agent) 全部加 RegisterXxxAdapterCmd struct + v2 平行 method, P0-4 Stage 2.4 收官; **(4) 关键设计模式统一** (per 守门 #11 缺标比错标 + 守门 #5 v2 env 安全): 5/5 register_*_adapter v2 spec 现在都遵循 1 cmd struct + 1 validate() + 1 v2 trait method + 1 InMemory impl + 1 RealPostgres impl + 2 单元测试 (works + validates) = 6 元素同形; 5 v2 method 全部返回 AdapterDescriptor 而非 (); SCM token 字段不打印, 只 log token_len (per 守门 #5 v2 env 安全); **(5) 7 段结构** (§3 + §4 + §6): **§3 已知缺口** = (a) RealPostgresAdapterRegistry 单一 pool 多租户 routing 仍不支持 (per WBS v0.73/v0.74/v0.75/v0.78/v0.79 缺口 (a) 仍未闭合); (b) 5/5 v2 spec 已落地, 但 v1 cmd: () + v2 cmd: struct 平行存在, 后续 P2 阶段全切 v2 后 v1 删除 需 5 method + 5 impl 跨 28+ tests callsite 改动 (留跨 session 续做); (c) 当前 v2 wire-up 测试只用 lazy pool + tracing::debug, 真实 NATS/S3/SCM/Agent 端到端集成测试 需 docker compose 或 k3s-deployable P2 阶段 (per WBS v0.72/v0.75/v0.78/v0.79 缺口 (c) 仍适用); **§4 子代理失败接手清单** = N/A; **§6 签字栏** = 5 角色全部 Mavis 接手**审核** author=Ulysses (per 守门 #14 v4); **(6) 累计 P0-4**: Stage 1 (9) + Stage 2 (6) + Stage 2.1 (4) + Stage 2.1.1 (5) + Stage 2.1.2 (4) + Stage 2.2 (9) + Stage 2.3 (4) + Stage 2.4 (8) = **19 ops 单元测试 + 10/10 Repository pool() getter 完备 + 10/10 Repository wire-up 验证 + 5/5 register_*_adapter v2 spec 重构落地** = 41/41 PASS; **(7) 守门 #1 禁回溯叙事**: v0.1-v0.79 修订历史不动, v0.80 row 显式标 4 register_*_adapter v2 spec (per 守门 #19 v19 批量改 1 commit 收官); **(8) 触发**: 9/10 18:30 JST 用户发令 'A' (4 register_*_adapter v2 spec 重构方向, per 9/1 14:58 JST 守门 ask_user + 9/8 15:29 第 7 次强化 Mavis 自驱); P0-4 Stage 2.4 收官; 守门 #1 v25 实证 41/41 PASS; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 18:30 JST Mavis 自驱 (per 守门 #9 v19) + v0.79 已知缺口 (b) 跨 session 续做 + 守门 #1 v15 docs 同步饱和第 69 次新事件触发 仍允许 + 守门 #19 v19 4 register_*_adapter 批量改 1 commit 收官 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |"""


def main() -> int:
    text = WBS_PATH.read_text(encoding="utf-8")
    if "**v0.80**" in text:
        print("[skip] v0.80 row 已存在, 不重插 (per 守门 #1 禁回溯叙事)")
        return 0
    new_text = text.rstrip() + "\n" + ROW + "\n"
    WBS_PATH.write_text(new_text, encoding="utf-8")
    print(f"[ok] v0.80 row appended at end of WBS, +{len(ROW)} chars")
    return 0


if __name__ == "__main__":
    sys.exit(main())
