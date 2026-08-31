#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""HANDOFF-ST-001 v0.3: 4 项 Ulysses 拍板结果 + 跨 session 续交接"""
import sys
from pathlib import Path

REPO = Path(r"D:/Star/HANDOFF-ST-001.md")

NEW_S3 = """## §3 Ulysses 拍板结果 (2026-08-31 22:45 JST, per ask_user 4-step questionnaire)

| # | 决策点 | Ulysses 选择 | 落地动作 |
|---|---|---|---|
| Q1-D | AGENTS.md §5"5 域独立 Lead"命名解读 | **(a)+(c) 历史命名 + disclaimer, 不映射** | AGENTS.md §4 守门 #3 + §5 仓库拓扑 双向加 disclaimer — 5 域是历史治理命名 (5 位真人 Lead 问责结构), 22 domain-* 是 DDD bounded context, 两者非同一分类, 不建立业务子域↔DDD 映射. (commit a61b85d) |
| Q10-P | P0 token 预算超支后续 | **(b) 接受 P0-1 现完成度 + 暂停跨 session 续** | 当前 session 至此收尾; 跨 session 续入口见 §5 |
| Q11-P | ST 测试层级 | **(a) 保持 3 层级** (单元 + IT + ST) | --all-targets 432 err 现状下不扩层, 维持原 3 层 |
| Q12-P | 文档治理详细程度 | **(a) 维持三层** (PHASE 报告 + Q&A 报告 + commit message) | token 预算压力下不升级, 维持原 3 层 |

---"""

OLD_S3_PREFIX = "## §3"
OLD_S4_PREFIX = "## §4"

text = REPO.read_text(encoding="utf-8")
lines = text.splitlines(keepends=True)
new_lines = []
i = 0
while i < len(lines):
    line = lines[i]
    if line.startswith(OLD_S3_PREFIX):
        # 替换整段 §3
        new_lines.append(NEW_S3 + "\n")
        # 跳到 §4 之前
        while i < len(lines) and not lines[i].startswith(OLD_S4_PREFIX):
            i += 1
        continue
    if line.startswith(OLD_S4_PREFIX):
        # 在 §4 之后追加 v0.3 + §5
        new_lines.append(line)  # ## §4 修订历史
        i += 1
        # 读取剩余直到文件结束
        remaining = "".join(lines[i:])
        # 找 v0.2 行结束位置
        v02_marker = "真实尝试脚本入档 scripts/p0_h2_3domain_migration.py |"
        v02_idx = remaining.find(v02_marker)
        if v02_idx >= 0:
            insert_pos = v02_idx + len(v02_marker)
            new_v03 = """
| v0.3 | 2026-08-31 | 架构师 (Mavis 接手 agent per DEC-008) | 4 项 Ulysses 拍板结果 (Q1-D a+c / Q10-P b / Q11-P a / Q12-P a) + 跨 session 续交接 (Q10-P b 推荐拍板) + §5 新增 "下个 session 入口" 段; 本 session 至此收尾 (per 22:45 JST 4 项拍板 + 守门 #1+#9+#12+#15 跨 stage 全过) |"""
            remaining = remaining[:insert_pos] + new_v03 + remaining[insert_pos:]
        # 追加 §5
        new_v05 = """

---

## §5 跨 session 续入口 (per Q10-P b 拍板)

**当前 session 至此收尾** (token 1.4M/2.0M = 70% 接近上限), 下次 session 续 H2-EXT 5 domain + P0-2/3/4.

### 5.1 H2-EXT 5 domain 改造顺序 (估 0.5-0.8M token)

| 顺序 | domain | 类型不兼容决策 | 字段扩展 | 估 token |
|---|---|---|---|---|
| 1 | domain-comment | 无 (context.rs 存在但 lib.rs 无 pub mod) | 无 | 0.05M |
| 2 | domain-tenant | `user_id` 已 Uuid (兼容) | 加 `tenant_policy_id: Option<Uuid>` 到 star_context | 0.1M |
| 3 | domain-project | `user_id` 已 Uuid (兼容) | 加 `workspace_ids: Vec<Uuid>` 到 star_context | 0.1M |
| 4 | domain-identity | `device_id: DeviceId` 强类型 → Uuid 重构 | 无 | 0.2M |
| 5 | domain-work-item | `device_id: Option<String>` → `Option<Uuid>` 业务语义重设 (String 是 hostname? JWT token? 需 Ulysses 拍板) | 无 | 0.2M |
| 6 | H2 原 3 domain service.rs 改造 | feedback/validation/integration service.rs 内部 ~150+ 调用点 Uuid ↔ UserId/TenantId/ProjectId 转换 | 无 | 0.6-0.8M |
| **合计** | 8 domain 全部 | | | **1.1-1.6M** |

### 5.2 P0-2/3/4 token 预算 (估 1.3M)

| 阶段 | 内容 | 估 token | 依赖 |
|---|---|---|---|
| P0-2 | ApiError 映射 (api crate 的 ApiError 跟 domain Error 双向映射) | 0.3M | H2 完成 |
| P0-3 | application crate 真实编排 (跨域 service 调用) | 0.6M | P0-2 完成 |
| P0-4 | infrastructure adapter (DB/KMS/Credential broker 等) | 0.4M | P0-3 完成 |
| **合计** | | **1.3M** | |

### 5.3 跨 session 续 Blockers (5 项)

1. **H2-EXT 类型不兼容决策**: domain-identity DeviceId→Uuid 重构, domain-work-item device_id String→Uuid 业务语义重设 (需 Ulysses 拍板 String 原义是 hostname/JWT token/其他)
2. **star_context 字段扩展**: workspace_ids (Vec<Uuid>) + tenant_policy_id (Option<Uuid>) 加到 star-context 的 ActorContext struct (跟 is_agent_session 同模式)
3. **H2 原 3 domain service.rs 改造**: feedback/validation/integration 内部 ~150+ 调用点 Uuid ↔ 强类型 ID 转换, 可选 (a) 业务侧加 UserId::from(actor.user_id) 显式转换 vs (b) port trait 拆 Uuid + 强类型 双层 (per Q2-D A2 上游推荐 (b))
4. **5 域 Lead 真人到位**: per 8/21 JST 拒绝兼任硬约束, P3-C/E/F 阻塞 1 (per AGENTS.md §4 守门 #3)
5. **P3-B 拍板**: B.5 OpenClaw / B.6 Hermes 凭证 (per AGENTS.md §7 待办 #5-7, 仍 1 阻塞)

### 5.4 下次 session 第 1 步 (建议)

```bash
# 1. 读 HANDOFF-ST-001.md v0.3 (本文) + AGENTS.md v0.26
# 2. git log --oneline -10 看最新 HEAD
# 3. cargo check --workspace --all-targets 重新实测 (per Q9-T A9 数字有时效性, 必须实测, 不得沿用 v0.3 数字)
# 4. 续 H2-EXT 5 domain (按 §5.1 顺序)
# 5. 续 P0-2 ApiError 映射 (H2 完成后)
```"""
        new_lines.append(remaining)
        break
    new_lines.append(line)
    i += 1

REPO.write_text("".join(new_lines), encoding="utf-8")
print("[OK] HANDOFF-ST-001 v0.3 写入完成")
