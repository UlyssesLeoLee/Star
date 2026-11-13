# star-api-rest K3s deployment status

> **Status: retired legacy deployment material. Do not use this directory as an operator runbook.**

The previous instructions in this directory depended on a local container build and a Linux/WSL-side cluster workflow. They are not the supported Star deployment path. The retained manifest and build files are not evidence of a supported Star API image or deployment pipeline.

## Supported application deployment

The current documented deployment path covers the Star Canvas services only:

- Guide: [`docs/deployment/star-canvas-services-deploy.md`](../../docs/deployment/star-canvas-services-deploy.md)
- Entry point: [`scripts/start-star-backend.ps1`](../../scripts/start-star-backend.ps1)

It requires an already provisioned and reachable K3s context, with the required namespace, image pull secret, and monitoring CRD. It applies the Canvas manifest; it does not create, start, or recover a VM or cluster.

The Multipass provider script currently supports a read-only preflight and a guarded provisioning flow. Provisioning creates a new guest; it is not a safe substitute for starting or resuming an existing guest. The provider enforces a maximum of two registered test instances, counts stopped and deleted records, serializes provisioning with a cross-process lock, fails closed on unknown inventory, and never purges automatically.

## star-api-rest status

No supported Star API image build, publish, or deployment flow is provided by this legacy directory. Do not run its retained `build-and-deploy.sh` entry point. Before this service can be deployed, its image identity, build/publish contract, and existing-cluster deployment workflow must be reviewed and implemented as a separate Star delivery change.
