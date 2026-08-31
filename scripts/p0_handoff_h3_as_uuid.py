#!/usr/bin/env python3
"""
H3 (per Q4-I/A4): 统一 as_uuid() 签名
- 22 domain 强类型 ID as_uuid() 返回 Uuid (非 &Uuid)
- tuple 构造保留为宏内部/测试用法
- 在宏注释里注明 From<Uuid> 是推荐主构造方式
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DOMAINS = [
    "domain-agent", "domain-agent-windows", "domain-ai", "domain-audit", "domain-automation",
    "domain-board", "domain-cli", "domain-collaboration", "domain-comment", "domain-context",
    "domain-dashboard", "domain-development", "domain-feedback", "domain-form", "domain-identity",
    "domain-integration", "domain-kms", "domain-local-runtime", "domain-notification",
    "domain-permission", "domain-planning", "domain-project", "domain-relation", "domain-report",
    "domain-scm", "domain-search", "domain-tenant", "domain-theme", "domain-validation",
    "domain-work-item", "domain-workflow", "domain-workspace", "domain-worktree",
]

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    total = 0
    for d in DOMAINS:
        # 改 macros.rs (独立文件)
        f = WORKSPACE / d / "src" / "macros.rs"
        if f.exists():
            text = f.read_text(encoding="utf-8")
            if "pub fn as_uuid(&self) -> &uuid::Uuid" in text:
                new_text = text.replace(
                    "pub fn as_uuid(&self) -> &uuid::Uuid { &self.0 }",
                    "pub fn as_uuid(&self) -> uuid::Uuid { self.0 }\n            /// 推荐主构造方式: XxxId::from(uuid) 或 XxxId(uuid) tuple 构造 (仅宏内部/测试)",
                )
                if not DRY_RUN and new_text != text:
                    f.write_text(new_text, encoding="utf-8")
                if new_text != text:
                    print(f"{d}/src/macros.rs patched (as_uuid -> Uuid)")
                    total += 1
        # 改 lib.rs 顶部 (macro_rules! 直接定义)
        f = WORKSPACE / d / "src" / "lib.rs"
        if f.exists():
            text = f.read_text(encoding="utf-8")
            if "pub fn as_uuid(&self) -> &uuid::Uuid" in text:
                new_text = text.replace(
                    "pub fn as_uuid(&self) -> &uuid::Uuid { &self.0 }",
                    "pub fn as_uuid(&self) -> uuid::Uuid { self.0 }\n            /// 推荐主构造: XxxId::from(uuid) 或 tuple XxxId(uuid) (内部/测试)",
                )
                if not DRY_RUN and new_text != text:
                    f.write_text(new_text, encoding="utf-8")
                if new_text != text:
                    print(f"{d}/src/lib.rs patched (as_uuid -> Uuid)")
                    total += 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
