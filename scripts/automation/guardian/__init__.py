"""guardian: 守门 v3x 5 候选 脚手架 (per WBS-001 v0.56 + 守门 #14 v3 永久代签).

status: 待 Ulysses 拍板激活 (per 9/1 14:58 + 9/8 16:08 拍板必带推荐项).

5 候选:
- v27 子代理 RPC 失败 fallback 必填 (per 守门 #9 实证 P3-A.6/A.7)
- v28 拍板必带推荐项格式校验 (per 9/8 16:08 第 8 次强化)
- v29 docs 同步饱和 40+ 次主动告警 (per 守门 #12 v15 死循环饱和)
- v30 Mavis 永久代签适用边界 (per 守门 #14 v3 升级)
- v31 5 域 Lead 真人到位追溯签字机制 (per 守门 #14 v2 拍板 D 维持)

跨 session 续: 拍板激活后, 改 AGENTS.md §4.1.1 行 status + docs/automation-design.md §1.4 行.
"""

__version__ = "0.1-scaffold"
