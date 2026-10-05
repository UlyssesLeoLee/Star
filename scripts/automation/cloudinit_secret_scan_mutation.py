#!/usr/bin/env python3
"""cloudinit_secret_scan.py 的变异测试（守门 #19 v19 派生脚本）。

per 2026-10-04 教训, 门禁的变异测试必须满足三条, 缺一条门禁就只是装饰:
  1. 解析/执行结果为 0 必须区分"真的干净"与"检查根本没跑" → 每个用例断言
     **诊断文本本身出现**, 而不只是 exit code。否则"因为错误的原因变红/变绿"
     与正确结果无法区分。
  2. 必须有对照组(不注入的干净样本), 否则无法区分"门禁能发现违规"与
     "测试装置永远失败"。
  3. 必须有反例守卫(正常写法必须放行), 防止把宽泛规则写进去变成永久误报。

2026-10-05 实证: 本文件第一次跑时 Mutation 2 (密钥藏注释行) 绿了 ——
即第二版扫描器把"注释"变成了绕过密钥检测的藏身处。抓到它的正是这里。
"""
from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

# Windows 中文控制台默认 GBK 编码, 直接 print 子进程解出来的 UTF-8 中文会
# UnicodeEncodeError 把测试装置自己搞崩 ——
# per 2026-10-04 教训: "因错误原因失败"与"真的失败"必须可区分, 崩掉的测试不算数。
# 修法: 强制 stdout/stderr 走 UTF-8 (errors=replace 兜底)。
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

SCRIPT = Path(__file__).with_name("cloudinit_secret_scan.py")

# 期望诊断文本: 用例失败时若这条子串不出现, 说明"因错误原因"失败。
# 诊断片段统一用 ASCII, 避免与控制台编码耦合。
CASES: list[tuple[str, str, int, str | None]] = [
    # (用例名, 内容, 期望 exit, 期望出现在输出中的诊断片段)
    (
        "control_clean",
        "#cloud-config\npackage_update: true\npackages:\n  - curl\n",
        0,
        "PASS",
    ),
    (
        "mut_pem_key_value",
        "package_update: true\nwrite_files:\n  - path: /tmp/k\n    content: |\n"
        "      -----BEGIN RSA PRIVATE KEY-----\n      MIIBOgIBAAJBAKj34GkxFhD90vcNLYLInFEX6Ppy1tPf9\n"
        "      -----END RSA PRIVATE KEY-----\n",
        1,
        "PEM",
    ),
    (
        "mut_key_hidden_in_comment",
        "#cloud-config\npackage_update: true\n# -----BEGIN RSA PRIVATE KEY-----\n",
        1,
        "PEM",
    ),
    (
        "mut_github_token",
        "package_update: true\nruncmd:\n  - echo ghp_abcdefghijklmnopqrstuvwxyz0123456789\n",
        1,
        "github token",
    ),
    (
        "mut_jwt_private_value",
        'write_files:\n  - path: /etc/star/jwt\n    content: "JWT_PRIVATE_KEY_PEM: MIIEvQ"\n',
        1,
        "JWT",
    ),
    (
        "guard_variable_name_in_comment_is_allowed",
        "#cloud-config\n# guard5: this file carries no JWT_PRIVATE_KEY_PEM / JWT_PUBLIC_KEY_PEM value\n"
        "package_update: true\n",
        0,
        "PASS",
    ),
]


def main() -> int:
    passed = failed = 0
    with tempfile.TemporaryDirectory() as td:
        for name, content, want_code, want_diag in CASES:
            p = Path(td) / f"{name}.yaml"
            p.write_text(content, encoding="utf-8")
            proc = subprocess.run(
                [sys.executable, str(SCRIPT), str(p)],
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
            )
            out = proc.stdout + proc.stderr
            code_ok = proc.returncode == want_code
            diag_ok = want_diag is None or want_diag in out
            if code_ok and diag_ok:
                print(f"  PASS  {name}  (exit={proc.returncode}, want {want_code}, diag '{want_diag}' seen)")
                passed += 1
            else:
                print(f"  FAIL  {name}")
                print(f"        exit  = {proc.returncode} (want {want_code}) -> {'ok' if code_ok else 'WRONG'}")
                print(f"        diag  = want '{want_diag}' -> {'ok' if diag_ok else 'MISSING'}")
                print(f"        output= {out.strip()[:300]}")
                failed += 1

    print(f"\nRESULT: PASS={passed} FAIL={failed} TOTAL={passed + failed}")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
