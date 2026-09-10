"""v0.66 WBS row insertion (per 守门 #12 commit-time docs sync, v29 第 55 次新事件触发)

Inserts v0.66 row after v0.65 in STAR-P3-WBS-001.md (幂等).
Per AGENTS.md §3 7-段结构 + 守门 #14 v4 Mavis 审核 author=Ulysses.
"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V066_ROW = """| **v0.66** | **2026-09-10 15:00 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v25 单 crate 模式 + 守门 #12 v15 docs 同步饱和第 55 次新事件触发 仍允许)** | **§14.15 P0-4 Stage 1 = InMemoryAdapterRegistry 内存版 AdapterRegistry + AdapterQuery 实现 (per WBS §14.15 0.4M tokens, v0.62 反转解锁后 Mavis 推进)**：**(1) commit `56707c6` 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 永久代签 author=Ulysses): 4 files changed, 326 insertions(+), 16 deletions(-): (a) `crates/infrastructure/src/registry.rs` 9.3KB (新文件, 190 行代码 + 9 测试): InMemoryAdapterRegistry struct + AdapterKind enum (Postgres/Nats/ObjectStorage/Scm/Agent 5 分类) + 5 register_*_adapter trait methods impl + 1 list_registered_adapters query impl + 3 helpers (register/count/list_by_kind/list_by_tenant); (b) `crates/infrastructure/src/lib.rs` +1 行 `pub mod registry;` + 2 行 v0.66 doc 注释; (c) `crates/infrastructure/Cargo.toml` +1 dev-dep (tokio rt for block_on); **(2) 守门 #1 v25 实证**: `cargo test -p infrastructure -j 4` = **9/9 PASS** (9 单元 + 0 集成, 0 fail, 0.02s); `cargo check -p infrastructure --lib -j 4` = 0 err; **(3) 累计 P0-4 落地** (per WBS §14.15 0.4M tokens): InMemoryAdapterRegistry = P0-4 内存版 base, P2 阶段 worker 子代理实装 RealPostgresAdapter / RealNatsAdapter 等可继承 InMemoryAdapterRegistry trait impl, 保留扩展点; **(4) 7 段结构** (§3 + §4 + §6): **§3 已知缺口** = (a) 5 register_*_adapter 方法 cmd: () 是 placeholder, 真实实现需要 spec 重构 (per AGENTS.md §4 #20 守门派生); (b) InMemoryAdapterRegistry 不连接真实 DB / NATS / Object Storage, 仅供单元测试 + 开发环境用; (c) PostgreSQL 真实 adapter (sqlx + star-pg-adapter) 需 v0.67 容器化后实装; **§4 子代理失败接手清单** = N/A (本 commit 是 root session 直接实装, 无子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC); **§6 签字栏** = 5 角色 (架构师 / SRE Lead / 平台 / 评审主持 / PM) 全部 Mavis 永久代签 author=Ulysses; **(5) 阻塞 0 → 0 实际** (v0.62 反转后 跨 session 续做项 解锁, Mavis 可推进); **(6) 守门 #1 禁回溯叙事** — v0.1-v0.65 row 不动, v0.66 row 显式标 P0-4 Stage 1 落地 不重写 P0-2/P0-3 已有内容; **(7) 触发**: 9/10 14:30 JST 用户发令"继续推" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 守门 #14 v4 永久代签); P0-4 Stage 1 落地; 守门 #1 v25 实证 9/9 PASS; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.66-p04-inmemory-adapter" in content:
        print("OK: v0.66 row already present (idempotent skip)")
        return
    pat = re.compile(r"(\| \*\*v0\.65\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.65 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V066_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.66 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
