# GitOps 骨架 (Argo CD + Kargo) — dev → staging

> **选型依据**: [ADR-0054](../../../docs/architecture/2026-08-26-upgrade/adr/0054-cd-delivery-stack-argocd-kargo-rollouts.md)
> **落地日期**: 2026-10-04
> **拍板**: per `ask_72c00a6f28daadd5538fa004` scope_opt1 + env_opt1（本机 k3s 单集群，dev → staging 两级）
> **许可**: Argo CD v3.5.3 / Kargo v1.12.1 / Argo Rollouts v1.10.0 均 Apache-2.0（实测 2026-10-04，见 ADR-0054 §2.1）

## 目录结构

```
deploy/gitops/
├── argocd/argocd-applications.yaml   # AppProject + dev/staging 两个 Application
└── kargo/kargo-dev-staging.yaml      # Project + Warehouse + dev/staging 两个 Stage
```

**本目录不存放实际业务 manifest**。既有清单保持原位，由 Argo CD 引用：

| 路径 | 类型 | 用途 |
|---|---|---|
| `deploy/k3s-local/` | kustomize | dev 环境（含 envoy 独立 deployment、bff） |
| `deploy/helm/star/` | helm chart | staging 环境 |

## 职责边界

```
     CI (GitHub Actions)
            │ 构建 / 签名 / 推送镜像
            ▼
   Kargo Warehouse ──► Freight (不可变版本单元)
            │
            ▼
   Stage: dev  ──git-write──► deploy/dev 分支
            │ (自动, 5m soak)
            ▼
   Stage: staging ──git-write──► deploy/staging 分支 + PR
            │ (人工合并)
            ▼
   Argo CD reconcile ──► k3s
```

- **Kargo 只管晋级**（哪个版本进哪个环境、谁批的），不碰集群状态
- **Argo CD 只管调谐**（Git 声明 → 集群实际），不做审批
- **staging 刻意不设 `automated`**，否则会绕过 Kargo 审批，使晋级链路形同虚设

## ⚠️ 未实跑验证声明

per 守门 #11「缺标比错标安全」——本目录为**骨架**，以下均**未经实跑验证**：

| 项 | 状态 | 首次落地须做 |
|---|---|---|
| Kargo CRD 字段（`kargo.akuity.io/v1alpha1`） | ❌ 未实跑 | `kubectl apply --dry-run=server -f kargo/` |
| Argo CD `AppProject` / `Application` 字段 | ❌ 未实跑 | `argocd app diff star-dev` |
| Warehouse 镜像坐标 `ghcr.io/ulyssesleolee/star` | ⚠️ 推测值 | 与 CI 实际推送目标对齐后再启用订阅 |
| `git-write` 目标分支 `deploy/dev` / `deploy/staging` | ⚠️ 推测值 | 分支需先创建 |
| `path: deploy/k3s-local` 的版本写入位置 | ⚠️ 待定 | Kargo 写的是镜像 tag 覆写文件，需确认落点 |

YAML **语法**已通过 `python -c "yaml.safe_load_all(...)"` 校验（2026-10-04 实测，三文件全 OK）。

## 刻意未做的事

1. **未开 `prune: true`** — Git 中误删 manifest 目录会直接删除集群资源。开启前须先在 staging 验证 `PruneLast=true` + `allowEmpty=false` 的实际行为。
2. **未引入 `argocd-image-updater`** — 该组件已迁至 `argoproj-labs`（非 argoproj 主 org），成熟度低于主 org 组件；且 ArtifactHub 扫描报告称其 v1.2.1 存在大量漏洞告警，需先用 SBOM 工具复现核实（见 ADR-0054 §6.2 待办 #2）。
3. **未配置 Argo Rollouts** — 渐进式发布门禁属第二阶段（staging → prod 扩展时接入），当前 `star-api-rest` 等仍为普通 Deployment。
4. **未改 CI 侧** — 现有 11 个 workflow 保持不变，避免同时引入额外变量。

## 桌面发布链不在本目录

Tauri 桌面产物不由 k3s 承载，走**独立发布链**：`tauri signer`（minisign）→ updater channel。**不经过 Kargo / Argo CD**。详见 ADR-0054 §5.1。
