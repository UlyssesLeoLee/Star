# Brief: WLD5-HOST-AUTH-BRIDGE-20261023

**Agent**: worker
**Phase**: WBS-NEXT-20261023
**Created**: 2026-10-23 22:39:15

---

## 目标
推进 WLD-5 host authentication bridge。复核 frontend root、实际 auth/session owner、Run Directory 与 Group session wiring。只有仓库存在真实可验证的 host token/session contract 时才可最小接线；否则产出审计/决策报告，指出 embedding host 必须提供的 contract，保持 Hooks/Run/Group API disabled。

## 基线证据
预 brief dev HEAD=08f2df723c663a41d6d48faf29fcad916b4abddd；brief commit 后精确 base 由协调者记录。现有 RunDirectoryHostSession 是宿主注入接口，含非 secret generation 与 getAccessToken；Providers 收到 session 时共享其 callback/generation；RootLayout 当前不传 session。Group API 的 missing/401 invalidation、generation stale-response fence、403 scoped handling 已有实现和测试；不重复或放宽。须在当前 base 重新确认 auth owner 是否存在。
CodeGraph 可用索引 up-to-date，但重建 DB 被 MCP 锁定 (EPERM)；没有 check_index_coverage 工具，不得声称 coverage 已验证。

## 文件清单
仅写 docs/reports/PHASE-WLD5-HOST-AUTH-BRIDGE-20261023.md。若发现真实 auth provider，先向协调者给出权威来源、权限/过期语义和拟需文件；不得扩 manifest，不虚构 token issuer/cookie setter/browser storage/demo credential。

## 门禁与验收
不得 mock/seed auth、持久化/日志化 token、伪装无 session 为 authenticated。401/missing token fail closed；403 是 scoped denial；过期 generation 回调不可回填；mutation 不重放。报告逐项记录 RootLayout、Providers、RunDirectoryHostSession、Group API 与测试的实际行为；比较 host token callback 与 same-origin BFF 的信任边界、轮换、撤权、origin、测试和所需 owner；缺外部契约时推荐下一步并保持禁用。纯文档运行 git diff --check。
提交前确认只改 manifest 报告；回报 base/head/commit、证据、结论、阻塞与 integration 建议。不 push、不 rebase、不清理 worktree。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
