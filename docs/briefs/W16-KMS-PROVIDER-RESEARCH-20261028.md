# Brief: W16-KMS-PROVIDER-RESEARCH-20261028

**Agent**: explorer
**Phase**: W16-KMS-PROVIDER-RESEARCH-20261028
**Created**: 2026-10-28 04:12:55

---

## Assignment

Research current production KMS/envelope-encryption options for W16, emphasizing open-source options usable commercially without field-of-use/seat/usage restrictions, an active community, and a replaceable Rust adapter. Write only `docs/reviews/W16-KMS-PROVIDER-RESEARCH-20261028.md` plus this dispatcher-created brief and its dispatcher audit log. Do not edit application source, other documentation, or another branch. Do not commit, rebase, merge or push.

## Context and constraints

- Star must support both the Vercel deployment and local/CI Multipass-managed K3s guests; the test phase may use at most two Multipass instances. Do not assume Docker Desktop.
- A provider must support authenticated envelope encryption/wrapped-key operations, durable key identity across process restarts, key rotation, auditability, backup/recovery and bounded resource use. It must be reachable from Vercel serverless and from the Rust service in K3s without exposing host daemons or arbitrary shell.
- Keep owner-local transactions, Run/Project authorization, data lifecycle, idempotency and business facts in Star. Provider integration is an adapter; avoid coupling Star business contracts to a vendor's wire concepts.
- Rust API/client license and the deployed server product license are separate. Do not reject copyleft solely because it is copyleft; evaluate actual commercial use, modification, linkage, deployment, distribution and corresponding-source obligations. Exclude non-commercial, field-of-use, source-available, or paid-seat/usage restrictions on the required core capability.
- Use current primary sources only for technical and license claims: official documentation, canonical source repository license/README/releases and official Rust crate/repo. Record publication/release dates and retrieve current evidence; don't rely on memory.
- The source agent must not inspect secrets/environment variables. This is research only; no implementation.

## Scope

Compare a small set of viable approaches, including a self-hostable OSS key service (for example OpenBao Transit, if verified), a cloud KMS adapter where useful, and any stronger candidates found. For each, cover:
1. canonical license and exactly what component it applies to;
2. community activity evidence over the last 12 months and public maintenance channel;
3. envelope encryption API, authentication/RBAC, key versions/rotation, audit, recovery/backup, HA/state requirements;
4. Vercel network/auth constraints and a two-guest Multipass/K3s deployment model;
5. Rust client/API maturity, license, update recency and dependency closure risks;
6. operational cost/complexity, single points of failure, and fail-closed behavior.

End with a recommendation and a provider-neutral Rust adapter contract plus go/no-go checklist. Separate confirmed facts from inference and open questions. Cite each current claim with direct official links. Do not produce a marketing recommendation or state that a vendor is approved before the full distribution/dependency/license audit.

## Deliverable format

Write a concise comparison report with evidence table, dated activity checks, architecture fit, recommendation, risks, adapter seam and next acceptance tests. No code, tests or build work is required.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
