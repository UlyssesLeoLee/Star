# Star Canvas services deployment

This guide covers the Star Canvas backend services and their Envoy ingress on an existing K3s cluster. It does not create virtual machines or clusters. For local or CI clusters, select one of the K3s contexts managed through Multipass; keep the current test inventory within the two-instance limit.

## Deployed inventory

The manifest is [`deploy/star-canvas-services-k3s.yaml`](../../deploy/star-canvas-services-k3s.yaml). It declares 15 Kubernetes resources, including:

| Resource | Count | Names |
| --- | ---: | --- |
| Canvas backend Deployments and Services | 3 each | `canvas-engine`, `domain-canvas`, `canvas-realtime` |
| Ingress Deployment and Service | 1 each | `envoy-edge` |
| HorizontalPodAutoscaler | 1 | `canvas-realtime-hpa` |
| NetworkPolicy | 2 | backend default-deny and Envoy-to-backend access |
| Namespace | 1 | `star-system` |
| ConfigMaps | 2 | backend logging configuration and Envoy routing |
| ServiceMonitor | 1 | `star-canvas-services-monitor` |

There are **three Canvas backend services and four Kubernetes Service objects** in this manifest; the fourth Service exposes Envoy. The `star-api-rest` service remains separate and is not included in these counts or this manifest. Its existing image build path is `deploy/k3s-local/Dockerfile`.

The manifest contains no unrelated workloads or service-account permissions. It keeps only the `ghcr-pull-secret` reference needed by private GHCR images; provision that secret through the approved secret manager before deployment. Workload pods do not receive Kubernetes API tokens. Do not place registry credentials in source control or shell history.

The Kargo Warehouse also needs a separate read credential for private GHCR image scanning in the `star` Project namespace, labeled `kargo.akuity.io/cred-type: image` and scoped to the matching repository URLs. This is separate from the Kubernetes `ghcr-pull-secret`; provision it through the approved secret manager. No credential is stored in this repository, and Kargo reconciliation has not been live-validated here.

## Images and publishing

The GitHub Actions workflow is [`publish-star-canvas-services.yml`](../../.github/workflows/publish-star-canvas-services.yml). It tests the three Canvas crates and publishes these existing Star GHCR image names:

- `ghcr.io/ulyssesleolee/star-canvas-engine`
- `ghcr.io/ulyssesleolee/star-domain-canvas`
- `ghcr.io/ulyssesleolee/star-canvas-realtime`

The workflow preserves its `main`/`dev`, version-tag, manual-dispatch, and image-tag behavior. Publishing reads `GHCR_TOKEN` only from the repository secret store; credentials are not accepted as dispatch inputs. Kargo watches these three image repositories and selects semantic-version tags. Branch and short-SHA image tags are not Kargo promotion candidates.

## Prerequisites

- Provide a kubeconfig file and choose a context from that file explicitly; scripts never rely on the current context or an implicit kubeconfig.
- Namespace `star-system` exists in that context.
- Secret `ghcr-pull-secret` exists in `star-system` and has read access to the three private GHCR images.
- The `ServiceMonitor` custom resource definition is installed. Without the Prometheus Operator CRD, applying the complete manifest fails; install the CRD first or use an environment-specific manifest that intentionally omits monitoring.
- `kubectl` can reach the selected cluster. No Docker Desktop installation is needed by this deployment path.

## Deploy

In PowerShell, set the context name from the kubeconfig inventory, then run the Star-only deployment script:

```powershell
$env:KUBECONFIG = 'C:\path\to\k3s.yaml'
$env:STAR_K3S_CONTEXT = '<your-k3s-context>'
./scripts/start-star-backend.ps1
```

The script accepts `-Context` and `-Kubeconfig` parameters as alternatives to `STAR_K3S_CONTEXT` and `KUBECONFIG`. It validates the selected kubeconfig file and context, cluster access, namespace, and the presence of `ghcr-pull-secret` without reading or printing secret contents. It applies the manifest and waits for the three Canvas backend Deployments and Envoy Deployment to become ready. It does not build images or create Multipass instances.

The Envoy Service exposes HTTP on NodePort `30080`; its admin port is not exposed through the Service. Routes are limited to the Canvas API, MCP, and Canvas WebSocket paths present in the manifest.

## Verify

The verification script checks the three Canvas backend health endpoints only. From a Bash environment with `kubectl`, the explicit kubeconfig file, and its selected context available:

```bash
export STAR_K3S_KUBECONFIG='/path/to/k3s.yaml'
export STAR_K3S_CONTEXT='<your-k3s-context>'
bash scripts/verify-star-services.sh
```

It creates and cleans up only the port-forward processes it started. It does not terminate unrelated `kubectl` processes. `star-api-rest` is a separate verification target and is deliberately not counted here.

For a deployment status snapshot, use the selected context explicitly:

```powershell
kubectl --context $env:STAR_K3S_CONTEXT -n star-system get deployments,services,hpa,networkpolicy
```

## Boundaries

- The publishing workflow does not publish an API image; no new API registry contract is introduced here.
- The K3s manifest does not provision virtual machines, clusters, databases, or message brokers.
- Multipass and K3s infrastructure lifecycle remains owned by the host infrastructure workflow. This deployment guide only targets an already provisioned cluster.
- Applying the manifest may create or update resources in `star-system`; it does not delete the namespace or clean up other cluster resources.
