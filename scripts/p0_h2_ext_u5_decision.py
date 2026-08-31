#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""H2-EXT #5 拍板 + 4 项 Ulysses 拍板结果写入 HANDOFF v0.5 / AGENTS v0.28"""
from pathlib import Path

REPO = Path(r"D:/Star")

# === HANDOFF v0.5 ===
HANDOFF = REPO / "HANDOFF-ST-001.md"
text = HANDOFF.read_text(encoding="utf-8")

# 在 v0.4 后追加 v0.5
v04_marker = "| v0.4 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | H2-EXT #1-#3 落地 (3 commits: 9d08f80 / b6f6e2a / 7f611b0), 净修 507 err (797 → 290, 跨 9 crate); 守门 #1 实证: star-context 21/21 pass + workspace --lib 0 err + H2-EXT 3/5 完成; H2-EXT #4 domain-identity (DeviceId→Uuid 重构) + #5 domain-work-item (String→Uuid 需 Ulysses 拍板 String 原义) 跨 session 续; session 至此收尾 (per 2026-09-01 07:56 JST 新 session 启动, 2026-09-01 09:50 JST 收尾) |"
v05_row = """| v0.5 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 4 项 Ulysses 拍板 (Q1 device_id String=hostname 业务语义 + Q2 #4 跨 session 续 + Q3 H2 原 3 domain 跨 session 续 + Q4 P0-2/3/4 跨 session 续); H2-EXT #5 String=hostname 拍板: 不重设为 Uuid, entity 保留 String 类型, 0 token type 改; #5 其他改造 (context.rs 删除 + port/service dead import) 估 0.05M 跨 session 续; session 至此收尾 (per 2026-09-01 08:32 JST 拍板, token 1.95M/2.0M = 97% 紧) |"""

if v04_marker in text:
    text = text.replace(v04_marker, v04_marker + "\n" + v05_row)
    print("[OK] HANDOFF v0.5 row 追加")

# 加 §7 拍板记录
v05_section = """

---

## §7 4 项 Ulysses 拍板记录 (2026-09-01 08:32 JST, ask_user 4-step questionnaire)

| # | 决策点 | Ulysses 选择 | 落地影响 |
|---|---|---|---|
| Q1 | H2-EXT #5 domain-work-item `device_id: Option<String>` 业务语义 | **hostname (设备主机名)** | entity 保留 String 类型, 不重设为 Uuid, 0 token type 改. #5 改造简化: 仅删 context.rs + port/service dead import (估 0.05M) |
| Q2 | H2-EXT #4 domain-identity (DeviceId 强类型 → Uuid) | **(a) 跨 session 续** | 估 0.2M token 跨 session 续. 入口 = HANDOFF v0.4 §6 |
| Q3 | H2 原 3 domain (feedback/validation/integration) service.rs 改造 | **(a) 跨 session 续** | 估 0.6-0.8M token 跨 session 续. 入口 = HANDOFF v0.3 §5.1 #6 |
| Q4 | P0-2/3/4 (ApiError + application + infrastructure) | **(a) 跨 session 续** | 估 1.3M token 跨 session 续. 入口 = HANDOFF v0.3 §5.2 |

**session token 1.95M/2.0M (97%) 紧张**, 4 项全部"跨 session 续"是默认安全选项, 符合守门 #1 阶段 1 实证已经收官 (--lib 0 + clippy 0 + fmt 0 + 21/21 test).

**5 项 Blocker 更新** (per HANDOFF v0.3 §5.3):
1. ✅ H2-EXT #5 String 业务语义已拍板 = hostname (无需 type 改, 仅 context 子模块删除)
2. ⏳ H2-EXT #4 DeviceId → Uuid 重构: 跨 session 续
3. ⏳ H2 原 3 domain service.rs 改造: 跨 session 续
4. ⏳ 5 域 Lead 真人到位: 等 Ulysses
5. ⏳ P3-B 拍板: B.5 OpenClaw / B.6 Hermes 凭证
"""

if "## §7 4 项 Ulysses 拍板记录" not in text:
    text = text + v05_section
    print("[OK] HANDOFF §7 拍板记录追加")
HANDOFF.write_text(text, encoding="utf-8")
print("[OK] HANDOFF v0.5 完成")

# === AGENTS v0.28 ===
AGENTS = REPO / "AGENTS.md"
agents_text = AGENTS.read_text(encoding="utf-8")

v27_marker = "| v0.27 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | H2-EXT #1-#3 落地 (3 commits: 9d08f80 + b6f6e2a + 7f611b0), 守门 #1 v1 实证 --all-targets 797 → 290 err (净修 507, 跨 9 crate 分布); star-context 21/21 pass + workspace --lib 0 err + clippy 0 err + fmt exit 0 实证; H2-EXT #4 domain-identity (DeviceId→Uuid) + #5 domain-work-item (String→Uuid 需 Ulysses 拍板) 跨 session 续 (估 0.4M token); session 至此收尾 |"
v28_row = """| v0.28 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 4 项 Ulysses 拍板 (per ask_user 4-step questionnaire) + H2-EXT #5 String=hostname 业务语义拍板落地 (无需 type 改, 简化 0.05M 改造); Q2/Q3/Q4 全部跨 session 续 (per session token 1.95M/2.0M = 97% 紧); 5 项 Blocker 更新: #5 业务语义已拍板, #4 #H2原3 #P0-2/3/4 跨 session 续; session 至此收尾 |"""

if v27_marker in agents_text:
    agents_text = agents_text.replace(v27_marker, v27_marker + "\n" + v28_row)
    print("[OK] AGENTS v0.28 row 追加")

# 加 §4 守门 #3 拍板备注 (Q1 是 #5 String 业务语义, 跟 DDD 业务语义相关, 跟 #3 5 域独立 Lead 无关)
# 但 #5 domain-work-item 业务语义属 P0/P1 拍板范畴, 不动 #3
# Q2/Q3/Q4 跨 session 续决策属 Q10-P token 预算同类型, 不需 #3

AGENTS.write_text(agents_text, encoding="utf-8")
print("[OK] AGENTS v0.28 完成")
