#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""HANDOFF-ST-001 v0.4 + AGENTS v0.27 写入"""
from pathlib import Path

REPO = Path(r"D:/Star")

# === HANDOFF v0.4 ===
HANDOFF = REPO / "HANDOFF-ST-001.md"
text = HANDOFF.read_text(encoding="utf-8")

# 在 v0.3 后面追加 v0.4
v03_marker = "| v0.3 | 2026-08-31 | 架构师 (Mavis 接手 agent per DEC-008) | 4 项 Ulysses 拍板结果"
v04_row = """| v0.4 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | H2-EXT #1-#3 落地 (3 commits: 9d08f80 / b6f6e2a / 7f611b0), 净修 507 err (797 → 290, 跨 9 crate); 守门 #1 实证: star-context 21/21 pass + workspace --lib 0 err + H2-EXT 3/5 完成; H2-EXT #4 domain-identity (DeviceId→Uuid 重构) + #5 domain-work-item (String→Uuid 需 Ulysses 拍板 String 原义) 跨 session 续; session 至此收尾 (per 2026-09-01 07:56 JST 新 session 启动, 2026-09-01 09:50 JST 收尾) |"""

if v03_marker in text:
    text = text.replace(v03_marker, v03_marker + "\n" + v04_row)
    print("[OK] HANDOFF v0.4 row 追加")
else:
    print("[WARN] v0.3 marker not found")

# 加 §6 跨 session 续 v0.4 总结
v04_section = """

---

## §6 跨 session 续 v0.4 总结 (2026-09-01 09:50 JST)

**H2-EXT 5 domain 改造进度 3/5 完成**:

| # | domain | 状态 | commit | 字段扩展 | 估 token |
|---|---|---|---|---|---|
| 1 | domain-comment | ✅ | 9d08f80 | (无) | 0.05M (实测 ~0.15M) |
| 2 | domain-tenant | ✅ | b6f6e2a | + `tenant_policy_id: Option<Uuid>` + `is_platform_operator()` helper | 0.1M |
| 3 | domain-project | ✅ | 7f611b0 | + `workspace_ids: Vec<Uuid>` 字段 | 0.1M |
| 4 | domain-identity | ⏳ 跨 session 续 | — | (DeviceId 强类型 → Uuid 重构) | 0.2M |
| 5 | domain-work-item | ⏳ 跨 session 续 + 等 Ulysses 拍板 | — | (String → Uuid 业务语义重设) | 0.2M |

**守门 #1 实证 (新 session 启动后)**:

| 阶段 | 命令 | 结果 |
|---|---|---|
| --lib | cargo check --workspace --lib | 0 err |
| --all-targets | cargo check --workspace --all-targets | **290 err** (跨 9 crate, 数字时效性 per Q9-T A9 不得沿用 797 或 432) |
| clippy | cargo clippy --workspace --lib | 0 err |
| fmt | cargo fmt --all --check | exit 0 |
| star-context test | cargo test -p star-context --lib | 21/21 pass |

**290 err 跨 9 crate 分布** (新 baseline):
- domain-feedback 77 (H2 原 3 domain 之一, 最大头)
- domain-worktree 51 (其它 domain, 跟 H2-EXT 无关)
- domain-local-runtime 50
- domain-board 39
- domain-agent 37
- domain-identity 30 (H2-EXT #4)
- domain-relation 4
- domain-project 1 (剩 1 err, 跟 H2-EXT #3 强类型转换相关)
- infrastructure 1

**H2-EXT #4 #5 跨 session 续 (估 0.4M token)**:
- #4 domain-identity: DeviceId 强类型 → Uuid 重构 (entity 改 + 跨 service/invariant)
- #5 domain-work-item: device_id String → Uuid 业务语义重设 (需 Ulysses 拍板 String 原义: hostname? JWT token? 其他?)

**H2 原 3 domain 改造 (估 0.6-0.8M token 跨 session 续)**:
- domain-feedback 77 err 是 H2 原 3 domain 改造大头, 模式跟 #1 #2 #3 一样, 但 service.rs 内部 actor.user_id 当 UserId 用 / actor.tenant_id 当 TenantId 用的 call sites 更多
"""

if "## §6 跨 session 续 v0.4 总结" not in text:
    text = text + v04_section
    print("[OK] HANDOFF §6 v0.4 总结追加")
HANDOFF.write_text(text, encoding="utf-8")
print("[OK] HANDOFF v0.4 完成")

# === AGENTS v0.27 ===
AGENTS = REPO / "AGENTS.md"
agents_text = AGENTS.read_text(encoding="utf-8")

# 在 v0.26 后追加 v0.27
v26_marker = "| v0.26 | 2026-08-31 | 架构师 (Mavis 接手 agent per DEC-008) | 4 项 Ulysses 拍板落地"
v27_row = """| v0.27 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | H2-EXT #1-#3 落地 (3 commits: 9d08f80 + b6f6e2a + 7f611b0), 守门 #1 v1 实证 --all-targets 797 → 290 err (净修 507, 跨 9 crate 分布); star-context 21/21 pass + workspace --lib 0 err + clippy 0 err + fmt exit 0 实证; H2-EXT #4 domain-identity (DeviceId→Uuid) + #5 domain-work-item (String→Uuid 需 Ulysses 拍板) 跨 session 续 (估 0.4M token); session 至此收尾 |"""

if v26_marker in agents_text:
    agents_text = agents_text.replace(v26_marker, v26_marker + "\n" + v27_row)
    print("[OK] AGENTS v0.27 row 追加")
else:
    print("[WARN] v0.26 marker not found")

# 加 v18 派生规
v17_marker = "| v17 | **H2 范围扩量触发** (per 2026-08-31 22:00 JST 真实尝试"
v18_row = """| v18 | **H2-EXT 5 domain 跨域字段扩展触发** (per 2026-09-01 09:50 JST 真实尝试 commit 9d08f80 + b6f6e2a + 7f611b0): HANDOFF-ST-001 §5.1 H2-EXT 5 domain 改造 3/5 完成, star_context 共享 ActorContext 字段扩展 (tenant_policy_id / workspace_ids + is_platform_operator helper), 净修 507 err (797 → 290, 跨 9 crate), 守门 #1 阶段 1 实证 (--lib 0 + clippy 0 + fmt 0 + 21/21 test pass), 阶段 2 (--all-targets 0) 待 #4 #5 + H2 原 3 domain 改造完成; 5 项 Blocker 跨 session 续 (per HANDOFF v0.4 §5.3): H2-EXT #4 #5 类型不兼容 (DeviceId 强类型 + String→Uuid 业务语义) + H2 原 3 domain service.rs 改造 (~150+ call sites) + 5 域 Lead 真人 + P3-B 拍板 |"""

if v17_marker in agents_text:
    agents_text = agents_text.replace(v17_marker, v17_marker + "\n" + v18_row)
    print("[OK] AGENTS §4.1 v18 派生追加")
else:
    print("[WARN] v17 marker not found")

AGENTS.write_text(agents_text, encoding="utf-8")
print("[OK] AGENTS v0.27 完成")
