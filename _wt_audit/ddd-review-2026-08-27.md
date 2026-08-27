# 当前进度 worktree - DDD Review 资料

生成时间: 2026-08-27 14:45 JST

main HEAD: 4b3b8dc

feature/ai-ide-compat HEAD: 245cf56


## wt-phase-c-flow-review

**Path**: $p  
**HEAD**: df23deb  
**Ahead main**: 6  **Behind main**: 3  


### Commits (vs main)

```
df23deb 2026-08-27 07:20:03 +0900 docs(upgrade): v0.2 浠ｇ瑙勫垯鍙嶈浆 (鍏ㄩ儴鍏佽 Ulysses 绛惧悕, per 2026-08-27 07:16 JST)
c25d261 2026-08-26 23:30:37 +0900 docs(upgrade): P1 闃绘柇椤规眹鎬?(3 瀛愪唬鐞?cross-validate)
2abfa46 2026-08-26 23:28:43 +0900 docs(phase-c-review): 瀛愪唬鐞?B context/resources/flows 涓€鑷存€у鏌ユ姤鍛?(18 spec / 26 finding)
245cf56 2026-08-26 23:28:13 +0900 fix(adr): 琛?5 浠?ADR 鍒?architecture/2026-08-26-upgrade/adr/ 淇?P0-1 鏂摼
bc23d6c 2026-08-26 21:08:34 +0900 chore: 鍒犻櫎 11 浠藉凡鍚堝苟 commit 鐨勫簾寮冭剼鎵嬫灦(_commit_*.txt + _sow_fe_*.md)
876a2a7 2026-08-26 19:56:55 +0900 docs(upgrade): Phase C 54 浠?spec 鑽夋 (Mavis 鍗曞共绗?1 杞?
```

### Diff stat (vs main)

```
 .../2026-08-26-upgrade/INTERFACE-REVIEW-B.md       | 460 +++++++++++++++++++++
 .../2026-08-26-upgrade/P1-BLOCKERS-SUMMARY.md      | 123 ++++++
 docs/architecture/2026-08-26-upgrade/README.md     | 162 ++++++++
 .../adr/0021-zero-vendor-cooperation.md            | 113 +++++
 .../2026-08-26-upgrade/adr/0022-ide-placement.md   | 138 +++++++
 .../adr/0023-version-control-provider.md           | 117 ++++++
 .../adr/0024-ide-session-identity.md               | 112 +++++
 .../adr/0025-vendor-adapter-anti-contamination.md  | 130 ++++++
 .../arch/01-current-architecture-analysis.md       | 163 ++++++++
 .../arch/02-ide-capability-boundary.md             |  98 +++++
 .../arch/03-star-ai-compat-arch.md                 | 246 +++++++++++
 .../arch/04-star-ide-gateway-arch.md               | 129 ++++++
 .../arch/05-gitgit-compat-arch.md                  | 119 ++++++
 .../2026-08-26-upgrade/arch/06-threat-model-nfr.md | 105 +++++
 .../spec/acceptance/01-unknown-agent-test.md       |  51 +++
 .../acceptance/02-zero-knowledge-agent-test.md     |  65 +++
 .../spec/acceptance/03-unknown-ide-test.md         |  61 +++
 .../2026-08-26-upgrade/spec/acceptance/04-mvp.md   |  86 ++++
 .../spec/acceptance/05-phase2.md                   |  38 ++
 .../spec/acceptance/06-phase3.md                   |  24 ++
 .../spec/acceptance/07-adr-list.md                 |  30 ++
 .../spec/acceptance/08-risk-register.md            |  28 ++
 .../spec/acceptance/09-agent-instructions-spec.md  |  84 ++++
 .../spec/acceptance/10-ide-instructions-spec.md    |  58 +++
 .../spec/acceptance/11-token-efficiency.md         |  62 +++
 .../spec/acceptance/12-capability-discovery.md     |  71 ++++
 .../spec/acceptance/13-schema-stability.md         |  46 +++
 .../spec/acceptance/14-performance-requirements.md |  43 ++
 .../spec/acceptance/15-final-acceptance.md         |  54 +++
 .../acceptance/16-ecosystem-research-summary.md    |  71 ++++
 .../spec/acceptance/17-master-plan-update.md       |  82 ++++
 .../2026-08-26-upgrade/spec/agent-api/01-schema.md | 102 +++++
 .../2026-08-26-upgrade/spec/cli/01-cli-spec.md     |  87 ++++
 .../spec/context/01-context-api.md                 | 101 +++++
 .../spec/context/02-code-intelligence-arch.md      |  61 +++
 .../spec/context/03-code-navigation-arch.md        |  48 +++
 .../spec/context/04-context-graph.md               |  45 ++
 .../spec/flows/01-agent-task-lifecycle.md          |  65 +++
 .../spec/flows/02-agent-lease-heartbeat.md         |  53 +++
 .../spec/flows/03-agent-resume.md                  |  62 +++
 .../spec/flows/04-multi-agent.md                   |  48 +++
 .../spec/flows/05-universal-submit.md              |  63 +++
 .../spec/flows/06-error-recovery.md                |  52 +++
 .../spec/flows/07-audit-model.md                   |  56 +++
 .../spec/flows/08-event-model.md                   |  56 +++
 .../2026-08-26-upgrade/spec/ide-api/01-schema.md   |  74 ++++
 .../2026-08-26-upgrade/spec/mcp/01-mcp-spec.md     |  77 ++++
 .../spec/resources/01-workspace-protocol.md        |  72 ++++
 .../spec/resources/02-worktree-protocol.md         |  81 ++++
 .../spec/resources/03-agent-identity.md            |  67 +++
 .../spec/resources/04-ide-session-identity.md      |  68 +++
 .../spec/resources/05-agent-permission-model.md    |  66 +++
 .../spec/resources/06-ide-permission-model.md      |  31 ++
 .../spec/rest/01-rest-strategy.md                  |  73 ++++
 .../spec/vcs/01-version-control-provider.md        |  63 +++
 .../spec/vcs/02-gitgit-provider.md                 |  48 +++
 .../spec/vcs/03-github-gitlab-compat.md            |  55 +++
 .../spec/vcs/04-fallback-strategy.md               |  84 ++++
 58 files changed, 4927 insertions(+)
```

### Touched file list (vs main, top-level dirs)

```
docs/
```

---

## wt-phase-c-interface-review

**Path**: $p  
**HEAD**: 8d79dae  
**Ahead main**: 5  **Behind main**: 2  


### Commits (vs main)

```
8d79dae 2026-08-27 07:20:03 +0900 docs(upgrade): v0.2 浠ｇ瑙勫垯鍙嶈浆 (鍏ㄩ儴鍏佽 Ulysses 绛惧悕, per 2026-08-27 07:16 JST)
c754426 2026-08-26 23:28:43 +0900 docs(phase-c-review): 瀛愪唬鐞?A 鎺ュ彛涓€鑷存€у鏌ユ姤鍛?(9 spec / 30 finding)
245cf56 2026-08-26 23:28:13 +0900 fix(adr): 琛?5 浠?ADR 鍒?architecture/2026-08-26-upgrade/adr/ 淇?P0-1 鏂摼
bc23d6c 2026-08-26 21:08:34 +0900 chore: 鍒犻櫎 11 浠藉凡鍚堝苟 commit 鐨勫簾寮冭剼鎵嬫灦(_commit_*.txt + _sow_fe_*.md)
876a2a7 2026-08-26 19:56:55 +0900 docs(upgrade): Phase C 54 浠?spec 鑽夋 (Mavis 鍗曞共绗?1 杞?
```

### Diff stat (vs main)

```
 .../2026-08-26-upgrade/INTERFACE-REVIEW-A.md       | 1152 ++++++++++++++++++++
 docs/architecture/2026-08-26-upgrade/README.md     |  162 +++
 .../adr/0021-zero-vendor-cooperation.md            |  113 ++
 .../2026-08-26-upgrade/adr/0022-ide-placement.md   |  138 +++
 .../adr/0023-version-control-provider.md           |  117 ++
 .../adr/0024-ide-session-identity.md               |  112 ++
 .../adr/0025-vendor-adapter-anti-contamination.md  |  130 +++
 .../arch/01-current-architecture-analysis.md       |  163 +++
 .../arch/02-ide-capability-boundary.md             |   98 ++
 .../arch/03-star-ai-compat-arch.md                 |  246 +++++
 .../arch/04-star-ide-gateway-arch.md               |  129 +++
 .../arch/05-gitgit-compat-arch.md                  |  119 ++
 .../2026-08-26-upgrade/arch/06-threat-model-nfr.md |  105 ++
 .../spec/acceptance/01-unknown-agent-test.md       |   51 +
 .../acceptance/02-zero-knowledge-agent-test.md     |   65 ++
 .../spec/acceptance/03-unknown-ide-test.md         |   61 ++
 .../2026-08-26-upgrade/spec/acceptance/04-mvp.md   |   86 ++
 .../spec/acceptance/05-phase2.md                   |   38 +
 .../spec/acceptance/06-phase3.md                   |   24 +
 .../spec/acceptance/07-adr-list.md                 |   30 +
 .../spec/acceptance/08-risk-register.md            |   28 +
 .../spec/acceptance/09-agent-instructions-spec.md  |   84 ++
 .../spec/acceptance/10-ide-instructions-spec.md    |   58 +
 .../spec/acceptance/11-token-efficiency.md         |   62 ++
 .../spec/acceptance/12-capability-discovery.md     |   71 ++
 .../spec/acceptance/13-schema-stability.md         |   46 +
 .../spec/acceptance/14-performance-requirements.md |   43 +
 .../spec/acceptance/15-final-acceptance.md         |   54 +
 .../acceptance/16-ecosystem-research-summary.md    |   71 ++
 .../spec/acceptance/17-master-plan-update.md       |   82 ++
 .../2026-08-26-upgrade/spec/agent-api/01-schema.md |  102 ++
 .../2026-08-26-upgrade/spec/cli/01-cli-spec.md     |   87 ++
 .../spec/context/01-context-api.md                 |  101 ++
 .../spec/context/02-code-intelligence-arch.md      |   61 ++
 .../spec/context/03-code-navigation-arch.md        |   48 +
 .../spec/context/04-context-graph.md               |   45 +
 .../spec/flows/01-agent-task-lifecycle.md          |   65 ++
 .../spec/flows/02-agent-lease-heartbeat.md         |   53 +
 .../spec/flows/03-agent-resume.md                  |   62 ++
 .../spec/flows/04-multi-agent.md                   |   48 +
 .../spec/flows/05-universal-submit.md              |   63 ++
 .../spec/flows/06-error-recovery.md                |   52 +
 .../spec/flows/07-audit-model.md                   |   56 +
 .../spec/flows/08-event-model.md                   |   56 +
 .../2026-08-26-upgrade/spec/ide-api/01-schema.md   |   74 ++
 .../2026-08-26-upgrade/spec/mcp/01-mcp-spec.md     |   77 ++
 .../spec/resources/01-workspace-protocol.md        |   72 ++
 .../spec/resources/02-worktree-protocol.md         |   81 ++
 .../spec/resources/03-agent-identity.md            |   67 ++
 .../spec/resources/04-ide-session-identity.md      |   68 ++
 .../spec/resources/05-agent-permission-model.md    |   66 ++
 .../spec/resources/06-ide-permission-model.md      |   31 +
 .../spec/rest/01-rest-strategy.md                  |   73 ++
 .../spec/vcs/01-version-control-provider.md        |   63 ++
 .../spec/vcs/02-gitgit-provider.md                 |   48 +
 .../spec/vcs/03-github-gitlab-compat.md            |   55 +
 .../spec/vcs/04-fallback-strategy.md               |   84 ++
 57 files changed, 5496 insertions(+)
```

### Touched file list (vs main, top-level dirs)

```
docs/
```

---

## wt-phase-c-acceptance-review

**Path**: $p  
**HEAD**: 6fe1910  
**Ahead main**: 5  **Behind main**: 2  


### Commits (vs main)

```
6fe1910 2026-08-27 07:20:04 +0900 docs(upgrade): v0.2 浠ｇ瑙勫垯鍙嶈浆 (鍏ㄩ儴鍏佽 Ulysses 绛惧悕, per 2026-08-27 07:16 JST)
0e5b11a 2026-08-26 23:28:44 +0900 docs(phase-c-review): 瀛愪唬鐞?C vcs/acceptance/arch 涓€鑷存€у鏌ユ姤鍛?(27 spec / 6 P1 + 12 P2)
245cf56 2026-08-26 23:28:13 +0900 fix(adr): 琛?5 浠?ADR 鍒?architecture/2026-08-26-upgrade/adr/ 淇?P0-1 鏂摼
bc23d6c 2026-08-26 21:08:34 +0900 chore: 鍒犻櫎 11 浠藉凡鍚堝苟 commit 鐨勫簾寮冭剼鎵嬫灦(_commit_*.txt + _sow_fe_*.md)
876a2a7 2026-08-26 19:56:55 +0900 docs(upgrade): Phase C 54 浠?spec 鑽夋 (Mavis 鍗曞共绗?1 杞?
```

### Diff stat (vs main)

```
 .../2026-08-26-upgrade/INTERFACE-REVIEW-C.md       | 426 +++++++++++++++++++++
 docs/architecture/2026-08-26-upgrade/README.md     | 162 ++++++++
 .../adr/0021-zero-vendor-cooperation.md            | 113 ++++++
 .../2026-08-26-upgrade/adr/0022-ide-placement.md   | 138 +++++++
 .../adr/0023-version-control-provider.md           | 117 ++++++
 .../adr/0024-ide-session-identity.md               | 112 ++++++
 .../adr/0025-vendor-adapter-anti-contamination.md  | 130 +++++++
 .../arch/01-current-architecture-analysis.md       | 163 ++++++++
 .../arch/02-ide-capability-boundary.md             |  98 +++++
 .../arch/03-star-ai-compat-arch.md                 | 246 ++++++++++++
 .../arch/04-star-ide-gateway-arch.md               | 129 +++++++
 .../arch/05-gitgit-compat-arch.md                  | 119 ++++++
 .../2026-08-26-upgrade/arch/06-threat-model-nfr.md | 105 +++++
 .../spec/acceptance/01-unknown-agent-test.md       |  51 +++
 .../acceptance/02-zero-knowledge-agent-test.md     |  65 ++++
 .../spec/acceptance/03-unknown-ide-test.md         |  61 +++
 .../2026-08-26-upgrade/spec/acceptance/04-mvp.md   |  86 +++++
 .../spec/acceptance/05-phase2.md                   |  38 ++
 .../spec/acceptance/06-phase3.md                   |  24 ++
 .../spec/acceptance/07-adr-list.md                 |  30 ++
 .../spec/acceptance/08-risk-register.md            |  28 ++
 .../spec/acceptance/09-agent-instructions-spec.md  |  84 ++++
 .../spec/acceptance/10-ide-instructions-spec.md    |  58 +++
 .../spec/acceptance/11-token-efficiency.md         |  62 +++
 .../spec/acceptance/12-capability-discovery.md     |  71 ++++
 .../spec/acceptance/13-schema-stability.md         |  46 +++
 .../spec/acceptance/14-performance-requirements.md |  43 +++
 .../spec/acceptance/15-final-acceptance.md         |  54 +++
 .../acceptance/16-ecosystem-research-summary.md    |  71 ++++
 .../spec/acceptance/17-master-plan-update.md       |  82 ++++
 .../2026-08-26-upgrade/spec/agent-api/01-schema.md | 102 +++++
 .../2026-08-26-upgrade/spec/cli/01-cli-spec.md     |  87 +++++
 .../spec/context/01-context-api.md                 | 101 +++++
 .../spec/context/02-code-intelligence-arch.md      |  61 +++
 .../spec/context/03-code-navigation-arch.md        |  48 +++
 .../spec/context/04-context-graph.md               |  45 +++
 .../spec/flows/01-agent-task-lifecycle.md          |  65 ++++
 .../spec/flows/02-agent-lease-heartbeat.md         |  53 +++
 .../spec/flows/03-agent-resume.md                  |  62 +++
 .../spec/flows/04-multi-agent.md                   |  48 +++
 .../spec/flows/05-universal-submit.md              |  63 +++
 .../spec/flows/06-error-recovery.md                |  52 +++
 .../spec/flows/07-audit-model.md                   |  56 +++
 .../spec/flows/08-event-model.md                   |  56 +++
 .../2026-08-26-upgrade/spec/ide-api/01-schema.md   |  74 ++++
 .../2026-08-26-upgrade/spec/mcp/01-mcp-spec.md     |  77 ++++
 .../spec/resources/01-workspace-protocol.md        |  72 ++++
 .../spec/resources/02-worktree-protocol.md         |  81 ++++
 .../spec/resources/03-agent-identity.md            |  67 ++++
 .../spec/resources/04-ide-session-identity.md      |  68 ++++
 .../spec/resources/05-agent-permission-model.md    |  66 ++++
 .../spec/resources/06-ide-permission-model.md      |  31 ++
 .../spec/rest/01-rest-strategy.md                  |  73 ++++
 .../spec/vcs/01-version-control-provider.md        |  63 +++
 .../spec/vcs/02-gitgit-provider.md                 |  48 +++
 .../spec/vcs/03-github-gitlab-compat.md            |  55 +++
 .../spec/vcs/04-fallback-strategy.md               |  84 ++++
 57 files changed, 4770 insertions(+)
```

### Touched file list (vs main, top-level dirs)

```
docs/
```

---

## wt-phase-d-p1-fix

**Path**: $p  
**HEAD**: 0e00318  
**Ahead main**: 4  **Behind main**: 1  


### Commits (vs main)

```
0e00318 2026-08-27 08:34:13 +0900 fix(upgrade): P1 闃绘柇椤?15/15 淇 (瀛愪唬鐞?A 缁堝 commit)
245cf56 2026-08-26 23:28:13 +0900 fix(adr): 琛?5 浠?ADR 鍒?architecture/2026-08-26-upgrade/adr/ 淇?P0-1 鏂摼
bc23d6c 2026-08-26 21:08:34 +0900 chore: 鍒犻櫎 11 浠藉凡鍚堝苟 commit 鐨勫簾寮冭剼鎵嬫灦(_commit_*.txt + _sow_fe_*.md)
876a2a7 2026-08-26 19:56:55 +0900 docs(upgrade): Phase C 54 浠?spec 鑽夋 (Mavis 鍗曞共绗?1 杞?
```

### Diff stat (vs main)

```
 .../2026-08-26-upgrade/P1-FIX-SUMMARY.md           | 263 ++++++++++++++++++++
 docs/architecture/2026-08-26-upgrade/README.md     | 162 ++++++++++++
 .../adr/0021-zero-vendor-cooperation.md            | 113 +++++++++
 .../2026-08-26-upgrade/adr/0022-ide-placement.md   | 138 +++++++++++
 .../adr/0023-version-control-provider.md           | 117 +++++++++
 .../adr/0024-ide-session-identity.md               | 112 +++++++++
 .../adr/0025-vendor-adapter-anti-contamination.md  | 130 ++++++++++
 .../arch/01-current-architecture-analysis.md       | 163 ++++++++++++
 .../arch/02-ide-capability-boundary.md             |  98 ++++++++
 .../arch/03-star-ai-compat-arch.md                 | 274 +++++++++++++++++++++
 .../arch/04-star-ide-gateway-arch.md               | 129 ++++++++++
 .../arch/05-gitgit-compat-arch.md                  | 127 ++++++++++
 .../2026-08-26-upgrade/arch/06-threat-model-nfr.md | 105 ++++++++
 .../spec/acceptance/01-unknown-agent-test.md       |  51 ++++
 .../acceptance/02-zero-knowledge-agent-test.md     |  65 +++++
 .../spec/acceptance/03-unknown-ide-test.md         |  61 +++++
 .../2026-08-26-upgrade/spec/acceptance/04-mvp.md   |  86 +++++++
 .../spec/acceptance/05-phase2.md                   |  38 +++
 .../spec/acceptance/06-phase3.md                   |  24 ++
 .../spec/acceptance/07-adr-list.md                 |  30 +++
 .../spec/acceptance/08-risk-register.md            |  28 +++
 .../spec/acceptance/09-agent-instructions-spec.md  |  84 +++++++
 .../spec/acceptance/10-ide-instructions-spec.md    |  58 +++++
 .../spec/acceptance/11-token-efficiency.md         |  62 +++++
 .../spec/acceptance/12-capability-discovery.md     |  71 ++++++
 .../spec/acceptance/13-schema-stability.md         |  46 ++++
 .../spec/acceptance/14-performance-requirements.md |  43 ++++
 .../spec/acceptance/15-final-acceptance.md         |  54 ++++
 .../acceptance/16-ecosystem-research-summary.md    |  71 ++++++
 .../spec/acceptance/17-master-plan-update.md       |  82 ++++++
 .../2026-08-26-upgrade/spec/agent-api/01-schema.md | 234 ++++++++++++++++++
 .../2026-08-26-upgrade/spec/cli/01-cli-spec.md     | 112 +++++++++
 .../spec/context/01-context-api.md                 | 101 ++++++++
 .../spec/context/02-code-intelligence-arch.md      |  61 +++++
 .../spec/context/03-code-navigation-arch.md        |  48 ++++
 .../spec/context/04-context-graph.md               |  45 ++++
 .../spec/flows/01-agent-task-lifecycle.md          |  74 ++++++
 .../spec/flows/02-agent-lease-heartbeat.md         |  53 ++++
 .../spec/flows/03-agent-resume.md                  |  85 +++++++
 .../spec/flows/04-multi-agent.md                   |  48 ++++
 .../spec/flows/05-universal-submit.md              |  77 ++++++
 .../spec/flows/06-error-recovery.md                |  52 ++++
 .../spec/flows/07-audit-model.md                   |  56 +++++
 .../spec/flows/08-event-model.md                   |  56 +++++
 .../2026-08-26-upgrade/spec/ide-api/01-schema.md   |  93 +++++++
 .../2026-08-26-upgrade/spec/mcp/01-mcp-spec.md     | 133 ++++++++++
 .../spec/resources/01-workspace-protocol.md        |  72 ++++++
 .../spec/resources/02-worktree-protocol.md         |  81 ++++++
 .../spec/resources/03-agent-identity.md            |  67 +++++
 .../spec/resources/04-ide-session-identity.md      |  68 +++++
 .../spec/resources/05-agent-permission-model.md    |  66 +++++
 .../spec/resources/06-ide-permission-model.md      |  31 +++
 .../spec/rest/01-rest-strategy.md                  |  84 +++++++
 .../spec/vcs/01-version-control-provider.md        |  63 +++++
 .../spec/vcs/02-gitgit-provider.md                 |  48 ++++
 .../spec/vcs/03-github-gitlab-compat.md            |  55 +++++
 .../spec/vcs/04-fallback-strategy.md               |  91 +++++++
 57 files changed, 4939 insertions(+)
```

### Touched file list (vs main, top-level dirs)

```
docs/
```

---

## wt-phase-d-skeleton

**Path**: $p  
**HEAD**: 6f3c90a  
**Ahead main**: 4  **Behind main**: 1  


### Commits (vs main)

```
6f3c90a 2026-08-27 08:34:26 +0900 feat(phase-d): STAR CLI / MCP / Context 3 crate 鏋佺畝楠ㄦ灦 (瀛愪唬鐞?B 缁堝 commit)
245cf56 2026-08-26 23:28:13 +0900 fix(adr): 琛?5 浠?ADR 鍒?architecture/2026-08-26-upgrade/adr/ 淇?P0-1 鏂摼
bc23d6c 2026-08-26 21:08:34 +0900 chore: 鍒犻櫎 11 浠藉凡鍚堝苟 commit 鐨勫簾寮冭剼鎵嬫灦(_commit_*.txt + _sow_fe_*.md)
876a2a7 2026-08-26 19:56:55 +0900 docs(upgrade): Phase C 54 浠?spec 鑽夋 (Mavis 鍗曞共绗?1 杞?
```

### Diff stat (vs main)

```
 Cargo.lock                                         | 149 +++++++++++++
 Cargo.toml                                         |   4 +
 crates/star-cli/Cargo.toml                         |  25 +++
 crates/star-cli/src/commands/agent.rs              | 162 ++++++++++++++
 crates/star-cli/src/commands/mod.rs                |  12 +
 crates/star-cli/src/commands/submit.rs             |  30 +++
 crates/star-cli/src/commands/task.rs               |  94 ++++++++
 crates/star-cli/src/error.rs                       |  29 +++
 crates/star-cli/src/main.rs                        |  73 ++++++
 crates/star-cli/src/output.rs                      |  29 +++
 crates/star-context/Cargo.toml                     |  21 ++
 crates/star-context/src/lib.rs                     |  90 ++++++++
 crates/star-context/src/template.rs                |  30 +++
 crates/star-mcp/Cargo.toml                         |  25 +++
 crates/star-mcp/src/error.rs                       |  27 +++
 crates/star-mcp/src/main.rs                        | 134 +++++++++++
 crates/star-mcp/src/tools/create_merge_request.rs  |  22 ++
 crates/star-mcp/src/tools/create_worktree.rs       |  22 ++
 crates/star-mcp/src/tools/find_references.rs       |  22 ++
 crates/star-mcp/src/tools/get_code_context.rs      |  22 ++
 crates/star-mcp/src/tools/get_context.rs           |  22 ++
 crates/star-mcp/src/tools/get_current_task.rs      |  22 ++
 crates/star-mcp/src/tools/get_issue.rs             |  22 ++
 crates/star-mcp/src/tools/get_pipeline_status.rs   |  22 ++
 crates/star-mcp/src/tools/get_symbol.rs            |  22 ++
 crates/star-mcp/src/tools/get_workspace.rs         |  22 ++
 crates/star-mcp/src/tools/get_worktree.rs          |  22 ++
 crates/star-mcp/src/tools/mod.rs                   |  25 +++
 crates/star-mcp/src/tools/request_review.rs        |  22 ++
 crates/star-mcp/src/tools/run_validation.rs        |  22 ++
 crates/star-mcp/src/tools/search_code.rs           |  22 ++
 crates/star-mcp/src/tools/search_issues.rs         |  22 ++
 crates/star-mcp/src/tools/submit.rs                |  22 ++
 docs/architecture/2026-08-26-upgrade/README.md     | 162 ++++++++++++++
 .../adr/0021-zero-vendor-cooperation.md            | 113 ++++++++++
 .../2026-08-26-upgrade/adr/0022-ide-placement.md   | 138 ++++++++++++
 .../adr/0023-version-control-provider.md           | 117 ++++++++++
 .../adr/0024-ide-session-identity.md               | 112 ++++++++++
 .../adr/0025-vendor-adapter-anti-contamination.md  | 130 +++++++++++
 .../arch/01-current-architecture-analysis.md       | 163 ++++++++++++++
 .../arch/02-ide-capability-boundary.md             |  98 ++++++++
 .../arch/03-star-ai-compat-arch.md                 | 246 +++++++++++++++++++++
 .../arch/04-star-ide-gateway-arch.md               | 129 +++++++++++
 .../arch/05-gitgit-compat-arch.md                  | 119 ++++++++++
 .../2026-08-26-upgrade/arch/06-threat-model-nfr.md | 105 +++++++++
 .../spec/acceptance/01-unknown-agent-test.md       |  51 +++++
 .../acceptance/02-zero-knowledge-agent-test.md     |  65 ++++++
 .../spec/acceptance/03-unknown-ide-test.md         |  61 +++++
 .../2026-08-26-upgrade/spec/acceptance/04-mvp.md   |  86 +++++++
 .../spec/acceptance/05-phase2.md                   |  38 ++++
 .../spec/acceptance/06-phase3.md                   |  24 ++
 .../spec/acceptance/07-adr-list.md                 |  30 +++
 .../spec/acceptance/08-risk-register.md            |  28 +++
 .../spec/acceptance/09-agent-instructions-spec.md  |  84 +++++++
 .../spec/acceptance/10-ide-instructions-spec.md    |  58 +++++
 .../spec/acceptance/11-token-efficiency.md         |  62 ++++++
 .../spec/acceptance/12-capability-discovery.md     |  71 ++++++
 .../spec/acceptance/13-schema-stability.md         |  46 ++++
 .../spec/acceptance/14-performance-requirements.md |  43 ++++
 .../spec/acceptance/15-final-acceptance.md         |  54 +++++
 .../acceptance/16-ecosystem-research-summary.md    |  71 ++++++
 .../spec/acceptance/17-master-plan-update.md       |  82 +++++++
 .../2026-08-26-upgrade/spec/agent-api/01-schema.md | 102 +++++++++
 .../2026-08-26-upgrade/spec/cli/01-cli-spec.md     |  87 ++++++++
 .../spec/context/01-context-api.md                 | 101 +++++++++
 .../spec/context/02-code-intelligence-arch.md      |  61 +++++
 .../spec/context/03-code-navigation-arch.md        |  48 ++++
 .../spec/context/04-context-graph.md               |  45 ++++
 .../spec/flows/01-agent-task-lifecycle.md          |  65 ++++++
 .../spec/flows/02-agent-lease-heartbeat.md         |  53 +++++
 .../spec/flows/03-agent-resume.md                  |  62 ++++++
 .../spec/flows/04-multi-agent.md                   |  48 ++++
 .../spec/flows/05-universal-submit.md              |  63 ++++++
 .../spec/flows/06-error-recovery.md                |  52 +++++
 .../spec/flows/07-audit-model.md                   |  56 +++++
 .../spec/flows/08-event-model.md                   |  56 +++++
 .../2026-08-26-upgrade/spec/ide-api/01-schema.md   |  74 +++++++
 .../2026-08-26-upgrade/spec/mcp/01-mcp-spec.md     |  77 +++++++
 .../spec/resources/01-workspace-protocol.md        |  72 ++++++
 .../spec/resources/02-worktree-protocol.md         |  81 +++++++
 .../spec/resources/03-agent-identity.md            |  67 ++++++
 .../spec/resources/04-ide-session-identity.md      |  68 ++++++
 .../spec/resources/05-agent-permission-model.md    |  66 ++++++
 .../spec/resources/06-ide-permission-model.md      |  31 +++
 .../spec/rest/01-rest-strategy.md                  |  73 ++++++
 .../spec/vcs/01-version-control-provider.md        |  63 ++++++
 .../spec/vcs/02-gitgit-provider.md                 |  48 ++++
 .../spec/vcs/03-github-gitlab-compat.md            |  55 +++++
 .../spec/vcs/04-fallback-strategy.md               |  84 +++++++
 89 files changed, 5655 insertions(+)
```

### Touched file list (vs main, top-level dirs)

```
Cargo.lock
Cargo.toml
crates/
docs/
```

---

