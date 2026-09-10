#!/usr/bin/env python3
"""
automation-design §4.30 + registry v0.15 追加脚本 (per 守门 #12 v21 [P] docs 同步).

触发: 2026-09-10 19:10 JST P3-D.5 5 文档 IPA SEC 合规性修复落地 (commit 0d6cecf) 后 docs 同步.
范围: §4.30 IPA SEC 合规性 fix task card + registry v0.15 修订历史.
"""
import sys
from pathlib import Path

AUTOMATION_DESIGN_PATH = Path("D:/Star/docs/automation-design.md")
REGISTRY_PATH = Path("D:/Star/scripts/automation/registry.md")


# §4.30 content (P3-D.5 IPA SEC 合规性 修复)
SECTION_4_30 = """


### 4.30 P3-D.5 IPA SEC 合规性修复 (per 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准", 2026-09-10 19:10 JST)

> **触发**: 2026-09-10 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 79 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 79 次新事件, docs 同步允许) + 守门 #1 v19 (1 commit 5 文件批量改 收官) + 守门 #1 禁回溯叙事 (不重写 §0-§10 旧内容, v0.1.1/v0.1.2/v1.2 反转行显式标) + 守门 #10 (commit author=Ulysses) + 守门 #11 (缺标比错标, 5 处不合规显式列) + 守门 #14 v2/v3/v4 (5 域 Lead Mavis 临时代签 + Mavis 永久代签 + v0.62 反转 Mavis 审核 author=Ulysses) + 守门 #9 v19 (Mavis 自驱, 19:06 JST 拍板 → 19:10 JST 1 commit 5 文件 ~4 分钟) + 守门 #9 v27 (RPC 失败 fallback 3 段, 真实验证文件 + 字节数 + 段头结构) + 守门 #13 W/T/M (本批不涉及, 维持前批 14+15 张表 100% 覆盖)
> **落档文件** (关联 commit `0d6cecf`, 6 files / 500 insertions / 5 deletions):
> - `docs/requirements/SRS-CANVAS-001.md` **v1.2** (46,071 bytes, 12 段, 加 §10 5 角色签字栏 + renumber §9 → §11 修订履历, +22 -4 lines, 修复前 10 段缺独立 5 角色签字栏 段)
> - `docs/requirements/SRS-CANVAS-GAMIFY-001.md` **v0.1.1** (73,053 bytes, 12 段, 加 §10 5 角色签字栏 + renumber §9 → §11 修订履历, +866 bytes, 修复前 10 段 5 角色签字栏 在附录 D)
> - `docs/design/DD-CANVAS-001.md` **v0.1.2** (47,269 bytes, 14 段, 加 §11 NFR 详细 6 类 29 项 (性能 6 / 可靠性 3 / 安全 6 / 易用 5 / 可观测 3 / ARG 6 + MU-CONS-01) + §12 5 角色签字栏 + §13 修订历史, +6819 bytes, 修复前 11 段缺 NFR 独立段 + 5 角色签字栏 在附录 D + 修订历史 在附录 E)
> - `docs/design/DD-CANVAS-AGENT-001.md` **v0.1.1** (122,367 bytes, 15 段, 加 §13 5 角色签字栏 + §14 修订历史, +2018 bytes, 修复前 13 段 5 角色签字栏 在附录 D + 修订历史 在附录 E)
> - `docs/reports/COORDINATION-CHECK-001.md` **v0.1.1** (18,824 bytes, 8 段, 加 §0 文档信息 / 修订履历, +1435 bytes, 修复前 7 段 (§1-§7) 缺 §0 文档信息)
> - `scripts/automation/ipa_sec_compliance_fix.py` 22.7KB (idempotent Python 修复脚本, 5 文档 5 fix 函数 + main orchestrator + 守门 #11 缺标比错标 + 守门 #1 禁回溯叙事 显式标)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.5-1 | D5.5-1 | `docs/requirements/SRS-CANVAS-001.md` v1.2 (12 段 IPA SEC 模板) | A | **[P]** | `ipa_sec_compliance_fix.py::fix_srs_canvas_001` | 加 §10 5 角色签字栏 (per AGENTS.md §3 7 段结构) + renumber §9 → §11 修订履历; 字节数 45,202 → 46,071 (+869); 段数 10 → 12 |
| D5.5-2 | D5.5-2 | `docs/requirements/SRS-CANVAS-GAMIFY-001.md` v0.1.1 (12 段 IPA SEC 模板) | A | **[P]** | `ipa_sec_compliance_fix.py::fix_srs_canvas_gamify_001` | 加 §10 5 角色签字栏 + renumber §9 → §11 修订履历; 字节数 72,187 → 73,053 (+866); 段数 10 → 12; SRS-GAMIFY-001 §9 是 simple table 无 sub-headings (per grep), script `### 9.1` 模式失败, 改用 `## §9 修订履历\n\n\\| 版本` 模式 |
| D5.5-3 | D5.5-3 | `docs/design/DD-CANVAS-001.md` v0.1.2 (14 段 IPA SEC 模板, NFR 6 类 29 项 跨域) | A | **[P]** | `ipa_sec_compliance_fix.py::fix_dd_canvas_001` | 加 §11 NFR 详细 6 类 (性能 6 + 可靠性 3 + 安全 6 + 易用 5 + 可观测 3 + ARG 6 + MU-CONS-01) + §12 5 角色签字栏 + §13 修订历史 (v0.1/v0.1.1/v0.1.2 三行); 字节数 40,450 → 47,269 (+6819); 段数 11 → 14 |
| D5.5-4 | D5.5-4 | `docs/design/DD-CANVAS-AGENT-001.md` v0.1.1 (15 段 IPA SEC 模板) | A | **[P]** | `ipa_sec_compliance_fix.py::fix_dd_canvas_agent_001` | 加 §13 5 角色签字栏 + §14 修订历史 (v0.1/v0.1.1 两行); 字节数 120,349 → 122,367 (+2018); 段数 13 → 15 |
| D5.5-5 | D5.5-5 | `docs/reports/COORDINATION-CHECK-001.md` v0.1.1 (8 段 IPA SEC 模板) | A | **[P]** | `ipa_sec_compliance_fix.py::fix_coordination_check_001` | 加 §0 文档信息 / 修订履历 (0.1 文档信息 + 0.2 修订履历 v0.1/v0.1.1 两行); 字节数 17,389 → 18,824 (+1435); 段数 7 → 8; 跟其他 9 份 P3-D.5 文档结构对齐 |
| D5.5-6 | D5.5-6 | `scripts/automation/ipa_sec_compliance_fix.py` 22.7KB (idempotent Python 脚本) | A | **[P]** | (本脚本) | 5 文档 5 fix 函数 + main orchestrator + 守门 #11 缺标比错标 + 守门 #1 禁回溯叙事 显式标; 第 2 次跑 idempotent 跳过 (4/5 已修, 1/5 第二次才修 SRS-GAMIFY-001) |
| D5.5-7 | D5.5-7 | `docs/automation-design.md` §4.30 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.5-8 | D5.5-8 | `scripts/automation/registry.md` §3 v0.15 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.15 修订历史 (19:06 JST IPA SEC 拍板) |
| D5.5-9 | D5.5-9 | 1 commit author = `Ulysses <ulysses@mavis.local>` (commit `0d6cecf`) | A | **[P]** | (git commit) | 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 Mavis 审核 author=Ulysses; 不推 origin (守门 #1 反转后 R-05) |

**§4.30 任务卡维度判定**:
- R (Rerunnable): **是** (`ipa_sec_compliance_fix.py` idempotent, 二次跑同样结果)
- V (Volume): 否 (无子代理派发, Mavis 接手 root session 一次性, 0 子代理调用)
- S (Structural): **是** (5 文档 段头结构整改, 10-13 段 → 12-15 段, 段号 renumber §9 → §11 + 加 §10/§11/§12/§13/§14)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 0d6cecf) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 子代理调用 + 守门 #1 禁回溯叙事 不重写 §0-§10 旧内容 + 守门 #11 缺标比错标 5 处不合规显式列 + 守门 #14 v2/v3/v4 代签 / 审核 规则全备)

**§4.30 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow docs/requirements/SRS-CANVAS-001.md` 实证 v1.2 落档 (commit `0d6cecf`, 12 段, +22 -4)
- `git log -p --follow docs/requirements/SRS-CANVAS-GAMIFY-001.md` 实证 v0.1.1 落档 (commit `0d6cecf`, 12 段, +866)
- `git log -p --follow docs/design/DD-CANVAS-001.md` 实证 v0.1.2 落档 (commit `0d6cecf`, 14 段, +6819)
- `git log -p --follow docs/design/DD-CANVAS-AGENT-001.md` 实证 v0.1.1 落档 (commit `0d6cecf`, 15 段, +2018)
- `git log -p --follow docs/reports/COORDINATION-CHECK-001.md` 实证 v0.1.1 落档 (commit `0d6cecf`, 8 段, +1435)
- 5 处 IPA SEC 不合规修复 (per 守门 #11 缺标比错标): (1) SRS-001 缺 §10 5 角色签字栏 → 加; (2) SRS-GAMIFY-001 缺 §10 5 角色签字栏 → 加; (3) DD-001 缺 §11 NFR + §12 5 角色签字栏 + §13 修订历史 → 3 段全加; (4) DD-AGENT-001 缺 §13 5 角色签字栏 + §14 修订历史 → 2 段加; (5) COORDINATION-CHECK 缺 §0 文档信息 → 加
- 5 文档段号 renumber: SRS-001 §9 → §11 修订履历; SRS-GAMIFY-001 §9 → §11 修订履历; DD-001 新加 §11/§12/§13 (与原 §11-§13 不冲突, 原 §11/§12/§13 跨域汇总的内容保留在 §10 测试用例之后 + 附录 A-E)
- 守门 #1 v19: 0 子代理调用 (IPA SEC 修复期), Mavis 接手 root session 一次性, 1 commit 5 文件
- 守门 #9 #3: 0 子代理调用 (IPA SEC 修复期), 不派二级子代理
- 守门 #9 v19: Mavis 自驱, 拍板后立即执行 (19:06 JST 拍板 → 19:10 JST 1 commit 5 文件 ~4 分钟)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签)
- 守门 #11 缺标比错标: 5 处 IPA SEC 不合规显式列 (3 SRS/2 DD/1 报告), 不沿用代签决策
- 守门 #13 DB W/T/M: 本批 IPA SEC 修复不涉及 DB schema, 维持前批 14+15 张表 100% 覆盖
- 守门 #14 v3: 5 角色签字栏 (新增) 全部 Mavis 接手代签 (修订人 + 审批者 author=Ulysses)
- 守门 #14 v4: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST v0.62 反转)
- 守门 #1 禁回溯叙事: 不重写 §0-§10 旧内容, v0.1.1/v0.1.2/v1.2 反转行显式标 (per 守门 #1 + 守门 #11 缺标比错标)

**§4.30 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本 IPA SEC 修复期 (root Mavis 接手 + ipa_sec_compliance_fix.py idempotent Python 脚本): ~0.05M tokens
- 累计 P3-D.5 全部 10 commit (4 SRS + 1 总册 BD + 2 专题 BD + 1 总册 DD + 2 专题 DD + 1 协调性检查 + 1 自审 + 1 IPA SEC 修复): ~3.41M tokens (2.84 SRE·周, 累计 10 commit)
- 后续 P3-D.6 启动实装 (P0 36 项): ~3-5M tokens
- 双核心 78 项 全部落地 (12 个月+): ~13-19M (13-19 SRE·周, per STAR-OLU-001)"""


# registry v0.15 row
V015_ROW = """| **v0.15** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (5 文档已落, 索引同步) + §3 v0.15**: P3-D.5 5 文档 IPA SEC 合规性修复 (per 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准"), 关联 commit `0d6cecf` (6 files / 500 insertions / 5 deletions, per 守门 #1 v15 docs 同步饱和第 79 次新事件触发 + 守门 #1 v19 批量改 1 commit 收官 + 守门 #1 禁回溯叙事不重写 §0-§10 旧内容 + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #14 v3 Mavis 永久代签 + 守门 #11 缺标比错标 5 处不合规显式列): 5 文档 段头结构整改 (10-13 段 → 12-15 段, 段号 renumber §9 → §11 + 加 §10/§11/§12/§13/§14) — (a) `docs/requirements/SRS-CANVAS-001.md` v1.2 (12 段, 加 §10 5 角色签字栏, +22 -4); (b) `docs/requirements/SRS-CANVAS-GAMIFY-001.md` v0.1.1 (12 段, 加 §10 5 角色签字栏, +866); (c) `docs/design/DD-CANVAS-001.md` v0.1.2 (14 段, 加 §11 NFR 6 类 29 项 + §12 5 角色签字栏 + §13 修订历史, +6819); (d) `docs/design/DD-CANVAS-AGENT-001.md` v0.1.1 (15 段, 加 §13 5 角色签字栏 + §14 修订历史, +2018); (e) `docs/reports/COORDINATION-CHECK-001.md` v0.1.1 (8 段, 加 §0 文档信息 / 修订履历, +1435); + `scripts/automation/ipa_sec_compliance_fix.py` 22.7KB idempotent Python 脚本 | **2026-09-10 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标 5 处不合规显式列 + 守门 #1 v19 批量改 1 commit 收官)** |"""


def main() -> int:
    """Append §4.30 to automation-design.md and v0.15 row to registry.md."""
    # Append §4.30 to automation-design.md
    if not AUTOMATION_DESIGN_PATH.exists():
        print(f"ERROR: {AUTOMATION_DESIGN_PATH} not found", file=sys.stderr)
        return 1

    auto_content = AUTOMATION_DESIGN_PATH.read_text(encoding="utf-8")
    print(f"Read {len(auto_content)} bytes from {AUTOMATION_DESIGN_PATH.name}")

    if "### 4.30 P3-D.5 IPA SEC 合规性修复" in auto_content:
        print(f"  {AUTOMATION_DESIGN_PATH.name}: §4.30 already exists, skipping")
    else:
        if not auto_content.endswith("\n"):
            auto_content = auto_content + "\n"
        new_content = auto_content + SECTION_4_30 + "\n"
        AUTOMATION_DESIGN_PATH.write_text(new_content, encoding="utf-8")
        print(f"  Appended §4.30 ({len(SECTION_4_30)} chars)")
        print(f"  New file size: {len(new_content)} bytes")

    # Append v0.15 row to registry.md
    if not REGISTRY_PATH.exists():
        print(f"ERROR: {REGISTRY_PATH} not found", file=sys.stderr)
        return 1

    reg_content = REGISTRY_PATH.read_text(encoding="utf-8")
    print(f"Read {len(reg_content)} bytes from {REGISTRY_PATH.name}")

    if "**v0.15**" in reg_content:
        print(f"  {REGISTRY_PATH.name}: v0.15 already exists, skipping")
    else:
        if not reg_content.endswith("\n"):
            reg_content = reg_content + "\n"
        new_content = reg_content + V015_ROW + "\n"
        REGISTRY_PATH.write_text(new_content, encoding="utf-8")
        print(f"  Appended v0.15 row ({len(V015_ROW)} chars)")
        print(f"  New file size: {len(new_content)} bytes")

    return 0


if __name__ == "__main__":
    sys.exit(main())
