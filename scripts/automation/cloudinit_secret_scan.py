#!/usr/bin/env python3
"""Cloud-init 密钥材料扫描器（守门 #5 v2 派生）。

为什么需要独立成脚本而不是随手 grep —— 两次踩坑, 两次都只有变异测试能抓到:

  1. 2026-10-05 第一版用 Select-String 扫, 命中了文件里"本文件不含 secret"
     这句**注释**里的 JWT_PRIVATE_KEY_PEM 字样 —— 误报。守门若天天误报就会被
     当噪声忽略, 真漏了密钥反而没人看见。
  2. 于是改成"跳过全部注释行" —— **这引入了一个更严重的漏洞**: 变异测试注入
     "# -----BEGIN RSA PRIVATE KEY-----" 到注释行, 扫描器返回 exit 0 (放行)。
     即"注释"成了一个绕过密钥检测的藏身处。

  正确规则 (本版实现): 区分两类模式, 而不是一刀切跳过注释。
    - COMMENT_SENSITIVE  (含实际密钥材料) → 注释行**照扫**, 宁可误报
    - COMMENT_SAFE      (仅提及变量名)    → 注释行跳过, 避免噪声

  per 2026-10-04 教训: 证否要换判据 + 门禁必须做变异测试并带对照组。
  "扫描器报 0 命中"不能证明文件干净; 而且扫描器自身的变异测试比门禁本身更重要 ——
  这次抓到漏洞的是注入测试, 不是 review。
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

# Windows 中文控制台默认 GBK, print 中文诊断标签会 UnicodeEncodeError。
# 门禁脚本自己崩掉 = 门禁形同虚设(退出码不可信)。故强制 UTF-8 输出。
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")


def _p(label: str, pattern: str, comment_sensitive: bool) -> tuple[str, re.Pattern[str], bool]:
    return (label, re.compile(pattern), comment_sensitive)


# (label, regex, comment_sensitive)
PATTERNS: list[tuple[str, re.Pattern[str], bool]] = [
    # --- comment_sensitive=True: 注释里出现即为泄露, 一律扫 ---
    _p("PEM 私钥块", r"-----BEGIN [A-Z ]*PRIVATE KEY-----", True),
    _p("PEM 证书块", r"-----BEGIN CERTIFICATE-----", True),
    _p("aws_access_key_id", r"AKIA[0-9A-Z]{16}", True),
    _p("github token", r"gh[pousr]_[A-Za-z0-9]{36,}", True),
    _p(
        "裸长 base64 串(>=80 连续字符)",
        r"(?<![A-Za-z0-9+/=])[A-Za-z0-9+/]{80,}={0,2}(?![A-Za-z0-9+/=])",
        True,
    ),
    # --- comment_sensitive=False: 提及变量名本身正常(本文件头就在声明"不含 secret") ---
    _p("JWT 私钥 PEM 值", r"JWT_PRIVATE_KEY_PEM\s*[:=]\s*\S", False),
    _p("JWT 公钥 PEM 值", r"JWT_PUBLIC_KEY_PEM\s*[:=]\s*\S", False),
    _p("私钥文件写入", r"/etc/rancher/k3s/[\w./-]*\.(pem|key|crt)\b", False),
]


def is_comment(line: str) -> bool:
    return line.lstrip().startswith("#")


def scan_text(text: str) -> list[tuple[int, str, str]]:
    findings: list[tuple[int, str, str]] = []
    for lineno, line in enumerate(text.splitlines(), start=1):
        comment = is_comment(line)
        for label, pat, comment_sensitive in PATTERNS:
            # 注释行仍要检查 comment_sensitive 的模式 —— 见模块 docstring 的踩坑记录
            if comment and not comment_sensitive:
                continue
            if pat.search(line):
                findings.append((lineno, label, line.strip()))
    return findings


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: cloudinit_secret_scan.py <file> [file...]", file=sys.stderr)
        return 2

    findings: list[tuple[str, int, str, str]] = []
    scanned = 0

    for arg in sys.argv[1:]:
        path = Path(arg)
        if not path.is_file():
            print(f"FAIL: not a file: {path}", file=sys.stderr)
            return 2
        findings.extend(
            (str(path), lineno, label, line) for lineno, label, line in scan_text(path.read_text(encoding="utf-8"))
        )
        scanned += 1

    if scanned == 0:
        print("FAIL: no files scanned (scanner degenerate) — treating as invalid, not clean")
        return 2

    if findings:
        print(f"FAIL: {len(findings)} potential secret material hit(s)")
        for path, lineno, label, line in findings:
            print(f"  {path}:{lineno}  [{label}]  {line[:120]}")
        return 1

    print(f"PASS: {scanned} file(s) scanned, 0 secret material found")
    return 0


if __name__ == "__main__":
    sys.exit(main())
