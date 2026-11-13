#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.tests — 单测包 (unittest, pytest 兼容)。

sys.path bootstrap: 保证 `automation.agent_runtime` 在 pytest 与
unittest discover 两种入口下均可解析 (repo 无 pytest 配置, per TASK 验证命令)。
"""

import os
import sys

_PARENT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
if _PARENT not in sys.path:
    sys.path.insert(0, _PARENT)
