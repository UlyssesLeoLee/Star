# Brief: P3-F.6

**Agent**: worker
**Phase**: P3-F
**Created**: 2026-09-02 02:38:59

---

F.6 推 origin (R-05 反转) (per docs/automation-design.md v0.1 §4.5 + WBS §5 F.6 + §14.4 B-8)

scope: scripts/automation/git_push.py 落真实 git push 3 branch (main + feature/ai-ide-compat + wt branch) 到 https://github.com/UlyssesLeoLee/Star.git
base: 094284b (per automation v0.1)
mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 实证 + 守门 #20 v2)
交付:
  1. scripts/automation/git_push.py 新建, GitPushHelper 类 (push / validate / scan_secret 3 方法)
  2. 推 3 branch + secret 扫描 (.env / API key / PAT)
  3. 守门 #1+#6+#9+#12 实证: 推 0 失败 + author Ulysses 唯一 + secret 0 命中 + docs commit
  4. scripts/automation/__tests__/git_push_test.py 3 测试 (validate + scan_secret + dry_run push)
守门: cargo check --workspace --lib 0 err + python smoke_test.py 5/5 + author Ulysses + 1 commit 1 wt
docs: commit message 含 scripts/automation/git_push.py 路径 + 引用 WBS §5 F.6
已知: 推 origin 9/1 23:59 JST 失败 (github.com 443 不可达 + 无 PAT/GITHUB_TOKEN), wt 内 dry_run=True 默认, 真推需 Ulysses 提供 PAT 跨 session 续
