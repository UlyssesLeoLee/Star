#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""HANDOFF-ST-001: 追加 §5 跨 session 续入口"""
from pathlib import Path

REPO = Path(r"D:/Star/HANDOFF-ST-001.md")

S5 = """

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

text = REPO.read_text(encoding="utf-8")
if "## §5" not in text:
    REPO.write_text(text + S5, encoding="utf-8")
    print("[OK] §5 追加完成")
else:
    print("[SKIP] §5 已存在")
