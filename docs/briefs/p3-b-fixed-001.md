# Brief: p3-b-fixed-001

**Agent**: worker
**Phase**: P3-B-FIXED
**Created**: 2026-09-03 07:59:49

---

蓝方 26 项代修剩余 9 项实装 (0.22M token, 估 ~30 分钟): #2 7 supporting spec 头部声明加 disclaimer / #5 2 spec (identity/local-runtime) requirements §23.x → basic-design 修 / #6 5 文件 7 vs 6 supporting crate 统一 / #7 5 文件 {display} 占位符替换 / #18 2 spec (identity/permission) tenant 隐式依赖 / #19 domain-integration-spec 加 work-item / #20 domain-relation-spec 统一方向 / #23 ASCII §2.3 依赖图统一箭头方向 / #25 7 supporting spec 加附录 B:边界清单 章节。14 docs 1 commit 落档 (per 守门 #12 commit-time docs 同步)。不派子代理 (per 守门 #9 #3 实证 5/5 RPC 不可靠), 改用 refactor_template.py 子类 + subprocess.run 替代。
