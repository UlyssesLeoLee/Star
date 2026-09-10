"""comment_signing.py — 任务卡留言 ed25519 签名/验证 (per 守门 v35)

Per docs/guardian/v35_gpg_signed_comments.md §1.1 + §1.2:
  - 每条留言带 _signature + _signed_payload + _signing_actor 3 字段
  - canonical payload = "id|ts|author|body|blocks|tags" 6 字段
  - 验证走 cryptography library ed25519 (GPG CLI 不可用降级, per 已知缺口 #4)

跟现有守门关系:
  - #5 env 安全派生: 同理 author 字段必可信, 0 验证 = 子代理可伪造
  - v33 v0.2 智能 check_blocked: 权威 actor 留言必签名有效 (软约束 v0.1)
  - v34 30min 探活: 触发 docs/briefs/ 留言签名验证

已知缺口 (per v35 已知缺口):
  - #1 现有留言反向兼容: legacy 留言无签名 → verify 返 False, 仍按 BLOCK 处理 (per 缺标比错标)
  - #2 软约束: 签名无效 → warn log, 仍按 BLOCK 处理 (per 缺标比错标 #11)
  - #3 GPG key 落地: 真实 key pair 用 ed25519 cryptography library 生成 (PEM 格式, 无密码)
  - #4 GPG CLI 不可用 → 改用 cryptography library (等价 RSA/ed25519 加密保证)
  - #5 Mavis 跟 Ulysses 共享 key 引发责任混淆: v0.1 简化, v0.2 拆 Mavis 独立 key
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import base64
import logging
from pathlib import Path
from typing import Optional, Tuple

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)

logger = logging.getLogger(__name__)

# 默认 key 目录: docs/keys/ (per 守门 v35 §1.4 落地清单 #4)
DEFAULT_KEYS_DIR = Path("docs/keys")

# 权威 actor 列表 (跟 dispatcher.check_blocked 一致, per 守门 v33 v0.2)
AUTHORITATIVE_ACTORS = ("Ulysses", "Mavis", "architect")


def _key_paths(actor: str, keys_dir: Path = DEFAULT_KEYS_DIR) -> Tuple[Path, Path]:
    """actor 的 private + public PEM 路径 (per 守门 v35 §1.4)."""
    name = actor.lower()
    return (keys_dir / f"{name}.private.pem", keys_dir / f"{name}.public.pem")


def ensure_keypair(actor: str, keys_dir: Path = DEFAULT_KEYS_DIR) -> Tuple[Ed25519PrivateKey, Ed25519PublicKey]:
    """确保 actor 的 ed25519 key pair 存在, 不存在自动生成 (per 守门 v35 §1.4).

    Returns:
        (private_key, public_key) tuple
    """
    priv_path, pub_path = _key_paths(actor, keys_dir)
    keys_dir.mkdir(parents=True, exist_ok=True)
    if priv_path.exists() and pub_path.exists():
        try:
            priv = serialization.load_pem_private_key(priv_path.read_bytes(), password=None)
            pub = serialization.load_pem_public_key(pub_path.read_bytes())
            if isinstance(priv, Ed25519PrivateKey) and isinstance(pub, Ed25519PublicKey):
                return priv, pub
        except Exception as e:  # noqa: BLE001 - 容错 corrupt key
            logger.warning("[v35] %s key corrupt, regenerating: %s", actor, e)
    # 生成新 key pair
    priv = Ed25519PrivateKey.generate()
    pub = priv.public_key()
    priv_pem = priv.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),  # per v35 已知缺口 #5 v0.2 加密
    )
    pub_pem = pub.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo,
    )
    priv_path.write_bytes(priv_pem)
    pub_path.write_bytes(pub_pem)
    logger.info("[v35] generated ed25519 keypair for %s at %s", actor, priv_path)
    return priv, pub


def build_signed_payload(comment: dict) -> str:
    """构造 canonical payload string (per 守门 v35 §1.1).

    Fields: id|ts|author|body|blocks|tags
    """
    tags = comment.get("tags") or []
    return "|".join([
        str(comment.get("id", "")),
        str(comment.get("ts", "")),
        str(comment.get("author", "")),
        str(comment.get("body", "")),
        str(comment.get("blocks", False)),
        ",".join(str(t) for t in tags),
    ])


def sign_comment(comment: dict, actor: str = "Mavis", keys_dir: Path = DEFAULT_KEYS_DIR) -> dict:
    """签 1 条 comment dict, 返回带 3 字段的 copy (per 守门 v35 §1.1).

    Args:
        comment: 留言 dict (id/ts/author/body/blocks/tags 至少)
        actor: 签名 actor (per 守门 v35 §1.2 默认 Mavis per 守门 #14 v3 永久代签)

    Returns:
        新 dict, 加 _signature + _signed_payload + _signing_actor 3 字段
    """
    priv, _ = ensure_keypair(actor, keys_dir)
    payload = build_signed_payload(comment)
    sig = priv.sign(payload.encode("utf-8"))
    out = dict(comment)
    out["_signature"] = base64.b64encode(sig).decode("ascii")
    out["_signed_payload"] = payload
    out["_signing_actor"] = actor
    return out


def verify_comment(comment: dict, keys_dir: Path = DEFAULT_KEYS_DIR) -> bool:
    """验 1 条 comment 签名 (per 守门 v35 §1.2).

    Returns:
        True = 签名有效 (pub.verify 0 err)
        False = 签名无效 / 缺失 / payload 跟 comment 字段不一致 / key 缺失
    """
    sig_b64 = comment.get("_signature")
    payload = comment.get("_signed_payload")
    actor = comment.get("_signing_actor") or comment.get("author")
    if not sig_b64 or not payload or not actor:
        return False
    # 检查 payload 跟当前 comment 字段一致 (per v35 §1.2 "comment 字段不一致 → False")
    if payload != build_signed_payload(comment):
        return False
    try:
        sig = base64.b64decode(sig_b64)
        _, pub = ensure_keypair(actor, keys_dir)
        pub.verify(sig, payload.encode("utf-8"))
        return True
    except Exception as e:  # noqa: BLE001 - 容错
        logger.debug("[v35] signature verify failed for %s: %s", actor, e)
        return False


def is_authoritative_actor(actor: Optional[str]) -> bool:
    """是否权威 actor (per 守门 v33 v0.2 + 守门 v35 §1.3)."""
    if not actor:
        return False
    return actor in AUTHORITATIVE_ACTORS
