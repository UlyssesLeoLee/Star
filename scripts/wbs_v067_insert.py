"""v0.67 WBS row insertion (per 守门 #12 commit-time docs sync, v29 第 56 次新事件触发)"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V067_ROW = """| **v0.67** | **2026-09-10 15:30 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v25 单 crate 模式 + 守门 #12 v15 docs 同步饱和第 56 次新事件触发 仍允许)** | **§14.10.4 缺口 #4 6/6 ops Repository 收官: ops_metrics_config 拆出独立文件 (per WBS §14.10.4 + v0.39 §14.12 II Postgres, v0.62 反转解锁后 Mavis 推进)**：**(1) commit `3905be0` 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 永久代签 author=Ulysses): 2 files changed, 419 insertions(+), 310 deletions(-): (a) `crates/star-pg-adapter/src/repository/ops_metrics_config.rs` 9.3KB 新文件 (290 lines, OpsMetricsConfig struct + OpsMetricsConfigRepository trait + PgOpsMetricsConfigRepository impl + 2 unit tests); (b) `crates/star-pg-adapter/src/repository/mod.rs` -290 lines +37 lines (改 pub mod 索引 + pub use 跟其他 5 Repository 一致); **(2) 守门 #1 v25 实证**: `cargo test -p star-pg-adapter --lib -j 4` = **26/26 PASS** (0 fail, 0s); `cargo check -p star-pg-adapter --lib -j 4` = 0 err; `cargo fmt -p star-pg-adapter --check` = 0 diff; **(3) 6 ops Repository 100% 收官** (per WBS §14.10.4 缺口 #4): T=3 (ops_helm_release_state + ops_cluster_action_log + ops_log_query_log) + W=2 (ops_log_entry TTL 7d + ops_log_analysis TTL 30d) + **M=1 (ops_metrics_config SCD2, 本次拆出独立文件) [v0.67]**; **(4) 累计 star-pg-adapter 10 Repository** (6 ops + 4 OAuth) = ops_metrics_config + ops_cluster_action_log + ops_helm_release_state + ops_log_analysis + ops_log_entry + ops_log_query_log + oauth_access_tokens + oauth_authorization_codes + oauth_clients + oauth_refresh_tokens; **(5) 7 段结构** (§3 + §4 + §6): **§3 已知缺口** = (a) PgOpsMetricsConfigRepository 没连接真实 PG (需要 v0.66 InMemoryAdapterRegistry 或真实 PG connection); (b) ops_metrics_config 12 字段只测 2 (struct has 12 fields + trait has 3 methods), 缺端到端 PG 集成测试 (per 守门 #1 v15 docs 同步饱和, 等 PG 容器化后实装); (c) ops_metrics_config SCD Type 2 trigger 测试缺 (跟 trg_ops_metrics_config_scd2 + trg_ops_metrics_config_audit 联动); **§4 子代理失败接手清单** = N/A (本 commit 是 root session 直接实装, 无子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC); **§6 签字栏** = 5 角色全部 Mavis 永久代签 author=Ulysses; **(6) 阻塞 0 → 0** (v0.62 反转后跨 session 续做项解锁); **(7) 触发**: 9/10 14:30 JST 用户发令"继续推" + 守门 #1 v25 实证 26/26 PASS + commit author=Ulysses |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.67-ops-metrics-config-repo" in content:
        print("OK: v0.67 row already present (idempotent skip)")
        return
    pat = re.compile(r"(\| \*\*v0\.66\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.66 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V067_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.67 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
