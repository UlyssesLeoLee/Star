"""T-HR-* 单元测试: Hook Registry CRUD + schema 校验 + 热更新 (per DD §5.1 T-HR-1~T-HR-10)."""
# SPDX-License-Identifier: MIT OR Apache-2.0

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

WT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(WT))
sys.path.insert(0, str(WT / "scripts"))


from scripts.automation.hooks.hook_registry import (  # noqa: E402
    HookRegistry,
    Hook,
    _minimal_validate,
    _full_validate,
    HOOK_SCHEMA,
)


class TestSchemaValidation(unittest.TestCase):
    """T-HR-3: schema 校验."""

    def test_minimal_validate_valid(self):
        hook_def = {
            "name": "test_valid",
            "event_type": "PreToolUse",
            "action_type": "pre",
            "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
        }
        self.assertIsNone(_minimal_validate(hook_def))

    def test_minimal_validate_missing_name(self):
        hook_def = {
            "event_type": "PreToolUse",
            "action_type": "pre",
            "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
        }
        err = _minimal_validate(hook_def)
        self.assertIn("invalid name", err)

    def test_minimal_validate_invalid_event_type(self):
        hook_def = {
            "name": "test_bad",
            "event_type": "NotARealEvent",
            "action_type": "pre",
            "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
        }
        err = _minimal_validate(hook_def)
        self.assertIn("invalid event_type", err)

    def test_minimal_validate_invalid_action_type(self):
        hook_def = {
            "name": "test_bad_action",
            "event_type": "PreToolUse",
            "action_type": "super_block",
            "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
        }
        err = _minimal_validate(hook_def)
        self.assertIn("invalid action_type", err)

    def test_minimal_validate_priority_out_of_range(self):
        hook_def = {
            "name": "test_prio",
            "event_type": "PreToolUse",
            "action_type": "pre",
            "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
            "priority": 2000,  # > 1000
        }
        err = _minimal_validate(hook_def)
        self.assertIn("invalid priority", err)

    def test_full_validate_combines(self):
        """_full_validate 用 jsonschema (可选) + 最小自检."""
        hook_def = {
            "name": "test_full",
            "event_type": "PreToolUse",
            "action_type": "pre",
            "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
        }
        self.assertIsNone(_full_validate(hook_def))


class TestRegistryCRUD(unittest.TestCase):
    """T-HR-1, T-HR-4, T-HR-5, T-HR-7, T-HR-9: CRUD 操作."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_reg_")
        self.registry_path = Path(self.tmpdir) / "registry.json"
        self.registry_path.write_text('{"version":"1.0","hooks":[]}', encoding="utf-8")
        self.registry = HookRegistry(self.registry_path, builtin_names=["pre_tool_use_guard"])
        self.registry.load(builtin_hooks=[])

    def tearDown(self):
        self.registry.stop()

    def test_register_new_hook(self):
        """T-HR-4: register 新 hook."""
        h = self.registry.register({
            "name": "my_hook",
            "event_type": "PreToolUse",
            "action_type": "pre",
            "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
            "description": "test",
        })
        self.assertEqual(h.name, "my_hook")
        self.assertTrue(h.enabled)
        # 持久化验证
        data = json.loads(self.registry_path.read_text(encoding="utf-8"))
        self.assertEqual(len(data["hooks"]), 1)
        self.assertEqual(data["hooks"][0]["name"], "my_hook")

    def test_register_duplicate_raises(self):
        """T-HR-5: register 重名 hook → FileExistsError."""
        self.registry.register({
            "name": "my_hook",
            "event_type": "PreToolUse",
            "action_type": "pre",
            "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
        })
        with self.assertRaises(FileExistsError):
            self.registry.register({
                "name": "my_hook",
                "event_type": "PreToolUse",
                "action_type": "pre",
                "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
            })

    def test_register_builtin_name_raises(self):
        with self.assertRaises(PermissionError):
            self.registry.register({
                "name": "pre_tool_use_guard",
                "event_type": "PreToolUse",
                "action_type": "pre",
                "handler": "builtin:pre_tool_use_guard",
            })

    def test_enable_disable(self):
        """T-HR-7: enable/disable 修改 enabled 标志."""
        self.registry.register({
            "name": "toggle_test",
            "event_type": "PostToolUse",
            "action_type": "post",
            "handler": "scripts.automation.hooks.user.user_post_audit:handler",
        })
        # 默认 enabled=true
        h = self.registry.get("toggle_test")
        self.assertTrue(h.enabled)

        self.registry.disable("toggle_test")
        h = self.registry.get("toggle_test")
        self.assertFalse(h.enabled)
        self.assertIsNotNone(h.disabled_at)

        self.registry.enable("toggle_test")
        h = self.registry.get("toggle_test")
        self.assertTrue(h.enabled)

    def test_archive(self):
        """T-HR-9: archive 设置 archived=true."""
        self.registry.register({
            "name": "archive_test",
            "event_type": "PostToolUse",
            "action_type": "post",
            "handler": "scripts.automation.hooks.user.user_post_audit:handler",
        })
        self.registry.archive("archive_test")
        h = self.registry.get("archive_test")
        self.assertTrue(h.archived)
        self.assertFalse(h.enabled)  # archive 后默认禁用
        self.assertIsNotNone(h.archived_at)

    def test_archive_builtin_raises(self):
        """T-HR-8: archive builtin hook → PermissionError."""
        # 先注入 1 个 builtin (模拟)
        self.registry._hooks["pre_tool_use_guard"] = Hook(
            name="pre_tool_use_guard",
            event_type="PreToolUse",
            action_type="pre",
            handler="builtin:pre_tool_use_guard",
            is_builtin=True,
        )
        with self.assertRaises(PermissionError):
            self.registry.archive("pre_tool_use_guard")

    def test_list_by_event_filters_and_sorts(self):
        """T-HR-10: list_by_event 按 priority 升序."""
        for prio in [200, 10, 100]:
            self.registry.register({
                "name": f"prio_{prio}",
                "event_type": "PreToolUse",
                "action_type": "pre",
                "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
                "priority": prio,
            })
        hooks = self.registry.list_by_event("PreToolUse")
        priorities = [h.priority for h in hooks]
        self.assertEqual(priorities, sorted(priorities))
        self.assertEqual(priorities[0], 10)


class TestRegistryReload(unittest.TestCase):
    """T-HR-1 + §NFR-A-1 fail-open: registry.json 损坏时 reload."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_reg_reload_")
        self.registry_path = Path(self.tmpdir) / "registry.json"

    def tearDown(self):
        try:
            self.registry.stop()
        except Exception:
            pass

    def test_load_missing_file_returns_empty(self):
        """fail-open: registry.json 不存在 → 仅 builtin hook."""
        registry = HookRegistry(self.registry_path)
        result = registry.load(builtin_hooks=[])
        self.assertEqual(result, [])

    def test_load_corrupt_json_fails_open(self):
        """§NFR-A-1: registry.json JSON parse 失败 → fail-open."""
        self.registry_path.write_text("{this is not valid json", encoding="utf-8")
        registry = HookRegistry(self.registry_path)
        result = registry.load(builtin_hooks=[])
        # 不抛异常, 返回 builtin (这里是空)
        self.assertEqual(result, [])

    def test_reload_picks_up_changes(self):
        """热更新: registry.json 变化后 reload()."""
        self.registry_path.write_text('{"version":"1.0","hooks":[]}', encoding="utf-8")
        registry = HookRegistry(self.registry_path)
        registry.load(builtin_hooks=[])
        self.assertEqual(len(registry.list_all()), 0)

        # 写入 1 个 hook
        new_data = {
            "version": "1.0",
            "hooks": [{
                "name": "added_via_reload",
                "event_type": "PreToolUse",
                "action_type": "pre",
                "handler": "scripts.automation.hooks.builtin.pre_tool_use_guard:handler",
            }],
        }
        self.registry_path.write_text(json.dumps(new_data), encoding="utf-8")
        registry.reload(builtin_hooks=[])
        self.assertEqual(len(registry.list_all()), 1)
        self.assertIsNotNone(registry.get("added_via_reload"))


if __name__ == "__main__":
    unittest.main()