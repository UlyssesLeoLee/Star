{
  "fixture_version": "v1",
  "module": "sa-08",
  "type": "domain-dev",
  "method": "POST",
  "scenario": "implement-domain-agent",
  "description": "SA-08 domain-dev: 实现 domain-agent crate (per AGENTS.md §4.2 实装前一致性门 47 packages, 34 domain-* 命名)",
  "request": {
    "target_crate": "domain-agent",
    "implementation_type": "port + service + adapter",
    "include_unit_tests": true,
    "include_integration_tests": true
  },
  "response_200": {
    "dev_id": "dev-uuid-008",
    "files_created_count": 12,
    "lines_added": 850,
    "test_count": 24,
    "guard_passes": ["#7 0 unsafe", "#13 a/b/c/d", "#10 author Ulysses"]
  },
  "fixture_assertion": {
    "dev_complete": true,
    "guard_passes_count": 3
  }
}
