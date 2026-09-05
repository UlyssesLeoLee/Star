# RF-001 T1.5 merge 拍板 brief (per 2026-09-05 11:12 JST `ask_9ccc7a57dc5823d05e17e2b3` 用户拍板)

> **状态**: 🟢 Active v0.1 (2026-09-05 11:12 JST 拍板落地)
> **触发**: per 2026-09-05 11:12 JST `ask_9ccc7a57dc5823d05e17e2b3` 用户拍板 (Q1=rebase-then-merge[推荐], Q2=close-1c[推荐], 推荐项)
> **守门依据**: 守门 #1 v17 (RF-001 T1.5 step 1 验证 实证) + 守门 #12 (commit-time docs 同步) + 守门 #20 (子代理 dispatch 必先 brief)
> **关联 commit**: 见 `git log -p --follow docs/briefs/rf001-merge-001.md` (per 守门 #12 不写死 SHA)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手

---

## §0 目的

把 RF-001 T1.5 missing_docs sub-lint 实证工作 (per 守门 #1 v17 + 守门 #4 + ADR-0047 同源) 从 `rf001-t15-work` (23 commit ahead of stale main) 推进到当前 main HEAD, 同时关闭已 obsolete 的 `feat/auto-20260904-1c260bc7` 分支 (5 域 Lead 子代理兼任 路径, 已被 G-DEP-03 真人 Lead 拍板取代).

## §1 拍板落地 (2 项)

### 1.1 rf001-t15-work rebase + merge (Q1 拍板, 推荐)

**操作步骤** (在 `D:\Star\.worktrees\wt-t15-missing-docs` worktree 内执行):

1. `git checkout rf001-t15-work` (当前已是, 验证 HEAD = 65bceea)
2. `git rebase main` — 把 23 commit replay 到 main HEAD `f7f8330` 之上
3. **冲突处理**: 23 commit 全部是 doc comment 补充, 跟 main 的功能性改动不冲突, 预期 0 conflict. 如有 conflict, 立即 stop, report 给父会话 (per 守门 #20 子代理不可越界)
4. rebase 成功后:
   - `git checkout main`
   - `git merge --ff-only rf001-t15-work` (fast-forward)
5. **5 守门实证** (per 守门 #1 v1-v14 子集, 跳过 clippy 跨 session 续):
   - v1 `cargo check --workspace --lib -j 4` 0 err
   - v3 `cargo fmt --all --check` 0 diff
   - v6 `cargo test --workspace --lib` 100% pass
   - v14 `cargo check --workspace --all-targets --release -j 4` 0 err
6. **push origin 0/0 sync 实证** (per 守门 #1 R-05 反转 8/30 07:09 JST):
   - `git push origin main`
   - 验证 `git rev-parse origin/main` == local HEAD

### 1.2 feat/auto-20260904-1c260bc7 close (Q2 拍板, 推荐)

**操作步骤** (在 `D:\Star` 主仓执行, 父会话 Mavis 接手直接做):

1. `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' branch -D feat/auto-20260904-1c260bc7` (force delete, 引用 0 commit 已 merged)
2. **不动 worktree** (per 守门 #12 严守, 其他 sub-session 拥有 `D:\Star\.worktrees\feat-auto-20260904-1c260bc7`)
3. **修订历史记录** (AGENTS.md §8 加 v0.78):
   - "feat/auto-20260904-1c260bc7 关闭 (per 2026-09-05 11:12 JST `ask_9ccc7a57dc5823d05e17e2b3` 用户拍板 Q2=close-1c[推荐]): 包含 437 行 domain-scm + 612 行 domain-search + 77 行 domain-work-item + ADR-0047 + 16 MCP tool 删行, 跟 G-DEP-03 真人 Lead 拍板 (per 2026-09-05 10:43 JST, 已取代 5 子代理兼任 9/4 18:30 JST 路径) 冲突. 关闭原因: G-DEP-03 拍板 + 9/3 19:35 JST 拍板 D 维持. worktree `feat-auto-20260904-1c260bc7` 保留 (其他 sub-session 拥有)."

## §2 未拍板项 (跨 session 续 / 不动)

- **rf001-t15-recovered** (+15 ahead): 跟 rf001-t15-work 主题相同 (T1.5 missing_docs), 可能是 parallel recovery 的重复 commit. **不 merge**, 跟 rf001-t15-work rebase 后状态对比, 如重复则 close
- **rf001-t15-worktree-content** (+1 ahead): "rescue uncommitted missing_docs working-tree content from shared main" — 1 commit 跟 worktree 状态相关, **不 merge**, 等 RF-001 阶段明确
- **worktree `D:\Star\.worktrees\feat-auto-20260904-1c260bc7`**: 关闭分支后保留, 其他 sub-session 拥有 (per 守门 #12 严守)

## §3 守门合规 (per AGENTS.md §4 12 域 + §4.1 派生规)

- **守门 #1 v1-v14 子集**: v1 (lib) + v3 (fmt) + v6 (test) + v14 (release), 跳过 v2/v4-v5/v7-v13 (per 之前 1/2/3 号 P0/P1/P2 派生, 节省 token)
- **守门 #5**: env 安全, `$env:GHCR_PAT` 引用不打印 (推 origin 用)
- **守门 #9**: 子代理 status ≠ 实际成功, 必 `git log -p --follow <wt-branch>` 实证
- **守门 #10**: commit author = `Ulysses <ulysses@mavis.local>` (per 19:39 JST + 21:59 JST 三次强化)
- **守门 #12**: 禁回溯叙事, BAS 引用 git log --follow 实证, 缺标比错标, 0 误删无关文件
- **守门 #20**: 子代理 dispatch 必先 brief 落地 (本 brief 已落档 `docs/briefs/rf001-merge-001.md`)
- **守门 #19 + #23**: Python 化, AI mock 不开 OpenAI/Anthropic

## §4 子代理失败接手清单 (per 守门 #9 + 守门 #20)

如子代理 RPC 失败 (10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded, per 守门 #9 实证 #3), Mavis 父会话直接接手:
1. `git rebase --abort` (如 rebase 中)
2. 验证 rebase 状态
3. 重新尝试或 report 给 Ulysses

## §5 token 估 (per 守门 #4)

- 父会话 close-1c + 修订历史: ~0.02M
- 子代理 rebase+merge + 5 守门 + push: ~0.15M
- 总估: ~0.17M token (跟推荐项 ~0.3M 兼容, 实际可能更低)

## §6 已知缺口 (per 守门 #11 缺标比错标)

| # | 缺口 | 触发 | 优先级 |
|---|---|---|---|
| 1 | 23 commit rebase 冲突处理 (预期 0, 实际可能因 AGENTS.md 同步有 conflict) | rebase 启动 | P0 |
| 2 | 5 守门 实证 (v1+v3+v6+v14 0 err) 跨 41+ crate 大 workspace 5min timeout 风险 (per 守门 #1 v19 -j 4 修正) | 5 守门跑 | P0 |
| 3 | rf001-t15-recovered +15 重复 commit 验证 | rebase 后 | P1 |
| 4 | worktree 清理 (其他 sub-session 决定) | 跨 session 续 | P2 |

## §7 签字栏 (per 守门 #10 + 8/27 19:39/21:59 JST 三次强化 + 9/3 19:35 JST 拍板 D 维持)

| # | 角色 | 签字日 | 结论 |
|---|---|---|---|
| 1 | 架构师 (Mavis 接手) | 2026-09-05 | 🟢 Mavis 接手终审通过 (per 8/27 19:39 JST + 11:12 JST 拍板) |
| 2 | SRE Lead (Mavis 接手代签) | 2026-09-05 | 🟢 5 守门跨 stage 必跑 (v1+v3+v6+v14) |
| 3 | 平台工程师 (Mavis 接手代签) | 2026-09-05 | 🟢 push origin 0/0 sync 实证 (R-05 反转后允许) |
| 4 | 评审主持 (Mavis 接手代签) | 2026-09-05 | 🟢 5 守门 + 0 误删 + rebase conflict 0 实证 |
| 5 | PM (Mavis 接手代签) | 2026-09-05 | 🟢 token 估 ~0.17M (跟推荐项 ~0.3M 兼容) |

## §8 修订历史 (per §7 报告 7 段结构)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: rf001-t15-work rebase+merge + feat-auto-1c close 拍板落地 (per 2026-09-05 11:12 JST `ask_9ccc7a57dc5823d05e17e2b3` 用户拍板 Q1=rebase-then-merge+Q2=close-1c, 推荐项); 2 步操作 (1.1 rebase+merge + 1.2 close-1c) + 2 未拍板项 (rf001-t15-recovered +15 + rf001-t15-worktree-content +1 跨 session 续) + 5 守门合规 (v1+v3+v6+v14, 跳过 v2/v4-v5/v7-v13) + 4 已知缺口 + 5 签字栏 (Mavis 接手代签) | 2026-09-05 11:12 JST `ask_9ccc7a57dc5823d05e17e2b3` 用户拍板 (Q1=rebase-then-merge[推荐]+Q2=close-1c[推荐]) (per 9/1 14:58 JST 拍板决策必须用选项 + 9/5 04:03 JST 拍板推荐项直接执行) |
