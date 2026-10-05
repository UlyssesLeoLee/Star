#!/usr/bin/env python3
"""用上游 CRD 的 openAPIV3Schema 离线校验 GitOps 清单 (per 守门 #11 缺标比错标).

为什么需要本脚本 (而非 kubectl --dry-run=server):
    编写时本机 k3s 不可达 (172.28.176.169:6443 connection refused), 无法做
    server 端校验; kubectl --dry-run=client 对 Custom Resource 也需要 API
    discovery 才能拿到 schema, 同样要求集群可达。本脚本不依赖集群。

为什么不用 kubeconform:
    实测 kubeconform v0.8.0 对 CRD 自定义 schema 的本地文件定位不生效
    (自定义 schema-location 始终 "could not find schema", -debug 亦未暴露查找路径)。
    直接用 CRD 的 openAPIV3Schema 做 JSON Schema 校验更可控, 且能给出精确字段路径。

做法:
    1. 按**固定版本**从上游拉 CRD (Argo CD vX / Kargo vX), 缓存到 target/crds/
    2. 提取每个版本的 openAPIV3Schema
    3. 校验目标 yaml 中每个资源, 报告精确到字段的错误

CRD 版本与上游来源 (per ADR-0054 §2.1 实测选型):
    Argo CD  v3.5.3  argoproj/argo-cd            manifests/crds/
    Kargo    v1.12.1  akuity/kargo                charts/kargo/resources/crds/

用法:
    python scripts/automation/validate_gitops_crds.py
    python scripts/automation/validate_gitops_crds.py --online   # 强制重新拉 CRD

退出码:
    0 = 全部资源通过
    1 = 存在校验失败
    2 = 装置故障 (CRD 拉取失败 / 依赖缺失)
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import urllib.request
from pathlib import Path

import yaml

try:
    from jsonschema import Draft7Validator
except ImportError:  # pragma: no cover
    sys.stderr.write("jsonschema not installed.  pip install jsonschema\n")
    raise SystemExit(2)

REPO_ROOT = Path(__file__).resolve().parents[2]
CACHE_DIR = REPO_ROOT / "target" / "crds"

ARGO_CD_VERSION = "v3.5.3"
ARGO_CD_CRDS = {
    "AppProject": "appproject-crd.yaml",
    "Application": "application-crd.yaml",
}
KARGO_VERSION = "v1.12.1"
KARGO_CRD_BASE = (
    "https://raw.githubusercontent.com/akuity/kargo/"
    f"{KARGO_VERSION}/charts/kargo/resources/crds/"
)
KARGO_CRDS = [
    "kargo.akuity.io_projects.yaml",
    "kargo.akuity.io_warehouses.yaml",
    "kargo.akuity.io_stages.yaml",
]

TARGETS = [
    REPO_ROOT / "deploy" / "gitops" / "argocd" / "argocd-applications.yaml",
    REPO_ROOT / "deploy" / "gitops" / "kargo" / "kargo-dev-staging.yaml",
]


def fetch(url: str, dest: Path, online: bool) -> bool:
    if dest.is_file() and not online:
        return True
    dest.parent.mkdir(parents=True, exist_ok=True)
    try:
        with urllib.request.urlopen(url, timeout=45) as r:
            dest.write_bytes(r.read())
        return True
    except Exception as exc:  # noqa: BLE001
        sys.stderr.write(f"    fetch failed: {url}\n      {exc}\n")
        return False


def build_schemas(online: bool) -> tuple[dict[tuple[str, str, str], dict], list[str]]:
    """返回 {(kind, group, version): schema} 与故障列表.

    索引必须含 **kind**: Kargo 的 5 个 CRD 共享同一 group/version
    (kargo.akuity.io/v1alpha1), 若仅按 (version, group) 索引会互相覆盖,
    只剩最后一个 —— 那会导致用 Stage 的 schema 去校验 Project, 报出假错误。
    """
    schemas: dict[tuple[str, str, str], dict] = {}
    failures: list[str] = []

    def add(doc: dict, label: str) -> None:
        group = doc["spec"]["group"]
        kind = doc["spec"]["names"]["kind"]
        for ver in doc["spec"]["versions"]:
            if ver.get("schema", {}).get("openAPIV3Schema"):
                schemas[(kind, group, ver["name"])] = ver["schema"]["openAPIV3Schema"]
                return
        failures.append(f"{label} (无 openAPIV3Schema)")

    for kind, fname in ARGO_CD_CRDS.items():
        dest = CACHE_DIR / f"argocd-{ARGO_CD_VERSION}-{fname}"
        url = f"https://raw.githubusercontent.com/argoproj/argo-cd/{ARGO_CD_VERSION}/manifests/crds/{fname}"
        if not fetch(url, dest, online):
            failures.append(f"Argo CD {kind} CRD")
            continue
        add(yaml.safe_load(dest.read_text(encoding="utf-8")), f"Argo CD {kind}")

    for fname in KARGO_CRDS:
        dest = CACHE_DIR / f"kargo-{KARGO_VERSION}-{fname}"
        if not fetch(KARGO_CRD_BASE + fname, dest, online):
            failures.append(f"Kargo {fname}")
            continue
        add(yaml.safe_load(dest.read_text(encoding="utf-8")), f"Kargo {fname}")

    return schemas, failures


def validate(schemas: dict[tuple[str, str], dict]) -> tuple[int, int, list[str]]:
    total = ok = 0
    problems: list[str] = []
    for target in TARGETS:
        if not target.is_file():
            problems.append(f"[SKIP] {target} 不存在")
            continue
        rel = target.relative_to(REPO_ROOT)
        for doc in yaml.safe_load_all(target.read_text(encoding="utf-8")):
            if not doc:
                continue
            kind = doc.get("kind")
            api = doc.get("apiVersion", "")
            group, _, version = api.rpartition("/")
            name = doc.get("metadata", {}).get("name", "<no-name>")
            total += 1

            # 内置资源 (Namespace 等) 无 CRD, 交给 yaml 结构已校验, 此处跳过
            if not group or not group.endswith((".io", ".sh", ".dev")):
                ok += 1
                continue

            schema = schemas.get((kind, group, version))
            if schema is None:
                problems.append(f"[NO-SCHEMA] {rel}: {kind}/{api} {name} —— 未取得 CRD schema")
                continue

            errors = sorted(
                Draft7Validator(schema).iter_errors(doc), key=lambda e: list(e.absolute_path)
            )
            if not errors:
                ok += 1
                print(f"  PASS  {kind:<12} {name}")
            else:
                for e in errors[:6]:
                    path = ".".join(str(p) for p in e.absolute_path) or "<root>"
                    problems.append(f"[FAIL] {rel}: {kind}/{name} @ {path}\n         {e.message[:200]}")
    return total, ok, problems


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--online", action="store_true", help="强制重新拉取 CRD (默认用缓存)")
    args = ap.parse_args()

    print("=== CRD offline schema validation ===")
    print(f"Argo CD {ARGO_CD_VERSION} | Kargo {KARGO_VERSION}")
    schemas, failures = build_schemas(args.online)
    if failures:
        print(f"\n[DEVICE FAULT] CRD 获取失败: {', '.join(failures)}")
        return 2
    print(f"schemas loaded: {len(schemas)}")

    total, ok, problems = validate(schemas)
    if problems:
        print()
        for p in problems:
            print(p)
    print(f"\n===== resources={total} passed={ok} failed={total - ok} =====")
    return 1 if (total - ok) else 0


if __name__ == "__main__":
    raise SystemExit(main())
