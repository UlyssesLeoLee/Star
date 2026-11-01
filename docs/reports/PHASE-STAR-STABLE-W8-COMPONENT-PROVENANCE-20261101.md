# PHASE-STAR-STABLE-W8-COMPONENT-PROVENANCE-20261101

> Status: 🟡 W8 evidence review complete; W8 acceptance and stable release remain open.
> Version: 0.2 | Snapshot date: 2026-11-01
> Scope: first-party provenance, current local Multipass inventory, and static deployment/add-on trace. No VM lifecycle operation was performed.

## §0 Purpose

Record current upstream and local evidence for the Multipass + K3s test infrastructure, identify source-to-runtime drift, and define what remains before W8 can be accepted. This is a bounded evidence report: it does not change deployment behavior or certify the complete release payload.

**Release chronology correction (2026-11-01):** Canonical's official GitHub release page marks Multipass **v1.17.0** as Latest and dates it **2026-10-19**. Canonical's 1.17.0 notes make HCS the new default Windows driver and describe a migration path from Hyper-V; this lane verified neither an upgrade nor a VM migration. The local host was rechecked read-only and remains **1.16.4+win / Hyper-V**. Sources: [official release page](https://github.com/canonical/multipass/releases), [Canonical release notes](https://canonical.com/multipass/docs/latest/reference/release-notes/).

## §1 Change / evidence matrix

| Area | Evidence at 2026-11-01 | Assessment |
|---|---|---|
| Multipass stable and maintenance | Canonical's GitHub release page marks **v1.17.0 Latest**, released **2026-10-19**. **1.16.5-rc1** and **1.17.0-rc1** remain prereleases in the release list. The GA notes make HCS the new default Windows driver, say it supports all Windows editions, and describe a Hyper-V migration path. Canonical's release page provides GitHub Issues, Discourse, and Matrix support channels. Sources: [releases](https://github.com/canonical/multipass/releases), [v1.17.0 release notes](https://github.com/canonical/multipass/releases/tag/v1.17.0), [Canonical release notes](https://canonical.com/multipass/docs/latest/reference/release-notes/). | Recent GA and prerelease activity plus named public support channels are evidence of active maintenance; no star-count inference used. HCS is now the GA Windows driver direction, not a prerelease-only feature. The official release notes mention migration support, but this lane did not verify the path. |
| Local Multipass host/inventory (read-only) | Rechecked during this correction: client/daemon **1.16.4+win**; `local.driver=hyperv`. Inventory: **1** registered instance, `informed-impala`, **Stopped**, Ubuntu **26.04 LTS**, no IPv4; remaining capacity under the two-instance cap: **1**. | The checked-out launcher requires Multipass >=1.17.0 and `local.driver=hcs`; the repo README still names **1.17.0-rc1/HCS**. Local host remains below the target version and on the prior driver. Do not assume or claim an upgrade/migration succeeded. The guest's K3s and add-on state remains unknown. |
| Guest image | The launcher explicitly uses image `24.04`; current inventory reports an Ubuntu 26.04 LTS guest. Canonical's Multipass image documentation lists `26.04`/`resolute` as a current LTS image and describes the `lts` alias as dynamic: [image list](https://canonical.com/multipass/docs/latest/reference/command-line-interface/images/). | Image identity is not consistent across the existing guest and new-launch path. Pinning the series alone is not immutable image provenance; record the selected image hash/build identifier as part of W8. K3s official requirements say modern Linux/Ubuntu is expected, but do not establish this exact Ubuntu 26.04 + K3s combination as tested: [requirements](https://docs.k3s.io/installation/requirements). |
| K3s upstream stable | Official release notes identify **v1.37.1+k3s1**, released **2026-09-30**, as latest stable. The upstream table lists Kubernetes 1.37.1, Kine 0.17.1, SQLite 3.53.4, etcd 3.7.1-k3s3, containerd 2.3.4-k3s1, runc 1.4.2, Flannel 0.28.4, metrics-server 0.9.0, Traefik 3.7.13, CoreDNS 1.14.7, Helm-controller 0.17.9, and local-path-provisioner 0.0.37. Sources: [official v1.37 release notes](https://docs.k3s.io/release-notes/v1.37.X), [release tag](https://github.com/k3s-io/k3s/releases/tag/v1.37.1%2Bk3s1). | K3s release cadence and September component updates provide recent maintenance evidence. Windows is not a native K3s host; Linux guests are the intended runtime here. K3s support channels are the official Issues/Discussions and Rancher Slack K3s channel: [FAQ](https://docs.k3s.io/faq). |
| K3s version installed by Star bootstrap | `deploy/multipass/cloud-init-k3s-gitops.yaml` invokes `https://get.k3s.io` without `INSTALL_K3S_VERSION`; the separate fix cloud-init also does not set a K3s version. K3s uses a native Linux service/systemd path in the guest. | Version is floating at install time. The inventory VM is stopped and no guest version was queried, so do not claim it runs v1.37.1. Pin installer version and verify downloaded installer/binary digests before acceptance. |
| Packaged K3s components | Star disables Traefik and ServiceLB in the install arguments, but does not pin/override K3s's other packaged components. The upstream version matrix provides candidate versions only; they are not evidence of the local guest's actual component versions. | Record post-install `k3s --version`, node/runtime/CNI/CSI state, packaged component image digests and storage provisioner. The current deployment has no CSI add-on pinned separately; packaged local-path storage remains part of the K3s closure. |
| GitOps add-ons | Repo declares cert-manager **v1.18.2**, Argo Rollouts **v1.10.0**, Argo CD **v3.5.3**, and Kargo **v1.12.1**. License texts at those upstream tags are available: [cert-manager](https://github.com/cert-manager/cert-manager/blob/v1.18.2/LICENSE), [Argo CD](https://github.com/argoproj/argo-cd/blob/v3.5.3/LICENSE), [Argo Rollouts](https://github.com/argoproj/argo-rollouts/blob/v1.10.0/LICENSE), [Kargo](https://github.com/akuity/kargo/blob/v1.12.1/LICENSE). `cloud-init-fix-gitops.yaml` corrects Rollouts namespace/RBAC and installs Kargo from its versioned OCI chart. | Versions are declared, but downloaded manifests/chart and their transitive images are not pinned by digest or included in a generated end-to-end SBOM. The launcher passes only `cloud-init-k3s-gitops.yaml`; the separate fix file and `install-kargo.sh` are not automatically composed by that launcher. The selected cloud-init still contains the known Rollouts namespace and Kargo `install.yaml` defects described in the fix file, so the correction is not part of the normal launch path. The separate fix cloud-init fetches Helm's mutable `main` installer. |
| Star workloads and monitoring | `deploy/star-canvas-services-k3s.yaml` is a non-deployable render template until three Canvas image digest placeholders are replaced; Envoy in this manifest has a digest. The manifest defines a `ServiceMonitor`, while the Multipass bootstrap does not install Prometheus Operator/its CRD. `deploy/helm/star` identifies itself as a deprecated placeholder with no active workload templates. | The template and its monitoring dependency are not proof of a running Star Canvas deployment. The separate `deploy/k3s-local` path is for `star-api-rest`; its Dockerfile copies the broad `crates` workspace. That context still needs classification/filtering for the Star-only artifact policy and is tracked outside this lane. |
| Commercial-use / redistribution evidence | Multipass **v1.17.0** declares **GPL-3.0** ([tag license](https://github.com/canonical/multipass/blob/v1.17.0/LICENSE)); the locally installed v1.16.4 tag carries the same license ([tag license](https://github.com/canonical/multipass/blob/v1.16.4/LICENSE)). K3s v1.37.1 declares **Apache-2.0** ([tag license](https://github.com/k3s-io/k3s/blob/v1.37.1%2Bk3s1/LICENSE)). The declared add-on project licenses above are Apache-2.0. cert-manager's generated [license inventory](https://github.com/cert-manager/cert-manager/blob/v1.18.2/LICENSES) includes additional third-party licenses, including MIT, BSD, and MPL-2.0. Apache's official redistribution terms require preserving the license, notices, and change notices where applicable: [Apache-2.0 text](https://www.apache.org/licenses/LICENSE-2.0). | These licenses do not bar commercial use. Redistribution duties depend on the actual packaging/linking/modified-file form. The headline project licenses are not a complete release clearance: guest OS packages, all K3s/add-on image contents, charts, build tools, and transitive dependency notices still need an artifact-level SBOM/license review. No component is rejected merely for being copyleft. |

## §2 Verification summary

| Check | Result |
|---|---|
| Worktree branch and baseline | `codex/star-stable-w8-component-provenance-20261101` at `2a57fa0d329be830735d6657677236e195673362` |
| Multipass version / selected driver | Read-only: `1.16.4+win` / `hyperv` |
| Multipass JSON inventory | Read-only: 1 record; `informed-impala`, Stopped, Ubuntu 26.04 LTS; remaining cap 1/2 |
| Upstream version/license/support review | Official Canonical release page and docs plus tagged license/official docs linked in §1; as-of 2026-11-01, GitHub marks v1.17.0 Latest (2026-10-19). Canonical's release-notes table contains a `September XX` placeholder and shows last updated 2026-10-16, so the GitHub GA release entry is the date evidence used here. |
| VM/K3s/add-on runtime verification | Not run. No instance start/stop/create/delete/purge, no guest command, and no cluster connection was attempted. |
| Static deployment review | Read selected launcher/cloud-init, fix cloud-init, add-on installer, current Canvas K3s manifest, deprecated Helm placeholder, and local API deployment Dockerfile. |
| Document/JSONL parse and `git diff --check` | Pass: JSONL parsed as 1 record with required audit keys; report contains all 8 required sections; `git diff --check` exit 0. |

## §3 Known gaps and W8 blockers

1. **Host driver mismatch:** current official stable is Multipass 1.17.0 GA, while local is 1.16.4/Hyper-V and the launcher requires >=1.17.0/HCS. The repo README still names the old 1.17.0-rc1 snapshot. Select and verify the Windows host path before executing the launcher; do not silently lower its preflight or assume the documented migration path is verified here.
2. **Guest image identity:** one existing Ubuntu 26.04 guest versus launcher-pinned Ubuntu 24.04. Decide the supported series and record exact image hash/build provenance.
3. **K3s reproducibility:** bootstrap installs a moving K3s channel and mutable Helm installer. Pin versions and verify installer, binary, chart, manifest, and image digests.
4. **Bootstrap composition:** the standard launcher uses the cloud-init file with known Rollouts/Kargo defects and does not automatically apply the separate fix file. Reconcile to one tested bootstrap path before the next VM run.
5. **Cluster/add-on evidence:** stopped guest means actual K3s version, packaged Flannel/metrics-server/local-path-provisioner, add-on versions, CRDs, readiness, network, and persistent storage are unknown. No two-instance behavior or capacity isolation has been proven.
6. **Manifest readiness:** Canvas digests are placeholders, and the ServiceMonitor CRD provider is not part of the current Multipass bootstrap. A future test must resolve both explicitly.
7. **Commercial distribution closure:** current project-level license facts are positive for commercial use, but no SBOM/license scan covers the complete guest, image, chart, installer, and packaged artifact closure. The broad `deploy/k3s-local` build context also needs a separate Star-only payload gate.
8. **Capacity:** current inventory consumes one of two allowed records. Only one slot remains; the existing guest's owner, contents, and reusability were not inspected by starting it.

W8 remains open until a selected host/guest/K3s profile is reproducible, the corrected bootstrap succeeds, artifact-level payload/license evidence exists, and the approved two-instance acceptance matrix passes with actual API/readiness, isolation, CNI/CSI/storage, upgrade/recovery, and bounded-resource results. This report does not close the stable release.

## §4 Sub-agent failure and handoff

No child agent was dispatched in this bounded evidence lane. Handoff to the coordinator:

- Treat the local version/driver mismatch and launcher/fix-file composition as blocking evidence, not a recommendation to alter or purge the current guest.
- Coordinate a separate owner-approved execution lane for the remaining one-instance capacity and the two-instance acceptance. Preserve the stopped instance until its ownership/reuse is resolved.
- Keep the artifact-context finding and release SBOM/license closure in their separately owned lanes; this report did not modify those files.

## §5 Guard rules

| # | Guard | Evidence/status |
|---:|---|---|
| 1 | Do not treat `Stopped` as deleted or free capacity. | One record is counted; no deletion or purge. |
| 2 | Do not exceed the test cap of two Multipass inventory records. | Current inventory is 1/2. |
| 3 | Do not use Docker Desktop as a prerequisite; K3s runs in Linux guest VMs. | Scope and repo bootstrap use Multipass/Linux guest. |
| 4 | Distinguish upstream release from installed runtime version. | Multipass upstream stable is v1.17.0 GA; the local host is 1.16.4+win/Hyper-V. K3s local version is unknown because the guest is stopped. |
| 5 | Pin versions and digests for reproducible installs and artifacts. | K3s and Helm installer are currently mutable; blocker recorded. |
| 6 | Commercial use is allowed where license terms permit; honor distribution obligations. | GPL-3.0/Apache-2.0 facts recorded; full closure remains open. |
| 7 | Do not claim component/image verification from tags or templates alone. | Digests, SBOM, and runtime inspection remain open. |
| 8 | Keep product payload Star-only. | Broad local build context is flagged for separate filtering/audit. |
| 9 | Leave active D2 and other lanes/worktrees untouched. | Only this registered report is added; no unrelated path changed. |
| 10 | Do not modify `dev`/`main`, push, or operate Multipass instances in this lane. | No branch integration, push, or VM lifecycle operation performed. |

## §6 Review sign-off

| Role | Reviewer | Decision |
|---|---|---|
| Architecture | Mavis 接手审核 | 🟡 Source-to-runtime mismatches and acceptance boundary recorded; W8 remains open. |
| SRE Lead | Mavis 接手审核 | 🟡 Two-instance, readiness, recovery, and resource behavior not run. |
| Platform | Mavis 接手审核 | 🟡 Multipass/K3s versions, guest, add-ons, and install mutability evidence recorded. |
| Review Facilitator | Mavis 接手审核 | 🟢 Lane stayed within registered files and made no VM or branch changes. |
| PM | Mavis 接手审核 | 🟡 No stable release approval; distribution closure and user-visible acceptance remain open. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| 0.1 | 2026-11-01 | Mavis review | Record official upstream/version/license/support evidence, current read-only inventory, repo drift, and remaining W8 acceptance gates. | STAR stable release W8 component provenance brief. |
| 0.2 | 2026-11-01 | Mavis review | Correct Multipass chronology: v1.17.0 is GA/latest since 2026-10-19 with HCS as Windows default; retain the verified local 1.16.4+win/Hyper-V mismatch and explicitly leave upgrade/migration unverified. Add official source and date-basis note. | STAR-STABLE-W8-MULTIPASS-VERSION-CORRECTION-20261101 follow-up. |
