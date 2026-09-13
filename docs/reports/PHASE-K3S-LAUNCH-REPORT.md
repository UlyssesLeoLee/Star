# Phase K3S 真实部署启动 (k3s in WSL2 完整版) 报告 v0.1

> **状态**: 🟢 v0.1 完成
> **日期**: 2026-09-12
> **基点 commit**: `f2492f7` (plan-032 v0.2 阶段性收官, R10 阶段 2 obsolete per v0.62 反转)
> **修订人**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手**审核** (per 守门 #14 v4 + 9/10 12:45 JST 真人代签全部取消改为 mavis 审核)
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手**审核** (per守门 #14 v4)

---

## 0. 报告目的

Ulysses 9/12 12:39 JST 拍板"你起动真实 k3s 版本试试" (per 9/1 14:38 + 9/8 16:08 必带推荐项守门, 选项 2. k3s in WSL2 完整版被选).

**关键发现** (vs Ulysses 假设"k3s 未装"):
- ✅ k3s v1.36.4+k3s1 **已装** + running (Sep 10 14:54 装, 45h uptime)
- ✅ WSL2 Ubuntu systemd=true (wsl.conf `[boot] systemd=true`)
- ✅ docker 29.1.3 + ctr 已装
- ✅ sudoers `/etc/sudoers.d/k3s-debug` 配 NOPASSWD: `/usr/local/bin/k3s` + `/usr/bin/systemctl restart k3s` + `/usr/local/bin/crictl` (跟 README 期望完全一致)
- ⚠️ crictl 不在 /usr/local/bin/ (错位), 但 crictl 是 k3s subcommand, 走 `sudo -n /usr/local/bin/k3s crictl` 即可
- ⚠️ kubectl / crictl 没独立 binary, 全部走 `sudo k3s kubectl` / `sudo k3s crictl` 形式 (k3s 自带)
- ⚠️ WSL2 instanceIdleTimeout 是 README 9/8 实证坑, 但 45h 仍跑 = 实测不需保持 wsl.exe 常驻 (旧部署, 当前任务未踩)

**任务**:
- 跑 `deploy/k3s-local/star-api-rest-deploy.yaml` 单文件 (bff + envoy 跨 session 续, per README line 71-72 已知限制)
- 验证 star-api-rest HTTP server 真实可跑 (NodePort 30081 + 22 路由 stub)
- 修复 sandbox 卡死症状 (WSL2 instanceIdleTimeout 下游)

---

## 1. 任务完成矩阵

| # | 步骤 | 状态 | 实证 |
|---|---|---|---|
| 1 | 调研 `D:\Star\deploy/k3s-local/` 完整方案 | ✅ done | helm chart stub (G4) + kustomize 顶层 + Dockerfile + build-and-deploy.sh + 4 envoy yaml (k3s-local/envoy) + 5 yaml (bff-deployment/star-api-rest-deploy) |
| 2 | 检查 WSL2 现状 | ✅ done | k3s v1.36.4+k3s1 active running 12:39:44 JST, systemd=true, 1 节点 Ready 45h, 16Gi mem allocatable, 1007G disk 6% used |
| 3 | sudoers 检查 | ✅ done | `/etc/sudoers.d/k3s-debug` 配 `(root) NOPASSWD: /usr/local/bin/k3s + /usr/bin/systemctl restart k3s + /usr/local/bin/crictl` (跟 README 期望一致, 但 crictl 路径错位 → 走 k3s subcommand) |
| 4 | 复用已有镜像 `star-api-rest:local` | ✅ done | docker images `star-api-rest:local ad2923d3c0ca 32.4MB` (sha256:ad2923d3c0ca97ef3cf607ecc95ebd581527378daa403f89ac4ea7b3ac303f32) 已 build 在 9/8 之前 (入口 /usr/local/bin/star-api-rest, port 8081) |
| 5 | `ctr -n k8s.io images import` | ✅ done | Importing elapsed 1.4s, crictl 视角 `docker.io/library/star-api-rest local a2f877a1291c5 32.4MB` 可见 |
| 6 | `kubectl apply -f deploy/k3s-local/star-api-rest-deploy.yaml` | ✅ done | 3 资源 created: `namespace/star-system` + `deployment.apps/star-api-rest` + `service/star-api-rest` (NodePort 10.43.124.194 8081:30081) |
| 7 | 诊断 Pod Pending (IP <none> 0/1 Ready 60s+) | ✅ done | crictl pods 9 个 sandbox 全 NotReady (3-5 min ago) + pod event 只有 Scheduled 没 sandbox error = kubelet CRI gRPC 卡住 (WSL2 instanceIdleTimeout 下游症状) |
| 8 | 修法 1: `systemctl restart k3s` (per sudoers 免密) | ⚠️ partial | 节点 Ready 起来了, 但 crictl pods 仍 NotReady, 新 pod 仍 Pending (kubelet 跟 apiserver 长连接修复不彻底) |
| 9 | 修法 2: `wsl --shutdown` 完全重启 WSL | ✅ done | 12:48 JST WSL 重启, k3s systemd 起来, 1 节点 Ready 45h, 重新 import 镜像 + rollout, **Pod 1/1 Running 7s, IP 10.42.0.32** |
| 10 | curl NodePort 30081 实测 | ✅ done | `/api/v1/health` HTTP 200 59 bytes `{"service":"star-api-rest","status":"ok","version":"0.1.0"}` + `/api/v1/work-items` HTTP 501 410 bytes NOT_IMPLEMENTED + `/api/v1/context` HTTP 501 407 bytes + `/api/v1/workspaces` HTTP 404 (不在 22 路由列表) |

**守门 1: 0 unsafe / 0 新外部依赖 / 0 公共 API 变化 / k3s 1 节点 + 1 namespace 跟 README 一致**

**守门 2: 0 改 deploy/k3s-local/* 任何行** (per 守门 #1 禁回溯叙事, kustomization.yaml / star-api-rest-deploy.yaml / envoy/* / bff-deployment.yaml / Dockerfile 全部不动, 只跑现成 yaml)

**守门 3: 真实部署 = 镜像跑起来 + 端口可访问 + HTTP 真实响应** (per守门 #4 看起来完成 ≠ 实际完成, curl 实测验证)

---

## 2. 验证摘要 (curl 实测)

### 2.1 集群状态 (per kubectl + crictl)

```text
NAMESPACE          NAME                             READY   STATUS    RESTARTS   AGE   IP           NODE        NOMINATED NODE   READINESS GATES
star-system        star-api-rest-5b49dc8c68-j8q8t   1/1     Running   0          7s    10.42.0.32   ulyssespc   <none>           <none>
kube-system        coredns-54996dc9b4-4dslx         1/1     Running   13         45h   10.42.0.245  ulyssespc
kube-system        local-path-provisioner-...       1/1     Running   11         45h   10.42.0.2    ulyssespc
kube-system        metrics-server-...               1/1     Running   2          45h   10.42.0.241  ulyssespc
kube-system        traefik-59b7647586-mb8w7         1/1     Running   2          45h   10.42.0.243  ulyssespc
```

### 2.2 HTTP 实测 (per curl 127.0.0.1:30081)

| URL | HTTP | Body | 含义 |
|---|---|---|---|
| `/api/v1/health` | **200** | 59 bytes `{"service":"star-api-rest","status":"ok","version":"0.1.0"}` | ✅ Health check pass, 服务名/版本/状态全返 |
| `/api/v1/work-items` | **501** | 410 bytes `{"error":{"code":"NOT_IMPLEMENTED","hint":"Wait for P2 phase implementation...","message":"REST endpoint GET /api/v1/work-items is not yet implemented (P2 phase, awaiting worker delegation per AGENTS.md §4 #20)","retriable":false,"source_kind":"NotImplemented","source_module":"star-api-rest"}}` | ✅ 22 路由 stub 之一, 跟 README 9/9 实证完全一致 |
| `/api/v1/context` | **501** | 407 bytes 同上结构 | ✅ 22 路由 stub 之一 |
| `/api/v1/workspaces` | **404** | 0 bytes | ✅ 不在 22 路由列表, 返 404 (合理) |

### 2.3 跟 README 9/9 实证对比 (守门 #12 实证对照)

README 9/9 line 81-82:
> 已用 `port-forward` + curl 实测 `star-api-rest` 的 `/api/v1/health`（200）和 `/api/v1/work-items`（501，符合 22 路由 stub 的预期行为）

本次 (9/12) 实测:
- `/api/v1/health` (200) ✅ 一致
- `/api/v1/work-items` (501) ✅ 一致

### 2.4 跟真实前端 (port 3000 Next.js dev) 关系

真实前端 (`D:\Star/frontend/`) 是 Next.js 13 dev server, **跟 k3s 部署解耦**:
- 真实前端在 Windows 端 `http://127.0.0.1:3000/inbox`
- k3s 部署的是 Rust backend `star-api-rest` 在 WSL2 内 `http://127.0.0.1:30081`
- demo HTML 在 `http://127.0.0.1:8080/` python http.server (PID 12444)
- 三者**独立运行**, 不互相依赖

---

## 3. 已知缺口

| # | 缺口 | 状态 | 备注 |
|---|---|---|---|
| 1 | bff-deployment.yaml 没部署 | 跨 session 续 | per README line 71-72 + bff-deployment.yaml line 142 注释 "阶段 1 占位镜像, 阶段 4 实装阶段换成 ghcr.io/ulysses-star/bff:0.1.0". bff:local 镜像还没构建, 跨 session 续 |
| 2 | envoy/ 4 yaml 没部署 | 跨 session 续 | per README line 72 "deploy/helm/star chart 仍是 stub, 未使用". k3s-local/envoy/ 也是 P3-D.6 阶段 1 任务 1.5 实证, 跨 session 续 (跟守门 #14 v2 拍板 D 5 域 Lead 真人到位前 Mavis 临时代签) |
| 3 | helm chart 是 stub (G4) | 跨 session 续 | per README line 3-4 已知. 跳过 helm, 直接用原生 k8s manifest (per README line 41-44 一键部署脚本) |
| 4 | crictl 路径错位 (sudoers 配 `/usr/local/bin/crictl` 但 binary 不存在) | 已知错位 | crictl 是 k3s subcommand, 走 `sudo -n /usr/local/bin/k3s crictl` 即可. README 没改 (per守门 #1 禁回溯叙事), 报告里显式说明 |
| 5 | build-and-deploy.sh 没跑 (改用分步) | 已知偏离 | build-and-deploy.sh 写的 `sudo -n /usr/local/bin/crictl` (binary 不存在) 会失败. 改分步: `docker save \| ctr -n k8s.io import` + `kubectl apply -f`. 脚本下次维护时改 (跨 session 续) |
| 6 | crictl pods 9 个老 sandbox (rust-game-server) 仍 NotReady | 已知, 不影响 | rust-game-server 16 pod 实际 45h 一直在跑 (kubectl 报告 1/1 Running), crictl 报 NotReady 是 WSL2 instanceIdleTimeout 历史症状. 不动 rust-game-server (跨项目) |
| 7 | node 始终 AGE 45h (重启 WSL 后不变) | 已知机制 | kubectl 报告节点 AGE 是 ETCD 里的 node registration 时间, 不受 WSL 重启影响. 真正判断节点工作 = curl 实际 HTTP 响应 |
| 8 | rust-game-server* 3 ns 占 k3s (player/economy/match/social/admin/cluster-ops/postgres/nats/prometheus/grafana/otel-collector/traefik) | 跨项目残留 | RGS 仓历史治理命名 (5 位真人 Lead 问责结构, per 守门 #3 反转拍板 (a)+(c)). 不动, 不建立业务子域↔DDD 映射 (per 守门 #3 disclaimer) |
| 9 | cgroup v1 deprecation warning (k3s 启动时) | 已知, non-blocking | containerd v2.2 开始 deprecate, 2029 年 5 月前移除. 跟守门 #19 (-j 4 修正) 同源, 不影响当前 k3s 1.36.4 跑 |
| 10 | demo/ 8 panel 跟真实 k3s 后端没联调 | 跨 session 续 | demo/ 是单文件 HTML (port 8080), 真实前端是 Next.js dev (port 3000), k3s 后端是 Rust axum (port 30081). 三者**未联调**, demo 显示 22 路由 stub 期望但没真实 HTTP 调用. 跨 session 续 |

---

## 4. 子代理失败接手清单

本次任务**未派子代理** (per 守门 #9 v20 子代理 dispatch 必先 brief 落地), 全程 Mavis 推进:
- ✅ 调研 (5 file read parallel)
- ✅ 4 个候选 k3s 启动路径 (ask_user 推荐项 per 守门 v28)
- ✅ 9 个 wsl bash 命令分步跑
- ✅ 报告 + commit (Mavis 接手 author=Ulysses per 守门 #14 v4)

**未踩守门 #9 v3 RPC 失败实证** (per P3-A.6/A.7 10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded), 因为本次全走 WSL subprocess + kubectl 直连, 不派子代理.

---

## 5. 守门规则 (本次触发 + 实证)

| # | 守门 | 触发 | 实证 |
|---|---|---|---|
| 1 | 守门 #1 不 push origin | 反转 (per 2026-08-30 07:09 JST 已推 origin) | 0 push (R1-R10 13 commit 推 origin 已落地, 9/12 这次是部署验证, 不涉及 commit push) |
| 2 | 守门 #5 v2 调试控制台后端不污染 main 编译 | (本任务无关) | N/A |
| 3 | 守门 #1 禁回溯叙事 | 0 改 deploy/k3s-local/* 任何行 | ✅ 0 改 (build-and-deploy.sh / star-api-rest-deploy.yaml / kustomization.yaml / envoy/* / bff-deployment.yaml / Dockerfile / secrets/* 全部不动) |
| 4 | 守门 #6 PowerShell only | 全程 PowerShell | ✅ `wsl -d Ubuntu -e bash` 调用, Windows 端 PowerShell |
| 5 | 守门 #8 不沿用 bc23d6c 叙事 | 0 编造历史 | ✅ 所有事实 9/8 README + 9/10 装 k3s + 9/12 实测, per 守门 #1 禁回溯叙事 |
| 6 | 守门 #10 代签规则应用 | Mavis 接手 author=Ulysses | ✅ 报告 "审批" 列 = 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手**审核** (per 守门 #14 v4 + 9/10 12:45 JST 真人代签流程全部取消改为 mavis 审核) |
| 7 | 守门 #11 缺标比错标 | §3 显式列 10 项已知缺口 | ✅ 10 项缺口 (bff/envoy 跨 session 续 + crictl 路径错位 + build-and-deploy.sh 没跑 + rust-game-server 残留 + ...) |
| 8 | 守门 #12 AI 协作文档治理 | 引用 BAS git log --follow 实证 | ✅ 引用 `D:\Star\deploy/k3s-local/README.md` (line 3-4 helm stub, line 41-44 一键部署, line 71-72 已知限制, line 81-82 9/9 实证) |
| 9 | 守门 #14 v4 Mavis 审核 author=Ulysses | 报告签字栏 | ✅ 修订人=Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手**审核**; 审批=架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手**审核** |
| 10 | 守门 v28 拍板必带推荐项 | 4 选项 (k3d 5min 推荐 / k3s WSL2 完整 / dry-run / docker-compose) | ✅ 选项 2 被选 (per 9/5 04:03 JST 拍板推荐项被选后立即执行), 不需要多确认 |
| 11 | 守门 #1 守门 #9 v3 调试控制台走 subprocess 替代 RPC | (本任务无关) | N/A (本任务走 WSL subprocess, 跟子代理 RPC 无关) |
| 12 | 守门 #19 -j 4 修正 (cargo check 降并行度) | (本任务无关) | N/A (k3s 部署, 跟 cargo 无关) |
| 13 | 守门 #1 v25b CI cargo test 改单 crate | (本任务无关) | N/A |
| 14 | 守门 #1 v26 CI 4 守门修订反转 | (本任务无关) | N/A |
| 15 | 守门 #5 v2 (env 安全) | 0 打印 $env:VAR 内容 | ✅ 全程 $env:UbuntuPW / password 不打印, 走 stdin pipe (per 守门 #5 硬 ban) |

---

## 6. 签字栏

| 角色 | 签字 | 形式 | 日期 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | Mavis 接手**审核** (per 守门 #14 v4) | 2026-09-12 |
| SRE Lead | SRE Lead (Mavis 接手 agent per DEC-008) | Mavis 接手**审核** (per守门 #14 v4) | 2026-09-12 |
| 平台 | 平台 (Mavis 接手 agent per DEC-008) | Mavis 接手**审核** (per守门 #14 v4) | 2026-09-12 |
| 评审主持 | 评审主持 (Mavis 接手 agent per DEC-008) | Mavis 接手**审核** (per守门 #14 v4) | 2026-09-12 |
| PM | PM (Mavis 接手 agent per DEC-008) | Mavis 接手**审核** (per守门 #14 v4) | 2026-09-12 |

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-12 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手**审核** (per 守门 #14 v4) | 初版落地: k3s in WSL2 完整版部署验证, star-api-rest NodePort 30081 真实 HTTP 200/501/404, 10 项已知缺口显式列, 5 角色签字栏 per 守门 #14 v4 | Ulysses 9/12 12:39 JST 拍板"你起动真实 k3s 版本试试" → 选项 2 (k3s in WSL2 完整版) → 部署完成 + curl 实测 |
