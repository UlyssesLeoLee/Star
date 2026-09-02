# scripts/automation/registry.md — Agent 交互自动化脚本索引

> **文档版本**: v0.1 (2026-09-02)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**: 2026-09-02 00:39 JST Ulysses 指令"所有涉及与 agent 交互的功能点,都应该尽可能使用 python 脚本" + 拍板 "新建 docs/automation-design.md + scripts/automation/ 落档"
> **依赖**: `docs/automation-design.md` v0.1 (§6 基类骨架 + §6.8 索引)
> **校验**: `python scripts/automation/registry_check.py` 校验索引一致性

---

## 0. 索引说明

本索引跟踪 `scripts/automation/` 下所有 python 脚本的:
- **路径**: 相对仓库根的路径
- **用途**: 1 行简述
- **调用方**: 哪些任务卡 (§4 任务卡表) 调用
- **末次 commit**: 7 字符短码 (per 守门 #1 禁回溯叙事)
- **状态**: 🟢 完成 / 🟡 stub / 🔴 阻塞

**约束 (per 守门 #12 派生 v2)**:
- 任何 [P] 子项落档后必更新本索引
- 索引跟实际脚本不一致 → `registry_check.py` 输出 warning, 不阻塞 CI

---

## 1. 脚本索引表

| 脚本路径 | 用途 | 调用方 | 末次 commit | 状态 |
|---|---|---|---|---|
| `scripts/automation/__init__.py` | 包初始化, 暴露 4 基类 + CLI | 全部 | TBD | 🟢 完成 |
| `scripts/automation/dispatcher.py` | 子代理 dispatch 基类 (per §3.1 + §6.1) | H2-1/H2-2/H2-3/H2-4/H2-5 (refactor_template 调用) | TBD | 🟡 stub (invoke / verify / collect_output 待对接 Mavis task 调度) |
| `scripts/automation/cli_helper/__init__.py` | cli_helper 子包初始化 | 全部 | TBD | 🟢 完成 |
| `scripts/automation/cli_helper/base.py` | CLI 调用基类 (per §3.2 + §6.2) | P3-B.1/B.2/B.5/B.6/B.7/B.8/B.9, P3-C.7, P3-D.2/D.3/D.5/D.6, P3-E.4/E.6, P3-F.5/F.6 | TBD | 🟡 stub (cargo / git / wt 子命令待补全) |
| `scripts/automation/refactor_template.py` | 代码改造基类 (per §3.3 + §6.3) | H2-1/H2-2/H2-3/H2-4/H2-5, 后续 P0-1 19 脚本改写 | TBD | 🟡 stub (子类化 + git stash rollback 待补) |
| `scripts/automation/judge.py` | 任务卡 [P]/[S]/[M] 判定 CLI (per §2.3 + §6.5) | WBS 任务卡全过初判 | TBD | 🟢 完成 (WBS 41 子项初判已落档 §4 任务卡表) |
| `scripts/automation/smoke_test.py` | 4 基类 smoke 验证 (per §6.6) | CI 守门基线 step 6 | TBD | 🟢 完成 (4 case 跑通) |
| `scripts/automation/registry_check.py` | 索引一致性校验 (per §6.7) | CI 守门基线 step 7 | TBD | 🟢 完成 (warning 不阻塞) |
| `scripts/automation/charts_p0_setup.py` | P0 图表基础设施 + C01 完整跑通 (per docs/briefs/P3-CHARTS-P0.md) | CHARTS-P0 阶段 1 (Recharts 3 依赖 + crates/domain-report 12 Rust + frontend 4 文件 + 19/19 测试) | TBD | 🟢 完成 (16 文件写入, 19/19 测试 pass, 0 err / 0 clippy) |

**说明**:
- 末次 commit 列填 `TBD` = 本批次 v0.1 初版, commit 落地后回填
- 状态 🟡 stub = 框架落地, 真实对接 (Mavis task 调度 / cargo 子命令 / git stash) 待续
- 状态 🟢 完成 = 框架 + smoke + 真实对接都完成 (本批次 4 份: __init__ × 2 + judge + smoke_test + registry_check)

---

## 2. 子代理任务索引 (per dispatcher.py 落档)

| task_id | brief 路径 | output 路径 | status.json 路径 | 调用方 |
|---|---|---|---|---|
| (待续) | `docs/briefs/<task_id>.md` | `docs/briefs/<task_id>.output.md` | `docs/briefs/<task_id>.status.json` | (待续) |

**说明**: 本表 v0.1 初版为空, 跨 session 续做 (H2 强类型重构 / P3-B.5/B.6 真实 e2e 等) 落档后回填。

---

## 3. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 8 份脚本索引 (__init__.py × 2 + dispatcher + cli_helper/base + refactor_template + judge + smoke_test + registry_check), 任务卡调用方映射, 状态 🟢/🟡 分档 | 2026-09-02 00:39 JST 拍板 "新建 docs/automation-design.md + scripts/automation/ 落档" + 守门 #12 派生 v2 |

---

## 4. 引用文档

- `docs/automation-design.md` v0.1 (上游: 设计文档)
- `scripts/automation/smoke_test.py` (校验脚本, 跑通 4 case)
- `scripts/automation/registry_check.py` (校验脚本, 索引一致性)
- `AGENTS.md` §4.1 守门派生 v19/v20/v21 (待追加, per §5 守门基线)
