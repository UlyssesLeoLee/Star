#!/usr/bin/env python3
"""arch-live 文档页截图预览生成器.

用途
    为 docs/arch-live/*.html 生成整页 PNG 预览, 用于 (a) 人工核对渲染,
    (b) 视觉回归对比. 纯本地文件, 不访问网络.

守门
    守门 #19 v19 (agent 交互 Python 化) —— 浏览器交互一律走本脚本, 不手敲一次性命令.
    守门 #1 累积规 —— 任何阶段缺 "实测渲染证据" = 守门不完整.

用法
    pwsh -NoProfile -File scripts/automation/arch_live_shot.py
    python scripts/automation/arch_live_shot.py --pages index.html 03-request-lifecycle.html
    python scripts/automation/arch_live_shot.py --self-test

fail-closed
    * 页面数为 0  → exit 2 (区别于"渲染成功 0 违规")
    * 任意一页截图缺失/过小 → exit 2
    * playwright 不可用 → exit 3 (环境缺失, 不是文档问题)
"""

from __future__ import annotations

import argparse
import pathlib
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parents[2]
DOCS = REPO / "docs" / "arch-live"
OUT = DOCS / ".preview"
VIEWPORT = "1440,900"
# 必须 > 揭示兜底(1200ms) + 数字滚动动画(1100ms), 否则整页截图会拍到"数字还在滚"的中间态,
# 让预览图出现 408/117 这种既不是 0 也不是真值的假数字 (实测踩过)。
WAIT_MS = "3200"

# 页面清单 = 文档契约, 少一页就说明文档被删了 (对比 README 索引)
PAGES = [
    "index.html",
    "01-crate-map.html",
    "02-navigation-model.html",
    "03-request-lifecycle.html",
    "04-agent-runtime.html",
    "05-worktree-canvas.html",
    "06-data-model.html",
    "07-deploy-topology.html",
    "08-guardrails.html",
]


def playwright_cmd() -> pathlib.Path | None:
    for rel in (
        "frontend/node_modules/.bin/playwright.cmd",
        "frontend/node_modules/.bin/playwright",
        "node_modules/.bin/playwright.cmd",
    ):
        cand = REPO / rel
        if cand.exists():
            return cand
    return None


def shoot(pw: pathlib.Path, page: str) -> tuple[bool, str]:
    src = DOCS / page
    dst = OUT / (page.replace(".html", ".png"))
    if not src.exists():
        return False, f"源页面不存在: {src}"
    cmd = [
        str(pw), "screenshot",
        "--full-page",
        f"--viewport-size={VIEWPORT}",
        f"--wait-for-timeout={WAIT_MS}",
        src.resolve().as_uri(),
        str(dst),
    ]
    try:
        # encoding 必须显式给 utf-8: Windows 默认 GBK 解码子进程输出, 遇到非 ASCII
        # 会抛 UnicodeDecodeError, 表现为"截图脚本崩了"而其实只是控制台编码 (实测踩过)。
        proc = subprocess.run(cmd, cwd=str(REPO), capture_output=True, timeout=120,
                              encoding="utf-8", errors="replace")
    except subprocess.TimeoutExpired:
        return False, "playwright 超时(120s)"
    if proc.returncode != 0:
        tail = (proc.stderr or proc.stdout or "").strip().splitlines()[-3:]
        return False, f"exit={proc.returncode} {' | '.join(tail)}"
    if not dst.exists() or dst.stat().st_size < 4096:
        return False, f"产物缺失或过小: {dst}"
    return True, f"{dst.stat().st_size // 1024} KiB"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--pages", nargs="*", default=None)
    ap.add_argument("--self-test", action="store_true",
                    help="注入一个必然不存在的页面, 验证 fail-closed 是否真的报错")
    args = ap.parse_args()

    if args.self_test:
        ok, msg = False, "x"
        # 自测: 走一遍"页面缺失"分支, 断言它返回 False
        fake = shoot_result_probe()
        print(f"self-test: missing-page 判定 = {fake} (期望 False)")
        return 0 if fake is False else 2

    pages = args.pages or PAGES
    if not pages:
        print("FAIL: 页面清单为空, 无法区分'渲染成功'与'什么都没做'", file=sys.stderr)
        return 2

    pw = playwright_cmd()
    if pw is None:
        print("FAIL: 未找到 playwright (frontend/node_modules/.bin/playwright.cmd)", file=sys.stderr)
        print("      修法: cd frontend; npm ci", file=sys.stderr)
        return 3

    OUT.mkdir(parents=True, exist_ok=True)
    failed: list[tuple[str, str]] = []
    for page in pages:
        ok, msg = shoot(pw, page)
        print(f"{'OK  ' if ok else 'FAIL'} {page:<28} {msg}")
        if not ok:
            failed.append((page, msg))

    if failed:
        print(f"\nFAIL: {len(failed)}/{len(pages)} 页未生成预览", file=sys.stderr)
        for p, m in failed:
            print(f"  - {p}: {m}", file=sys.stderr)
        return 2
    print(f"\n全部 {len(pages)} 页预览已生成: {OUT}")
    return 0


def shoot_result_probe() -> bool:
    """自测用: 故意请求不存在的页面, 断言判定为失败."""
    pw = playwright_cmd()
    if pw is None:
        return True  # 环境缺失, 视为无法通过自测
    ok, _ = shoot(pw, "__not_a_real_page__.html")
    return ok


if __name__ == "__main__":
    raise SystemExit(main())
