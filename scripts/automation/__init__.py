# scripts/automation/__init__.py
# Agent 交互自动化基类库 (per docs/automation-design.md v0.1)
#
# 模块:
#   - dispatcher: 子代理 dispatch 基类 (per §3.1)
#   - cli_helper.base: CLI 调用基类 (per §3.2)
#   - refactor_template: 代码改造基类 (per §3.3)
#   - generate_ac_matrix: AC 矩阵生成器 (T.1 已实装范例, per §3.4)
#   - judge: 任务卡 [P]/[S]/[M] 判定 CLI (per §2.3)
#   - smoke_test: 4 基类 smoke 验证 (per §6.6)
#   - registry_check: 索引一致性校验 (per §6.7)
#
# 约束 (per 守门 #1 v1 + 守门 #12 + 缺标比错标):
#   - 标准库 only: re / csv / pathlib / argparse / sys / json / subprocess (无第三方)
#   - 入口 `if __name__ == "__main__":` 可直接 `python scripts/automation/<module>.py`
#   - 跨平台: Windows PowerShell / WSL / macOS / Linux 抽象
#   - audit_log 必填, 落 `docs/reports/<phase>.log`

__version__ = "0.1.0"
__all__ = [
    "dispatcher",
    "cli_helper",
    "refactor_template",
    "judge",
    "smoke_test",
    "registry_check",
]
