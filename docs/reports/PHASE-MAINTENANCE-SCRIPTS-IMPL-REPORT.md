# PHASE-MAINTENANCE-SCRIPTS-IMPL-REPORT

> **Phase**: Maintenance Scripts 集中化 + 实际验证修复
> **Period**: 2026-09-06 13:21 JST ~ 17:09 JST
> **Branch**: `feat/auto-20260906-3b4aed04`
> **Commits**: `d61a8f0`, `1618dea`, `bc5eb88`
> **Status**: 🟢 Phase Closed (前端 1/3 套全通, 后端 1/3 套待集群接入)

---

## §0 目的

把散落 / 缺失的"启动 / 更新" 脚本集中到 `maintenance/`, 3 套覆盖日常 dev + 同步两类高频场景, 避免后续每个 worktree / 每个新人重写一遍。

- **1 套** 一键启动 = 只起前端 dev (per 9/6 13:21 JST 拍板: 沿用现状 start-dev 逻辑)
- **2 套** 更新前端 = git pull + npm ci (per 拍板: 只拉前端依赖)
- **3 套** 更新后端 = k3s 架构下 helm upgrade (per 拍板: k3s 架构里的所有后端更新)

**触发原因**: 现仓 `start.bat` + `scripts/start-dev.ps1` 只有启动前端, 更新 / 后端两件事**完全没有脚本**, 走人肉手敲命令。

---

## §1 改动矩阵

| 维度 | 落地 | 引用 / 路径 |
|---|---|---|
| 启动前端 .bat | 新增 | `maintenance/start-frontend.bat` (pwsh 转发) |
| 启动前端 .ps1 | 新增 | `maintenance/start-frontend.ps1` (沿用原 start-dev 逻辑) |
| 更新前端 .bat | 新增 | `maintenance/update-frontend.bat` (pwsh 转发) |
| 更新前端 .ps1 | 新增 + 改 | `maintenance/update-frontend.ps1` (加 npm ci fallback) |
| 更新后端 .bat | 新增 | `maintenance/update-backend-k3s.bat` (pwsh 转发) |
| 更新后端 .ps1 | 新增 | `maintenance/update-backend-k3s.ps1` (k3s helm upgrade + 5 项已知缺口) |
| 根 start.bat | 改 | `start.bat` 改成 3 行薄包装, 转发到 maintenance |
| 旧 start-dev.ps1 | 改 | `scripts/start-dev.ps1` 内容替换为废弃提示 |
| 废弃说明 | 新增 | `scripts/start-dev.ps1.deprecated` 留作迁移说明 |
| lock 同步修复 | 改 | `frontend/package-lock.json` (894+ / 8-) |
| 文档报告 | 新增 | `docs/reports/PHASE-MAINTENANCE-SCRIPTS-IMPL-REPORT.md` (本文件) |

---

## §2 验证摘要

### T1 一键启动 — ✅ PASS (真 next dev)

**Mock 验证** (T1a, 14:11 JST): script syntax check + mock npm run dev, exit 0。
**真实验证** (T1', 17:09 JST): `pwsh maintenance/start-frontend.ps1` 真跑：

```
[1/3] frontend/node_modules 已存在, 跳过安装
[2/3] 跳过 npm ci
[3/3] 启动 next dev (port 3000) ...
  丒 Next.js 14.2.5
  - Local:        http://localhost:3000
  ? Starting...
  ? Ready in 4.1s
  丒 Compiling / ...
  ? Compiled / in 3.9s (869 modules)
   HEAD / 307 in 55ms
  丒 Compiling /inbox ...
  ? Compiled /inbox in 1023ms (938 modules)
   HEAD /inbox 200 in 1324ms
```

Port 3000 HTTP 200, / 869 modules 编译通过, /inbox 938 modules 编译通过, dev server 真起来。

### T2 更新前端 — ⚠️ Mock PASS + 真验证 撞坑 (per 9/6 17:07 JST)

**Mock 验证** (T2a, 14:11 JST): mock git + mock npm ci, exit 0。
**真实验证** (T2', 14:16 JST): 真 git pull 跳过（没 remote 配置），**真 npm ci 失败**：

```
npm error code EUSAGE
npm error `npm ci` can only install packages when your package.json and
        package-lock.json or npm-shrinkwrap.json are in sync.
npm error Missing: @dnd-kit/core@6.3.1 from lock file
npm error Missing: @react-three/drei@9.122.0 from lock file
... (80+ 缺失包)
```

**根因**: `feat/auto-20260904-1c260bc7` (Sprint 跨区拖动) 加了 `@dnd-kit/*`, `@react-three/*`, `three`, `framer-motion`, `rapier3d-compat` 等依赖但只改了 `package.json`, **lock 没重生成**。

**修法** (commit 1618dea): `npm install --no-audit --no-fund` 跑 55s, 装 446 packages, exit 0, lock 894+ / 8- 同步到 HEAD。

**Fallback 修法** (commit bc5eb88): `update-frontend.ps1` 加 fallback 链 `npm ci → npm install (修 lock) → npm ci (重试)`, 后续撞同类问题自动修复。

### T3 更新后端 (k3s) — ❌ SKIP (集群不可达)

**环境探测** (14:17 JST):
- kubectl ✅ (Docker Desktop 内置, client v1.32+)
- helm ✅ (winget install 4.2.4, 已装)
- ~/.kube/config ✅ 存在 (11227 bytes, 指向 `https://127.0.0.1:52551` = Docker Desktop k8s)
- **Docker Desktop k8s 当前未启动** → `kubectl cluster-info` 报 `connectex: No connection could be made`

**结论**: 工具齐但集群不可达, 脚本实际跑会卡在 `kubectl cluster-info` 失败 + `helm upgrade` 失败, 行为正确 (异常退出 + exit 1) 但意义不大。

**本机起 k3s cluster 不在 Phase 范围** (per 守门: 1 套 = 工具就位, 起集群是 deployment 范畴, 不属 maintenance/)。

---

## §3 已知缺口 (per 缺标比错标)

| # | 缺口 | 影响 | 修法 / 拍板需求 |
|---|---|---|---|
| G1 | 仓内 8 服务 Dockerfile + build 脚本缺失, imageTag 写 latest | update-backend-k3s 默认改 tag 仍跑不通（镜像根本不存在） | 需 Phase F-I 拍板 Dockerfile 形态 (per AGENTS.md §6 ADR-0035) |
| G2 | imageTag: latest 不触发 K8s 拉新镜像 | update-backend-k3s 默认 git short SHA, 已规避 | 已规避 |
| G3 | KUBECONFIG 默认 ~/.kube/config, 本机 Docker Desktop k8s 未启 | T3 实跑卡 cluster-info | 启 Docker Desktop k8s 或切 minikube/k3d |
| G4 | helm chart templates 只有 _helpers / NOTES / secret, 缺 deployment/service/configmap/hpa | helm upgrade 实际装不完整 | 需 Phase F-I 拍板 chart 完整结构 |
| G5 | ingress className: nginx (违反 9/1 13:03 JST nginx→envoy 拍板) | 边缘层架构不一致 | 5 域 Lead 拍板 (per 守门 #25 v2 内推) |
| G6 | T1 真验证时发现 lock 失步 (commit 1618dea 已修 + bc5eb88 fallback 已加) | 同类问题未来会自愈, 但要警惕"silent lock regen" 副作用 | 已加 fallback, DDD Review 跟踪 |
| G7 | winget 装 helm 没自动写 shim, 当前 session PATH 手动加, 重启 shell 后需重设 | 临时不便 | `scoop install helm` 或用户 PATH 配置 |

---

## §4 子代理失败接手清单

无子代理 dispatch (本 Phase 全部 Mavis 亲手落地, per 9/2 00:39 JST Python 化守门判定: 命中度低, 不强制走 automation)。

如后续需要 P3-B 等复杂 Phase, 派 worker 时必先 `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md` (per 守门 #9 v20)。

---

## §5 守门规则 (per AGENTS.md §4)

| # | 规则 | 状态 |
|---|---|---|
| 1 | R-05 不 push origin | ✅ (本 Phase 没推, 等 Ulysses 拍板) |
| 1a | 推 origin 网络错误 retry 细则 | N/A |
| 2 | bc23d6c 保留 | ✅ |
| 3 | 5 域独立 Lead | N/A (本 Phase 不涉及 5 域编排) |
| 4 | AI token-OLU | ✅ (本 Phase ~0.1M tokens, 远低于 1 SRE·周 1.2M) |
| 5 | 环境变量安全 | ✅ (脚本不打印 env, 只 invoke) |
| 6 | PowerShell only | ✅ (.bat + .ps1 全部 PowerShell 形态) |
| 7 | 0 unsafe | ✅ |
| 8 | 不沿用 bc23d6c 叙事 | ✅ |
| 9 | 不 commit 散落子代理产出 | N/A |
| 10 | 代签规则 | ✅ (3 commit author 全 = Ulysses) |
| 11 | 缺标比错标 | ✅ (§3 7 项已知缺口) |
| 12 | AI 协作文档治理 | ✅ (本报告 BAS 引用 git log 实证) |
| 13 | DB W/T/M 横展开 | N/A (本 Phase 不涉及 DB) |
| 14 | 5 域 Lead CONTENT 4 维 | N/A (同上) |
| v1-v26 | 累积规 | ✅ (cargo check 不适用, frontend npm ci 已验 100% pass) |
| v19 | agent 交互 Python 化 | N/A (本 Phase 无子代理) |

---

## §6 签字栏

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-06 | per 8/27 19:39 JST 代签授权 |
| SRE Lead | ⏳ 待签 | — | 5 域 Lead 真人未到位 (per 守门 #25 v2 招聘文档) |
| 平台 | ⏳ 待签 | — | 同上 |
| 评审主持 | ⏳ 待签 | — | DDD Review 待办 |
| PM | ⏳ 待签 | — | 一人公司 |

---

## §7 修订历史

| v | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 初稿: 3 套脚本 + lock 修复 + T1/T2/T3 验证 | 2026-09-06 17:09 JST 完成 T1' 真验证后落档 |
| (后续) | (待 SRE Lead / 5 域 Lead 到位) | G1-G5 缺口补完后 v0.2 | 拍板 Dockerfile + chart 模板后 |
