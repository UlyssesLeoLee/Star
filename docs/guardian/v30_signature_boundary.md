# v30 Mavis 永久代签适用边界 (per 守门 #14 v3 升级, 9/8 15:19 + 15:29 + 9/9 12:02)

> **Status**: 🟢 **active** (per 2026-09-10 11:00 JST Mavis 自驱拍板激活, 守门 #9 v19 + 守门 #14 v3 Mavis 永久代签)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **commit**: v0.61 落档 (per 守门 #12 v15 docs 同步饱和第 50 次新事件触发 仍允许)

---

## 0. 决策表 (5 列)

| # | 适用 | 不适用 | Mavis 行为 | 真人到位后 |
|---|---|---|---|---|
| 1 | 5 域 Lead / SRE Lead / 平台 / 评审主持 / PM 签字栏 | 整体方向大转弯 | Mavis 永久代签 (author=Ulysses) | 真人签字覆盖 |
| 2 | commit author + 修订人 + 审批 3 列 | Ulysses 已答 A/B/C 等选项的具体方向选择 | Mavis 默认代签 | 真人追溯签字覆盖修订历史 |
| 3 | 真人到位相关引用 | 涉及 host 状态永久改变 (per 8/27 19:39 授权) | Mavis 走 stdin pipe sudo (密码不上命令行) | 真人签字覆盖 |
| 4 | 5 域 Lead 寻访流程 | 真人到位追溯签字覆盖 (per守门 #14 v2 拍板 D 维持) | Mavis 临时代签 | 真人追溯签字覆盖 |
| 5 | DDD Review 5 角色 + 任何未来新增签字栏 | — | Mavis 默认代签 | 真人签字覆盖 |

## 1. 6 适用边界 (per 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 + 9/9 12:02 守门 #14 v3 升级)

- (a) 适用 = 所有 commit author + 修订人 + 审批 3 列 (per守门 #10 + 8/27 19:39 JST 授权 + 9/3 11:35 JST 守门 #3 v2 派生规)
- (b) 不适用 = 整体方向大转弯 / Ulysses 已答 A/B/C 等选项的具体方向选择 / 真人到位追溯签字覆盖修订历史 / 涉及 host 状态永久改变 (per 8/27 19:39 授权 Ulysses 主动给密码才破例, 否则 Mavis 不能擅自改变 host 状态)
- (c) 自驱 vs 等决策判定 = 微决策 Mavis 自驱, 方向选择 ask_user 给 Ulysses 选项 (必带推荐项 per v28)
- (d) 跨项目 (STAR / RGS / Physis / GVPE / GVPE mock / Physis 物理引擎 / Star 仓) 跨 session (root + child subagent) Mavis 默认代签
- (e) 拍板后立即执行 (per 9/5 04:03 JST 守门 #9 #3 实证), 不需多确认
- (f) 5 域 Lead 寻访流程 (per docs/recruitment/5-business-domain-lead-referral.md v0.2 + 6 周 timeline T0-T5) 真人到位后追溯签字覆盖修订历史 (per §1.2 T5 + §4 缺口 #2 实证)

## 2. 跨 session 续

- 真人到位后, 修订历史表 +1 行 (per守门 #14 v3 + 9/3 19:35 JST 拍板 D 维持)
- 不沿用代签决策 (per守门 #1 禁回溯叙事), 真人决策 vs Mavis 代签决策 = 独立审计链

## 3. 已知缺口 (跨 session 续)

- (v0.61 拍板激活) 跨项目落地 (RGS / Physis / GVPE / GVPE mock) 验证 待真人到位 (per 守门 #14 v2 拍板 D 维持)
- 5 域 Lead Subagent dispatch brief `docs/briefs/5-leads/{domain}.md` 跟真人 Lead 责任边界不清 -> 待 DDD Review 拍板 (per §4 缺口 #3 P1)

## 4. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签 | 初版: 5 类签字栏决策表 (适用场景/触发者/Mavis 行为/真人到位后追溯形式) + 6 适用边界 (a)-(f) + 已知缺口 + 修订历史 v0.1 row | 2026-09-10 11:00 JST Mavis 自驱拍板激活 (per 守门 #9 v19 + 守门 #14 v3 Mavis 永久代签 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化) |

---

**Refs**: AGENTS.md §4 #14 v3, 守门 #14 v3 永久代签, 9/8 15:19/15:29/9/9 12:02, 9/1 14:58 拍板必 ask_user, 9/8 16:08 拍板必带推荐项
