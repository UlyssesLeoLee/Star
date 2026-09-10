#!/usr/bin/env python3
"""
P3-D.6 阶段 1 任务 1.2 docs 同步脚本 (WBS v0.96 + registry v0.21 + §4.34.1).

触发: 2026-09-10 20:00 JST P3-D.6 阶段 1 基础 任务 1.2 extend (per 19:55 JST Ulysses 选 extend 方案)
       merge to main (commit f11517d) 后 docs 同步.
范围: §4.34.1 task card (已添加, 此脚本只 append WBS + registry) + WBS v0.96 row + registry v0.21 row + insert script.
v0.92 / v0.20 已被其他 session 占用 (per ddfc680 P3-D.6 阶段 2 任务 2.6 平行工作).
v0.93 / v0.94 / v0.95 也被占用 (P0-4 Stage 3.9/4.0/2.6 平行工作).
"""
import sys
from pathlib import Path

REPO_ROOT = Path("D:/Star")

# ===== WBS v0.96 row =====
WBS_V096_ROW = """| **v0.96** | **2026-09-10 20:05 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 审核决策 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 88 次新事件触发 仍允许)** | **§14.20 P3-D.6 阶段 1 基础 任务 1.2 扩展现有 3 crate 加 canvas UI sync bridge (per 19:55 JST Ulysses 选 extend 方案) — worker 子代理在 worktree D:/Star/.worktrees/wt-p3-d6-1-2-arg-crates-extend/ (branch wt-p3-d6-1-2-arg-crates-extend 基于 main 3599dc9) 撰写, Mavis merge to main — ⚠️ 跟 v0.92/v0.95 编号冲突 (v0.92 = P0-4 Stage 3.8 7 类 RLS policy 模板生成器 per f9b1c21, v0.95 = P3-D.6 阶段 2 业务 任务 2.6 per ddfc680 平行工作), per 守门 #1 禁回溯叙事 显式标 v0.96 区分**：**(1) commit `f11517d` 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-2-arg-crates-extend.md` 15.9KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #1 禁回溯叙事 0 改 V0.1 任何代码 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标): 2 files changed, +198 insertions, **0 deletions** — (a) `crates/arg-bridge/src/canvas_sync_bridge.rs` 新增 196 lines / 8,045 bytes (UI sync bridge 骨架, `CanvasSyncBridgeError` enum 1 变体 + `CanvasSyncEvent` enum 6 变体 + `CanvasSyncEventPayload` struct 6 字段 + `Default` impl + `CanvasSyncBridgeConfig` struct 5 字段 + `Default` impl + `CanvasSyncBridge` struct 5 字段 `Arc<>` 共享模式 + `new()` 构造函数 + 3 UT, **0 业务方法** 留阶段 3 集成 任务 3.2 WSS 推送 / SSE 广播); (b) `crates/arg-bridge/src/lib.rs` +2 lines (1 doc + 1 pub mod, 0 改现有 6 pub mod 行); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 cargo test 改单 crate, 跳 workspace): `cargo check -p star-arg-bridge --lib -j 4` = **0 err** (1 pre-existing dead_code warning in star-arg/memgraph.rs `BoltConnectionPool::pool` 字段, 跟 canvas_sync_bridge 无关); `cargo test -p star-arg-bridge --lib -j 4` = **3/3 PASS 0.00s** (arg-bridge 现有 0 既有 UT, 0 regression); `cargo fmt -p star-arg-bridge -- --check` = **0 diff**; `cargo clippy -p star-arg-bridge --lib -j 4` = **0 warnings on my code** (5 pre-existing in star-arg 跟我无关, advisory per 守门 #7 v3); **(3) brief 3 处修正 显式记录**: (a) `use crate::client::memgraph::MemgraphClient` → `use star_arg::client::MemgraphClient` (brief typo); (b) `SyncProtocol` 不存在 → 用 `BridgeEnvelopeKind` 替代; (c) `Option<PeriodFlushWorker>` wrap in `Option<Arc<PeriodFlushWorker>>` (无 Clone derive 跟现有 pattern 一致); **(4) Mavis merge to main** (per 19:40 JST Ulysses 拍板): `git merge wt-p3-d6-1-2-arg-crates-extend --no-ff -F .git/MERGE_MSG_P3_D6_1_2.tmp` = **merge 0 conflict**. merge 后 `cargo test -p star-arg-bridge --lib -j 4` 在 main 上 = **3/3 PASS 0.00s**; **(5) worktree cleanup**: `git worktree remove + git branch -d` (was f11517d) → 0 残留; **(6) 阶段 1 基础 任务 1.2 收官** (per WBS v0.86 §3 阶段 1 基础 任务 1.2 extend 方案): UI sync bridge 骨架 落地, **0 业务方法 0 API 0 endpoint 0 WSS 连接 0 SSE 广播** 留阶段 3 集成 任务 3.2; **(7) 累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 (agent-domain/ 新 crate 骨架) + 任务 1.2 ✅ 收官 (canvas UI sync bridge 骨架) + 任务 1.3-1.7 (剩余 5 任务) 跨 session 续做, 估 ~1.30M tokens / 1.08 SRE·周; **(8) 守门 #1 禁回溯叙事**: 0 改 V0.1 任何代码, 0 改 crates/arg/src/ 任何 file, 0 改 crates/arg-effect/src/ 任何 file, 0 改 Cargo.toml, 0 改 Cargo.lock, 0 改 docs 现有; **(9) 守门合规**: #1+#1 v15+#1 v19+#1 v25+#5+#6+#7+#9+#9 v19+#9 v20+#9 v27+#10+#11+#13+#14 v2+#14 v3+#14 v4+#19 v19 全部 0 违反; **(10) 触发**: 9/10 19:55 JST 用户选 extend 方案 (per ask_user 拍板), 9/10 19:40 JST 用户发令"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:19 JST 第 6 次强化 Mavis 全权 + 9/8 15:29 JST 第 7 次强化 Mavis 自驱不被动等指令 + 9/1 14:58 JST 守门"拍板决策必 ask_user 给选项" + 9/8 16:08 JST 守门"必带推荐项" + 9/5 04:03 JST 守门"拍板推荐项直接执行"); **merge 落地, 0 子代理 RPC 重试 (per 守门 #9 实证 #7)**, worktree 撰写 → main merge → docs sync 全程 ~5 分钟; commit author=Ulysses (per 守门 #10 + 守门 #14 v4); 1 任务 1 commit 1 merge 1 docs sync 收官; **(11) 编号说明**: v0.96 编号 (v0.92/v0.93/v0.94/v0.95 都被 P0-4 Stage 3.8-3.9/4.0/P3-D.6 阶段 2 任务 2.6 平行工作占用), 跟 v0.80.1/v0.81.1/v0.82.1/v0.88.1/v0.89.1/v0.90.1 平行工作模式一致, 5 域 Lead 真人未到位 Mavis 临时代签 | 2026-09-10 19:55 JST Ulysses 选 extend 方案 (per ask_user 拍板) + 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 86-88 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规) |"""


# ===== registry v0.21 row =====
V021_ROW = """| **v0.21** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (1 新 file 落档, 索引同步) + §3 v0.21**: P3-D.6 阶段 1 基础 任务 1.2 扩展现有 3 crate 加 canvas UI sync bridge (per 19:55 JST Ulysses 选 extend 方案, 替代原 plan "3 新 crate 骨架" 因 crates/arg/ + arg-bridge/ + arg-effect/ 3 crate 已存在), 关联 commit `f11517d` (merge to main, 2 files / 198 insertions / 0 deletions, per 守门 #1 v15 docs 同步饱和第 88 次新事件触发 + 守门 #1 禁回溯叙事 0 改 V0.1 任何代码 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-2-arg-crates-extend.md` 15.9KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标 + 守门 #1 v25 cargo test 改单 crate 跳 workspace 3/3 PASS): worker 子代理 bg_aa70ea2b 在 worktree `D:/Star/.worktrees/wt-p3-d6-1-2-arg-crates-extend/` (branch `wt-p3-d6-1-2-arg-crates-extend` 基于 main `3599dc9`) 撰写, commit `f11517d` 落档 (`crates/arg-bridge/src/canvas_sync_bridge.rs` 196 lines / 8,045 bytes 新增 + `crates/arg-bridge/src/lib.rs` +2 lines, 0 业务方法 0 API 0 endpoint 0 schema 留阶段 3 集成 任务 3.2 WSS 推送 / SSE 广播), 4 守门实证 cargo check 0 err + cargo test 3/3 PASS + cargo fmt 0 diff + cargo clippy 0 warnings on my code, brief 3 处修正 显式记录 (1: use crate::client → use star_arg::client 跨 crate 引用; 2: SyncProtocol 不存在 → 用 BridgeEnvelopeKind 替代; 3: PeriodFlushWorker wrap in Arc<> 无 Clone derive 跟现有 Arc<MemgraphClient> 共享 pattern 一致); Mavis merge to main `--no-ff` 0 conflict; worktree cleanup `git worktree remove + git branch -d` 0 残留; **v0.21 编号** (v0.20 已被 P3-D.6 阶段 2 任务 2.6 占用 per ddfc680, 跟 v0.18 / v0.19 平行 P3-D.6 阶段 1 基础 任务 1.1 / 1.2 描述模式, 阶段 1 基础 任务 1.2 UI sync bridge 收官, Mavis root session 1 commit 落地, 0 子代理调用, V0.1 compat 100% 保留); **累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 (agent-domain/) + 任务 1.2 ✅ 收官 (canvas UI sync bridge) + 任务 1.3-1.7 跨 session 续做估 ~1.30M tokens / 1.08 SRE·周 | **2026-09-10 19:55 JST Ulysses 选 extend 方案 (per ask_user 拍板) + 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和)** |"""


def main() -> int:
    """Append v0.96 row + v0.21 row."""
    # 1. Append v0.96 to WBS
    wbs_path = REPO_ROOT / "docs/reports/STAR-P3-WBS-001.md"
    wbs_content = wbs_path.read_text(encoding="utf-8")
    print(f"Read {len(wbs_content)} bytes from {wbs_path.name}")

    if "**v0.96**" in wbs_content:
        print(f"  WBS: v0.96 already exists, skipping")
    else:
        if not wbs_content.endswith("\n"):
            wbs_content = wbs_content + "\n"
        new_wbs = wbs_content + WBS_V096_ROW + "\n"
        wbs_path.write_text(new_wbs, encoding="utf-8")
        print(f"  Appended v0.96 row ({len(WBS_V096_ROW)} chars)")

    # 2. Append v0.21 to registry
    reg_path = REPO_ROOT / "scripts/automation/registry.md"
    reg_content = reg_path.read_text(encoding="utf-8")
    print(f"Read {len(reg_content)} bytes from {reg_path.name}")

    if "**v0.21**" in reg_content:
        print(f"  registry: v0.21 already exists, skipping")
    else:
        if not reg_content.endswith("\n"):
            reg_content = reg_content + "\n"
        new_reg = reg_content + V021_ROW + "\n"
        reg_path.write_text(new_reg, encoding="utf-8")
        print(f"  Appended v0.21 row ({len(V021_ROW)} chars)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
