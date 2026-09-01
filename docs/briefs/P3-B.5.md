# Brief: P3-B.5

**Agent**: worker
**Phase**: P3-B
**Created**: 2026-09-02 02:38:59

---

B.5 OpenClaw 真实集成 e2e (per docs/automation-design.md v0.1 §4.1 + WBS §1 B.5)

scope: scripts/automation/integration_e2e.py 落 5 endpoint × 4 method OpenClaw wiremock stub
base: 094284b (per automation v0.1)
mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 实证 + 守门 #20 v2)
交付:
  1. scripts/automation/integration_e2e.py 新建, 5 endpoint stub: /v1/agents, /v1/sessions, /v1/messages, /v1/tools/invoke, /v1/cost
  2. 4 method 覆盖: GET (list/retrieve), POST (create/start), PUT (update), DELETE (close)
  3. 5 endpoint × 4 method = 20 case, 每个 case 返 wiremock 格式 response
  4. scripts/automation/__tests__/integration_e2e_test.py 5 测试 (每个 endpoint 1 测试)
守门: cargo check --workspace --lib 0 err + python smoke_test.py 5/5 + author Ulysses + 1 commit 1 wt
docs: commit message 含 scripts/automation/integration_e2e.py 路径 + 引用 WBS §1 B.5
已知: 5 endpoint 待 Ulysses 拍板 (per WBS §1 9/2 23:59 JST 选 1 拍板 + 共享脚本优先), 真实凭证 (B.5 mock 备选 per 29692a7)
