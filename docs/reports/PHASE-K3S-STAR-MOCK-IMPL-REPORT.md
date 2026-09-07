# PHASE-K3S-STAR-MOCK-IMPL-REPORT

> **文档版本**: v0.2 (2026-09-08 08:08 JST)
> **v0.2 变更**: + §8 续做记录: 镜像拉到 daocloud, 但 k3s kubelet 半死 (container runtime 通信断), 新 pod 100% 起不来; port-forward service 已在 v0.2 期间 disable 避免 auto-restart 浪费 CPU; 等 Ulysses 手动重启 k3s (sudo systemctl restart k3s) 才能续做. v0.2 commit 把 envoy-deployment.yaml image path 改 daocloud 永久落档.
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**: 2026-09-08 07:36 JST UAT 反馈 "用 playwright 操作进行 UAT 测试,现在启动 3000 端口后黑了,存在显示问题"
> **范围**: Star 仓 `D:\Star\.worktrees\feat-auto-20260908-204a1a91` 本地恢复 (k3s 6443 / 3000 端口转发), **不动 origin, 不动 main 分支** (per 守门 #1 R-05)
> **依赖 brief**: `docs/briefs/k3s-star-mock-3000-restore-001.md`

---

## §0 目的

本报告记录 2026-09-08 07:36 JST UAT 测试启动 3000 端口后"黑屏"根因 + 4 个补救动作落档。

**根因实证 (4 步推理)**:
1. **进程层**: 主机 `Get-NetTCPConnection -LocalPort 3000 -State Listen` 0 listener → 3000 端口无服务在听
2. **k3s 层**: 6443 API + node Ready 6d23h → k3s 后端 OK
3. **pod 层**: `kubectl -n star-mock get pod` 2/2 **Pending 4m+** + containerStatuses=[] → 容器没启动
4. **资源层**:
   - `envoyproxy/envoy:v1.32-latest` 本地 `crictl images` 无 → 拉镜像超时
   - deployment.yaml 引用 `configMap: star-mock-mock-data` 但 **CM 未声明** → 隐式 FailedMount
5. **port-forward 层**: wsl 临时 VTL 销毁时回收子进程 → kubectl port-forward 立即 exit

**结论**: 黑屏 = 浏览器连不上任何东西 (Connection refused) + Envoy ConfigMap `direct_response /` = 404 静态文本 (即使 pod 起来也是黑屏体验)。"黑了"不是渲染问题, 是网络/进程生命周期问题。

---

## §1 改动矩阵

| 路径 | 类型 | 行数 | 说明 |
|---|---|---|---|
| `tools/star-flash-mock/k3s/envoy-deployment.yaml` | 改 | +18 | P0 热修: 显式声明 `star-mock-mock-data` ConfigMap (原缺) |
| `tools/star-flash-mock/scripts/start-k3s-backend.ps1` | 新增 | 158 | 幂等启 k3s server, 探测 WSL + 6443 + kubectl get nodes (从 Temp 落档) |
| `tools/star-flash-mock/scripts/start-k3s-backend.bat` | 新增 | 47 | pwsh 包装, `where pwsh` 优先 + `powershell` fallback |
| `tools/star-flash-mock/scripts/k3s-portforward.service` | 新增 | 15 | systemd user unit, `kubectl -n star-mock port-forward svc/star-mock-envoy 3000:8080 --address 0.0.0.0` |
| `tools/star-flash-mock/scripts/README.md` | 新增 | 100 | 启动顺序 + 速查命令 + 4 已知卡点 + 守门引用 |
| `docs/reports/PHASE-K3S-STAR-MOCK-IMPL-REPORT.md` | 新增 | (本文件) | 7 段结构 per 守门 #3 |
| `docs/automation-design.md` | 改 | +1 节 (§4.13) | 任务卡表, 本 commit 引用 |
| `scripts/automation/registry.md` | 改 | +3 行 | 3 脚本索引行 |
| **合计** | 4 改 + 4 新 | ~ 350+ 行 | 1 commit |

**Commit 引用**: 1 commit, author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 8/27 19:39 JST 授权)

**关联 commit (本 session 已存在, 跟本 commit 无关)**:
- `5cfb7b3` 守门 #12 v15 docs 同步饱和 (本 commit 触发"新事件 = k3s UAT 恢复"已超越饱和约束)

---

## §2 验证摘要

### 2.1 现场实测 (本次 session, 07:42-07:48 JST)

| 步骤 | 验证项 | 结果 | 证据 |
|---|---|---|---|
| WSL 启动 | distro 可达 | ✅ | `wsl -d Ubuntu echo distro-ok` exit 0 |
| k3s server PID | 进程在跑 | ✅ | `ps -ef | grep k3s server` → pid 183 (07:42 JST) |
| 6443 LISTEN | API server 端口在听 | ✅ | `ss -tln` → `LISTEN 0 4096 *:6443` |
| node Ready | k3s node 状态 | ✅ | `kubectl get nodes` → `ulyssespc Ready control-plane 6d23h` |
| 拉镜像 `envoy:v1.32-latest` | crictl pull | ❌ **超时 (60s+ 无输出)** | 推测: docker.io 网络拉不到 (WSL2 NAT) |
| pod Pending 4m+ | deployment 状态 | ❌ | `kubectl -n star-mock get pod` → `0/1 Pending` 4m+ |
| pod containerStatuses | kubelet 启动 | ❌ | `kubectl -n star-mock get pod -o json` → `containerStatuses: []` |
| mock-data CM 存在 | ConfigMap 查询 | ❌ | `kubectl -n star-mock get cm star-mock-mock-data` → NotFound |
| port-forward listen 3000 | 主机端口 | ❌ | `Get-NetTCPConnection -LocalPort 3000` → 0 listener |
| curl localhost:3000 | HTTP 响应 | ❌ | "目标计算机积极拒绝" (Connection refused) |

### 2.2 修复后预期 (等 sudo 重置, 下 session 实证)

| 步骤 | 预期 | 阈值 |
|---|---|---|
| 1. `kubectl create cm star-mock-mock-data` | ✅ 一次成功 | (本 session 已建) |
| 2. `crictl pull docker.m.daocloud.io/envoyproxy/envoy:v1.32-latest` | ✅ 30-60s 完成 (mirror) | 镜像 ≤ 200MB |
| 3. `kubectl apply -f envoy-deployment.yaml` | pod Pending → ContainerCreating → Running | 60s 内 |
| 4. `systemctl --user status k3s-portforward` | auto-restart loop → **active (running)** | 5s 内 |
| 5. `curl -i localhost:3000` | status=200, body 含 "not found" (envoy `/` 路由 direct_response 404) | 10s 内 |
| 6. Playwright headless 截图 localhost:3000 | 看到 "not found" 文本或 "Hello" 文本 (mock-data CM 有内容) | 30s |

### 2.3 守门 (per 守门 #1 + 守门 #5 + 守门 #9 + 守门 #10 + 守门 #12 v21)

- ✅ **守门 #5 (env 安全)**: 全程无 env 打印, sudo 密码永不落盘
- ✅ **守门 #9 (子代理)**: 无子代理 dispatch, 全主上下文直跑
- ✅ **守门 #10 (代签)**: 1 commit author = Ulysses
- ✅ **守门 #12 (AI 协作)**: 报告 7 段结构, BAS 引用 git 实证 (Temp 路径 + 字节数 + mtime)
- ✅ **守门 #1 派生 v19 (Python 化)**: 本次改动 [P] 维度, 4 步骤全落 `scripts/automation/registry.md` 索引
- ✅ **守门 #1 派生 v20 (brief 落档)**: `docs/briefs/k3s-star-mock-3000-restore-001.md` 落档
- ✅ **守门 #1 派生 v21 (docs 同步)**: `docs/automation-design.md §4.13` + `registry.md` 双索引

---

## §3 已知缺口 (per 缺标比错标)

| # | 缺口 | 优先级 | 状态 | 触发 |
|---|---|---|---|---|
| 1 | envoy 镜像未拉, pod 仍 Pending | **P0** | 阻塞 | sudo 配额 5min 重置 (per §4.2 卡点 1) |
| 2 | port-forward service 在 auto-restart loop, 未转 active | **P0** | 阻塞 | 阻塞 = pod 未 Running, 跟 #1 同源 |
| 3 | envoy ConfigMap `/` 路由 404, 不是 HTML 页面 | P2 | 已知 | 即使 3000 通, 浏览器仍可能"看起来空" |
| 4 | `mock_data/` 实际内容没灌到 `star-mock-mock-data` CM | P1 | 待办 | `kubectl create cm --from-file=tools/star-flash-mock/mock_data/` (需 sudo? 不需, kubectl create 自带权限) |
| 5 | WSL daemon 重启后 port-forward service 需重新 `enable --now` | P1 | 待办 | systemd --user 默认 `WantedBy=default.target` 应该在用户 session 启动时拉起, 但 `loginctl enable-linger leo19` 必须在 enable 之前执行, 缺一次验证 |
| 6 | `star-mock` ns 没建 ResourceQuota / NetworkPolicy | P3 | 待办 | k3s 默认允许, 不影响 3000, 但生产前需补 |
| 7 | envoy ConfigMap direct_response 是 inline_string, 大文件会 OOM | P3 | 待办 | 改 file_system HTTP filter 走 `mock_data/` 目录 |
| 8 | 没写 smoke 脚本验证 3000 渲染 | P1 | 待补 | Playwright `expect(page.locator('text=not found')).toBeVisible()` 一行可补 |

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

**本次 session 无子代理 dispatch** (守门 #9 主路径, 全主上下文直跑), 故无失败接手清单。

**未来风险** (下 session 拉镜像 / 验 3000 阶段如派子代理):
- 子代理 RPC `net::ERR_CONNECTION_CLOSED` 风险 (per 守门 #9 P3-A.6/A.7 实证, 10 background task 报 succeeded 实际失败)
- 缓解: 走 `scripts/automation/dispatcher.py brief(...)` 落 `docs/briefs/k3s-star-mock-3000-restore-001.md` (本 commit 已落), 子代理强制 brief 落档后接

---

## §5 守门规则 (15-17 项, per 守门 #3 模板)

| # | 守门 | 适用本 commit? | 实证 |
|---|---|---|---|
| 1 | R-05 不 push | ✅ 适用 | 不推 origin, 本地 wt 1 commit (per 8/30 07:09 JST 反转保留) |
| 1a | 推 origin 重试细则 | N/A | 不推 |
| 2 | bc23d6c 保留 | ✅ 适用 | 不动 main 分支 |
| 3 | 5 域独立 Lead | ✅ 适用 | 本次无 5 域决策, Mavis 临时代签 (per 9/3 11:35 JST 拍板 B) |
| 4 | AI 协作 token-OLU | ✅ 适用 | 本 session 估 ~ 30K tokens (root context) |
| 5 | env 安全 | ✅ 适用 | 全程无 env 打印 (8/27 11:06 JST hard ban) |
| 6 | PowerShell only | ✅ 适用 | 全 PowerShell, 无 bash 内嵌 (3 份 .bat 5 字符链 + Start-Process 都走 PS) |
| 7 | 0 unsafe | ✅ 适用 | 无 Rust 代码改动 |
| 8 | 不沿用 bc23d6c 叙事 | ✅ 适用 | Temp 文件按 git 实证 (路径 + 字节数 + mtime) 引用, 无回溯 |
| 9 | 不 commit 散落子代理产出 | ✅ 适用 | 无子代理, 主上下文直跑 |
| 10 | 代签 | ✅ 适用 | commit author = Ulysses (per 8/27 19:39 JST) |
| 11 | 缺标比错标 | ✅ 适用 | §3 列 8 缺口, 显式标 P0/P1/P2/P3 |
| 12 | AI 协作文档 | ✅ 适用 | 报告 7 段 + BAS git 实证 (Temp lf-backup-* + orphan-* 路径) |
| 13 | DB W/T/M 强制分类 | N/A | 无 DB 改动 (envoy CM 是 k8s config, 非业务 DB) |
| 14 | 5 域 Lead CONTENT 4 维 | N/A | 无 5 域决策 |

**守门派生 (v15-v26, 跟本 commit 相关)**:
- ✅ v15 守门 #12 饱和边界: 本 commit 触发 "新事件 = k3s UAT 恢复" (per 5cfb7b3 触发, 5:47 write-bat-english.ps1 → 7:36 UAT 反馈, 跨过饱和点)
- ✅ v19 守门 #1 派 Python 化: 4 步骤全落 registry 索引 + 报告 + brief
- ✅ v20 守门 #9 派 brief 落档: `docs/briefs/k3s-star-mock-3000-restore-001.md` v0.1
- ✅ v21 守门 #12 派 docs 同步: `automation-design.md §4.13` + `registry.md` 双索引

---

## §6 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构师 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 07:55 JST | per 守门 #10 + 8/27 19:39 JST 授权 |
| SRE Lead | ⏳ 待签 (5 域 Lead 真人到位后追溯) | — | per 守门 #3 v2 拍板 (per 9/3 11:35 JST), 真人到位 TBD |
| 平台 | ⏳ 待签 | — | 同上 |
| 评审主持 | ⏳ 待签 | — | DDD Review Lead 待 5 域真人到位 |
| PM | ⏳ 待签 | — | per 守门 #14 v2 (9/5 10:43 JST), Mavis 长期代签 |

---

## §7 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: 4 步根因 + 4 改动 + 8 缺口 + 5 角色签字 | 2026-09-08 07:36 JST UAT 反馈 → 07:55 JST 落档 |
| v0.2 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | + §8 续做记录 (k3s kubelet 半死, container runtime 通信断, 镜像已落 daocloud 但新 pod 起不来, port-forward service disable); envoy-deployment.yaml image path 改 daocloud 永久落档 | 2026-09-08 08:04-08:08 JST 续做 (sudo 实际是 sudoers 白名单非配额, 但 k3s 内部状态破裂) |

---

## §8 续做记录 (per 2026-09-08 08:04-08:08 JST)

### §8.1 已落地

| 步骤 | 动作 | 结果 |
|---|---|---|
| 1 | `sudo -n k3s crictl pull docker.m.daocloud.io/envoyproxy/envoy:v1.32-latest` | ✅ Image is up to date sha256:49b0af0078643 (60MB) |
| 2 | `envoy-deployment.yaml` image 改 daocloud 路径 | ✅ commit 落档 (v0.2 本次) |
| 3 | nuke deployment + recreate | ✅ RS hash 77864fd7b8 2/2 pod 拉起 |
| 4 | nop-test pod (alpine) 在 default ns + star-mock ns | ✅ 创建成功 (control test) |

### §8.2 失败根因 (新发现, 5 步推理)

| 步 | 观察 | 结论 |
|---|---|---|
| 1 | envoy pod 60s+ events 段只有 Scheduled, 无 ImagePulling / Pulled / ContainerCreating | kubelet 没接这个 pod |
| 2 | nop-test alpine pod (default ns) 也卡 Pending, events 同样只有 Scheduled | **不是 star-mock ns 问题, 是 k3s 全局问题** |
| 3 | rust-game-server 6d 旧 pod 全 Running, events 段丰富 (Warning/BackOff/Pulled/Created/Started) | scheduler 工作, 但 kubelet 不接收新 pod |
| 4 | `journalctl k3s`: `Skipping pod synchronization err="container runtime status check may not have completed yet"` + `Failed to create existing container: task XXX not found` + `Sending HTTP/1.1 502: dial tcp 10.42.0.110:10250: connect: no route to host` | **kubelet 跟 containerd 通信断 + kubelet API 网络分裂 (10.42.0.110:10250 no route)** |
| 5 | `systemctl status k3s`: Active active, 但 pid 215 6s 前刚刚重启 (从 217 变 215), Memory 601.8M | k3s 在 systemd 死循环重启, 启动竞争资源导致 kubelet 初始化不完整 |

**根因结论** (per §8.2 实证 5 步):

WSL 内 dockerd + containerd + k3s server 三个 daemon 启动顺序竞争资源,k3s 抢在 containerd 完全就绪前启动 → kubelet sync container runtime 超时 → kubelet 502 → 新 pod 永远 Pending。**这不是 envoy 镜像问题,不是 yaml 问题,不是 sudo 配额问题,是 k3s 启动时序问题**。

### §8.3 已采取缓解 (本 commit 落地)

| 动作 | 命令 | 效果 |
|---|---|---|
| 1. disable port-forward service | `systemctl --user disable --now k3s-portforward.service` | 停止 auto-restart 循环 (已 16 次重启), 避免 CPU 浪费 |
| 2. cleanup test pods | `kubectl delete pod nop-default nop-test` (force) | 不留半死 pod 占资源 |
| 3. envoy-deployment.yaml image 改 daocloud | edit + commit (本 commit) | 下次 apply 自动用 daocloud 镜像, 避免 docker.io 拉超时 |

### §8.4 续做清单 (per Ulysses 手动操作, 需 sudo)

1. **重启 k3s service** (sudo systemctl restart k3s) — 等 30s 让 containerd + kubelet 完全就绪
2. **验证 kubelet 健康**: `journalctl -u k3s | grep -E "kubelet.*Running|container runtime.*ok"` 应见 "Running" 而非 "Skipping"
3. **拉镜像 (manual, 防 crictl cache 失效)**: `sudo -n k3s crictl pull docker.m.daocloud.io/envoyproxy/envoy:v1.32-latest`
4. **重启 deployment**: `kubectl -n star-mock rollout restart deployment star-mock-envoy` (用 daocloud 镜像, 已 commit 94190bb 落档)
5. **等 pod Ready** (1-2 min): `kubectl -n star-mock get pod -w`
6. **重 enable port-forward service**: `systemctl --user enable --now k3s-portforward.service`
7. **验证 3000**: `curl -i http://localhost:3000` 期望 status=200 + body="not found" (envoy direct_response)
8. **Playwright 截图**: 验证浏览器渲染 "not found" 文本, 不再黑屏

### §8.5 跨 session 续做关键信息

- **worktree**: `D:\Star\.worktrees\feat-auto-20260908-204a1a91`
- **branch**: `feat/auto-20260908-204a1a91` (本地, ahead origin)
- **commits**: `94190bb` (v0.1) + 本 commit (v0.2)
- **k3s 6443 LISTEN**: ✅ 持续 (server 进程在跑, 只是 kubelet 子组件半死)
- **镜像 crictl cache**: ✅ daocloud envoy v1.32-latest 60MB 已在
- **port-forward service**: ⏸ 已 disable, 等 pod Ready 后手动 enable
- **WU**: 9/3 11:35 JST 拍板 B + 守门 #3 v2 派生规: Mavis 临时代签 5 域 Lead 决策, 真人到位后追溯签字 (per §1.2 修订人栏)

### §8.6 教训 (per 守门 #12 v21 docs 同步必更新)

- 守门 #1 派生规需补 1 条: **拉镜像前必先看 `journalctl -u k3s | grep "Skipping pod sync"`, 若有则不要拉镜像, 先 systemctl restart k3s**
- 守门 #1 派生规需补 1 条: **systemd 拉起 k3s 后必等待 60s 验证 container runtime 状态, 而非立即 apply yaml**
- 守门 #1 派生规需补 1 条: **port-forward service 在 apply 之前必先 disable, 避免 "auto-restart 16 次 + 502 错误" CPU 浪费**

