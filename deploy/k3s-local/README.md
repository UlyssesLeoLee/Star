# star-api-rest — 本机 k3s 快速部署

跳过 `deploy/helm/star`（该 chart 目前是 stub，见 G4：只有 `NOTES.txt`/`_helpers.tpl`/`secret.yaml`，
没有 Deployment/Service 模板），直接用本目录的原生 k8s manifest 部署。

## 现状（2026-09-08 实证）

8 个 chart 里声明的 "服务" crate 中，目前只有 **`star-api-rest`** 有真实的
`fn main()` 可执行体（axum HTTP server，绑端口 + `axum::serve`）。其余
`context/sa/sse/webhook/cache/saga` 只有 lib crate，没有 `main.rs`；
`star-cli`/`star-ops` 是 CLI/运维工具，不适合当长驻 Deployment 跑。
所以本目录目前只覆盖 `star-api-rest` 一个服务。

## 前提

- 目标机器已装好 k3s（含内置 containerd + crictl）、docker（或兼容的镜像构建 CLI）、kubectl
- 运行部署脚本的用户对 `/usr/local/bin/k3s` 和 `/usr/local/bin/crictl` 有免密 sudo：
  ```
  # /etc/sudoers.d/k3s-local
  <user> ALL=(root) NOPASSWD: /usr/local/bin/k3s, /usr/local/bin/crictl
  ```
  没配免密的话脚本会在导入/校验步骤失败并打印手动命令，自己 `sudo` 跑一遍即可。
  `kubectl` 本身是 `/usr/local/bin/k3s` 的符号链接，默认读 root-only 的
  `/etc/rancher/k3s/k3s.yaml`，脚本统一走 `sudo -n /usr/local/bin/k3s kubectl`，
  不依赖 `KUBECONFIG` 环境变量或 `~/.kube/config` 是否存在。
- 在 k3s 所在的 Linux/WSL 环境里跑脚本，不是 Windows PowerShell。
- **若 k3s 跑在 WSL2 里**：WSL2 对每个发行版实例有一个独立于 `vmIdleTimeout`
  （整个轻量 VM 的空闲超时）的**实例级空闲超时**，默认约 15 秒——
  只要没有任何 `wsl.exe` 客户端连着该发行版（哪怕里面有 systemd 常驻服务/k3s 在跑），
  整个发行版用户态（PID 1 及以下，含 k3s）就会被销毁重建，和内核/轻量 VM 本身
  是否存活无关（`vmmemWSL` 进程、`journalctl --list-boots` 的 boot ID 可以全程不变）。
  表现为 `journalctl -u k3s` 里看似"崩溃重启"的循环，但其实是宿主 Windows 侧在拆卸
  整个 WSL 实例。2026-09-09 实测：`.wslconfig` 里加 `[general] instanceIdleTimeout=-1`
  在 WSL 2.4.13.0 上不被识别（`unrecognized configuration key`，`wsl --update`
  又因网络 403 拉不到新版本），所以这台机器上目前没有配置层面的根治办法，只能靠
  **保持至少一个 `wsl.exe` 客户端常驻连接**该发行版（例如后台跑
  `wsl -d <Distro> -- sleep infinity`，或 Windows 登录时用计划任务跑
  `wsl.exe --exec dbus-launch true`）来防止发行版被拆掉。部署/验证前先确认有常驻连接，
  否则构建/`kubectl apply` 中途可能因为发行版被拆而失败。

## 一键部署

```bash
./deploy/k3s-local/build-and-deploy.sh
```

脚本做的事（对应 `crates/star-api-rest/src/main.rs` 的行为，22 路由 stub 目前返 501）：

1. `docker build` — 用本目录 `Dockerfile`，context 为仓库根目录（需要根目录的
   `Cargo.toml`/`Cargo.lock`/`rust-toolchain.toml`/`crates/`）。
2. **导入到 k3s containerd 的 `k8s.io` namespace**（不是默认的 `default` namespace）——
   `docker save | k3s ctr -n k8s.io images import -`。这是本部署能跑起来的关键坑：
   kubelet 只认 `k8s.io` namespace 里的镜像，落错 namespace 会导致
   `imagePullPolicy: Never` 的 Pod 卡 `ErrImageNeverPull`。
3. 用 `crictl images`（kubelet 实际看到的视角，不是 `ctr images ls`）校验镜像可见，
   看不到就直接失败退出，不会往下部署一个必挂的 Pod。
4. `kubectl apply -f star-api-rest-deploy.yaml` + `rollout status` 等 Pod 就绪。

## 手动分步（脚本失败时对照排查）

```bash
docker build -t star-api-rest:local -f deploy/k3s-local/Dockerfile .
docker save star-api-rest:local | sudo /usr/local/bin/k3s ctr -n k8s.io images import -
sudo /usr/local/bin/crictl images | grep star-api-rest
kubectl apply -f deploy/k3s-local/star-api-rest-deploy.yaml
kubectl -n star-system get pods -o wide
```

## 已知限制

- 其余 7 个 chart 服务未部署（无 `main.rs` 或不适合常驻）。
- `deploy/helm/star` chart 仍是 stub，未使用。

## 已解决的历史问题（存档，供排障参考）

- **2026-09-08 曾记录**「`kubectl logs`/`exec`/`port-forward` 对所有 Pod 都返回
  `pods/log ... not found`，疑似 apiserver↔kubelet 子资源代理问题」——2026-09-09
  查明并非独立集群 bug，而是上面「若 k3s 跑在 WSL2 里」那条 WSL 实例空闲拆卸问题的
  下游症状：kubelet↔apiserver 的长连接被反复中断的发行版重建打断。保持
  `wsl.exe` 常驻连接后，`kubectl logs`/`exec`/`port-forward` 均恢复正常，已用
  `port-forward` + curl 实测 `star-api-rest` 的 `/api/v1/health`（200）和
  `/api/v1/work-items`（501，符合 22 路由 stub 的预期行为）。
