"""v0.65 ApplicationError test assertions 批量更新 (per self-review Finding 3+4)

跟 helper signature 同步:
- source_module: "application" → domain-specific ("domain-xxx" / "infrastructure")
- source_kind: lowercase → TitleCase ("validation" → "Validation" 等)
- not_found("X") → not_found("module", "X")

幂等: 已 fix 跳过.
"""
import re
import sys
from pathlib import Path

APP_LIB = Path("crates/application/src/lib.rs")

# Test name → (source_module, source_kind TitleCase)
TEST_UPDATES = {
    "work_item_error_not_found_maps_to_application_not_found": ("domain-work-item", "Validation"),
    "workspace_error_permission_denied_maps_to_application_permission_denied": ("domain-workspace", "Policy"),
    "worktree_error_invalid_transition_maps_to_application_invalid_state": ("domain-worktree", "Validation"),
    "search_error_not_found_maps_to_application_not_found": ("domain-search", "Validation"),
    "scm_error_idempotency_conflict_maps_to_application_conflict": ("domain-scm", "External"),
    "validation_error_internal_maps_to_application_internal": ("domain-validation", "Internal"),
    "infrastructure_error_not_found_maps_to_application_not_found": ("infrastructure", "Validation"),
    "infrastructure_error_conflict_maps_to_application_conflict": ("infrastructure", "External"),
    "infrastructure_error_internal_maps_to_application_internal": ("infrastructure", "Internal"),
}

def main():
    if not APP_LIB.exists():
        print(f"FAIL: {APP_LIB} not found", file=sys.stderr)
        sys.exit(1)
    content = APP_LIB.read_text(encoding="utf-8")
    if 'source_module, "domain-work-item"' in content or 'source_kind, "Validation"' in content or 'assert_eq!(app_err.source_module, "domain-work-item")' in content:
        # already updated; check whether some still need it
        pass
    n_updates = 0
    # Update test name context: find each test function, then replace source_module/source_kind within
    for test_name, (module, kind) in TEST_UPDATES.items():
        # Find test function
        idx = content.find(f"fn {test_name}(")
        if idx < 0:
            print(f"WARN: {test_name} not found, skipping")
            continue
        # Find end of function (next #[test] or end of mod)
        next_test = content.find("\n    #[test]", idx + 1)
        if next_test < 0:
            next_test = content.find("\n}", idx + 1) + 2
        block = content[idx:next_test]
        new_block = block
        # Replace source_module = "application" → "domain-xxx" / "infrastructure"
        new_block = re.sub(
            r'source_module, "application"',
            f'source_module, "{module}"',
            new_block,
        )
        # Replace source_kind = "validation" → "Validation" (TitleCase)
        new_block = re.sub(
            r'source_kind, "validation"',
            f'source_kind, "{kind}"',
            new_block,
        )
        new_block = re.sub(
            r'source_kind, "policy"',
            'source_kind, "Policy"',
            new_block,
        )
        new_block = re.sub(
            r'source_kind, "external"',
            'source_kind, "External"',
            new_block,
        )
        new_block = re.sub(
            r'source_kind, "internal"',
            'source_kind, "Internal"',
            new_block,
        )
        if new_block != block:
            content = content[:idx] + new_block + content[next_test:]
            n_updates += 1
            print(f"OK: {test_name} → module='{module}', kind='{kind}'")
    # Update the 6-field structure test: not_found("test-resource") → not_found("application", "test-resource")
    if 'ApplicationError::not_found("test-resource")' in content:
        content = content.replace(
            'ApplicationError::not_found("test-resource")',
            'ApplicationError::not_found("application", "test-resource")',
        )
        n_updates += 1
        print("OK: 6-field structure test callsite updated")
    APP_LIB.write_text(content, encoding="utf-8")
    print(f"OK: {n_updates} updates done")


if __name__ == "__main__":
    main()
