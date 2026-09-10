"""v0.99.4 重新评估 WBS row 同步脚本 (per 守门 #12 v21 [P] docs 同步必更新 WBS).

落地 3 改动:
1. docs/reports/STAR-P3-WBS-001.md append v0.99.4 row (idempotent)
2. scripts/automation/registry.md append v0.28 row
3. docs/automation-design.md append §4.34.6 任务卡 (P3-D.6 阶段 1 收官评估)
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import re
from pathlib import Path

ROOT = Path(r'D:/Star')

V094_WBS_ROW = (
    "| **v0.99.4** | **2026-09-10 21:20 JST** | **架构师 (Mavis 接手 agent per DEC-008) ? Mavis 审核 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 92 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 不重写 v0.99.1-v0.99.3 / v1.00-v1.03 row + 守门 #11 缺标比错标 11 缺口显式列)** | "
    "**§14.21 P3-D.6 阶段 1 收官 + 阶段 2 启动 重新评估** (per Ulysses 2026-09-10 21:18 JST 发令\"所有先进分支合并到 main 之后, 根据文档和代码状态重新评估更新 wbs\") | "
    "**收口**: (1) **git 状态**: HEAD = origin/main = `4185dc4` (本地 = origin 同步, 0 commit ahead, 0 commit behind, 不需 push); (2) **worktree 状态**: `git worktree list` 50 行, 含 **1 STALE** `wt-ex-02-star-mutex` (path 不存在, 需 `git worktree prune` + `git branch -D` 清理) + **44 ACTIVE** (per `git worktree list`, 实际都已 merge to main, 但 worktree 实体残留, 需批量 `git worktree remove --force` + `git branch -D`); "
    "(3) **守门 v3x 状态 (本 session 落地)**: 6 active + 2 obsolete + 3 候选 — "
    "**🟢 active**: v27 RPC fallback (9/10 11:00) + v28 推荐项格式 + v29 docs 同步饱和 + v32 Mavis 审核 author=Ulysses (v0.70) + **v33 任务卡留言机制 (v0.63)** + **v34 30min sustained 探活 (本次激活)**; "
    "**🟡 obsolete**: v30 永久代签边界 (v0.62 + v0.63 反转) + v31 5 域 Lead 真人到位追溯 (v0.62 + v0.63 反转); "
    "**🟡 候选**: **v35 v33 留言 GPG 签名 (P1 缺口, 本 session)** + **v36 audit log 1000+ 性能索引 (P2 缺口, 本 session)**; "
    "(4) **PreToolUse 安全拦截层 v0.1** (per SRS-PRE-TOOL-USE-GUARD-001.md + BD + DD + Spec + WBS-002): 5 module 落地 + 15 规则 (BLOCK 8 + ASK 5 + WARN 2) + 226 tests pass + commit `29b1b93` (5 docs + 21 文件); "
    "(5) **v33 任务卡留言机制 v0.1+v0.2+v0.3**: 4 module (`comment/list_comments/check_blocked/mark_read`) + 智能 check_blocked (parent_comment_id + 权威 actor 解除) + verify comments-read 软约束 (v0.3 缺口 #5 修复) + 6 file + 250+ tests pass; commits `6e1c1e9` (v0.1) + `1c2eb5e` (v0.2) + `572af5c` (v0.3); "
    "(6) **v0.62 + v0.63 反转落档** (per 守门 #14 v3 + #14 v4): 14 docs/recruitment/** + 2 守门 v30+v31 标 obsolete + AGENTS.md §4.1 v0.63 row 追加; commit `e053447` (18 文件改动); "
    "(7) **v25b 重号修复** (per AGENTS.md §4.1 (d) 守门 v3x 激活门槛 (d)): line 153 v25 → v25b (守门 #1 v25 CI cargo test 改单 crate 跳 workspace); line 152 v25 维持 (跟 v0.63 obsolete 标共存, 不重写 per 守门 #1 禁回溯叙事); commit `b360135` (2 文件改动); "
    "**待办**: (A) **P3-D.6 阶段 1 基础 任务 1.6** (3 份 brief 已落档 21:13-21:15: p3-d6-1-6-15-more-tables.md + p3-d6-1-6-p3d6-tables-impl.md + 1 份重复 需 0 改): 14+15 张 SQL DDL 落档 (A11 7 + A12 7 + G11 15 = 29 张表, 守门 #13 W/T/M 6M + 5T + 3W + 1T = 14 张新增, 0 改 V0.1 14 张 DDL), 估 0.30M tokens / 0.25 SRE·周; "
    "(B) **P3-D.6 阶段 1 基础 任务 1.7** (brief 已落档 21:15: p3-d6-1-7-25module-cross.md): 25 module 跨接口定型 (worktree + work-item + comment + notification + audit + search + setting + ...), 估 0.20M tokens; "
    "(C) **v1.01 5 已知缺口**: 1/5 闭合 (#5 cargo fmt baseline fix per commit `a1466aa` v1.03), 4/5 仍 缺口 (#1 dev env 无 Docker P0-4 / #2 rls_7_policy_gen.py 跨 26 表 idempotent 验证 / #3 PLATFORM_ADMIN password 部署 + sealed-secrets / #4 端到端测试 5 守门), 等 P2 阶段 worker 实跑; "
    "(D) **50 worktree 清理**: 1 STALE `wt-ex-02-star-mutex` + 44 ACTIVE 实际已 merge, 需批量 `git worktree prune` + `git worktree remove --force` + `git branch -D`; "
    "(E) **守门 v35 拍板激活** (5 落地文件 ~480 LOC, P1 缺口修复, 需 GPG key 落地 P0); "
    "(F) **守门 v36 拍板激活** (4 落地文件 ~490 LOC, P2 性能); "
    "(G) **PreToolUse 守门 v0.2 子代理 sandbox** (SRS 已知缺口 #1 P0 阻塞, ~2-3h token); "
    "**评估**: P3-D.6 阶段 1 5/7 收官 (任务 1.1-1.5 收), 阶段 2 业务 + P2 阶段 + 阶段 3-4 待续; **本 session 8 commit 实证 (per 守门 #1 v15 docs 同步饱和)**: `29b1b93` + `6e1c1e9` + `9b25fd6` + `e053447` + `b360135` + `1c2eb5e` + `5771bc7` + `572af5c` + `1843511` + `6d06091` + `4185dc4` = 11 commit; 跨项目 (STAR / RGS / Physis / GVPE 0 双向同步 per Ulysses 双仓并行守门 + v0.63 真人寻访 obsolete Mavis 永久代签) | "
    "2026-09-10 21:20 JST Mavis 自驱 (per 守门 #9 v19) + 守门 #1 v15 docs 同步饱和第 92 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 不重写 v0.99.1-v0.99.3 / v1.00-v1.03 + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #11 缺标比错标 11 缺口显式列 |"
)

V028_REG_ROW = (
    "| **v0.28** | **2026-09-10 21:20 JST** | 架构师 (Mavis 接手 agent per DEC-008) ? Mavis 审核 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 92 次新事件触发 仍允许) | "
    "P3-D.6 阶段 1 收官 + 阶段 2 启动重新评估 (per Ulysses 2026-09-10 21:18 JST 发令): "
    "v0.99.4 WBS row 落档 (本 session 8 commit 实证 + 11 缺口显式列 + P3-D.6 阶段 1 5/7 收 + 50 worktree 待清理 + 1 STALE `wt-ex-02-star-mutex` 需 git worktree prune); "
    "守门 v3x 状态: 6 active (v27/v28/v29/v32/v33/v34) + 2 obsolete (v30/v31) + 3 候选 (v35/v36 落档 v0.1 候选 + v37 跟 v1.01 5 已知缺口联动); "
    "v1.01 5 已知缺口: 1/5 闭合 (#5 cargo fmt baseline fix per commit `a1466aa` v1.03) + 4/5 仍 缺口 (#1 dev env / #2 RLS idempotent / #3 PLATFORM_ADMIN / #4 端到端) | "
    "2026-09-10 21:20 JST Mavis 自驱 (per 守门 #9 v19) + 守门 #1 v15 docs 同步饱和第 92 次新事件触发 仍允许 + 守门 #14 v4 Mavis 审核 author=Ulysses |"
)


def main():
    modified = []

    # 1. WBS row
    wbs_path = ROOT / 'docs' / 'reports' / 'STAR-P3-WBS-001.md'
    wbs_text = wbs_path.read_text(encoding='utf-8')
    if '| **v0.99.4**' not in wbs_text:
        # 找最后一行 (WBS 表末尾, 通常是分隔符或 foot note)
        lines = wbs_text.splitlines()
        # 找 v0.99.3 row 位置
        last_v_row_idx = max(
            i for i, line in enumerate(lines)
            if line.startswith('| **v0.')
        )
        # 在 v0.99.3 后面 + 1 行后插入
        insert_idx = last_v_row_idx + 1
        # 如果 v0.99.3 row 是 table 最后一行, 找 table 结束 (|---| 下一行)
        # 实际 WBS 表是连续的 v0.X rows, 直接在 v0.99.3 后面插
        new_lines = lines[:insert_idx] + [V094_WBS_ROW] + lines[insert_idx:]
        wbs_path.write_text('\n'.join(new_lines) + '\n', encoding='utf-8')
        modified.append('docs/reports/STAR-P3-WBS-001.md')

    # 2. registry.md v0.28 row
    reg_path = ROOT / 'scripts' / 'automation' / 'registry.md'
    reg_text = reg_path.read_text(encoding='utf-8')
    if '| **v0.28**' not in reg_text:
        lines = reg_text.splitlines()
        last_v_row_idx = max(
            i for i, line in enumerate(lines)
            if line.startswith('| **v0.')
        )
        insert_idx = last_v_row_idx + 1
        new_lines = lines[:insert_idx] + [V028_REG_ROW] + lines[insert_idx:]
        reg_path.write_text('\n'.join(new_lines) + '\n', encoding='utf-8')
        modified.append('scripts/automation/registry.md')

    # 3. automation-design.md §4.34.6 任务卡
    ad_path = ROOT / 'docs' / 'automation-design.md'
    ad_text = ad_path.read_text(encoding='utf-8')
    if '§4.34.6' not in ad_text:
        # 找 §4.34.5 位置
        sec4345_idx = ad_text.find('### §4.34.5')
        if sec4345_idx >= 0:
            # 在 §4.34.5 段后插 §4.34.6
            new_sec = (
                '\n\n### §4.34.6 P3-D.6 阶段 1 收官 + 阶段 2 启动重新评估 (v0.99.4 row, per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡)\n\n'
                '**触发**: per Ulysses 2026-09-10 21:18 JST 发令"所有先进分支合并到 main 之后, 根据文档和代码状态重新评估更新 wbs"\n\n'
                '**评估内容**:\n'
                '- (a) git 同步: HEAD = origin/main = `4185dc4` (本地 = origin, 0 ahead 0 behind)\n'
                '- (b) worktree 状态: 50 行, 1 STALE `wt-ex-02-star-mutex` + 44 ACTIVE (实际都 merge 了, 实体残留需清理)\n'
                '- (c) 守门 v3x 状态: 6 active (v27/v28/v29/v32/v33/v34) + 2 obsolete (v30/v31) + 3 候选 (v35/v36 落档 v0.1 + v37 待)\n'
                '- (d) PreToolUse 安全拦截层 v0.1 (commit `29b1b93`): 5 module + 15 规则 + 226 tests\n'
                '- (e) v33 任务卡留言机制 v0.1+v0.2+v0.3: 4 module + 智能解除 + verify 软约束 (250+ tests)\n'
                '- (f) v0.62 + v0.63 反转 (commit `e053447`): 14 docs/recruitment/** + 2 守门 obsolete 标\n'
                '- (g) v25b 重号修复 (commit `b360135`): 守门 v3x 激活门槛 (d) 同步\n'
                '- (h) v34 拍板激活 (commit `1843511`): 30min sustained 探活\n'
                '- (i) v35 + v36 候选 (commit `6d06091` + `4185dc4`): GPG 签名 + audit 索引\n\n'
                '**收口 (P3-D.6 阶段 1 5/7 收)**:\n'
                '- ✅ 任务 1.1 agent-domain: `eb73e0d` 之前\n'
                '- ✅ 任务 1.2 arg-bridge extend: `fd33ffb` 之前\n'
                '- ✅ 任务 1.3 canvas-collab: 之前\n'
                '- ✅ 任务 1.4 api 2 module: `2733c4d`\n'
                '- ✅ 任务 1.5 bff+envoy: `3975bf2`\n'
                '- ⏳ 任务 1.6 14+15 tables: 3 brief 21:13-21:15 落档, 未实装 (估 0.30M tokens)\n'
                '- ⏳ 任务 1.7 25 module cross: brief 21:15 落档, 未实装 (估 0.20M tokens)\n\n'
                '**v1.01 5 已知缺口** (per 守门 #11 缺标比错标):\n'
                '- #1 dev env 无 Docker (P0-4 阶段 0 实跑): ⏳ 仍 缺口, 等 P2 阶段 worker 实跑\n'
                '- #2 rls_7_policy_gen.py 跨 26 表 idempotent 验证: ⏳ 仍 缺口, P2 阶段 worker 实跑补\n'
                '- #3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装: ⏳ 仍 缺口, P2 阶段 worker 实跑\n'
                '- #4 端到端测试 5 守门 0 违反: ⏳ 仍 缺口, P2 阶段 worker 实跑补\n'
                '- #5 cargo fmt pre-existing baseline: ✅ 闭合 (per commit `a1466aa` v1.03, 31 file +224/-149)\n\n'
                '**待办** (per 守门 #11 缺标比错标):\n'
                '- (1) 派 worker 子代理实装任务 1.6 (14+15 tables DDL, brief 已落档)\n'
                '- (2) 派 worker 子代理实装任务 1.7 (25 module 跨接口定型, brief 已落档)\n'
                '- (3) 50 worktree 清理 (1 STALE + 44 ACTIVE 实体残留)\n'
                '- (4) 守门 v35 (GPG 签名) + v36 (audit 索引) 拍板激活\n'
                '- (5) PreToolUse 守门 v0.2 子代理 sandbox (P0 阻塞)\n'
                '- (6) P2 阶段 worker 子代理实跑触发 (等守门 #1 v15 + 4 守门满足)\n\n'
                '**作者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 反转 v0.62 Mavis 审核 author=Ulysses)\n'
            )
            new_text = ad_text[:sec4345_idx] + new_sec + ad_text[sec4345_idx:]
            ad_path.write_text(new_text, encoding='utf-8')
            modified.append('docs/automation-design.md')

    print(f'Modified {len(modified)} files:')
    for m in modified:
        print(f'  {m}')


if __name__ == '__main__':
    main()
