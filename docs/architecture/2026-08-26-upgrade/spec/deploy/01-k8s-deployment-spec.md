# Spec-01: Kubernetes Deployment 规范

> **状态**：Draft v0.1
> **日期**：2026-08-28
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**：per ADR-0037 §8 / 2026-08-27 21:59 JST 用户授权 / Phase I production rollout

## §1 目的
定义 Star 在 K8s 集群上的完整部署规范：8 个核心服务（star-cli / star-mcp / star-context / star-sa / star-sse / star-webhook / star-cache / star-saga）的 Deployment + Service + Ingress + ConfigMap + Secret。Phase I 是 MVP v1 → production 最后一公里。

## §2 8 服务清单
per `Cargo.toml [workspace] members` (commit 9723bae base)：

| 服务 | 镜像 (ghcr.io/ulysses-lee-lee) | 端口 | 副本数 | 启动命令 |
|------|-------------------------------|------|--------|----------|
| star-cli | /star-cli:latest | - | 1 (Job/Cron) | star --help |
| star-mcp | /star-mcp:latest | 8080 | 3 | star-mcp serve --transport http |
| star-context | /star-context:latest | 8081 | 3 | star-context serve |
| star-sa | /star-sa:latest | 8082 | 2 | star-sa serve |
| star-sse | /star-sse:latest | 8083 | 3 | star-sse serve |
| star-webhook | /star-webhook:latest | 8084 | 2 | star-webhook serve |
| star-cache | /star-cache:latest | 8085 | 2 | star-cache serve |
| star-saga | /star-saga:latest | 8086 | 2 | star-saga serve |

## §3 命名空间 + RBAC
- namespace: `star-system` (control plane) / `star-tenant-{id}` (per-tenant, 5 域业务)
- ServiceAccount: `star-mcp` / `star-sa` / `star-saga` 等 8 个 1:1 映射
- Role:
  - `star-cache-readonly` — 只读 (cache 层) → spec/cache/01 §4
  - `star-mcp-readwrite` — 读写 (MCP) → spec/agents/02 §3
  - `star-sa-cluster` — 集群级 (Git Provider)
- RoleBinding: ServiceAccount ↔ Role

## §4 Deployment 模板
per 服务 1 个 Deployment（star-mcp 示例）：

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: star-mcp
  namespace: star-system
  labels: { app: star-mcp, version: v0.1.0 }
spec:
  replicas: 3
  selector:
    matchLabels: { app: star-mcp }
  template:
    metadata:
      labels: { app: star-mcp }
    spec:
      serviceAccountName: star-mcp
      containers:
        - name: star-mcp
          image: ghcr.io/ulysses-lee-lee/star-mcp:latest
          imagePullPolicy: IfNotPresent
          ports: [{ containerPort: 8080, name: http }]
          envFrom:
            - configMapRef: { name: star-mcp-config }
            - secretRef: { name: star-mcp-secrets }
          resources:
            requests: { cpu: 100m, memory: 256Mi }
            limits:   { cpu: 500m, memory: 512Mi }
          livenessProbe:
            httpGet: { path: /healthz, port: 8080 }
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            httpGet: { path: /readyz, port: 8080 }
            initialDelaySeconds: 5
            periodSeconds: 5
```

## §5 Service + Ingress
- ClusterIP Service（默认 8 服务各 1 个）
- Ingress path:
  - `/mcp` → star-mcp:8080
  - `/sse` → star-sse:8083 (Server-Sent Events)
  - `/webhook` → star-webhook:8084
  - `/context` → star-context:8081
- TLS: cert-manager + Let's Encrypt
- Rate limit: nginx ingress annotations (`100rps` per service)

## §6 ConfigMap + Secret
- **ConfigMap** (公开配置):
  - `log-level: info` (生产默认)
  - `replica-overrides: {}` (per-tenant 副本覆盖)
  - `feature-flags: { mcp_resources: true, sse_replay: true, ... }`
- **Secret** (凭据, per 8/27 11:06 JST secret 安全):
  - `GITHUB_TOKEN` / `GITLAB_TOKEN` / `BITBUCKET_APP_PASSWORD` / `GITEA_TOKEN` (per spec/vcs/05 §4)
  - `WEBHOOK_SECRET` (per spec/services/03 §2)
  - `REDIS_URL` (per spec/cache/01 §5, Phase I+ 启用)
  - **所有凭据从 K8s Secret 注入，不进 ConfigMap，不进镜像层**

## §7 HPA + PDB
- **HPA** (HorizontalPodAutoscaler):
  - CPU > 70% / 内存 > 80% 触发扩缩
  - min 2 / max 10 副本 (mcp/sse 高负载)
  - min 1 / max 5 副本 (sa/webhook/cache/saga 中负载)
- **PDB** (PodDisruptionBudget):
  - `minAvailable: 1` (mcp/sse)
  - `maxUnavailable: 0` (单副本服务)

## §8 已知缺口
1. 真实 image registry (ghcr.io vs 自建 Harbor) 选型
2. 镜像扫描 (Trivy / Snyk) 接入 (Phase I+)
3. 5 业务域 Lead (Player/Economy/Match/Social/Admin) 真实身份签字 (per 8/21 JST 5 域独立 Lead)
4. 多 region 部署 (Phase I+)
5. 灰度发布 (Argo Rollouts / Flagger)
6. Disaster Recovery (Velero 备份)
7. 22 domain 真实数据源部署 (依赖 Phase H 真实数据完成)

## §9 引用文档
- adr/0037-phase-h-architecture.md
- adr/0038-phase-i-architecture.md
- spec/observability/01-monitoring-spec.md (Phase I)
- spec/sla/01-sla-spec.md (Phase I)
- spec/vcs/05-real-providers-spec.md (Phase F)
- spec/services/03-webhook-adapter-spec.md (Phase E)
- AGENTS.md

## §10 修订历史
| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：8 服务 + 命名空间 + Deployment 模板 + Service/Ingress + HPA + 7 已知缺口 | ADR-0037 §8 Phase I |
