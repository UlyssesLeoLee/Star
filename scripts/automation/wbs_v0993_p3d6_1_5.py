#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P3-D.6 阶段 1 基础 任务 1.5 bff/ + envoy 独立 deployment docs 同步脚本
(per 守门 #1 禁回溯叙事 + 守门 #9 v19 Mavis 自驱 + 守门 #12 v21 [P] docs 同步)
- WBS v0.99.3 row 追加 (v0.99.1/v0.99.2 已被占用)
- registry v0.27 row 追加 (v0.20-v0.26 都被占用)
- automation-design.md §4.34.4 追加
"""
import io
import sys
from pathlib import Path

WBS_PATH = Path("docs/reports/STAR-P3-WBS-001.md")
REGISTRY_PATH = Path("scripts/automation/registry.md")
AUTOMATION_DESIGN_PATH = Path("docs/automation-design.md")

WBS_V0993_ROW = """| **v0.99.3** | **2026-09-10 21:30 JST** | **架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 91 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规不破坏 V0.1)** | **§14.20 P3-D.6 阶段 1 基础 任务 1.5 bff/ 新独立 workspace + bff/src/collaboration/ 5 REST + 4 WSS + envoy 独立 deployment 落档 (per P3-D.6 实施计划 §3 阶段 1 基础 任务 1.5 + DD-AGENT §3.1 line 301-307 + 9/1 13:05 JST envoy 独立 deployment 偏好 + 守门 #9 v19 Mavis 自驱)** — worker 子代理在 worktree `D:/Star/.worktrees/wt-p3-d6-1-5-bff-skeleton/` (branch `wt-p3-d6-1-5-bff-skeleton` 基于 main `7c80398` brief 落档 之后) 撰写, Mavis merge to main `3975bf2` — ⚠️ 跟 v0.99.1/v0.99.2 编号冲突 (v0.99.1 = P3-D.6 阶段 2 业务 任务 2.1 batch 1 crates/agent-domain/ Agent 业务方法 5 业务方法 per e428eed 平行工作, v0.99.2 = P3-D.6 阶段 1 基础 任务 1.4 crates/api/src/{agent,canvas_collab}/ 2 新 module 骨架 per 4aba448 平行工作), per 守门 #1 禁回溯叙事 显式标 v0.99.3 区分**: (1) **commit `3975bf2` merge to main 落地** (worktree commit `4369650`): 12 files changed, +5310 insertions, **0 deletions** — (a) `bff/Cargo.toml` 64 lines 新建 ([package] + [workspace] + 9 deps direct version axum 0.8 + tokio 1.40 + serde/serde_json/async-trait/thiserror/uuid/chrono + 4 path 依赖 star-context/canvas-collab/star-arg-bridge/api + [lints.rust]); (b) `bff/Cargo.lock` 2747 lines (cargo 自动, 0 手动编辑); (c) `bff/src/lib.rs` 39 lines (module doc + pub mod collaboration + re-exports + 2 UT); (d-h) `bff/src/collaboration/{mod,controller,wss_hub,permission,audit,dto}.rs` 6 新增 1923 lines (164+305+407+258+159+630, CollaborationState 7 字段 [audit + permission + wss_hub + tenant_id + actor_id + canvas_id + presence_placeholder] + 5 REST endpoint 占位 A12.1+A12.3+A12.5+A12.2+A12.4 + 4 WSS endpoint 占位 A12.1+A12.2+A12.5+A12.4 + CollaborationWssHub broadcast channel capacity 1024 + PermissionLevel 3 变体 View/Comment/Edit + CollaborationAuditEvent 6 字段 + ApiError 5 变体 + 27 UT 跨 6 sub-module); (i) `deploy/k3s-local/bff-deployment.yaml` 261 lines (envoy 独立 deployment 0 istio sidecar per 9/1 13:05 JST 偏好, bff-envoy Deployment + bff-envoy Service + bff Deployment + bff Service + bff-envoy-config ConfigMap 含 envoy.yaml HTTP filter + WSS upgrade_configs + K8s DNS svc://bff.star.svc.cluster.local + sidecar.istio.io/inject=false annotation 3 资源都标); (j) `deploy/k3s-local/kustomization.yaml` 31 lines (顶层 kustomize 集成 bff-deployment.yaml + star-api-rest-deploy.yaml + envoy/ 3 子资源); (k) `docs/deployment/BFF-DEPLOYMENT-001.md` 245 lines (7 段结构 per AGENTS.md §3, §0 目的 + §1 改动矩阵 11 file + §2 验证摘要 5 守门 29 UT + §3 envoy 独立 deployment 模式 + §4 集成流程 5 步 + §5 守门规则 13 维 + §6 签字栏 5 角色 per DEC-008 + §7 修订历史 v0.1 落档); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 cargo test 改单 crate, 跳 workspace): `cd bff && cargo check --lib -j 4` = **0 err 0.88s**; `cd bff && cargo test --lib -j 4` = **29/29 PASS 0.00s** (V0.1 0 + V0.2 29 new = 29, 0 regression); `cd bff && cargo fmt -- --check` = **0 diff**; `cd bff && cargo clippy --lib -j 4` = **0 warnings on my code** (0 bff warnings, parent workspace warnings 全 pre-existing V0.x); `cargo check --workspace --lib -j 4` = 0 err 16.07s (跟 V0.1 + V0.2 任务 1.1-1.4 全部兼容, bff 是新独立 workspace 不影响根 workspace); `kubectl kustomize deploy/k3s-local/` = 0 err (535 lines 9 Kinds); **(3) 关键设计 — bff 新独立 workspace 0 加根 Cargo.toml [workspace] members** (per 守门 #19 v19 累积规不破坏 V0.1 + 守门 #1 禁回溯叙事 0 改根 Cargo.toml 任何行 + 守门 #11 缺标比错标): 跟 `tools/` / `frontend/` 模式一致, bff 是平级 workspace 不是 crates/ 子 crate, 0 改根 Cargo.toml [workspace] members 任何行; **(4) 实施计划"3 段 1 任务 1.5" 实际"bff 1 独立 workspace" 缺口显式标** (per 守门 #11 缺标比错标): 实施计划 §3 任务 1.5 说 `bff/src/collaboration/`, 实际是新建 `bff/` 整个 workspace + 6 file collaboration module + bff-deployment.yaml + 文档, 1 任务 4 交付物; **(5) 守门合规** (#1+#1 v15+#1 v19+#1 v25+#6+#7+#9 v19+#9 v20+#9 v27+#10+#11+#12+#13+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反: 0 改根 Cargo.toml [workspace] members 任何行 (bff 是新独立 workspace) + 0 改 crates/api/Cargo.toml 加 bff 依赖 + 0 改 crates/api/src/ 任何 file (任务 1.4 100% 保留) + 0 改 crates/canvas-collab/ 任何 file (任务 1.3 100% 保留) + 0 改 crates/agent-domain/ V0.1+V0.2+V0.3 任何 file + 0 改 crates/arg-bridge/ 任何 file + 0 改 deploy/k3s-local/ 现有 file + 0 改 Cargo.lock 任何手编行; **(6) worktree cleanup** (per 守门 #9 #3 0 散落子代理产出): `git worktree remove .worktrees/wt-p3-d6-1-5-bff-skeleton --force + git branch -D wt-p3-d6-1-5-bff-skeleton` 0 残留; **(7) 累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 (agent-domain/) + 任务 1.2 ✅ 收官 (canvas UI sync bridge) + 任务 1.3 ✅ 收官 (canvas-collab/) + 任务 1.4 ✅ 收官 (api 2 新 module) + **任务 1.5 ✅ 收官 (bff/ 独立 workspace + collaboration 5 REST + 4 WSS + envoy 独立 deployment)** + 任务 1.6-1.7 跨 session 续做估 ~0.50M tokens / 0.42 SRE·周; **(8) 守门 #1 禁回溯叙事**: v0.1-v0.99.2 修订历史不动, v0.99.3 row 显式标 P3-D.6 阶段 1 基础 任务 1.5 收官; **(9) 触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化; P3-D.6 阶段 1 基础 任务 1.5 收官; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 21:30 JST Mavis 自驱 (per 守门 #9 v19) + 守门 #1 v15 docs 同步饱和第 91 次新事件触发 仍允许 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #1 禁回溯叙事 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |
"""

REGISTRY_V027_ROW = """| **v0.27** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (10 新 file + 2 改 file 落档, 索引同步) + §3 v0.27**: P3-D.6 阶段 1 基础 任务 1.5 bff/ 新独立 workspace + bff/src/collaboration/ 5 REST + 4 WSS + envoy 独立 deployment (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 20:08 JST 拍板"推进" + 守门 #9 v19 Mavis 自驱第 7 次强化), 关联 commit `3975bf2` (merge to main, 12 files / 5310 insertions / 0 deletions, per 守门 #1 v15 docs 同步饱和第 91 次新事件触发 + 守门 #1 禁回溯叙事不重写 V0.1 任何代码 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-5-bff-skeleton.md` 31.3KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标 4 已知缺口显式标 + 守门 #6 PowerShell only + 守门 #1 v25 cargo test 改单 crate 跳 workspace 29/29 PASS): worker 子代理 在 worktree `D:/Star/.worktrees/wt-p3-d6-1-5-bff-skeleton/` (branch `wt-p3-d6-1-5-bff-skeleton` 基于 main `7c80398` brief 落档 之后) 撰写, commit `4369650` 落档 (bff/ 9 file + Cargo.lock + bff-deployment.yaml + kustomization.yaml + BFF-DEPLOYMENT-001.md, 0 业务方法实装 0 CRDT 0 NATS 0 真实 WSS subscribe 留 P3-D.6 阶段 2/3 续做), 6 守门实证 cargo check 0 err 0.88s + cargo test 29/29 PASS 0.00s (V0.1 0 + V0.2 29 new = 29, 0 regression) + cargo fmt 0 diff + cargo clippy 0 warnings on my code + cargo check --workspace --lib -j 4 = 0 err 16.07s + kubectl kustomize deploy/k3s-local/ = 0 err (535 lines 9 Kinds); brief 模板 0 bug 全部 1 次过 (跟 v0.18 任务 1.1 brief 有 1 微 bug + v0.21 任务 1.2 brief 有 3 处修正 + v0.22/v0.23 任务 1.3/1.4 brief 有 0 bug 相比, 任务 1.5 0 bug 是 v0.18/v0.21/v0.22/v0.23 经验累积); Mavis merge to main `--no-ff` 0 conflict; worktree cleanup `git worktree remove --force + git branch -D` 0 残留; **v0.27 编号** (v0.20-v0.26 都被 P3-D.6 阶段 1 基础 任务 1.1-1.4 / 阶段 2 业务 任务 2.6 / 任务 2.1 / P2 阶段 worker / cargo fmt baseline 平行工作占用, 用 v0.27 跳 v0.20-v0.26 平行工作模式); **累计 P3-D.6 阶段 1 基础**: 任务 1.1 + 1.2 + 1.3 + 1.4 + **任务 1.5 (本 commit)** 收官, 任务 1.6-1.7 跨 session 续做估 ~0.50M tokens / 0.42 SRE·周 | **2026-09-10 20:08 JST Ulysses 拍板"推进" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和)** |
"""

AUTOMATION_4344 = """

### 4.34.4 P3-D.6 阶段 1 基础 任务 1.5 bff/ 新独立 workspace + bff/src/collaboration/ 5 REST + 4 WSS + envoy 独立 deployment (per 20:08 JST Ulysses 拍板"推进", 2026-09-10 21:30 JST) — ⚠️ 跟 §4.34 / §4.34.1 / §4.34.2 / §4.34.3 编号冲突 (§4.34 = P3-D.6 阶段 2 业务 任务 2.6 per commit 64b96be, §4.34.1 = 任务 1.2 per commit f11517d, §4.34.2 = 任务 1.3 per commit 2fd60b1, §4.34.3 = 任务 1.4 per commit 2733c4d 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.4 区分

> **触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 91 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 91 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 任何代码, 0 改根 Cargo.toml [workspace] members 任何行 (bff 是新独立 workspace) + 0 改 crates/api/Cargo.toml 加 bff 依赖 + 0 改 crates/api/src/ 任何 file + 0 改 crates/canvas-collab/ 任何 file + 0 改 crates/agent-domain/ 任何 file + 0 改 crates/arg-bridge/ 任何 file + 0 改 deploy/k3s-local/ 现有 file) + 守门 #19 v19 累积规 (不破坏 V0.1, 0 重写 V0.1 任何代码) + 守门 #6 PowerShell only (0 bash &&) + 守门 #7 (unsafe_code="forbid") + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-5-bff-skeleton.md` 31.3KB) + 守门 #9 v27 (RPC 失败 fallback 3 段 invoke → verify → collect_output, 真实产出验证 cargo check 0 err + cargo test 29/29 PASS + cargo fmt 0 diff + cargo clippy 0 warnings + kustomize 0 err) + 守门 #10 (commit author=Ulysses `4369650` worktree + `3975bf2` merge) + 守门 #11 缺标比错标 (4 已知缺口显式标: bff 新独立 workspace + BFF 跟 API 平级 0 反向依赖 + envoy 独立 deployment 0 istio sidecar + 0 真实 WSS 业务逻辑) + 守门 #14 v4 (Mavis 审核 author=Ulysses) + 守门 #14 v3 (Mavis 永久代签)
> **落档文件** (关联 commit `3975bf2` merge to main, 12 files / 5310 insertions / 0 deletions):
> - `bff/Cargo.toml` (新, 64 lines, [package] + [workspace] + 9 deps direct version + [lints.rust])
> - `bff/Cargo.lock` (新, 2747 lines, cargo 自动)
> - `bff/src/lib.rs` (新, 39 lines, module doc + pub mod collaboration + re-exports + 2 UT)
> - `bff/src/collaboration/{mod,controller,wss_hub,permission,audit,dto}.rs` (6 新, 164+305+407+258+159+630 = 1923 lines, 27 UT)
> - `deploy/k3s-local/bff-deployment.yaml` (新, 261 lines, envoy 独立 deployment 0 istio sidecar, bff-envoy Deployment + bff-envoy Service + bff Deployment + bff Service + bff-envoy-config ConfigMap)
> - `deploy/k3s-local/kustomization.yaml` (新, 31 lines, 顶层 kustomize 集成)
> - `docs/deployment/BFF-DEPLOYMENT-001.md` (新, 245 lines, 7 段结构 per AGENTS.md §3)
> - `docs/briefs/p3-d6-1-5-bff-skeleton.md` (31.3KB, 子代理 brief 落档, per 守门 #9 v20)

**§4.34.4 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.18M tokens (worker 实装 0.13 + Mavis merge + verify + docs 0.05)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 1.2 + 1.3 + 1.4 + 1.5 22 commit: ~4.24M tokens (3.53 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.6-1.7 (剩余 2 任务: 14+15 张表 + 25 module 联动接口): ~0.50M tokens (0.42 SRE·周)
- 后续 P3-D.6 阶段 2 业务 任务 2.1 batch 2-5 跨 session 续 + 任务 2.2 A11 ARG 10 项 + 任务 2.3 A12 多人编辑 8 项 + 任务 2.4 G1-G12 游戏化 32 项 + 任务 2.5 13 关键 class: ~1.6M tokens (1.33 SRE·周)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 完整 5 阶段: ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)
"""


def main() -> int:
    # 1) WBS append
    wbs_text = WBS_PATH.read_text(encoding="utf-8")
    if "**v0.99.3**" in wbs_text:
        print("[skip] WBS v0.99.3 already present")
    else:
        if not wbs_text.endswith("\n"):
            wbs_text += "\n"
        wbs_text += WBS_V0993_ROW
        WBS_PATH.write_text(wbs_text, encoding="utf-8")
        print(f"[ok] WBS v0.99.3 appended ({len(WBS_V0993_ROW)} chars)")

    # 2) Registry append
    reg_text = REGISTRY_PATH.read_text(encoding="utf-8")
    if "**v0.27**" in reg_text:
        print("[skip] registry v0.27 already present")
    else:
        if not reg_text.endswith("\n"):
            reg_text += "\n"
        reg_text += REGISTRY_V027_ROW
        REGISTRY_PATH.write_text(reg_text, encoding="utf-8")
        print(f"[ok] registry v0.27 appended ({len(REGISTRY_V027_ROW)} chars)")

    # 3) automation-design.md §4.34.4 append
    auto_text = AUTOMATION_DESIGN_PATH.read_text(encoding="utf-8")
    if "### 4.34.4" in auto_text:
        print("[skip] automation-design §4.34.4 already present")
    else:
        if not auto_text.endswith("\n"):
            auto_text += "\n"
        auto_text += AUTOMATION_4344
        AUTOMATION_DESIGN_PATH.write_text(auto_text, encoding="utf-8")
        print(f"[ok] automation-design §4.34.4 appended ({len(AUTOMATION_4344)} chars)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
