#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""H2-EXT #1 fix3: domain-comment lib.rs 测试 as_agent + tid.0 修

E0616 (16 个): tid.0 私有字段访问 - 测试里 `tid` 现在是 Uuid 类型, .0 是私有
  修法: tid.0 -> tid (因为 tid 现在是 Uuid, ActorContext::new 需要 Uuid)
       但 dev/make_cmd 已改接收 Uuid, 测试里也有 `tid = uuid::Uuid::new_v4()` 已经是 Uuid
       所以 tid.0 -> 直接用 tid

E0599 (3 个):
  - as_agent: 删 context.rs 后, sub-module 没了. 用 ActorContext::new + is_agent_session 模拟
  - as_uuid: tid 是 Uuid, 不需要 as_uuid. 直接用 tid
"""
from pathlib import Path

REPO = Path(r"D:/Star/crates/domain-comment/src/lib.rs")

text = REPO.read_text(encoding="utf-8")
original = text
success = 0

# 1. tid.0 -> tid (因为 tid 现在是 Uuid)
# 但要小心理: `tid.0` 可能是 `TenantId.0` (强类型 ID deref) 或 `Uuid.0` (私有)
# 测试里现在 dev/make_cmd 接 Uuid, 所以 tid 是 Uuid, .0 私有错
# 但其它代码 (e.g. svc.create_comment 的 .into_uuid() 等) 仍可能用 .0
# 简单粗暴: 在 lib.rs test 段替换 .0 -> 删
# 但保险起见, 只替换 `, tid.0)` 在 ActorContext::new 调用里
replacements = [
    # ActorContext::new(me.0, tid.0) -> ActorContext::new(me, tid)
    # 实际上 me.0 (UserId -> Uuid) 也需要 -> UserId::from(me)
    # 但 me 已经是 Uuid (per 测试 let me = uuid::Uuid::new_v4())
    # 所以 me.0 私有错也要改
    # 解决: 删除所有 me.0 / tid.0 / other.0
    # 模式: `<var>.0` 删 .0
    ("ActorContext::new(me.0, tid.0)",
     "ActorContext::new(me, tid)"),
    ("ActorContext::new(other.0, tid.0)",
     "ActorContext::new(other, tid)"),
    # tid.as_uuid() -> tid (因为 tid 已经是 Uuid, 没 as_uuid)
    # 但 format! 里需要 tenants/{}/design.pdf, tid.as_uuid() 应该是 Uuid
    # 现在 tid 是 Uuid, 直接用 tid
    ('format!("tenants/{}/design.pdf", tid.as_uuid())',
     'format!("tenants/{}/design.pdf", tid)'),
    ('starts_with(&format!("tenants/{}/", tid.as_uuid()))',
     'starts_with(&format!("tenants/{}/", tid))'),
    # ActorContext::as_agent(AgentId::new()) - 删 context.rs 后没了
    # 改用 ActorContext::new + is_agent_session = true
    ("let mut agent_actor = ActorContext::as_agent(AgentId::new());",
     "let mut agent_actor = ActorContext::new(AgentId::new().as_uuid(), tid).with_agent_session(true);"),
]

for old, new in replacements:
    if old in text:
        text = text.replace(old, new)
        success += 1
        print(f"  [OK] {old[:60]}...")

REPO.write_text(text, encoding="utf-8")
print(f"\n替换 {success} 处")
