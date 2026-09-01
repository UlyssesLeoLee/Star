# Brief: P3-B.6

**Agent**: worker
**Phase**: P3-B
**Created**: 2026-09-02 02:38:59

---

B.6 Hermes 真实集成 e2e (per docs/automation-design.md v0.1 §4.1 + WBS §1 B.6)

scope: scripts/automation/integration_e2e.py 落 5 endpoint × 4 method Hermes wiremock stub (跟 B.5 共享脚本, 改 base_url + auth header)
base: 094284b (per automation v0.1)
mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 实证 + 守门 #20 v2)
交付:
  1. scripts/automation/integration_e2e.py 同 B.5, 加 HermesConfig dataclass (base_url / api_key / timeout)
  2. 5 endpoint stub: /v2/hermes/agents, /v2/hermes/sessions, /v2/hermes/messages, /v2/hermes/tools/invoke, /v2/hermes/cost
  3. 4 method 覆盖同 B.5, 5 × 4 = 20 case
  4. scripts/automation/__tests__/integration_e2e_test.py 加 5 测试 (跟 B.5 共享, 5/10 endpoint)
守门: cargo check --workspace --lib 0 err + python smoke_test.py 5/5 + author Ulysses + 1 commit 1 wt
docs: commit message 含 scripts/automation/integration_e2e.py 路径 + 引用 WBS §1 B.6
已知: 共享 B.5 脚本, B.5 收官后 B.6 直接 import 复用; Hermes 真实凭证 (B.6 mock 备选 per 29692a7)
