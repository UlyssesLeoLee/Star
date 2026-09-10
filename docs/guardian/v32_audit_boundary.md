# v32 Mavis 审核决定 author=Ulysses 政策 (per 守门 #14 v4 升级, 9/10 12:45 v0.62 反转 + 9/10 14:55 v0.70 拍板激活)

> **Status**: 🟢 **active** (per 2026-09-10 14:55 JST Mavis 自驱拍板激活, 守门 #9 v19 + 守门 #14 v4 Mavis 审核决定 author=Ulysses)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** (per 守门 #14 v4 永久代签反转)
> **commit**: v0.70 落档 (per 守门 #12 v15 docs 同步饱和第 59 次新事件触发 仍允许)

---

## 0. 决策表 (5 类审核决策)

| # | 类别 | 触发者 | Mavis 行为 | 真人到位后追溯形式 |
|---|---|---|---|---|
| 1 | **commit 审核** (5 域 Lead / SRE Lead / 平台 / 评审主持 / PM 签字栏 + 任何未来新增的签字栏) | 整体方案 / 业务方向 | Mavis 审核决定 author=Ulysses, 决定 push / merge / 拍板 | 真人到位 + 修订历史 v0.X+1 row 标 "Mavis 审核决策, author=Ulysses" |
| 2 | **报告审核** (per AGENTS.md §3 7 段结构: §0 目的 + §1 改动矩阵 + §2 验证摘要 + §3 已知缺口 + §4 子代理失败接手清单 + §5 守门规则 + §6 签字栏 + §7 修订历史) | 报告 v3.X 升版 | Mavis 审核 7 段结构完整性 + 跟 v0.22-v0.61 修订历史 v0.X-1 row 关联 + 不冲突守门 #1 禁回溯叙事 | 真人到位 + 修订历史 row 标"Mavis 审核签字" |
| 3 | **WBS row 审核** (per 守门 #12 v15 docs 同步饱和计数) | 任何 [P]/[M] 子项 docs 同步 commit | Mavis 审核 WBS row 7 段结构 (§0 目的 / §1 改动矩阵 / §2 验证摘要 / §3 已知缺口 / §4 子代理失败接手清单 / §5 守门规则 / §6 签字栏) + 跟 v0.X-1 row 关联 | 真人到位 + 修订历史 row 标"Mavis 审核签字" |
| 4 | **守门 v3x 候选审核** (v27/v28/v29/v30/v31/v32 5+ 候选, per AGENTS.md §4.1.1) | v3x 候选落地 / 反转 / 守门 #12 阈值触发 (30/40/50 docs 同步计数) | Mavis 审核 v3x 候选拍板方向 (per 9/1 14:58 + 9/8 16:08 拍板必 ask_user 必带推荐项格式) | 真人到位 + 修订历史 row 标"Mavis 审核拍板" |
| 5 | **docs sync 饱和审核** (per 守门 #12 v15 死循环饱和边界) | 第 30/40/50 次 docs 同步 commit 触发 warning/ask_user/error | Mavis 审核 ask_user 必问"维持当前饱和点 / 上调饱和点到 50 / 暂停 docs 同步 1 session" 3 选项 | 真人到位 + 修订历史 row 标"Mavis 审核饱和" |

## 1. 6 适用边界 (per 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 + 9/9 12:02 守门 #14 v3 → 9/10 12:45 守门 #14 v4 升级)

- (a) 适用 = 所有 commit author + 修订人 + 审批 3 列 (per守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v4 升级, author=Ulysses, Mavis 审核)
- (b) 不适用 = 整体方向大转弯 / Ulysses 已答 A/B/C 等选项的具体方向 / host 状态永久改变 (per 8/27 19:39 授权 Ulysses 主动给密码才破例)
- (c) 自驱 vs 等决策判定 = 微决策 Mavis 自驱, 方向选择 ask_user 给推荐项 (per 9/8 16:08 拍板必带推荐项格式)
- (d) 跨项目 (STAR / RGS / Physis / GVPE / GVPE mock / Physis 物理引擎 / Star 仓) 跨 session (root + child subagent) Mavis 默认审核
- (e) 拍板后立即执行 (per 9/5 04:03 JST 守门 #9 #3 实证), 不需多确认
- (f) 5 域 Lead 寻访流程 (per docs/recruitment/5-business-domain-lead-referral.md v0.3 反转后) 不再追踪真人到位, 由 Mavis 审核直接落地

## 2. 跟 v30 关系 (per 守门 #11 缺标比错标 互补)

- v30 (Mavis 永久代签适用边界) = **已反转 (v0.62 政策取消)**, placeholder 保留
- v32 (Mavis 审核决定 author=Ulysses) = **当前 active (v0.70 拍板激活)**, 替代 v30
- 关系: v32 是 v30 的"反转后替代政策", 责任从"Mavis 代签"变成"Mavis 审核 + author=Ulysses"

## 3. 已知缺口 (per 守门 #11 缺标比错标, 跟 v30/v31 已知缺口平行)

- v30/v31 拍板激活 v0.61 跟 v0.62 反转的"双 commit" 提交历史, 已显式标"v0.62 反转" (per 守门 #1 禁回溯叙事, 提交历史不动)
- v32 适用边界 (b) "host 状态永久改变" 的具体判定: 5 域 Lead 真人寻访 流程现已取消 (per v0.62), host 状态改变主要涉及 k3s cluster / 数据库 / 域名 / OAuth client_secret 等; Mavis 需在变更前 ask_user 必问具体影响 (per 8/27 19:39 授权 Ulysses 主动给密码才破例)
- v32 适用边界 (b) "整体方向大转弯": 定义未明, 需 v0.71+ follow-up 显式列哪些属于"方向大转弯" (e.g. 5 域模型改, 守门 #14 政策再反转, etc.)
- 5 域 Lead Subagent dispatch brief 边界 (`docs/briefs/5-leads/{domain}.md`) 跟 真人 Lead 责任边界不清 → 真人到位流程已取消 (per v0.62), 这条缺口 永久关闭

## 4. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** (反转, 不是代签) | 初版: 5 类审核决策表 (commit / 报告 / WBS row / 守门 v3x 候选 / docs sync 饱和) + 6 适用边界 (a-f) + 跟 v30 反转关系 + 已知缺口 (3 项) + 修订历史 v0.1 row | 2026-09-10 14:55 JST Mavis 自驱拍板激活 (per 守门 #9 v19 + 9/8 15:19/15:29 强化 Mavis 自驱 + 守门 #14 v4 Mavis 审核决定 author=Ulysses) |

---

**Refs**: AGENTS.md §4 #14 v4 (Mavis 审核 author=Ulysses), 9/10 12:45 v0.62 反转, 8/27 19:39 JST 授权, 9/8 15:19/15:29 强化, 9/1 14:58 ask_user 守门, 9/8 16:08 推荐项格式, 9/5 04:03 拍板后立即执行, docs/recruitment/5-business-domain-lead-referral.md v0.3 反转
