# W7 本地 K3s Star context 独立审阅 — 2026-11-02

## 决定

**W7 OPEN / BLOCKED；v1.0.6 NO-GO。** 部署脚本已接入 Cargo 闭包 context 生成器，原先直接复制整个 `crates/` 的绕行已消除；但生成器在 Windows 文件系统上会沿包目录内 junction 复制仓库外文件。隔离实验调用了实际 `_copy_local_package` 并确认外部 payload 进入 context。应先在单独的 generator + regression lane 修复并验证；本 docs-only lane 不修改代码。

## 范围与证据边界

本审阅只判断本地 K3s 入口是否调用过滤生成器、生成器的源码级输入边界，以及已有 focused tests 能证明什么。未做 Docker 构建、Multipass 操作或 K3s 部署。源码级 allowlist 接线证据不等于 archive、最终镜像、digest、SBOM/provenance、consumer、rollout 或 rollback 证明。

父协调器提供的 CodeGraph 记录显示索引含 1,907 个文件；本 lane 的 coverage 检查工具不可用，且 K3s generator 未包含在父协调器的两次图查询范围。本审阅对下列有限文件直接读取源码和测试；Dockerfile、shell 等使用源码回退。此结论不声称完成全仓或 graph coverage 审计。

## 审阅矩阵

| 检查项 | 结果 | 证据 |
|---|---|---|
| 构建入口 | 已接线 | `deploy/k3s-local/build-and-deploy.sh:31-35` 将显式 guest 与 image tag 传给 `scripts/automation/star_k3s_context.py`；不再在 shell 中复制完整 `crates/`。 |
| Dockerfile 输入 | 静态 allowlist 结构 | `deploy/k3s-local/Dockerfile:6-7` 是由生成器替换的 crate-copy marker；`star_k3s_context.py:628-697` 只为解析出的 local package 生成逐 crate COPY，并以显式 context 文件 inventory 生成摘要。模板本身明确不应直接作为 context。 |
| 本地 guest 部署 | 静态路径存在 | 生成器在显式已有 guest 内传输经过摘要核验的 archive，在 guest 运行 Docker build、导入 K3s containerd、核验 CRI 镜像可见性、apply manifest 并等待 rollout；本轮未实际执行。 |
| 普通 symlink/越界输入 | 有限防护 | `_repo_relative` 检查 resolved path 是否仍在 repository（`star_k3s_context.py:176-196`）；复制时拒绝 `Path.is_symlink()`。测试有 root LICENSE symlink 与 manifest path escape 覆盖。 |
| Windows junction/reparse point | **P1 阻塞** | `_copy_local_package` 在 `star_k3s_context.py:590-618` 以 `os.walk(..., followlinks=False)` 遍历 crate，并仅检查 `child.is_symlink()` / `source.is_symlink()`。Windows junction 的 `Path.is_symlink()` 为 False；实际隔离实验复制了 junction 目标中的仓库外文件。 |
| focused regression tests | 通过，但缺关键案例 | 模块运行 18 项，17 passed、1 skipped；skip 是 Windows symlink 创建权限不足。已覆盖 closure、普通 symlink、path escape、篡改、部署脚本接线和 guest readiness；未覆盖包目录内 junction。 |
| build-context archive / final image | 未验证 | 没有执行本地 archive-to-guest/build 流程或检查最终 image layers。 |
| digest / SBOM / provenance / consumer / rollout rollback | 未验证 | 没有发布证明、独立 consumer 校验、rollout/rollback 结果。 |

## 可复现的隔离 junction 实验

以下 PowerShell 输入通过 stdin 送给 Python。`TemporaryDirectory` 只在临时目录中创建仓库夹具和 junction，退出时清理该临时目录；`_copy_local_package` 是生产生成器中的实际函数。

```powershell
@'
import subprocess, tempfile
from pathlib import Path
from scripts.automation.star_k3s_context import _copy_local_package

with tempfile.TemporaryDirectory(prefix="star-w7-junction-review-") as tmp:
    root = Path(tmp) / "repo"
    package = root / "crates" / "domain-safe"
    package.mkdir(parents=True)
    (package / "Cargo.toml").write_text(
        '[package]\nname="domain-safe"\nversion="0.1.0"\n', encoding="utf-8"
    )
    outside = Path(tmp) / "outside"
    outside.mkdir()
    (outside / "payload.txt").write_text("outside repository payload", encoding="utf-8")
    junction = package / "assets"
    result = subprocess.run(
        ["cmd", "/c", "mklink", "/J", str(junction), str(outside)],
        capture_output=True, text=True,
    )
    print(f"junction_create_exit={result.returncode}")
    print(f"is_symlink={junction.is_symlink()}")
    print(f"is_junction={junction.is_junction()}")
    output = Path(tmp) / "context"
    _copy_local_package(root, {"manifest_path": str(package / "Cargo.toml")}, output)
    print(f"outside_payload_copied={(output / 'crates/domain-safe/assets/payload.txt').is_file()}")
'@ | rtk python -
```

实测结果：`junction_create_exit=0`、`is_symlink=False`、`is_junction=True`、`outside_payload_copied=True`。这证明包内 Windows junction 可绕过当前 symlink 检查，并把仓库外 payload 带入生成 context。该实验不证明这些字节必然进入最终镜像；但它足以否定当前 context allowlist 的完整性，并违反发布输入必须仅含获准 Star 内容的放行条件。

## 缺陷与建议

**P1 — generator 对 Windows junction 不 fail closed。** 修复与回归应在独立实现 lane 完成，建议：

1. crate 树递归期间拒绝所有 Windows junction/reparse point（并覆盖可用 Python 版本；不能依赖仅识别符号链接的 API）；
2. 对每个递归子目录和文件验证实际解析目标仍位于对应包根与 repository 根内；遇到未知 reparse 类型、解析失败或路径身份变化时 fail closed；
3. 新增一个 Windows 真实目录 junction 测试，junction 指向 repo 外含哨兵文件的临时目录，断言生成器拒绝且未生成/传输 context；
4. 保持现有闭包、game/legacy 拒绝、symlink、digest/tampering 与 guest readiness 覆盖；重新运行 focused tests 和静态验证。

已修复的 shell `cp -r crates/` 绕行与本次 junction 缺口是两个不同问题：前者的 build entry 已接入 generator；后者说明 generator 本身仍可能接受越界输入。W3 Canvas context 过滤与容器发布的静态检查不能替代本地 K3s generator 的审查，也不能用来宣称此 K3s context 安全。

## 检查结果

| 检查 | 结果 |
|---|---|
| `rtk python -m unittest scripts.automation.test_star_k3s_context` | 18 tests；17 passed，1 skipped（Windows symlink 权限） |
| `rtk python -m py_compile scripts/automation/star_k3s_context.py scripts/automation/test_star_k3s_context.py` | PASS |
| `bash -n deploy/k3s-local/build-and-deploy.sh` | PASS |
| 隔离 tempfile + 实际 `_copy_local_package` junction 复现 | **FAIL CLOSED 未实现**；外部哨兵文件被复制 |
| `rtk git diff --check` | PASS |
| Cargo metadata / Docker / Multipass / K3s | 未运行 |
| archive/image/digest/SBOM/provenance/consumer/rollout/rollback | 未验证 |

## 决策与剩余工作

- 仅接受“部署入口改用生成 context”的静态接线结果；过滤器完整性不通过。
- 先开独立 generator + regression 实现 lane，解决 junction/reparse point 越界并复测。
- 然后在受控目标环境实际生成 context archive，确认 inventory 和 digest；再完成 image、SBOM/provenance、独立 consumer、Multipass/K3s rollout 与 rollback 验收。
- W7 保持 OPEN / BLOCKED，稳定版 v1.0.6 保持 NO-GO。
- 本审阅没有修改代码、WBS、D2 README、其他 worktree、`D:\Star` 用户文件、`main` 或远端；没有运行 Docker、Multipass 或 K3s。
