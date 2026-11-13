#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=71c1bbd2d40287f971fcfe5c352738dc6bd015f082af5fc27f199088a1fa80ec
# MERGE (self:File {path:"scripts/automation/agent_runtime/tests/__init__.py"})
# @endcypher
"""agent_runtime.tests — 单测包 (unittest, pytest 兼容)。

sys.path bootstrap: 保证 `automation.agent_runtime` 在 pytest 与
unittest discover 两种入口下均可解析 (repo 无 pytest 配置, per TASK 验证命令)。
"""

import os
import sys

_PARENT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
if _PARENT not in sys.path:
    sys.path.insert(0, _PARENT)
