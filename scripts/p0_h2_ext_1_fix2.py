#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""H2-EXT #1 fix2: domain-comment lib.rs 测试 helper 改 Uuid + 内部 TenantId 包装

策略: dev/make_cmd 接受 Uuid, 内部 TenantId(uuid) 包装. 测试 call sites 不动.
"""
from pathlib import Path

REPO = Path(r"D:/Star/crates/domain-comment/src/lib.rs")

text = REPO.read_text(encoding="utf-8")

# 1. dev 函数签名: tid: TenantId -> tid: uuid::Uuid, 内部不变 (ActorContext::new(..., tid.0))
text = text.replace(
    "fn dev(tid: TenantId) -> ActorContext {\n        ActorContext::new(Uuid::new_v4(), tid.0).with_role(\"developer\")\n    }",
    "fn dev(tid: uuid::Uuid) -> ActorContext {\n        ActorContext::new(Uuid::new_v4(), tid).with_role(\"developer\")\n    }"
)

# 2. make_cmd 函数签名: tid: TenantId -> tid: uuid::Uuid, 内部 tenant_id: tid -> tenant_id: TenantId(tid)
# find make_cmd 块
old_make_cmd_start = "    fn make_cmd(tid: TenantId) -> CreateCommentCommand {\n        let me = uuid::Uuid::new_v4();\n        CreateCommentCommand {\n            tenant_id: tid,"
new_make_cmd_start = "    fn make_cmd(tid: uuid::Uuid) -> CreateCommentCommand {\n        let me = uuid::Uuid::new_v4();\n        CreateCommentCommand {\n            tenant_id: TenantId(tid),"
if old_make_cmd_start in text:
    text = text.replace(old_make_cmd_start, new_make_cmd_start)
    print("[OK] make_cmd 签名改")
else:
    print("[WARN] make_cmd 签名未找到")

REPO.write_text(text, encoding="utf-8")
