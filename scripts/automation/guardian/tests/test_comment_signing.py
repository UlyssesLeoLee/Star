"""test_comment_signing.py — 任务卡留言 ed25519 签名/验证 (per 守门 v35 + 守门 #5 派生)

覆盖:
  - sign + verify roundtrip (基本功能)
  - wrong actor key 验证失败 (negative test)
  - tampered body 验证失败 (完整性)
  - missing signature 验证失败 (legacy 兼容)
  - authoritative actor 识别 (per 守门 v33 v0.2 联动)
  - payload 跟 comment 字段不一致验证失败
  - Ulysses + Mavis + architect 3 actor 各自签名验证
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

# Add scripts/automation/ to sys.path (per 守门 #1 跟 test_audit_logger.py 一致)
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from guardian.comment_signing import (  # noqa: E402
    AUTHORITATIVE_ACTORS,
    build_signed_payload,
    ensure_keypair,
    is_authoritative_actor,
    sign_comment,
    verify_comment,
)


class TestSignVerifyRoundtrip(unittest.TestCase):
    """TC 1: sign + verify roundtrip (per 守门 v35 §1.1 + §1.2)."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp()
        self.keys_dir = Path(self.tmpdir) / "keys"

    def test_sign_and_verify_roundtrip(self):
        comment = {
            "id": "comment_001",
            "ts": "2026-09-10T22:00:00.000+09:00",
            "author": "Mavis",
            "body": "请加 X 字段",
            "blocks": False,
            "tags": ["requirement"],
        }
        signed = sign_comment(comment, actor="Mavis", keys_dir=self.keys_dir)
        # 3 字段必含
        self.assertIn("_signature", signed)
        self.assertIn("_signed_payload", signed)
        self.assertIn("_signing_actor", signed)
        self.assertEqual(signed["_signing_actor"], "Mavis")
        # 验签
        self.assertTrue(verify_comment(signed, keys_dir=self.keys_dir))


class TestWrongActorKey(unittest.TestCase):
    """TC 2: 错 actor key 验证失败 (negative test, per v35 已知缺口 #5)."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp()
        self.keys_dir = Path(self.tmpdir) / "keys"

    def test_ulysses_signature_fails_under_mavis_key(self):
        comment = {
            "id": "comment_002",
            "ts": "2026-09-10T22:01:00.000+09:00",
            "author": "Ulysses",
            "body": "已批准 deploy",
            "blocks": False,
            "tags": [],
        }
        # 用 Mavis 私钥签, 标 _signing_actor=Mavis (模拟伪造)
        forged = sign_comment(comment, actor="Mavis", keys_dir=self.keys_dir)
        # 改成 author=Ulysses 模拟冒充
        forged["author"] = "Ulysses"
        forged["_signing_actor"] = "Ulysses"
        # 此时 _signed_payload 跟新字段不一致, 验签必失败
        self.assertFalse(verify_comment(forged, keys_dir=self.keys_dir))


class TestTamperedBody(unittest.TestCase):
    """TC 3: body 被篡改验签失败 (per 守门 #5 完整性派生)."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp()
        self.keys_dir = Path(self.tmpdir) / "keys"

    def test_tampered_body_fails(self):
        comment = {
            "id": "comment_003",
            "ts": "2026-09-10T22:02:00.000+09:00",
            "author": "Mavis",
            "body": "原始内容",
            "blocks": False,
            "tags": [],
        }
        signed = sign_comment(comment, actor="Mavis", keys_dir=self.keys_dir)
        # 篡改 body
        signed["body"] = "被改的内容"
        # 验签必失败 (payload 跟字段不一致)
        self.assertFalse(verify_comment(signed, keys_dir=self.keys_dir))


class TestMissingSignature(unittest.TestCase):
    """TC 4: 缺签名验证失败 (legacy 兼容, per v35 已知缺口 #1)."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp()
        self.keys_dir = Path(self.tmpdir) / "keys"

    def test_legacy_comment_without_signature_fails(self):
        legacy = {
            "id": "comment_004",
            "ts": "2026-09-10T22:03:00.000+09:00",
            "author": "Mavis",
            "body": "legacy 留言无签名",
            "blocks": True,
            "tags": ["requirement"],
        }
        # 缺 _signature + _signed_payload → False
        self.assertFalse(verify_comment(legacy, keys_dir=self.keys_dir))

    def test_partial_signature_fails(self):
        partial = {
            "id": "comment_004b",
            "ts": "2026-09-10T22:03:00.000+09:00",
            "author": "Mavis",
            "body": "只有 _signature 没 payload",
            "blocks": True,
            "tags": [],
            "_signature": "aGVsbG8=",
        }
        self.assertFalse(verify_comment(partial, keys_dir=self.keys_dir))


class TestAuthoritativeActor(unittest.TestCase):
    """TC 5: 权威 actor 识别 (per 守门 v33 v0.2 + 守门 v35 §1.3 联动)."""

    def test_ulysses_is_authoritative(self):
        self.assertTrue(is_authoritative_actor("Ulysses"))

    def test_mavis_is_authoritative(self):
        self.assertTrue(is_authoritative_actor("Mavis"))

    def test_architect_is_authoritative(self):
        self.assertTrue(is_authoritative_actor("architect"))

    def test_sub_agent_is_not_authoritative(self):
        self.assertFalse(is_authoritative_actor("sub-agent"))

    def test_none_is_not_authoritative(self):
        self.assertFalse(is_authoritative_actor(None))

    def test_authoritative_actors_set_complete(self):
        self.assertEqual(AUTHORITATIVE_ACTORS, ("Ulysses", "Mavis", "architect"))


class TestMultipleActors(unittest.TestCase):
    """TC 6: Ulysses + Mavis + architect 3 actor 各自签名验证 (per v35 §1.4)."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp()
        self.keys_dir = Path(self.tmpdir) / "keys"

    def test_three_actors_sign_verify(self):
        for actor in ("Ulysses", "Mavis", "architect"):
            comment = {
                "id": f"comment_{actor}",
                "ts": "2026-09-10T22:04:00.000+09:00",
                "author": actor,
                "body": f"{actor} 留言",
                "blocks": False,
                "tags": [],
            }
            signed = sign_comment(comment, actor=actor, keys_dir=self.keys_dir)
            self.assertTrue(verify_comment(signed, keys_dir=self.keys_dir), f"{actor} roundtrip failed")

    def test_keypair_persistence(self):
        """ensure_keypair idempotent (per 守门 v35 §1.4)."""
        priv1, pub1 = ensure_keypair("Mavis", keys_dir=self.keys_dir)
        priv2, pub2 = ensure_keypair("Mavis", keys_dir=self.keys_dir)
        # 同一 actor 二次 ensure 应返同一 key (PEM 序列化等价)
        from cryptography.hazmat.primitives import serialization
        self.assertEqual(
            priv1.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption(),
            ),
            priv2.private_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PrivateFormat.PKCS8,
                encryption_algorithm=serialization.NoEncryption(),
            ),
        )


class TestPayloadConsistency(unittest.TestCase):
    """TC 7: _signed_payload 跟 comment 字段不一致验证失败 (per 守门 v35 §1.2)."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp()
        self.keys_dir = Path(self.tmpdir) / "keys"

    def test_payload_consistency_check(self):
        comment = {
            "id": "comment_007",
            "ts": "2026-09-10T22:05:00.000+09:00",
            "author": "Mavis",
            "body": "原始",
            "blocks": False,
            "tags": ["a", "b"],
        }
        signed = sign_comment(comment, actor="Mavis", keys_dir=self.keys_dir)
        # _signed_payload 跟 build_signed_payload 一致 → 通过
        self.assertEqual(signed["_signed_payload"], build_signed_payload(signed))
        # 篡改 tags
        tampered = dict(signed)
        tampered["tags"] = ["c", "d"]
        self.assertFalse(verify_comment(tampered, keys_dir=self.keys_dir))


if __name__ == "__main__":
    unittest.main()
