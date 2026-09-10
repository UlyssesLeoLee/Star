"""v0.65 ApplicationError helper callsite 批量更新 (per self-review Finding 3+4)

v0.65 修复: 5 helper functions 接受 source_module 参数 (跟 P0-2 star-api-rest 6-field 对齐, TitleCase kind).
策略: 不重写 From impl logic, 只更新 5 helper call sites 加 source_module 第一参数.

5 helper + 每个 domain 一对一映射:
- work_item callsite → "domain-work-item"
- workspace callsite → "domain-workspace"
- worktree callsite → "domain-worktree"
- search callsite → "domain-search"
- scm callsite → "domain-scm"
- validation callsite → "domain-validation"
- infrastructure callsite → "infrastructure"

幂等: 已加 source_module 跳过.
"""
import re
import sys
from pathlib import Path

APP_LIB = Path("crates/application/src/lib.rs")

# (From impl header line, source_module string)
IMPL_TO_MODULE = [
    ("From<domain_work_item::WorkItemError>", "domain-work-item"),
    ("From<domain_workspace::WorkspaceError>", "domain-workspace"),
    ("From<domain_worktree::WorktreeError>", "domain-worktree"),
    ("From<domain_search::SearchError>", "domain-search"),
    ("From<domain_scm::ScmError>", "domain-scm"),
    ("From<domain_validation::ValidationError>", "domain-validation"),
    ("From<infrastructure::InfrastructureError>", "infrastructure"),
]

def update_callsites_in_impl(content: str, impl_header: str, source_module: str) -> str:
    """Find impl block starting with header and update all 5 helper callsites inside."""
    idx = content.find(impl_header)
    if idx < 0:
        return content
    # Find the matching closing brace (next impl block start or end of file)
    end = content.find("\nimpl ", idx + 1)
    if end < 0:
        end = content.find("\n#[cfg(test)]", idx + 1)
    if end < 0:
        end = len(content)
    block = content[idx:end]
    new_block = block
    # Update not_found("X") → not_found("source_module", "X")
    new_block = re.sub(
        r'ApplicationError::not_found\("([^"]+)"\)',
        rf'ApplicationError::not_found("{source_module}", "\1")',
        new_block,
    )
    # Update permission_denied("X") → permission_denied("source_module", "X")
    new_block = re.sub(
        r'ApplicationError::permission_denied\("([^"]+)"\)',
        rf'ApplicationError::permission_denied("{source_module}", "\1")',
        new_block,
    )
    # Update invalid_state(EXPR, "hint") → invalid_state("source_module", EXPR, "hint")
    # Use a pattern that handles the multiline expression (e.to_string())
    new_block = re.sub(
        r'ApplicationError::invalid_state\(\s*([^,]+),\s*("[^"]+")\s*\)',
        rf'ApplicationError::invalid_state("{source_module}", \1, \2)',
        new_block,
    )
    # Update conflict(EXPR) → conflict("source_module", EXPR)
    new_block = re.sub(
        r'ApplicationError::conflict\(\s*([^)]+)\)',
        rf'ApplicationError::conflict("{source_module}", \1)',
        new_block,
    )
    # Update internal(EXPR) → internal("source_module", EXPR)
    new_block = re.sub(
        r'ApplicationError::internal\(\s*([^)]+)\)',
        rf'ApplicationError::internal("{source_module}", \1)',
        new_block,
    )
    return content[:idx] + new_block + content[end:]


def main():
    if not APP_LIB.exists():
        print(f"FAIL: {APP_LIB} not found", file=sys.stderr)
        sys.exit(1)
    content = APP_LIB.read_text(encoding="utf-8")
    if 'not_found("domain-work-item", "work-item")' in content:
        print("OK: helper callsites already updated (idempotent skip)")
        return
    for impl_header, source_module in IMPL_TO_MODULE:
        if impl_header not in content:
            print(f"WARN: {impl_header} not found, skipping")
            continue
        before = content
        content = update_callsites_in_impl(content, impl_header, source_module)
        if content == before:
            print(f"WARN: {impl_header} callsites unchanged (pattern mismatch?)")
        else:
            print(f"OK: {impl_header} → source_module = '{source_module}'")
    APP_LIB.write_text(content, encoding="utf-8")
    print(f"OK: callsite updates done ({APP_LIB})")


if __name__ == "__main__":
    main()
