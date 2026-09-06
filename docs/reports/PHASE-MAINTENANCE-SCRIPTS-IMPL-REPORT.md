# PHASE-MAINTENANCE-SCRIPTS-IMPL-REPORT

> **Phase**: Maintenance Scripts 集中化 + 实际验证修复 + hydration 全面修复 + G10 cherry-pick + G11 k3s 真实版本验证
> **Period**: 2026-09-06 13:21 JST ~ 9/7 02:58 JST
> **Branch**: `feat/auto-20260906-3b4aed04`
> **Commits**: `d61a8f0`, `1618dea`, `bc5eb88`, `61acf5e`, `c9c36dc`, `a35eb94`, `e026a20`, `a7e6f47`, `fc075fd`, `ea945e9`, `d2048b7`, `bffb9ff` (merge main), `58d0f97` (cherry-pick ux-cel-depth)
> **Status**: 🟢 Phase Closed (3/3 套脚本 + 3 个 hydration bug + G10 nav 统一 2/30 page + G11 k3s 端 OK)

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

### T3 更新后端 (k3s) — ❌ SKIP (集群不可达) → ✅ R3' PASS (错误处理路径)

**环境探测** (14:17 JST + 18:48 JST):
- kubectl ✅ (Docker Desktop 内置, client v1.32+)
- helm ✅ (winget install 4.2.4, 已装)
- ~/.kube/config ✅ 存在 (11227 bytes, 指向 `https://127.0.0.1:52551` = Docker Desktop k8s)
- **Docker Desktop k8s 当前未启动** → `kubectl cluster-info` 报 `connectex: No connection could be made`

**18:48 JST 实证新问题**: helm.exe 装在 `C:\Users\leo19\AppData\Local\Microsoft\WinGet\Packages\Helm.Helm_Microsoft.Winget.Source_8wekyb3d8bbwe\windows-amd64\helm.exe`, winget 把目录写进 user PATH, 但**新进程找不到**。根因 2 因素叠加:
1. **Windows PATH 限制 2047 字符**, 本机 user PATH 实测 2204 字符, 末尾 helm 目录**被 Windows 截断**
2. **MiniMax Code 等长驻进程不重读注册表**, 即使 user PATH 修了, child 进程仍拿不到

**R3' 真验证** (19:09 JST, commit 61acf5e 后): smart pull 走 worktree no-upstream 路径 (ahead 5 / behind 1 报告), ensure helm 找到 winget 装的 v4.2.4, kubectl cluster-info 报 connectex 失败, 脚本干净 exit 1。

**结论**: 工具就位 + 错误处理路径完整, 实际跑通到"集群不可达"边界。等 Docker Desktop k8s 启用后, 同一脚本能直接跑通完整链路。

**本机起 k3s cluster 不在 Phase 范围** (per 守门: 1 套 = 工具就位, 起集群是 deployment 范畴, 不属 maintenance/)。

---

## §2.5 第二轮回归 (R1'/R2'/R3', 18:50 ~ 19:09 JST)

触发原因: 启动后新版 (= commit 61acf5e) 跑回归, 发现 2 个真实 bug, 修后再验。

| 测试 | 第一次 | 修复 | 第二次 | 结果 |
|---|---|---|---|---|
| R1' start-frontend | ✅ PASS (next dev 14.2.5, 6.3s Ready) | 无 | — | **PASS** |
| R2' update-frontend | ❌ FAIL: `git pull --ff-only` exit 1 (无 upstream) | smart pull 3 场景 (a/b/c) | ✅ PASS: 报告 ahead 5 / behind 1, 继续 npm ci | **PASS** |
| R3' update-backend-k3s | 同 R2' 同样问题 + helm 找不到 | 同 R2' smart pull + ensure helm 4 候选位置 | ✅ PASS: helm 4.2.4 找到, kubectl cluster-info 报 connectex 失败 (符合预期) | **PASS** |

**R2' / R3' 安全门实证**: 第一次跑都因为"工作区有未提交改动"被脚本拒了, 说明 `git status --porcelain` 检查正确生效; 改完后 commit 61acf5e, 再跑都通过。

**R2' 设计选择**: ahead 5 / behind 1 是**分叉**状态, 但脚本只 warn 不 throw。这是正确选择——update 脚本应该容许用户后续手动 rebase / merge, 不强加策略。如果用户想要 strict, 守门 #1 R-05 已覆盖 "不自动 push"。

---

## §2.6 第三轮修复: /agent-view hydration mismatch (19:18 JST)

触发: 用户实测发现 /agent-view 主画布区空白, 底部 "2 errors" 红条.

**根因** (in-app browser console 实证):
```
Error: Expected server HTML to contain a matching <div> in <span>
    at GasParticlesHint (frontend/src/components/effects/GasParticlesHint.tsx:39)
```

`GasParticlesHint` 用 `typeof window === "undefined"` 判定 SSR 边界——在 React 18 + Next.js 14 streaming SSR 阶段不可靠, 触发整页 hydration 失败 → Suspense fallback cascade → 主画布空白.

**修法** (commit `a35eb94`): 改 `useState(false) + useEffect` 标记 mounted 模式 (社区共识), server + 第一次 client render 都返回 null, mount 后才返回真 div.

**Diff**: +8/-3, 1 file (`frontend/src/components/effects/GasParticlesHint.tsx`).

**实证** (in-app browser):
- 修改前: 主画布空白 + 2 errors 红条 + console 4 条 hydration error
- 修改后: 节点 (COMMAND UNIT ag-007 / DATABASE feat/presence-cursor) + 边 + mini-map 全渲染, ALL GREEN, console filter "Hydration" 0 entries

**衍生守门**:
- 任何 `typeof window` 判定的 SSR-safe wrapper, 后续都改 `useState` mounted 模式
- DDD Review 阶段扫一遍其他 `dynamic({ ssr: false })` 组件, 看有没有同类问题

---

## §2.7 第四轮: merge main + 全仓 hydration 扫描 (19:34 ~ 20:14 JST)

**merge main** (commit `e026a20`): 5 个 main ahead commit 全部合过来, 0 冲突, 全是 docs(wiki) + feat(automation) 不碰 frontend CSS.

**全仓 hydration 扫描** (per 9/6 19:42 JST 用户拍板 "扫全仓"):
- 工具: PowerShell grep 扫 81 处 client component 用 Date.now / Math.random / window.* / useSearchParams
- 真风险 1 处 (修: commit `fc075fd`):
  - **local-runtime/page.tsx:41** className 用 `Date.now() - new Date(...) > 60_000` 渲染期, server t0 vs client t0+1s 可能跨 60_000 边界 → text-warn/text-ok mismatch
  - 修法: useState `now` + useEffect setInterval 1s ticker, mount 前 now = 0 强制 text-ok
- False alarm 4 处 (经人工 review 排除):
  - **I18nProvider localStorage 5 处**: 已用 mounted gate 模式 (line 86-92 注释明确禁止 useState 初值读 storage)
  - **agent-view seed line 126**: 在 useEffect 内调用, SSR 不跑
  - **tooltip vw/vh line 102-103**: 在 useLayoutEffect `measure` callback 内, SSR 不跑
  - **tooltip useRef Math.random() id**: useRef 初值不渲染到 DOM

**累计 hydration 修复** (per 2026-09-06 20:14 JST):
1. `a35eb94` GasParticlesHint typeof window 判定 (commit a35eb94)
2. `a7e6f47` AgentViewContent derivedAt useMemo 内 new Date() (commit a7e6f47)
3. `fc075fd` local-runtime page className Date.now() (commit fc075fd)

**统一模板** (per 3 次修复实证):
```tsx
const [mounted, setMounted] = useState(false);
useEffect(() => { setMounted(true); }, []);
if (!mounted) return <safe-fallback/>;
// or: value = mounted ? computeReal() : defaultValue;
```

**根因总结**: React 18 + Next.js 14 streaming SSR 下, useState/useMemo 内的 `new Date() / Date.now() / Math.random()` 在 server t0 跟 client t0+δ 必然不同, 直接走 render 必 mismatch. mounted gate 模式让 first render 跟 server 一致, mount 后再走真实值.

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
| G7 | helm.exe 在 WinGet 包目录 (版本号路径), 不在常规位置 (per 9/6 18:48 JST 实证) | 脚本要 hard-code 4 个候选 + user PATH 扫 | 已修: Resolve-HelmExe 函数 (commit 61acf5e) |
| G8 | Windows PATH 限制 2047 字符, user PATH 超长末尾被截断 (per 9/6 18:48 JST 实证 user PATH 2204 → 截断) | G7 间接修复: ensure helm 走绝对路径, 不依赖 shell PATH | 已规避 (commit 61acf5e) |
| G9 | MiniMax Code 等长驻进程不重读注册表, child 进程拿不到新 user PATH (per 9/6 18:48 JST 实证) | 即使修 user PATH 也无效, 必须重启宿主 | 外部问题, 报告加 G9 标 N/A |
| G11 | ✅ 已验证 (per 9/7 02:58 JST WSL Ubuntu k3s server 实证): v1.36.3+k3s1 (gitCommit 5aed4d7beddeb3e67120da477c876ac9efd70318, go1.26.5, build 2026-08-04, linux/amd64); 单节点 ulyssespc Ready (5d18h, Ubuntu 24.04.3 LTS / kernel 5.15.167.4-microsoft-standard-WSL2, containerd 2.3.2-k3s2); 2 个 pod 异常 (battle-service + network-gateway CrashLoopBackOff 7h18m, 同期事件); rust-game-server ns 有 30+ pods (admin / cluster-ops / gm-backend / grafana / nats / prometheus 等 Running) | k3s 装在 WSL Ubuntu 内, kubeconfig 600 root 需手动 cp 到 ~/.kube/config | update-backend-k3s.ps1 现有 G3 (KUBECONFIG) + G4 (chart 缺 deployment) 仍需修才能用, 验证 kubectl OK 但 helm upgrade 会被 G4 卡住 |
| G10 | ✅ 已修 (per 9/7 02:45 JST cherry-pick a539816 + f207499, commit 58d0f97): (a) agent-view / projects 迁到 (app)/ 路由组 (自动获得 AppHeader); (b) AppShell 加 wide + fullBleed variant, AppHeader tab 加 shrink-0 whitespace-nowrap; (c) agent-view 删 core3d/Roguelike 模式 tabs (跟 sidebar 重复, 9/7 02:38 JST 用户拍板) | 影响: 2 个 page (projects / agent-view) 已统一, 剩余 28 个根目录 page (audit / board / canvas / automation 等) 仍走 root layout 缺 AppHeader | 后续: DDD Review 阶段考虑迁剩余 28 个 page, 或保留现状 (每个 page 自有 page-level nav) |

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
| v0.2 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 第二轮回归 R1'/R2'/R3': 修 2 个真 bug (smart pull + ensure helm), 加 G7-G9 已知缺口, §2.5 新增 | 2026-09-06 19:09 JST 完成 commit 61acf5e + R2''/R3' 实测后落档 |
| v0.3 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 第三轮修复: /agent-view hydration mismatch (commit a35eb94), §2.6 新增, 衍生守门 ("typeof window 改 useState mounted 模式") | 2026-09-06 19:26 JST 完成 in-app browser 实证 + commit a35eb94 后落档 |
| v0.4 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 第四轮: merge main (5 ahead commit) + 全仓 hydration 扫描 (81 处, 1 真风险修 / 4 false alarm 排除) + commit a7e6f47 + fc075fd, §2.7 新增, 3 次 hydration 修复统一模板 | 2026-09-06 20:14 JST 完成 in-app browser 实证 + commit fc075fd 后落档 |
| v0.5 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 第五轮: nav 统一讨论, 用户拍板先不干, 加 G10 已知缺口 (AppHeader 路由组不一致 + 溢出) | 2026-09-06 20:25 JST in-app browser 截图 + 4 选项拍板后落档 |
| v0.6 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 第六轮: 用户用 Claude Sonnet 5 自己改完 (per 9/7 02:38 JST), 我 cherry-pick ux-cel-depth 2 commit (a539816 + f207499) 到 worktree, 解 1 个 conflict (保留 HEAD hydration fix), G10 改已修 2/30 page | 2026-09-07 02:45 JST cherry-pick + commit 58d0f97 落档后 |
| v0.7 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 第七轮: G11 k3s 真实版本验证 (WSL Ubuntu 内 k3s v1.36.3+k3s1 在跑, 1 节点 Ready, 30+ pods Running + 2 个 CrashLoopBackOff 7h18m), 加 G11 已知缺口, 标 k3s 端 OK, G1-G4 部署端仍是硬缺口 | 2026-09-07 02:58 JST WSL 内 kubectl get nodes + kubectl get pods -A 实证后 |
| (后续) | (待 SRE Lead / 5 域 Lead / DDD Review) | 剩余 28 个根目录 page 迁 (app)/, G1-G4 k3s 部署补完, battle-service + network-gateway 7h18m CrashLoopBackOff 排查 | 拍板 Dockerfile + chart + 全量 page 迁后 |
