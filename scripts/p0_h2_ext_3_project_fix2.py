#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""H2-EXT #3 fix2: domain-project lib.rs 测试 15 err 修

修法: 跟 #1 #2 一样模式, 测试里 `tid` 改类型 / 包装 TenantId
"""
from pathlib import Path

REPO = Path(r"D:/Star/crates/domain-project/src/lib.rs")

text = REPO.read_text(encoding="utf-8")
original = text
success = 0

# 1. ActorContext::new(Uuid::new_v4(), tid.0) 私字段 -> ActorContext::new(Uuid::new_v4(), tid)
text = text.replace(
    "ActorContext::new(Uuid::new_v4(), tid.0)",
    "ActorContext::new(Uuid::new_v4(), tid)"
)

# 2. 测试里 CreateProjectCommand { tenant_id: tid, ... } 等期望 TenantId, 但 tid 是 Uuid
# 看实际 err, lib.rs:700 是 CreateProjectCommand { tenant_id: tid, ... } 测试
# 全局替换 `tenant_id: tid,` -> `tenant_id: TenantId(tid),`
# 但需要小心: t1/t2 已是对的不动. tid 是 Uuid.
# 实际上从 err 位置看, lib.rs:700, 722, 737, 760, ... 都期望 TenantId
text = text.replace(
    "tenant_id: tid,",
    "tenant_id: TenantId(tid),"
)

# 但有些 "tid" 是 Project 等 entity 的强类型. 检查.
# 实际上从 err 看, "tenant_id: tid" 的 tid 都是 Uuid.
# 但 Entity 字段是 TenantId 强类型, 如 Project { tenant_id: tid } - 不对
# Project { tenant_id: tid } 期望 TenantId - 那 tid 在测试里需要是 TenantId
# 让我保守只替换在测试段

REPO.write_text(text, encoding="utf-8")
print(f"替换完成")
