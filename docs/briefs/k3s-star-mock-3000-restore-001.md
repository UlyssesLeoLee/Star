# Brief: k3s star-mock 3000 端口恢复 commit (2026-09-08 JST)

> **目的**: 把"启动 3000 端口后黑了"根因 + 4 个补救动作合并为 1 commit,免 sudo 期间仍能落地

**触发**: 2026-09-08 07:36 JST Ulysses 反馈 "用 playwright 操作进行 UAT 测试,现在启动 3000 端口后黑了,存在显示问题"

**根因 (per 实证)**:
1. `envoyproxy/envoy:v1.32-latest` 本地无镜像,docker.io 拉超时 → pod Pending 4m
2. `envoy-deployment.yaml` 引用 `configMap: star-mock-mock-data` 但 yaml 内未声明此 CM → 隐式依赖缺失
3. WSL 内 `kubectl port-forward` 在 wsl 临时 VTL 销毁时被杀 → 进程不能常驻 → 需 systemd user service
4. sudo NOPASSWD 配额 5min,2 次 `sudo -n nohup k3s server` 耗尽 → crictl/ctr 拉镜像阻塞

**已落档成果 (本 session)**:
- k3s server running (WSL Ubuntu, v1.36.3, node Ready, 6443 LISTEN)
- namespace `star-mock` 创建
- deployment `star-mock-envoy` 2/2 (Pending, 镜像未到)
- service `star-mock-envoy` / `star-mock` ClusterIP
- configmap `star-mock-envoy-config` (envoy 路由) + `star-mock-mock-data` (空 CM 占位,补缺)
- `~/.config/systemd/user/k3s-portforward.service` 已建 (auto-restart 循环,等 pod Ready 即可生效)

**scope 范围 (本 commit)**:
- ✅ P0 热修 `tools/star-flash-mock/k3s/envoy-deployment.yaml` — 显式声明 `star-mock-mock-data` CM
- ✅ 收 3 份启动脚本进仓:
  - `tools/star-flash-mock/scripts/start-k3s-backend.ps1` (从 Temp lf-backup-* 落档, per 9/8 05:32 JST 拍板)
  - `tools/star-flash-mock/scripts/start-k3s-backend.bat` (pwsh 包装)
  - `tools/star-flash-mock/scripts/k3s-portforward.service` (systemd user unit, per 9/8 07:46 JST 落档)
- ✅ 加 `tools/star-flash-mock/scripts/README.md` (启动顺序 + sudo 注意 + 配额重置)
- ✅ 落报告 `docs/reports/PHASE-K3S-STAR-MOCK-IMPL-REPORT.md` (7 段结构 per 守门 #3)
- ✅ 落 `docs/automation-design.md` §4.13 (本次 commit 任务卡) + `scripts/automation/registry.md` 索引行 (per 守门 #12 v21)

**out of scope (等 sudo 重置,下 session 续)**:
- ⏸ crictl pull `envoyproxy/envoy:v1.32-latest` (or docker.m.daocloud.io mirror)
- ⏸ 重 apply deployment + 等 2/2 pod Ready
- ⏸ port-forward systemd service 自然转 active (从 auto-restart loop 切到 active running)
- ⏸ Playwright 验证 localhost:3000 渲染

**期望交付**:
- 1 commit, author = Ulysses (per 守门 #10 + 8/27 19:39 JST 授权)
- commit message 引用: 守门 #1 v19 (Python 化) + #9 v20 (brief 落档) + #10 (代签) + #12 v21 (docs 同步) + #13 (W/T/M 强制分类)
- 受影响文件:
  - `tools/star-flash-mock/k3s/envoy-deployment.yaml` (加 CM 声明)
  - `tools/star-flash-mock/scripts/start-k3s-backend.ps1` (新增)
  - `tools/star-flash-mock/scripts/start-k3s-backend.bat` (新增)
  - `tools/star-flash-mock/scripts/k3s-portforward.service` (新增)
  - `tools/star-flash-mock/scripts/README.md` (新增)
  - `docs/reports/PHASE-K3S-STAR-MOCK-IMPL-REPORT.md` (新增)
  - `docs/automation-design.md` (§4.13 新增 1 节)
  - `scripts/automation/registry.md` (1 索引行)

**验收标准 (per Ulysses 拍板: opt4 合 1 commit, 2026-09-08 07:51 JST ask_user)**:
- [ ] 8 份文件改动 + 新增全在 commit 内
- [ ] author=Ulysses 落档成功
- [ ] commit message 引用守门 + brief 路径 + automation-design §4.13
- [ ] 不动 main 分支 (当前在 feat-auto-20260908-204a1a91 wt)
- [ ] 不 apply 不推 (per 守门 #1 不 push)
