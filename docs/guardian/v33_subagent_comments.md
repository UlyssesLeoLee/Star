# v33 子代理任务卡留言机制 (per 守门 #9 v20 升级 + Jira 式异步需求注入)

> **Status**: 🟡 **Draft v0.1** (per 2026-09-10 19:54 JST Mavis 自驱, 待 Ulysses 拍板激活)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 Mavis 永久代签)
> **关联 commit**: 待落档
> **编号避让**: v32 已被 `v32_audit_boundary.md` 占用 (Mavis 审核 author=Ulysses 政策), v33 落到下一个空号位
> **守门基线**: 守门 #1+#9+#11+#12 v15+#13+#14 v3+#14 v4 7 项必过 (跟 #1 v15 docs 同步饱和联动, 1 commit 多文件)

---

## 0. 问题陈述 (Problem Statement)

**多子代理并行**的 3 大痛点 (per 2026-09-10 19:53 JST Ulysses 设计输入):

1. **跑重**: A 子代理做的事跟 B 重复, 浪费 token
2. **缺乏交流**: A 子代理改了 X, B 子代理不知道,后续基于过期 context 跑
3. **交流成本**: 子代理之间直接 RPC = O(N²) token, 根 session 中转 = O(N), 但根 session 也有 context 上限

**根因**: 子代理 dispatch 之后, 缺乏"异步需求注入管道"——A 想告诉 B 任何信息,要么 A 等 B 回应(同步,O(N²)),要么写 commit message 字符串(脆弱,无结构)。

**Jira 留言机制** = 完美的解药:每任务独立 thread,留言 = 需求/约束追加,@mention 触发自动通知,BLOCK 留言强制 enforce。

---

## 1. 设计方案 (Solution)

### 1.1 物理存储

每个 `task_id` 配套 1 个 append-only 留言文件:

```
docs/briefs/<task_id>.comments.jsonl   # JSON Lines, per 守门 #13 Transaction T
```

### 1.2 留言 Schema (per 评论 thread)

```json
{
  "id": "comment_001",
  "ts": "2026-09-10T19:55:00.000+09:00",
  "author": "Mavis",
  "actor_role": "orchestrator|sub-agent|architect|user|Ulysses",
  "body": "请在 src/domain/lib.rs 加 X 字段 (per Y spec §3.2)",
  "mentions": ["P3-D.6-1-2", "P3-D.6-1-3"],
  "blocks": false,
  "tags": ["requirement", "follow-up"],
  "parent_comment_id": null,
  "refs": ["docs/specs/X-spec.md §3.2", "commit_abc123"]
}
```

### 1.3 触发行为 (4 类)

| 留言属性 | 触发效果 | 实现 |
|---|---|---|
| **`@<task_id>` mention** | 被 @ 的 task 下次 brief 自动 append "Read COMMENTS" 段 | dispatcher.comment() 写 + brief() 拉 |
| **`blocks: true`** | 子代理 invoke **前必扫留言**, 如有 BLOCK 留言 → 不 invoke, 抛 `BlockedByCommentError` | dispatcher.invoke() 前 check |
| **`actor_role: "architect" \| "Ulysses"`** | 高优先级, 子代理 commit message 必引 comment_id | 强制 `comments-read: <id>` 字段 |
| **`tags: ["requirement"]`** | 当作新需求 (per 守门 #1 禁回溯) | 写入 WBS 任务卡 |
| **`tags: ["follow-up"]`** | 提示后续 task 该读 | 跟 #4 中转等价 |

### 1.4 读取时机 (3 阶段)

| 阶段 | 读取点 | 行为 |
|---|---|---|
| **brief 落档** | `dispatcher.brief()` | 拉所有 `@<this_task_id>` 的留言 → append 到 brief "Read COMMENTS" 段 |
| **invoke 前** | `dispatcher.invoke()` | 拉所有 `blocks: true` 未解除的留言 → 有 → 抛 `BlockedByCommentError`, 不 invoke; 0 → 继续 |
| **verify 时** | `dispatcher.verify()` | 检查 commit message 含 `comments-read: <id>` 字段 → 缺 → warn log (per 守门 #11 缺标比错标) |

### 1.5 brief 模板 (5 段 + 新增第 6 段 "Read COMMENTS")

```markdown
# <task_id>

## 1. Read FIRST (必读, 3-5 个文件路径)
- docs/briefs/<前序_task_id>.md
- docs/specs/<相关域>-spec.md
- crates/<相关 crate>/src/<关键文件>.rs line X-Y

## 6. Read COMMENTS (任务卡留言, 必读, 3-5 条)
- comment_001 by Mavis @ 2026-09-10 19:55: "请加 X 字段 (per Y spec)"
- comment_002 by Ulysses @ 2026-09-10 20:10: "BLOCK: 等 P3-D.6-1-1 完"
- comment_003 by Mavis @ 2026-09-10 20:30: "tag=requirement, 跟 Y spec"

## 2. DO  ## 3. DON'T  ## 4. Output  ## 5. Acceptance
(... 同前)
```

### 1.6 commit message 格式 (加 comments-read 字段)

```
feat(agent-domain): domain layer (per P3-D.6-1-1)
[原有字段]
comments-read: comment_001,comment_003  (per 守门 #14 v3 Mavis 永久代签)
comments-blocked: 0  (per 守门 #9 v20 brief 必先)
brief: docs/briefs/P3-D.6-1-1.md
```

---

## 2. 跟现有守门的关系 (Integration)

| 守门 | 跟留言机制的关系 |
|---|---|
| **#1 v15 docs 同步饱和** | 留言写 `docs/briefs/<id>.comments.jsonl`, 1 commit 多文件 (跟 brief 一起) |
| **#9 v20 brief 必先** | dispatcher 强制 brief 含 "Read COMMENTS" 段 |
| **#9 RPC 不可靠** | BLOCK 留言存 jsonl, 跨进程可见, 不依赖 RPC |
| **#9 commit 散落** | commit message 含 comments-read 字段, 根 session merge 时可追溯 |
| **#11 缺标比错标** | BLOCK 留言状态显式列, 缺标 = 缺 BLOCK 留言 = 子代理乱跑 |
| **#13 Transaction T** | comments.jsonl append-only, 不 truncate |
| **#14 v3 Mavis 永久代签** | 留言 author 默认 Mavis, commit author 一致 |
| **#27 RPC fallback** | BLOCK 留言 = 0 RPC 也能 enforce (本地读 jsonl) |

跟 v32 (Mavis 审核 author=Ulysses 政策) **不冲突**: v32 是 commit 审核, v33 是子代理协调。两者互补。

---

## 3. 落地清单 (Deliverables)

| # | 文件 | 改动 | 估 LOC |
|---|---|---|---|
| 1 | `docs/guardian/v33_subagent_comments.md` | 新增 (本文件) | 200 |
| 2 | `scripts/automation/dispatcher.py` | 扩 4 方法: `comment() / list_comments() / check_blocked() / mark_read()` + brief 加 "Read COMMENTS" 段 + invoke 前 check | +120 |
| 3 | `scripts/automation/guardian/tests/test_dispatcher_comments.py` | 新增 10+ TC | 250 |
| 4 | `scripts/automation/guardian/tests/test_e2e_dispatcher.py` | 扩 BLOCK 留言 → 抛异常 | +30 |
| **总计** | **4 文件** | **1 commit 多文件** (per #1 v15) | **~600 LOC** |

**估 token**: ~0.3M (per PreToolUse 守门 1:1 节省率, 设计已收敛)
**估时间**: ~15 min

---

## 4. 激活条件 (Activation)

per 守门 v3x 候选激活流程 (per AGENTS.md §4.1.1 + 9/1 14:58 + 9/8 16:08):

1. Mavis 走 `ask_user` 必带推荐项 (per 守门 v28 格式)
2. Ulysses 拍板 (激活 / 不激活 / 改方案)
3. 拍板后立即执行 (per 9/5 04:03)
4. commit author=Ulysses (per守门 #10 + 守门 #14 v3)
5. 修订历史表 +1 行 (per AGENTS.md §3 7 段结构)
6. WBS v0.X+1 升版同步 (per 守门 #12 v21 [P] docs 同步)

**当前状态**: 🟡 Draft v0.1, Mavis 自驱设计稿落档, 待 Ulysses 拍板激活。

---

## 5. 已知缺口 (per 缺标比错标)

| # | 缺口 | 严重度 | 缓解 |
|---|---|---|---|
| 1 | 留言 author 假冒风险 (Mavis 留言可被伪造成 Ulysses) | P1 | author 字段走 git commit author 模式, brief 必含 author 验证 |
| 2 | 留言无签名 (任何进程可改 jsonl) | P1 | 后续 v0.2 加 GPG 签名 (per 守门 #5 派生) |
| 3 | BLOCK 留言解除机制不明确 | P2 | 由 actor 写 `blocks: false` 的新留言显式解 block |
| 4 | 留言超过 1000 条时性能 | P2 | 后续 v0.2 加 index 索引 |
| 5 | 子代理 LLM 是否真读 "Read COMMENTS" 段无法强制 | P2 | verify 阶段检查 commit message 含 `comments-read:` 字段, 软约束 |

---

## 6. 修订履歴

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 19:54 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | 初版落档, 6 段 (问题/设计/守门整合/落地/激活/缺口+修订), 4 落地文件 ~600 LOC, 4 类触发行为 (@mention / BLOCK / architect-priority / tags), 3 阶段读取时机 (brief/invoke/verify), 编号避让 v32 落到 v33 | 2026-09-10 19:53 JST Ulysses 拍板"巧妙运用 jira 式任务卡内留言机制" + 19:54 JST "可以" |
