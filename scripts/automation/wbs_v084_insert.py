#!/usr/bin/env python3
"""
automation-design §4.31 + registry v0.16 追加脚本 (per 守门 #12 v21 [P] docs 同步).

触发: 2026-09-10 19:14 JST P3-D.5 IPA SEC 合规性 v2 修复落地 (commit 6f693ca) 后 docs 同步.
范围: §4.31 IPA SEC v2 细化修复 task card + registry v0.16 修订历史.
"""
import sys
from pathlib import Path

AUTOMATION_DESIGN_PATH = Path("D:/Star/docs/automation-design.md")
REGISTRY_PATH = Path("D:/Star/scripts/automation/registry.md")


# §4.31 content (P3-D.5 IPA SEC 合规性 v2 细化修复)
SECTION_4_31 = """


### 4.31 P3-D.5 IPA SEC 合规性 v2 细化修复 (per 19:14 JST Ulysses 再次拍板"确保这里的文档符合日本 IPA 标准", 2026-09-10 19:15 JST)

> **触发**: 2026-09-10 19:14 JST Ulysses 再次拍板"确保这里的文档符合日本 IPA 标准" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 81 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 81 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (不重写 §0-§12 旧内容, v1.3/v0.1.1 反转行显式标) + 守门 #10 (commit author=Ulysses) + 守门 #11 (缺标比错标, 2 处细化显式列) + 守门 #14 v3/v4 (5 域 Lead Mavis 临时代签 + Mavis 审核 author=Ulysses) + 守门 #9 v19 (Mavis 自驱, 19:14 JST 拍板 → 19:15 JST 1 commit 2 文件 ~1 分钟) + 守门 #1 v19 (本次不需 idempotent Python 脚本, 仅 2 文件各 +1 行)
> **落档文件** (关联 commit `6f693ca`, 2 files / 4 insertions):
> - `docs/requirements/SRS-CANVAS-AGENT-001.md` **v1.3** (1959 行, §0.2 修订履历 + §12 修订历史 各 +1 行 v1.2 (A12 多人编辑 v0.63 反转, 17:34 JST) + v1.3 (IPA SEC 修复, 19:10 JST), 13 段已合规无需加段, +3 lines)
> - `docs/requirements/SRS-CANVAS-GAMIFY-001.md` **v0.1.1** (1338 行, §11 修订履历 +1 行 v0.1.1 (IPA SEC 修复, 19:10 JST), 12 段已合规无需加段, +1 line)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.6-1 | D5.6-1 | `docs/requirements/SRS-CANVAS-AGENT-001.md` v1.3 (13 段已合规, 修订履历 v1.2 + v1.3 +1) | A | **[P]** | (Edit tool) | §0.2 修订履历 + §12 修订历史 各 +1 行 v1.2 (A12 多人编辑 v0.63 反转, 17:34 JST 拍板"多人编辑是要的") + v1.3 (IPA SEC 修复, 19:10 JST 拍板"确保这里的文档符合日本 IPA 标准"). 不重写 v0.1/v1.0/v1.1 旧内容 (per 守门 #1 禁回溯叙事). 段结构 13 段 (含 §11 签字栏 + §12 修订历史) 已合规, 无需加段. |
| D5.6-2 | D5.6-2 | `docs/requirements/SRS-CANVAS-GAMIFY-001.md` v0.1.1 (12 段已合规, 修订履历 v0.1.1 +1) | A | **[P]** | (Edit tool) | §11 修订履历 +1 行 v0.1.1 (IPA SEC 修复, 19:10 JST). 段结构 12 段 (含 §10 5 角色签字栏 + §11 修订履历) 已合规 (per §4.30 commit `0d6cecf` v0.1.1 修复). |
| D5.6-3 | D5.6-3 | `docs/automation-design.md` §4.31 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.6-4 | D5.6-4 | `scripts/automation/registry.md` §3 v0.16 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.16 修订历史 (19:14 JST IPA SEC v2 拍板) |
| D5.6-5 | D5.6-5 | 1 commit author = `Ulysses <ulysses@mavis.local>` (commit `6f693ca`) | A | **[P]** | (git commit) | 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 Mavis 审核 author=Ulysses; 不推 origin (守门 #1 反转后 R-05) |

**§4.31 任务卡维度判定**:
- R (Rerunnable): **是** (Edit tool 1 commit 2 文件 idempotent)
- V (Volume): 否 (无子代理派发, Mavis 接手 root session 一次性, 0 子代理调用)
- S (Structural): 否 (不修改段结构, 仅修订履历 +1 行)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 6f693ca) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 子代理调用 + 守门 #1 禁回溯叙事 不重写 §0-§12 旧内容 + 守门 #11 缺标比错标 2 处细化显式列 + 守门 #14 v2/v3/v4 代签 / 审核 规则全备)

**§4.31 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow docs/requirements/SRS-CANVAS-AGENT-001.md` 实证 v1.3 落档 (commit `6f693ca`, +3 lines: §0.2 + §12 修订履历 v1.2 + v1.3)
- `git log -p --follow docs/requirements/SRS-CANVAS-GAMIFY-001.md` 实证 v0.1.1 落档 (commit `6f693ca`, +1 line: §11 修订履历 v0.1.1)
- 2 处 IPA SEC 细化修复 (per 守门 #11 缺标比错标): (1) SRS-AGENT-001 §0.2 + §12 修订履历 缺 v1.2 (A12 多人编辑 v0.63 反转) + v1.3 (IPA SEC 修复) row → 加 2 行; (2) SRS-GAMIFY-001 §11 修订履历 缺 v0.1.1 (IPA SEC 修复) row → 加 1 行
- 0 段结构变更: 13 段 (SRS-AGENT-001) + 12 段 (SRS-GAMIFY-001) 跟 §4.30 commit `0d6cecf` 后一致
- 0 §0-§12 旧内容重写 (per 守门 #1 禁回溯叙事, v1.3/v0.1.1 反转行显式标)
- 守门 #1 v19: 0 子代理调用 (IPA SEC v2 修复期), Mavis 接手 root session 一次性
- 守门 #9 #3: 0 子代理调用 (IPA SEC v2 修复期), 不派二级子代理
- 守门 #9 v19: Mavis 自驱, 拍板后立即执行 (19:14 JST 拍板 → 19:15 JST 1 commit 2 文件 ~1 分钟)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签)
- 守门 #11 缺标比错标: 2 处 IPA SEC 细化显式列 (SRS-AGENT-001 修订履历缺 v1.2/v1.3 + SRS-GAMIFY-001 修订履历缺 v0.1.1)
- 守门 #14 v3: 不动 5 角色签字栏 (SRS-AGENT-001 §11 + SRS-GAMIFY-001 §10 已存在, 维持 Mavis 接手代签)
- 守门 #14 v4: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST v0.62 反转)
- 守门 #1 禁回溯叙事: 不重写 §0-§12 旧内容, v1.3/v0.1.1 反转行显式标 (per 守门 #1 + 守门 #11 缺标比错标)

**§4.31 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本 IPA SEC v2 细化修复期 (root Mavis 接手 + Edit tool 2 文件各 +1 行): ~0.02M tokens
- 累计 P3-D.5 全部 12 commit (4 SRS + 1 总册 BD + 2 专题 BD + 1 总册 DD + 2 专题 DD + 1 协调性检查 + 1 自审 + 1 IPA SEC 修复 + 1 IPA SEC v2 修复): ~3.43M tokens (2.86 SRE·周, 累计 12 commit)
- 后续 P3-D.6 启动实装 (P0 36 项): ~3-5M tokens
- 双核心 78 项 全部落地 (12 个月+): ~13-19M (13-19 SRE·周, per STAR-OLU-001)"""


# registry v0.16 row
V016_ROW = """| **v0.16** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (2 文档已落, 索引同步) + §3 v0.16**: P3-D.5 IPA SEC 合规性 v2 细化修复 (per 19:14 JST Ulysses 再次拍板"确保这里的文档符合日本 IPA 标准"), 关联 commit `6f693ca` (2 files / 4 insertions, per 守门 #1 v15 docs 同步饱和第 81 次新事件触发 + 守门 #1 禁回溯叙事不重写 §0-§12 旧内容 + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #14 v3 Mavis 永久代签 + 守门 #11 缺标比错标 2 处细化显式列): 2 文档 修订履历 +1 行 (段结构 13 段 + 12 段 已合规, 无需加段) — (a) `docs/requirements/SRS-CANVAS-AGENT-001.md` v1.3 (§0.2 修订履历 + §12 修订历史 各 +1 行 v1.2 (A12 多人编辑 v0.63 反转) + v1.3 (IPA SEC 修复), 13 段已合规, +3 lines); (b) `docs/requirements/SRS-CANVAS-GAMIFY-001.md` v0.1.1 (§11 修订履历 +1 行 v0.1.1 (IPA SEC 修复), 12 段已合规, +1 line) | **2026-09-10 19:14 JST Ulysses 再次拍板"确保这里的文档符合日本 IPA 标准" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标 2 处细化显式列)** |"""


def main() -> int:
    """Append §4.31 to automation-design.md and v0.16 row to registry.md."""
    # Append §4.31 to automation-design.md
    if not AUTOMATION_DESIGN_PATH.exists():
        print(f"ERROR: {AUTOMATION_DESIGN_PATH} not found", file=sys.stderr)
        return 1

    auto_content = AUTOMATION_DESIGN_PATH.read_text(encoding="utf-8")
    print(f"Read {len(auto_content)} bytes from {AUTOMATION_DESIGN_PATH.name}")

    if "### 4.31 P3-D.5 IPA SEC 合规性 v2 细化修复" in auto_content:
        print(f"  {AUTOMATION_DESIGN_PATH.name}: §4.31 already exists, skipping")
    else:
        if not auto_content.endswith("\n"):
            auto_content = auto_content + "\n"
        new_content = auto_content + SECTION_4_31 + "\n"
        AUTOMATION_DESIGN_PATH.write_text(new_content, encoding="utf-8")
        print(f"  Appended §4.31 ({len(SECTION_4_31)} chars)")
        print(f"  New file size: {len(new_content)} bytes")

    # Append v0.16 row to registry.md
    if not REGISTRY_PATH.exists():
        print(f"ERROR: {REGISTRY_PATH} not found", file=sys.stderr)
        return 1

    reg_content = REGISTRY_PATH.read_text(encoding="utf-8")
    print(f"Read {len(reg_content)} bytes from {REGISTRY_PATH.name}")

    if "**v0.16**" in reg_content:
        print(f"  {REGISTRY_PATH.name}: v0.16 already exists, skipping")
    else:
        if not reg_content.endswith("\n"):
            reg_content = reg_content + "\n"
        new_content = reg_content + V016_ROW + "\n"
        REGISTRY_PATH.write_text(new_content, encoding="utf-8")
        print(f"  Appended v0.16 row ({len(V016_ROW)} chars)")
        print(f"  New file size: {len(new_content)} bytes")

    return 0


if __name__ == "__main__":
    sys.exit(main())
