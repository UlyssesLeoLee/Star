# tools/star-flash-mock/scripts — k3s 启动 + port-forward 守护

> **触发**: 2026-09-08 07:36 JST UAT 实证 "启动 3000 端口后黑了" 根因修复 (per `docs/briefs/k3s-star-mock-3000-restore-001.md`)

## 1. 启动顺序 (3 步)

| 步 | 动作 | 脚本 | 说明 |
|---|---|---|---|
| 1 | WSL Ubuntu 内启 k3s server | `start-k3s-backend.ps1` / `.bat` | 幂等: 已起就 skip, 没起走 `sudo nohup k3s server`. 需 sudo 免密 (5min 配额限制, 见 §3). |
| 2 | apply envoy deployment + service + 2 CM | (手动 `kubectl apply -f k3s/*.yaml`) | envoy 镜像 + 2 CM 一次性建好. 见 `k3s/envoy-deployment.yaml` (含 P0 热修的 mock-data CM) + `k3s/star-mock-service.yaml` |
| 3 | 3000 端口转发守护 | `k3s-portforward.service` (systemd user unit) | 装到 `~/.config/systemd/user/` + `systemctl --user enable --now`. 需 pod Running 才转 active (pod Pending 时 auto-restart 循环, 不会 3000 listen). |

## 2. 速查命令

```bash
# 1. 启 k3s (幂等, 1s 出报告)
powershell -NoProfile -ExecutionPolicy Bypass -File tools\star-flash-mock\scripts\start-k3s-backend.ps1

# 2. apply 全部资源 (含 2 CM)
wsl -d Ubuntu -- bash -lc 'KUBECONFIG=~/.kube/config kubectl apply -f /mnt/d/Star/.worktrees/feat-auto-20260908-204a1a91/tools/star-flash-mock/k3s/'

# 3. 装 port-forward 守护 (1 次性, 重启 wsl 自动拉起)
wsl -d Ubuntu -- bash -lc 'mkdir -p ~/.config/systemd/user; cp /mnt/d/Star/.worktrees/feat-auto-20260908-204a1a91/tools/star-flash-mock/scripts/k3s-portforward.service ~/.config/systemd/user/; XDG_RUNTIME_DIR=/run/user/$(id -u) systemctl --user daemon-reload; XDG_RUNTIME_DIR=/run/user/$(id -u) systemctl --user enable --now k3s-portforward.service; loginctl enable-linger leo19'

# 4. 验证 3000
curl -i http://localhost:3000
# 预期: status=200, body="not found" (envoy direct_response 404 配置, 表明 3000 通 + envoy 在听)
```

## 3. 已知卡点 (per 2026-09-08 07:36 JST 实证)

| # | 卡点 | 触发 | 缓解 |
|---|---|---|---|
| 1 | sudo NOPASSWD 5min 配额 | 2 次 `sudo -n nohup k3s server` 就会耗尽 | 等 5-15 min 自动重置, 或 `sudo -K` 清缓存. 不可绕过 (per AGENTS.md §4 守门 #5 禁 env 打印). |
| 2 | `crictl pull docker.io/envoyproxy/envoy:v1.32-latest` 卡超时 | WSL2 containerd 默认 docker.io, 国内网络差 | 改 `docker.m.daocloud.io/envoyproxy/envoy:v1.32-latest` mirror, 或预 pull 到 crictl. 需 sudo (受限 §3.1) |
| 3 | port-forward 在 wsl 临时 VTL 销毁时被 kill | PowerShell `wsl -d Ubuntu -- bash -lc "..."` 退出时回收所有子进程 | 必须走 systemd user service (本目录 `k3s-portforward.service`), 不要靠 `nohup`/`setsid`/`Start-Process` 手动后台 |
| 4 | `k3s crictl` 默认 root 拥有, leo19 读不到 | `/etc/rancher/k3s/k3s.yaml` 是 root 写的 | kubectl 走 `KUBECONFIG=~/.kube/config` (leo19 拥有), 不读系统 k3s.yaml. apply/diff 用此路径 |

## 4. 脚本来源 (per 守门 #1 禁回溯叙事 + §4 实证)

| 脚本 | 来源 | commit 引用 |
|---|---|---|
| `start-k3s-backend.ps1` | `C:\Users\leo19\AppData\Local\Temp\lf-backup-start-k3s-backend.ps1` (2026-09-08 05:34 JST, 7032 字节) | 本 commit 落档, 原 Temp 文件保留作历史形态 |
| `start-k3s-backend.bat` | `C:\Users\leo19\AppData\Local\Temp\lf-backup-start-k3s-backend.bat` (2026-09-08 05:32 JST, 1685 字节) | 同上, 原 Temp `orphan-start-k3s-backend.bat` 5:32 同份 |
| `k3s-portforward.service` | `C:\Users\leo19\AppData\Local\Temp\k3s-portforward.service` (2026-09-08 07:46 JST 落档, 375 字节) | 同上 |

## 5. 守门引用

- **守门 #1** (R-05 不 push): 本仓改动本地, 不推 origin
- **守门 #5** (env 安全): 8/27 11:06 JST hard ban, 全程无 env 打印
- **守门 #9** (子代理 dispatch): 本次无子代理, 全主上下文直跑
- **守门 #10** (代签): commit author = Ulysses (per 8/27 19:39 JST 授权)
- **守门 #12** (AI 协作文档): 报告 7 段结构, BAS 引用 git 实证 (本 README §4)
