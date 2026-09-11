"""
providers/ — 5 provider 适配器 (per DD-MULTICA-RUNTIME-001 §2.1 物理 layout)

@deprecated v0.legacy — Replaced by Rust `star-registry::providers` module in R3 阶段
                (per ADR-0027 R3). 阶段 1 实装: commit `fadff8f`.
                Per 守门 #11 缺标比错标: 不删, 永久留档.

阶段 1: 5 named provider 占位
阶段 2: 扩到 25 named + BuiltinRuntimes 派生 (废弃, 跳过, 走 R3 Rust)

每个 provider 是一个简单的 dataclass, 包含:
- name: provider 标识
- default_cmd: 默认命令名
- env_path: MULTICA_<NAME>_PATH env 名
- env_model: MULTICA_<NAME>_MODEL env 名 (None for mcode/qwenpaw/zeroclaw per FR-4)
- min_version: 最低 semver (在 MinVersionGate.MIN_VERSIONS 中)
- 额外字段: codex Desktop app bundle 路径 (per Multica agents_probe.go:139-151)
"""
from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class ProviderSpec:
    """单 provider 规格 (per DD-MULTICA-RUNTIME-001 §3.2 Probe provider spec)"""
    name: str
    default_cmd: str
    env_path: str
    env_model: Optional[str]  # None for mcode/qwenpaw/zeroclaw per FR-4
    min_version: Optional[str]
    extra: Optional[dict] = None  # 特殊处理 (e.g. codex Desktop app bundle paths)


# 阶段 1: 5 named provider (per SRS FR-1 + Multica)
STAGE1_PROVIDERS = {
    "claude": ProviderSpec(
        name="claude", default_cmd="claude",
        env_path="MULTICA_CLAUDE_PATH", env_model="MULTICA_CLAUDE_MODEL",
        min_version="2.0.0",
    ),
    "codex": ProviderSpec(
        name="codex", default_cmd="codex",
        env_path="MULTICA_CODEX_PATH", env_model="MULTICA_CODEX_MODEL",
        min_version="0.100.0",
        extra={"desktop_app_bundle_paths": []},  # 阶段 2 实装, 暂空
    ),
    "mcode": ProviderSpec(
        name="mcode", default_cmd="mcode",
        env_path="MULTICA_MCODE_PATH", env_model=None,  # per FR-4 不读 model env
        min_version="0.1.2",
    ),
    "copilot": ProviderSpec(
        name="copilot", default_cmd="copilot",
        env_path="MULTICA_COPILOT_PATH", env_model="MULTICA_COPILOT_MODEL",
        min_version="1.0.0",
    ),
    "grok": ProviderSpec(
        name="grok", default_cmd="grok",
        env_path="MULTICA_GROK_PATH", env_model="MULTICA_GROK_MODEL",
        min_version="0.2.89",
    ),
}


__all__ = ["ProviderSpec", "STAGE1_PROVIDERS"]
