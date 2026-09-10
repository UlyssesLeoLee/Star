#!/usr/bin/env python3
"""v0.89 P0-4 Stage 3.5 = platform_admin PostgreSQL password generator

Per v0.88 已知缺口 (b) 'PASSWORD 占位需 env 替换' 跨 session 续做.

P0-4 阶段: 声明脚本 (实际跑由 P2 阶段 worker 子代理 + k3s-deployable 触发).
P2 阶段: 跑脚本生成 32 字节随机 password + 写 .env.star-platform-admin-password (不入 git)
         + 用 sealed-secrets controller 加密 + 替换 deploy/k3s-local/secrets/platform-admin-secret.yaml 占位.

Usage (P2 阶段):
    python scripts/secrets/platform_admin_password_gen.py \\
        --output .env.star-platform-admin-password \\
        --sealed-secret deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml

守门 #5 v2 env 安全:
- password 永远不入 git (.env.star-platform-admin-password + *.sealed.yaml 都加 .gitignore)
- 不打印 password (跟 v0.78 SCM token 字段一样, 只 log password_len)
- 32 字节 secrets.token_urlsafe 随机 (256 bits 熵)
"""
import argparse
import secrets
import sys
from pathlib import Path


def gen_password(length: int = 32) -> str:
    """生成 32 字节 url-safe 随机 password (per Python secrets.token_urlsafe 256 bits 熵)"""
    return secrets.token_urlsafe(length)


def write_env_file(path: Path, password: str) -> None:
    """写 .env.star-platform-admin-password (不入 git, 守门 #5 v2)"""
    content = f"""# v0.89 P2 阶段 generated (P0-4 阶段占位 'CHANGE_ME_AT_DEPLOY')
# 守门 #5 v2 env 安全: 此文件不入 git (per .gitignore 配 star-platform-admin-password)
# 用法: source .env.star-platform-admin-password 加载到 shell, 然后跑 sealed-secrets controller
# 轮换策略: 90d 自动轮换, per ADR-0043 + 守门 #5 v2
STAR_PLATFORM_ADMIN_DB_PASSWORD={password}
"""
    path.write_text(content, encoding="utf-8")


def write_sealed_yaml_template(path: Path, password: str) -> None:
    """写 SealedSecret yaml template (P2 阶段用 sealed-secrets controller 加密)"""
    # P0-4 阶段: 写明文 placeholder, P2 阶段用 sealed-secrets CLI 加密
    content = f"""# v0.89 P2 阶段 generated SealedSecret (P0-4 阶段占位)
# 实际部署: kubeseal --format yaml --secret .env.star-platform-admin-password > this file
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: platform-admin-db-password
  namespace: star-system
spec:
  encryptedData:
    password: <PLACEHOLDER_P2_RUNS_kubeseal_TO_ENCRYPT>
  template:
    metadata:
      name: platform-admin-db-password
      namespace: star-system
    type: Opaque
# 原始明文 password (仅本地, 不入 git):
# STAR_PLATFORM_ADMIN_DB_PASSWORD={password}
"""
    path.write_text(content, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="v0.89 platform_admin PG password generator (P0-4 阶段占位, P2 阶段实跑)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(".env.star-platform-admin-password"),
        help=".env 输出文件路径 (不入 git, per 守门 #5 v2)",
    )
    parser.add_argument(
        "--sealed-secret",
        type=Path,
        default=Path("deploy/k3s-local/secrets/platform-admin-secret.sealed.yaml"),
        help="SealedSecret yaml 输出路径 (P2 阶段 sealed-secrets CLI 加密)",
    )
    parser.add_argument(
        "--length",
        type=int,
        default=32,
        help="password 字节长度 (默认 32 = 256 bits 熵)",
    )
    args = parser.parse_args()

    password = gen_password(args.length)
    # 守门 #5 v2: 不打印 password 全文, 只 log 长度 + 写入文件
    print(
        f"[v0.89] 生成 password (len={len(password)}, entropy≈{len(password) * 6} bits)",
        file=sys.stderr,
    )
    write_env_file(args.output, password)
    print(f"[ok] 写入 {args.output} (不入 git, per .gitignore)", file=sys.stderr)
    write_sealed_yaml_template(args.sealed_secret, password)
    print(
        f"[ok] 写入 SealedSecret template {args.sealed_secret} (P2 阶段用 sealed-secrets CLI 加密)",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
