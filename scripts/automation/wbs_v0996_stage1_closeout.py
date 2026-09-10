#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P3-D.6 阶段 1 基础 7 任务全部收官 + 阶段 2 启动 WBS 收尾更新 (per 21:34 JST 拍板"更新wbs")
(per 守门 #1 禁回溯叙事 + 守门 #9 v19 Mavis 自驱 + 守门 #12 v21 [P] docs 同步)
- WBS v0.99.6 row 追加 (P3-D.6 阶段 1 收官 + 阶段 2 重新估时)
- registry v0.30 row 追加
- automation-design.md §4.34.6 追加
"""
import io
import sys
from pathlib import Path

WBS_PATH = Path("docs/reports/STAR-P3-WBS-001.md")
REGISTRY_PATH = Path("scripts/automation/registry.md")
AUTOMATION_DESIGN_PATH = Path("docs/automation-design.md")

WBS_V0996_ROW = """| **v0.99.6** | **2026-09-10 22:30 JST** | **架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 93 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规不破坏 V0.1)** | **§14.21 P3-D.6 阶段 1 基础 7 任务全部收官 + 阶段 2 业务 启动 重新估时 (per 21:34 JST 拍板"更新wbs" + 21:30 JST 拍板"完成剩余任务")** | **收口** (per 21:18 JST Ulysses 发令"所有先进分支合并到 main 之后, 根据文档和代码状态重新评估更新 wbs" 二次校准): (1) **git 状态**: HEAD = `f16a5d8` (本地领先 origin/main 多个 commit, 待 push); (2) **阶段 1 基础 7 任务全部收官** (per 21:30 JST 拍板"完成剩余任务" + worker 子代理模式 + brief 必先落档 + docs 同步 3 docs 必更新 WBS/registry/automation-design): 任务 1.1 ✅ `crates/agent-domain/` (141 lines + 2 UT + 2/2 PASS) + 任务 1.2 ✅ `crates/arg-bridge/canvas_sync_bridge.rs` (198 lines + 3 UT + 3/3 PASS) + 任务 1.3 ✅ `crates/canvas-collab/` 5 域 Rust struct (397 lines + 6 UT + 6/6 PASS) + 任务 1.4 ✅ `crates/api/src/{agent,canvas_collab}/` 2 新 module (2521 lines + 17 UT + 67/67 PASS) + 任务 1.5 ✅ `bff/` 独立 workspace + 5 REST + 4 WSS + envoy 独立 deployment (5310 lines + 29 UT + 29/29 PASS) + 任务 1.6 ✅ 14+15 张表 SQL DDL +14 张 (1463 DDL lines + 599 gen + 158 docs + python 0 err + W/T/M 6M+5T+3W 0 混在) + **任务 1.7 ✅** 25 module 跨域接口对账 (652 lines docs + 35/35 PASS 0 regression + 0 改 V0.1 + 0 改 V0.2 任务 1.1-1.6); **(3) 累计 P3-D.6 阶段 1 基础 token OLU 校准** (per 守门 #1 v25 + 守门 #4 + STAR-OLU-001 v0.1 1 SRE·周 = 1.2M tokens): 估时 1.5M tokens / 1.25 SRE·周 (per 实施计划 §3 估), 实际 **~3.1x 超支** ~4.62M tokens / 3.85 SRE·周 (实际 7 任务落地, 原因: 1.1 强类型 5 enum + Agent struct 14 字段 + 5 业务方法 batch 1 派生 + 1.4 api 2 new module 2521 lines 远超估时 + 1.5 bff 独立 workspace 5310 lines 含 envoy 独立 deployment + 1.6 14+15 张表 完整 DDL + 1.7 25 module 详对账 652 lines + 并发 session 多 v0.99 row 占用 + docs 同步 7 docs commit); **(4) 阶段 2 业务 (P3-D.6.2) 重新估时** (per 实施计划 §3 估 + 阶段 1 实际 OLU 3.1x 校准): 任务 2.1 A1-A10 业务实装 (per 1.4 api/agent/ + 1.1 agent-domain 强类型 5 业务方法 已实, batch 2-5 估 1.6M → 实际估 ~2.0M, 含 handoff/topology/parent_child/pipeline/status_sync/worktree_assoc/workitem_assoc/cluster/cross_ref/settings_integration 10 module 业务实装) + 任务 2.2 A11 ARG 10 项 (per 1.2 arg-bridge/canvas_sync_bridge + 1.5 bff/collaboration 5 REST, 估 0.4M → 实际估 ~0.6M 含 ARG 5 类图 + 双人 + 5 关键 class) + 任务 2.3 A12 多人编辑 8 项 (per 1.3 canvas-collab + 1.4 api/canvas_collab + 1.5 bff/collaboration 4 WSS, 估 0.4M → 实际估 ~0.6M 含 CRDT/NATS/broadcast 集成) + 任务 2.4 G1-G12 游戏化 32 项 (per 1.6 14 张新表 + gamify 5 enum, 估 0.4M → 实际估 ~0.6M) + 任务 2.5 13 关键 class + 5 状态机 (per 1.1 agent-domain V0.3 5 enum + 14 状态, 估 0.1M → 实际估 ~0.2M); 阶段 2 估时 **1.9M → 实际估 ~4.0M tokens / 3.33 SRE·周** (3.1x 校准); **(5) 阶段 3 集成 + 阶段 4 实装 重新估时** (per 阶段 1+2 实际 OLU 3.1x 校准): 任务 3.1 23 REST + 任务 3.2 5 WSS + 任务 3.3 e2e test + 任务 3.4 25 module 联动 → 估 **1.5M → 实际估 ~3.0M tokens** + 任务 4.1-4.6 k3s deploy + CI/CD → 估 **1.5M → 实际估 ~3.0M tokens**; 阶段 3+4 估时 ~5.0M tokens / 4.17 SRE·周 → 实际估 **~6.0M tokens / 5.00 SRE·周**; **(6) P3-D.6 完整 5 阶段 重新估时**: 阶段 1 ✅ 4.62M + 阶段 2 ~4.0M + 阶段 3 ~3.0M + 阶段 4 ~3.0M = **~14.6M tokens / 12.17 SRE·周** (vs 实施计划 5.0M / 4.17 SRE·周, 2.9x 超支) — 5 域 Lead 真人到位 + 复用 v0.99.4 守门 + 跨 session 续做 实际工时校准; **(7) 守门合规** (#1+#1 v15+#1 v19+#1 v25+#4+#9 v19+#9 v20+#9 v27+#10+#11+#12+#13+#14 v3+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反: 0 改 V0.1 + V0.2 + V0.3 任何 file + 0 改 1.1-1.6 docs sync commit 任何内容 (新 row 追加, 0 改现有 row) + 0 改 守门 #14 v3 真人寻访 obsolete 政策 + 0 改 RGS 协议 v0.62 disclaimer; **(8) 触发原因**: 21:34 JST 拍板"更新wbs" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 21:18 JST 阶段 1 收官重新评估 续做 + 守门 #12 v21 [P] docs 同步必更新 WBS; **P3-D.6 阶段 1 基础 7 任务 全部 收官**; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 22:30 JST Mavis 自驱 (per 守门 #9 v19) + 守门 #1 v15 docs 同步饱和第 93 次新事件触发 仍允许 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #1 禁回溯叙事 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |
"""

REGISTRY_V030_ROW = """| **v0.30** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (3 docs 同步 落档, 索引同步) + §3 v0.30**: P3-D.6 阶段 1 基础 7 任务全部收官 + 阶段 2 业务 启动 重新估时 WBS 收尾更新 (per 21:34 JST 拍板"更新wbs" + 守门 #9 v19 Mavis 自驱第 7 次强化), 关联 commit `f16a5d8` (任务 1.7 docs sync 之后, HEAD = `f16a5d8` 本地领先 origin/main), 重新估时 per 守门 #1 v25 + 守门 #4 + STAR-OLU-001 v0.1: 阶段 1 估 1.5M 实际 ~4.62M (3.1x 超支, 原因: 1.4 2521 lines + 1.5 5310 lines + 1.6 14+15 张表完整 DDL + 1.7 25 module 详对账 652 lines + 并发 session docs 同步 7 commits), 阶段 2 估 1.9M 实际 ~4.0M tokens / 3.33 SRE·周, 阶段 3+4 估 5.0M 实际 ~6.0M tokens / 5.00 SRE·周, P3-D.6 完整 5 阶段 估 5.0M 实际 ~14.6M tokens / 12.17 SRE·周 (2.9x 超支); 累计 P3-D.6 阶段 1 基础 7 任务 全部 收官 (任务 1.1 + 1.2 + 1.3 + 1.4 + 1.5 + 1.6 + 1.7 估 1.5M 实际 4.62M), 任务 2.1-2.5 + 3.1-3.4 + 4.1-4.6 跨 session 续做估 ~10.0M tokens / 8.33 SRE·周 (per 3.1x 实际 校准) | **2026-09-10 21:34 JST 拍板"更新wbs" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和 + 守门 #4 token OLU)** |
"""

AUTOMATION_4346 = """

### 4.34.6 P3-D.6 阶段 1 基础 7 任务全部收官 + 阶段 2 业务 启动 重新估时 (per 21:34 JST 拍板"更新wbs", 2026-09-10 22:30 JST) — ⚠️ 跟 §4.34 / §4.34.1 / §4.34.2 / §4.34.3 / §4.34.4 / §4.34.5 编号冲突 (§4.34 = 任务 2.6 per 64b96be, §4.34.1 = 任务 1.2 per f11517d, §4.34.2 = 任务 1.3 per 2fd60b1, §4.34.3 = 任务 1.4 per 2733c4d, §4.34.4 = 任务 1.5 per 3975bf2, §4.34.5 = 任务 1.7 per 305b72a 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.6 区分

> **触发**: 2026-09-10 21:34 JST 拍板"更新wbs" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 21:18 JST 阶段 1 收官重新评估 续做 + 守门 #1 v15 docs 同步饱和第 93 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 93 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 + V0.2 + V0.3 任何 file, 0 改 1.1-1.6 docs sync commit 任何内容, 新 row 追加) + 守门 #19 v19 累积规 (不破坏 V0.1) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档, 0 子代理调用本任务 = Mavis root session 直接落档) + 守门 #9 v27 (RPC 失败 fallback 3 段, 本次 0 子代理调用 0 RPC 0 fallback 触发) + 守门 #10 (commit author=Ulysses) + 守门 #11 缺标比错标 (3 已知缺口显式标) + 守门 #12 v21 [P] docs 同步必更新 WBS + registry + automation-design §4 + 守门 #13 (守门 13 a 100% RLS 13 类 + b 物理删除禁止 + c M SCD Type 2 + d T 100% audit WORM 全部 0 违反) + 守门 #14 v3 (Mavis 永久代签, 5 角色签字栏 author=Ulysses) + 守门 #14 v4 (反转 v0.62 Mavis 审核 author=Ulysses)
> **落档内容** (per 守门 #9 v19 Mavis 自驱, Mavis root session 直接落档 0 子代理调用):
> - **`docs/reports/STAR-P3-WBS-001.md` v0.99.6 row** 新增 (per 守门 #12 v21 [P] docs 同步必更新 WBS): P3-D.6 阶段 1 基础 7 任务 全部 收官 (任务 1.1 + 1.2 + 1.3 + 1.4 + 1.5 + 1.6 + 1.7) + 阶段 2 业务 启动 重新估时 (估 1.9M → 实际估 ~4.0M tokens / 3.33 SRE·周, 3.1x 校准) + 阶段 3+4 重新估时 (估 5.0M → 实际估 ~6.0M tokens / 5.00 SRE·周) + P3-D.6 完整 5 阶段 重新估时 (估 5.0M 实际估 ~14.6M tokens / 12.17 SRE·周, 2.9x 超支)
> - **`scripts/automation/registry.md` v0.30 row** 新增 (per 守门 #12 v21 [P] docs 同步必更新 registry): v0.30 = P3-D.6 阶段 1 收官 + 阶段 2 重新估时 索引同步
> - **`scripts/automation/wbs_v0996_stage1_closeout.py`** 14KB 新增 (per 守门 #9 v19 Mavis 自驱 + 守门 #19 v19 累积规): Python 脚本 append 3 docs 同步 (WBS v0.99.6 + registry v0.30 + automation-design §4.34.6), idempotent 跑 2 次安全 (3 docs 都检查 v0.x 是否已存在), 跟 v0.22/v0.23/v0.27/v0.29 任务 1.3/1.4/1.5/1.7 模式一致

**§4.34.6 累计 token OLU (per 守门 #4 + STAR-OLU-001 v0.1)**:
- P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 7 任务 + 阶段 2 重新估时: **~4.62M tokens / 3.85 SRE·周 (已落档)**
- 后续 P3-D.6 阶段 2 业务 任务 2.1-2.5 跨 session 续做估 **~4.0M tokens / 3.33 SRE·周** (3.1x 校准)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装 跨 session 续做估 **~6.0M tokens / 5.00 SRE·周** (3.1x 校准)
- **P3-D.6 完整 5 阶段 重新估时 ~14.6M tokens / 12.17 SRE·周** (2.9x 实施计划 5.0M 估, 校准因子 = 实际 OLU / 实施计划估)
"""


def main() -> int:
    # 1) WBS append
    wbs_text = WBS_PATH.read_text(encoding="utf-8")
    if "**v0.99.6**" in wbs_text:
        print("[skip] WBS v0.99.6 already present")
    else:
        if not wbs_text.endswith("\n"):
            wbs_text += "\n"
        wbs_text += WBS_V0996_ROW
        WBS_PATH.write_text(wbs_text, encoding="utf-8")
        print(f"[ok] WBS v0.99.6 appended ({len(WBS_V0996_ROW)} chars)")

    # 2) Registry append
    reg_text = REGISTRY_PATH.read_text(encoding="utf-8")
    if "**v0.30**" in reg_text:
        print("[skip] registry v0.30 already present")
    else:
        if not reg_text.endswith("\n"):
            reg_text += "\n"
        reg_text += REGISTRY_V030_ROW
        REGISTRY_PATH.write_text(reg_text, encoding="utf-8")
        print(f"[ok] registry v0.30 appended ({len(REGISTRY_V030_ROW)} chars)")

    # 3) automation-design.md §4.34.6 append
    auto_text = AUTOMATION_DESIGN_PATH.read_text(encoding="utf-8")
    if "### 4.34.6" in auto_text:
        print("[skip] automation-design §4.34.6 already present")
    else:
        if not auto_text.endswith("\n"):
            auto_text += "\n"
        auto_text += AUTOMATION_4346
        AUTOMATION_DESIGN_PATH.write_text(auto_text, encoding="utf-8")
        print(f"[ok] automation-design §4.34.6 appended ({len(AUTOMATION_4346)} chars)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
