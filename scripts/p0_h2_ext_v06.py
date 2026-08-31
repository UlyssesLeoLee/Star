#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""HANDOFF v0.6: Ulysses 拍板"所有"任务规划, 总 4-5M token 跨 4-6 session 续"""
from pathlib import Path

REPO = Path(r"D:/Star")

# === HANDOFF v0.6 ===
HANDOFF = REPO / "HANDOFF-ST-001.md"
text = HANDOFF.read_text(encoding="utf-8")

v05_marker = "| v0.5 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 4 项 Ulysses 拍板 (Q1 device_id String=hostname 业务语义 + Q2 #4 跨 session 续 + Q3 H2 原 3 domain 跨 session 续 + Q4 P0-2/3/4 跨 session 续); H2-EXT #5 String=hostname 拍板: 不重设为 Uuid, entity 保留 String 类型, 0 token type 改; #5 其他改造 (context.rs 删除 + port/service dead import) 估 0.05M 跨 session 续; session 至此收尾 (per 2026-09-01 08:32 JST 拍板, token 1.95M/2.0M = 97% 紧) |"
v06_row = """| v0.6 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | Ulysses 拍板"所有" (per ask_user "所有"选项, 2026-09-01 08:44 JST) = H2 全量收官 + P0-2/3/4 + docs 优化 + 等真人/凭证 全部要做; 总估 4-5M token 跨 4-6 session 续; 本次 session 收尾 (token 1.95M/2.0M = 97% 紧); 入口 HANDOFF v0.6 §8 跨 session 续执行计划 |"""

if v05_marker in text:
    text = text.replace(v05_marker, v05_marker + "\n" + v06_row)
    print("[OK] HANDOFF v0.6 row 追加")

# §8 跨 session 续执行计划
v06_section = """

---

## §8 Ulysses "所有" 拍板执行计划 (per 2026-09-01 08:44 JST)

**拍板**: 所有任务都要做 (a + b + c + d 全部). 总估 4-5M token, 跨 4-6 session 续.

### 8.1 执行顺序 (按 token budget 优先级 + 依赖关系)

| 序 | 任务 | 估 token | 依赖 | session |
|---|---|---|---|---|
| 1 | **H2-EXT #5 简化** (context.rs 删除 + port/service dead import, hostname 拍板 0 type 改) | 0.05M | 无 (本 session 已完成 hostname 拍板) | session #1 |
| 2 | **H2-EXT #4** domain-identity (DeviceId 强类型 → Uuid 重构) | 0.2M | 无 (类型不兼容需 entity 改) | session #1 |
| 3 | **H2 原 3 domain** service.rs 改造 (domain-feedback 77 err 大头 + validation/integration) | 0.6-0.8M | H2-EXT #4 #5 完成 (port trait 模式统一) | session #2 |
| 4 | **守门 #1 阶段 2** --all-targets 0 err 实证 | 0.05M | H2 原 3 domain 完成 | session #2 末 |
| 5 | **P0-2** ApiError 映射 (api crate ApiError ↔ domain Error) | 0.3M | 守门 #1 阶段 2 实证 | session #3 |
| 6 | **P0-3** application crate 真实编排 (跨域 service 调用) | 0.6M | P0-2 完成 | session #4 |
| 7 | **P0-4** infrastructure adapter (DB/KMS/Credential broker) | 0.4M | P0-3 完成 | session #5 |
| 8 | **守门 #1 阶段 3** (release mode test + 派生 v3) | 0.2M | P0-4 完成 | session #5 末 |
| 9 | **docs 优化** PHASE 模板标准化 + HANDOFF 自动生成 | 0.1M | 无 (跟代码独立) | session #1-6 任一 |
| 10 | **cargo doc** 实证 (守门 #1 派生 v4) | 0.05M | 无 (跟代码独立) | session #1-6 任一 |
| 11 | **5 域 Lead 真人到位** (P3-C/E/F 阻塞解除) | 0 | 等 Ulysses 真人 | (等) |
| 12 | **P3-B 拍板** B.5 OpenClaw / B.6 Hermes 凭证 | 0 | 等 Ulysses | (等) |

### 8.2 token 预算

- session #1-#5 各约 1M token
- 6 session 总 4-5M token (per STAR-OLU-001.md v0.1 1 SRE·周 = 1.2M token)
- 实际每次 session 不能超 2M token (model context window)
- 建议每次 session 1-1.5M 目标 (留 25-50% buffer)

### 8.3 跨 session 续入口

每次新 session 第一步:
```bash
# 1. 读 HANDOFF-ST-001.md v0.6 (本文件) + AGENTS.md 最新版
# 2. git pull (per 推 origin 落地)
# 3. git log --oneline -10 看最新 commit
# 4. cargo check --workspace --all-targets 重测 (per Q9-T A9 数字时效性, 必须实测, 不得沿用)
# 5. 续下一个任务 (per §8.1 顺序)
# 6. 完成后 commit + docs 同步 + HANDOFF/AGENTS 修订
```

### 8.4 session 边界守门

- 守门 #1 阶段 1 已收官 (本 session 实证 --lib 0 + clippy 0 + fmt 0 + 21/21 test)
- 守门 #1 阶段 2 待 §8.1 #4 实证 (--all-targets 0 err)
- 守门 #1 阶段 3 待 §8.1 #8 实证 (release test 100% pass)
- 守门 #15 死循环饱和约束持续生效 (新事件触发新 docs 同步)

### 8.5 风险点

1. **session token 累加**: 每次 session 1-1.5M, 跨 6 session 总 4-5M, AI context 物理限制
2. **跨域 type 风险**: H2-EXT #4 #5 + H2 原 3 domain service.rs 改造, entity / port trait / service 三层修改, 需谨慎
3. **守门 #9 实证**: 子代理 RPC 不可靠 (P3-A.6/A.7 实证), 任何委派需 git log 实证
4. **5 域 Lead 真人阻塞**: per 8/21 JST 拒绝兼任硬约束, P3-C/E/F 阶段需真人到位
5. **P3-B 凭证**: B.5 OpenClaw / B.6 Hermes 需 Ulysses 提供凭证
"""

if "## §8 Ulysses \"所有\" 拍板执行计划" not in text:
    text = text + v06_section
    print("[OK] HANDOFF §8 拍板执行计划追加")
HANDOFF.write_text(text, encoding="utf-8")
print("[OK] HANDOFF v0.6 完成")

# === AGENTS v0.29 ===
AGENTS = REPO / "AGENTS.md"
agents_text = AGENTS.read_text(encoding="utf-8")

v28_marker = "| v0.28 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 4 项 Ulysses 拍板 (per ask_user 4-step questionnaire) + H2-EXT #5 String=hostname 业务语义拍板落地 (无需 type 改, 简化 0.05M 改造); Q2/Q3/Q4 全部跨 session 续 (per session token 1.95M/2.0M = 97% 紧); 5 项 Blocker 更新: #5 业务语义已拍板, #4 #H2原3 #P0-2/3/4 跨 session 续; session 至此收尾 |"
v29_row = """| v0.29 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | Ulysses 拍板"所有" (a H2 全量收官 + b P0-2/3/4 + c docs 优化 + d 等真人/凭证) = 全部跨 session 续; 总估 4-5M token 跨 4-6 session; §7 #7-#12 跨 session 续执行计划入档 (HANDOFF v0.6 §8); 推 origin 实证 commit 541767f; session 至此真正收尾 (per 2026-09-01 08:44 JST "所有" 拍板 + 推 origin 完成) |"""

if v28_marker in agents_text:
    agents_text = agents_text.replace(v28_marker, v28_marker + "\n" + v29_row)
    print("[OK] AGENTS v0.29 row 追加")
AGENTS.write_text(agents_text, encoding="utf-8")
print("[OK] AGENTS v0.29 完成")
