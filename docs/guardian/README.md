# Guardian v3x 候选脚手架 (per WBS-001 v0.56)

> **Status**: 待 Ulysses 拍板激活 (per 9/1 14:58 + 9/8 16:08 拍板必带推荐项)

5 v3x 候选脚本 + docs 索引:

| 候选 | 脚本/docs | 触发事件 | 实证 |
|---|---|---|---|
| v27 子代理 RPC 失败 fallback | `scripts/automation/guardian/v27_rpc_fallback.py` | 子代理 dispatch 后 30s verify | 守门 #9 P3-A.6/A.7 RPC 不可靠 |
| v28 拍板必带推荐项格式校验 | `scripts/automation/guardian/v28_recommendation.py` | Mavis 终端触发 ask_user | 9/8 16:08 第 8 次强化 + 9/8 16:11 4 次"等 Ulysses 下一步" 违规 |
| v29 docs 同步饱和 40+ 次告警 | `scripts/automation/guardian/v29_docs_saturation.py` | docs 同步 commit 落地 | 守门 #12 v15 死循环饱和 |
| v30 Mavis 永久代签边界 | `docs/guardian/v30_signature_boundary.md` | 5 域 Lead / SRE Lead / 平台 / 评审 / PM 签字栏 | 守门 #14 v3 升级 |
| v31 5 域 Lead 真人到位追溯签字 | `docs/guardian/v31_lead_traceback.md` | 真人到位 | 守门 #14 v2 拍板 D 维持 |

## 激活门槛 (per v0.46 brief §2.4)

1. Mavis 走 `ask_user` 必带推荐项 (per v28 + 9/8 16:08)
2. 推荐项放第一个 (Ulysses 视觉优先)
3. 拍板后立即执行 (per 9/5 04:03 守门 #9 #3 实证)
4. 拍板后同步修 v25 重号 (line 152/153): 第 1 个 v25 维持, 第 2 个 v25 改 v25b, 行 154 v26 改 v25c
5. 任何 v3x 激活后**必更新** `docs/automation-design.md` §1.4 守门编号累计表 + `scripts/automation/registry.md`
6. 跨 session 续做: v3x 候选激活是 governance-level 决策, 跨 5 域 Lead + 真人到位后追溯

## 拍板后落地流程

```bash
# 1. 激活 v3x 候选 (Ulysses 拍板后, Mavis 立即执行)
ask_user --lint < payload.json  # v28 验证
python scripts/automation/guardian/v27_rpc_fallback.py --task-id <id> --brief <path> --worktree <name>
python scripts/automation/guardian/v29_docs_saturation.py  # 触发告警

# 2. 改 AGENTS.md §4.1.1 行 status: 待 → active
# 3. 改 docs/automation-design.md §1.4 行 status: 待 → active
# 4. 修订历史 +1 行 (per AGENTS.md §3 7 段结构 + §4 #14 v3)
```

---

**Refs**: v0.46 `1d19d5f`, AGENTS.md §4.1.1, docs/automation-design.md §1.4, 守门 #14 v3, 9/1 14:58, 9/8 16:08, 9/8 15:19, 9/8 15:29, 9/9 12:02
