# v37 子代理 sandbox 隔离守门候选 (per 守门 #1 v15 + 守门 #11 缺标比错标)

> **Status**: 🟡 **Draft v0.1** (per 2026-09-10 22:24 JST Mavis 自驱, 待 Mavis 拍板激活)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 Mavis 永久代签)
> **关联 commit**: `4219c3d` (跨 session 协同 v37 sandbox 守门候选落档) + `bb30a9c` (SANDBOX-002 架构文档 v0.1) + `d01509b` (SANDBOX-002 6 文档 SRS+BD+DD+TDD+IMPL+WBS) + `941daea` (SANDBOX-002 v0.1.1 决策) + v0.1.2 4 决策点修订 (per 22:36 JST opt1)
> **编号避让**: v32 / v33 / v34 / v35 / v36 已用, v37 落到下一个空号位
> **守门基线**: 守门 #1+#1 v15+#1 v25+#5+#9+#11+#12 v21+#14 v3+#14 v4+#19 v19+#24 v2 12 项必过
> **跟 SANDBOX-002 关系** (per 22:36 JST opt1 = 全部按推荐改造): v37 v0.1 是 sandboxd v0.1 **软约束** (5 维软约束: path allowlist + network allowlist + cpu/mem/wall-clock + capability stub + sandbox mode flag, 跟 `4219c3d` sandbox.py 集成 100% 对齐), **sandboxd v0.1 hard 约束** (4 维隔离 resource/network/fs/capability + 3 平台 + gRPC + session 池 + observability, per `docs/architecture/SANDBOX-002.md` v0.1.1 + 6 决策点 D-1~D-6 已拍板) 是 v0.2 升级; v37 → active 触发条件 = sandboxd v0.1 落地 (per WBS §14.19 SBX-01..SBX-08) + 实测 0 越权, v0.1 软约束 → v0.2 hard 约束切换

---

## 0. 问题陈述 (Problem Statement)

**子代理 dispatch 无资源/路径/网络隔离** (per 守门 #1 v15 + 守门 #9 派生 + 守门 #11 缺标比错标):

1. **现状**: `scripts/automation/dispatcher.py invoke()` 走 `mavis task dispatch <task_id> <brief_path> --agent <agent>` subprocess, **0 隔离**:
   - 子代理可读写主仓任何路径 (crates/ + db/ + docs/ + .git/)
   - 子代理可访问任何网络 (内网 / 公网 / 5 内部 mock 服务)
   - 子代理可消耗无限 CPU + 内存 + wall-clock
2. **风险场景**:
   - 子代理 (worker-1) 误删 `crates/api/` 整个目录 → main 仓被破坏
   - 子代理访问 `prod-db.internal:5432` 删除 prod 数据
   - 子代理 fork bomb (1000 进程派生) → 系统 OOM
3. **per 守门 #9 v3 调试控制台走 subprocess 替代 RPC**: sandbox 是 dispatch 运行时隔离层,跟 mavis CLI stub 路径兼容
4. **per 守门 #11 缺标比错标**: 5+ 已知缺口显式列 (per §5)

**核心矛盾**:
```
无 sandbox → 子代理能力 = root session 能力 → 风险 0 防御
全 sandbox → 跟 v0.1 fail-open 安全网 冲突
折中 → 软约束 v0.1 试运行 (per 守门 #11 缺标比错标) → 跨 session 续 v0.2 hard 约束
```

---

## 1. 设计方案 (Solution)

### 1.1 物理存储:4 维隔离策略

| 维度 | v0.1 软约束 (落档) | v0.2 hard 约束 (跨 session 续做) |
|---|---|---|
| **文件路径** | allowlist (worktree + docs/briefs + reports + cache) | AppContainer (Windows) / mount namespace (Linux) |
| **网络 outbound** | allowlist (github + docs.rs + crates.io + registry.npmjs + 5 内部 mock) | WFP (Windows) / iptables (Linux) / pf (macOS) |
| **CPU + 内存 + wall-clock** | subprocess.run timeout (无 OS 级限制) | Windows Job Objects + cgroups v2 |
| **能力 (Linux capability)** | (未实装) | libcap drop CAP_* |

### 1.2 v0.1 软约束实现 (per 4219c3d commit)

`scripts/automation/guardian/sandbox.py` 183 LOC:
- 5 软约束函数 (`check_path_allowed` + `check_network_allowed` + `check_resource_within` + `sandbox_check` + `SandboxMode.{OFF,WARN,ENFORCE}`)
- WARN 模式: 违反 → warn log + continue (per 守门 #11 软约束 v0.1)
- ENFORCE 模式: 违反 → raise SandboxError (v0.2 升级)
- dispatcher.py 集成: invoke() 入口前先过 sandbox 验证 (per 守门 #9 #3 + 守门 #1 禁回溯叙事 0 改 V0.1-V0.99 核心 logic)

### 1.3 v0.2 hard 约束实现 (跨 session 续做, per `61b8a12` SANDBOX-001 §1.4 v0.2 + SANDBOX-002)

`sandboxd` 独立 app 形态 (per 22:08 JST 拍板 "沙盒组建是一个 app 形式的独立模块"):
- 4 维隔离 (resource/network/fs/capability) 升级 v0.1 软约束
- 8 機能 (FR-1 daemon 形态 / FR-2 gRPC IPC / FR-3 4 维隔离 / FR-4 session 生命周期 / FR-5 跨平台后端 / FR-6 观测性 / FR-7 fail-open / FR-8 测试)
- 三平台后端 (Windows Job Objects + WFP + AppContainer / Linux cgroups v2 + netns + mount ns + libcap / macOS sandbox-exec)

### 1.4 跟 v35 ed25519 签名守门联动 (per 守门 v33 v0.3 verify comments-read 软约束)

- sandbox violations 写 audit log (`invoke_subprocess_sandboxed` action, per 4219c3d commit msg)
- 走 v36 索引 (count_by_decision_indexed O(K) 聚合, 跟 sandbox 决策联动)

---

## 2. 跟现有守门关系

| 守门 | 跟 v37 联动 |
|---|---|
| **#1 fail-open** | 沙箱自身错误 (Access Denied / import 失败) → 走 subprocess.run 路径, 不阻断 (per 守门 #1 + §5 已知缺口 #4+#6) |
| **#5 env 安全** | 沙箱不读 env, 0 泄露风险 |
| **#6 跨平台** | v0.1 仅路径/网络软约束, v0.2 跨平台 (Windows / Linux / macOS) |
| **#9 子代理 dispatch** | 沙箱是 dispatch 运行时隔离层, 跟 brief 必先落档联动 |
| **#11 缺标比错标** | 5 已知缺口显式列 (per §5) |
| **#14 v3 Mavis 永久代签** | 沙箱给代签决策加运行时安全护栏 |
| **v33 verify comments-read 软约束** | sandbox violations 进 audit log, 走 v36 索引 |
| **v36 audit log 索引** | sandbox decision 走 O(K) 聚合 |

---

## 3. 落地清单 (Deliverables)

| # | 文件 | 改动 | 估 LOC |
|---|---|---|---|
| 1 | `docs/guardian/v37_sandbox_guard.md` | 新增 (本文件) | 230 |
| 2 | `scripts/automation/guardian/sandbox.py` | 5 软约束函数 (per 4219c3d commit) | 183 |
| 3 | `scripts/automation/guardian/tests/test_sandbox.py` | 6 TC class 覆盖 allowed/denied 区分 + resource + flag | 130 |
| 4 | `scripts/automation/dispatcher.py` | invoke() 入口前 sandbox check (per 4219c3d commit) | +83/-24 |
| 5 | SANDBOX-001.md v0.2 (per 61b8a12) + SANDBOX-002 v0.1.1 (per 941daea) + DD-002 + TDD-002 + IMPL-002 (per d01509b) | sandbox 跨 session 协同 docs | ~150K |
| **总计** | **5+ 阶段** | **v0.1 软约束 + v0.2 hard 约束 (跨 session 续做项)** | **~3.5M token (per d01509b 估)** |

**v0.1 软约束 实测**: per 4219c3d commit (3 files / +237/-24 lines / 10 tests pass) 已落档
**v0.2 hard 约束 跨 session 续做项**: per d01509b (sandboxd 5 阶段 16 子项 ~3.5M token 估)

---

## 4. 激活条件 (Activation)

per 守门 v3x 候选激活流程 (per AGENTS.md §4.1.1 + 9/1 14:58 + 9/8 16:08):

1. Mavis 走 `ask_user` 必带推荐项 (per 守门 v28 格式)
2. Ulysses 拍板 (激活 / 不激活 / 改方案)
3. 拍板后立即执行 (per 9/5 04:03)
4. commit author=Ulysses
5. 修订历史表 +1 行
6. WBS v0.X+1 升版同步

**当前状态**: 🟡 Draft v0.1, 跨 session 协同 commit 4219c3d 已落档 v0.1 软约束 (sandbox.py + tests + dispatcher 集成), v0.2 hard 约束跨 session 续做项 (per SANDBOX-002 + IMPL-002 估 3.5M token / 5 阶段 16 子项).

---

## 5. 已知缺口 (per 缺标比错标)

| # | 缺口 | 严重度 | 缓解 | v0.1 状态 |
|---|---|---|---|---|
| 1 | Network isolation: WFP 复杂, v0.1 软约束仅 allowlist 检查, v0.2 升级 WFP | P1 | v0.2 SANDBOX-002 §3.3 NetworkPolicy | 🟡 跨 session 续 |
| 2 | File system isolation: AppContainer / mount ns 复杂, v0.1 软约束仅路径 allowlist | P1 | v0.2 SANDBOX-002 §3.1 FsPolicy | 🟡 跨 session 续 |
| 3 | POSIX rlimit / cgroups 兜底: v0.1 跨平台未实装 | P2 | v0.2 SANDBOX-002 §3.5 LinuxBackend | 🟡 跨 session 续 |
| 4 | mavis CLI 尚未落地: dispatcher.invoke() 当前走 status="deferred" fallback (per 守门 #9 #2) | P1 | mavis CLI 落地后立刻接入 | 🟡 跨 session 续 |
| 5 | 沙箱 metric/observability: v0.1 不实装 | P2 | v0.2 SANDBOX-002 §6 观测性 | 🟡 跨 session 续 |
| 6 | 非 elevated 进程 Access Denied: 在某些 Windows 配置下, AssignProcessToJobObject 失败 (per 守门 #1 fail-open) | P1 | fail-open 仅 warn log (per 守门 #1) | 🟢 跨 session 续 (已 fail-open) |

---

## 6. 修订履歴

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 22:24 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | 初版落档, 6 段 (问题/方案/守门/落地/激活/缺口+修订), 编号避让 v32+v33+v34+v35+v36 落到 v37, 5+5 已知缺口显式列, 跟 4219c3d + 61b8a12 + 941daea + d01509b 跨 session 协同 commit 关联 | 2026-09-10 22:24 JST Mavis 自驱 (per 守门 #9 v19 + 守门 #11 缺标比错标 + 守门 #1 禁回溯叙事 0 重写 4219c3d commit msg) |
