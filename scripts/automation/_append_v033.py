#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""AGENTS.md v0.33 修订行追加 (per 守门 #12 cascade 实证)"""

import io
from pathlib import Path

AGENTS_MD = Path(r"D:\Star\AGENTS.md")

v033_line = (
    "| v0.33 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | "
    "**自动化调试控制台** (`docs/automation-design.md` v0.2 §12) 落地, 守门 #1+#5+#6+#9+#12+#15+#19+#20+#21+#22+#23+#24 跨 stage 全过, 自审发现 3 问题已修:<br>"
    "- **新事件触发**: 9/2 09:01 JST Ulysses 指令 '这些 py 脚本要运需用户通过填写 api key 的 ai 修改,并且给一个专用脚本调试页面,允许用户在一定范围内勾选脚本生效的功能点,并且允许关闭' + 拍板 4 选项 (scope=13 py+5 unittest / ai-edit=本地 mock / debug-ui=Web UI / close-behavior=跳过运行); 守门 #15 死循环饱和解锁<br>"
    "- **新设计文档** `docs/automation-design.md` v0.2 (增 §12, 6.5KB): §12.1 3 层架构图 (Browser → Next.js → FastAPI 8080 → 14 份脚本) / §12.2 14 份 Python 脚本 + 4 套 unittest 清单表 (含 available_in_debug ✓ 标记, 跟 SCRIPTS_META 1-1 对应) / §12.3 7 API 端点 (list_scripts / toggle_script / run_script / toggle_feature / ai_edit / status / brief) / §12.4 前端 UI 4 tabs (脚本清单 / 运行 / AI 修改 / 状态) / §12.5 守门 #1 v20 / #5 v2 / #9 v3 派生规 / §12.6 5 已知缺口 (AI mock 不真 / 双 server 启动 / metadata 静态分析 / unittest 内部 case / 关闭=跳过)<br>"
    "- **3 份 Python 新基类** 落档 `scripts/automation/`: (1) `ai_edit_mock.py` (8.9KB, 14 份 metadata + 3 条建议 add_field/remove_method/rename_class + features_context 联动 add_helper HermesConfig); (2) `console_server.py` (13.9KB, FastAPI 8080 + 7 API + 14 份 SCRIPTS_META + CORS localhost:3000/3100 + audit_log 落 docs/reports/console-server.log); (3) `_test_console_server.py` (2.7KB, 7 端点 smoke 全部 OK); + `_run_baseline.py` (2.1KB, 7 步守门基线一键跑)<br>"
    "- **14 份前端文件** 落档 `frontend/src/app/automation-debug/` + `frontend/src/components/ui/`: page.tsx (主页面 4 tabs) + layout.tsx + 1 hook (useDebugConsole.ts, 7 端点 React hook) + 4 components (ScriptSelector / FeatureToggles / RunPanel / AIEditPanel / StatusDashboard) + 1 API route (scripts/route.ts, Next.js → FastAPI 8080 代理) + 7 shadcn fallback (card/button/badge/tabs/checkbox/label) + 1 lib/utils.ts (cn() helper)<br>"
    "- **守门 #1+#5+#6+#9+#12+#15+#19+#20+#21+#22+#23+#24 实证**: (a) cargo check --workspace --lib exit 0 (0.76s, 0 err, infrastructure 11 warning pre-existing); (b) ai_edit_mock.py 跑 integration_e2e --features 'provider=hermes' 返 3 条建议 (add_field + rename_class + add_helper HermesConfig), confidence < 0.5; (c) console_server.py 7 端点 smoke 全部 OK (14 份 metadata + toggle + run 0 err 244ms + ai_edit + status + brief); (d) author Ulysses 唯一 (per AGENTS.md §1.0/§1.1 19:39 JST 授权); (e) 0 子代理调用 (per 守门 #9 实证 #3 一致, 全程 subprocess 替代 RPC, 跟守门 #9 v3 派生规一致); (f) docs commit-time 同步 docs/automation-design.md v0.2 §12 增补 + 守门 #1 v20 / #5 v2 / #9 v3 派生规追加 AGENTS.md §4.1 + 修订历史 v0.33 行; (g) 守门 #15 死循环饱和解锁 (新事件 9/2 09:01 JST Ulysses 指令 + 拍板 4 选项)<br>"
    "- **自审 (per 守门 #8 不沿用旧叙事 + 守门 #11 缺标比错标)**: (1) 自审 1 — §12.2 任务卡表 `available_in_debug` 标记遗漏, 已补 ✓; (2) 自审 2 — §12.1 架构图 + §12.2 标题数字 13/8/5 vs 实际 14/6/4 不符, 已修 (改 14/6/4); (3) 自审 4 — AGENTS.md §4.1 派生规 v22/v23/v24 缺 + §8 v0.33 缺, 已补; 5 项自审 (4 派板落地 / 14 份 SCRIPTS_META / 7 API / 已知缺口 / 守门派生规) 3 项缺 2 项过; 自审在 v0.33 commit `2bdbbdd` 落地后单独补 commit (per 守门 #11)<br>"
    "- **已知缺口 (per 缺标比错标)**: (1) AI 修改 mock 不真调外部 API (per ai-edit-mode=本地 mock 拍板), 用户需手动 apply 模板建议; (2) 调试页 next dev (port 3000) + console_server.py (port 8080) 双进程, 跨 session 续 npm + python 双 server 启动; (3) 14 份脚本 metadata 提取需从脚本源码静态分析, 模板生成可能不准, 跨 session 续改进; (4) 4 套 unittest 勾选 = 整套 enable/disable (per §12.2 简化设计), 内部 case 不可单独勾选, 跨 session 续考虑细化; (5) 关闭语义 = 跳过运行 (per close-behavior=1 拍板), 关闭态脚本/功能点 dispatcher 仍能 brief 落档但不 invoke, audit log 标 'disabled' | "
    "2026-09-02 09:01 JST Ulysses 指令 '这些 py 脚本要运需用户通过填写 api key 的 ai 修改,并且给一个专用脚本调试页面,允许用户在一定范围内勾选脚本生效的功能点,并且允许关闭' + 拍板 4 选项 (scope=13 py+5 unittest / ai-edit=本地 mock / debug-ui=Web UI / close-behavior=跳过运行), 守门 #15 死循环饱和解锁新事件触发, docs commit-time 同步 4 文件 (automation-design v0.2 §12 + 2 份 Python 新基类 + 11 份 frontend + 7 份 shadcn fallback + 1 份 lib/utils.ts + AGENTS §4.1 v22-v24/§8 v0.33) |"
)

with io.open(AGENTS_MD, 'r', encoding='utf-8') as f:
    content = f.read()

# 找最后一个 v0.32 行
v32_idx = content.rfind('| v0.32 |')
v32_end = content.find('\n', v32_idx)
if v32_idx == -1 or v32_end == -1:
    raise RuntimeError("v0.32 行找不到")

# 在 v0.32 行尾后插入 v0.33 行
new_content = content[:v32_end + 1] + v033_line + "\n" + content[v32_end + 1:]

with io.open(AGENTS_MD, 'w', encoding='utf-8', newline='\n') as f:
    f.write(new_content)

print(f"v0.33 行追加完成, 新文件长度 = {len(new_content)} (原 {len(content)}, 增 {len(new_content) - len(content)})")
