# Brief: ContextPanel Tab 2 — 评论 (worker-2 / cp-tab-comment)

> **per AGENTS.md 守门 #9 v20**: 子代理 dispatch 必先落 brief
> **author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签
> **触发**: 2026-09-06 07:34 JST Ulysses 拍板 Q1=C / Q2=B

---

## §1 Objective & Why

**实装 ContextPanel Tab 2 "评论"** —— per `ui-3pane-arch.md` §1.4 line 193 设计,5 tab 之一,接 `domain-comment` InMemory Service。当前 `WorkItemDetailDrawer.tsx` line 561-563 自承"暂无评论 (Phase 2+ 接 store.comments 写)",**完全没实装**。

**Why**: 评论是协作核心;5 tab 容器是 worker-1 (cp-tab-property) 拍板的,本 worker 在 worker-1 合并后启动,本 worker 走独立 wt 不依赖 worker-1 commit (因为本 wt 也建自己的 ContextPanel 容器,但要从 worker-1 wt 拉 5 slot 约定)。

---

## §2 已知事实

- `crates/domain-comment/src/{entity,error,event,invariants,macros,port,service,value_object}.rs` 8 文件骨架在。
- `domain-comment::port::CommentPort` trait 暴露: `create_comment` / `list_comments` / `update_comment` / `add_reaction` / `delete_comment`。
- `domain-comment::service::InMemoryCommentService` 已存在 (InMemory store)。
- 没有 HTTP server,前端走 MSW (Q1=B1 拍板)。
- 5 wt 已建,本 worker wt = `feat/cp-tab-comment`。

---

## §3 Scope

**必做**:
1. MSW handler: `frontend/src/mocks/handlers/comment.ts` — 5 端点 (GET /api/comments?work_item_id=X / POST /api/comments / PATCH /api/comments/:id / POST /api/comments/:id/reactions / DELETE /api/comments/:id)。
2. MSW 注册: `frontend/src/mocks/handlers/_index.ts` 追加 5 handler。
3. Mock data: `frontend/src/mocks/data/comment.ts` — 至少 5 条样例 (含 @mention / reaction / 内部评论)。
4. 新建 `frontend/src/components/contextpanel/TabComment.tsx`:
   - 评论列表 (时间倒序, 分页 20/页)
   - 公开 / 内部评论 Tab 切换 (per spec `ui-3pane-arch.md` §1.4 "公开 + 内部 / @ 提及 / 反应")
   - 撰写框 (markdown, @提及, Ctrl+Enter 提交)
   - Reaction picker (emoji 6 个常用)
   - 删除 / 编辑 (本人/PM 可见)
5. i18n 4 语言 × 8 key = 32 行 (per STAR-I18N-EXTRACT.json)。
6. 测试: `TabComment.test.tsx` ≥ 5 测试 (render / post / edit / delete / reaction)。
7. 报告: `PHASE-CONTEXTPANEL-TAB2-COMMENT-REPORT.md` 7 段。

**不做**:
- ❌ 不实装 Tab 1/3/4/5 — slot 留空 + EmptyTabPlaceholder。
- ❌ 不起 HTTP server — 走 MSW。
- ❌ 不动 `WorkItemDetailDrawer.tsx` 主触发逻辑 — worker-1 改,本 wt 读 worker-1 合并后的 `ContextPanel.tsx` 接口。

---

## §4 Deliverable

| 文件 | 估行 |
|---|---|
| `frontend/src/mocks/handlers/comment.ts` | ~120 |
| `frontend/src/mocks/data/comment.ts` | ~80 |
| `frontend/src/mocks/handlers/_index.ts` | +5 行 |
| `frontend/src/components/contextpanel/TabComment.tsx` | ~350 |
| `frontend/src/components/contextpanel/__tests__/TabComment.test.tsx` | ~150 |
| `frontend/src/lib/i18n/{zh-CN,ja,en,ts}.ts` | +32 行 |
| `docs/reports/PHASE-CONTEXTPANEL-TAB2-COMMENT-REPORT.md` | 7 段 |

---

## §5 Acceptance Criteria

- [ ] MSW 5 端点全 200, curl 测试通过
- [ ] TabComment 列表 / 撰写 / reaction / 删除 全功能
- [ ] 公开 / 内部 tab 切换正常
- [ ] i18n 4 语言 key 全覆盖
- [ ] `pnpm test` 5/5 pass + 现有测试 100% 兼容
- [ ] 守门 4 项 0 err
- [ ] commit author = Ulysses, 引用 brief + 自动化档
- [ ] 7 段报告含 §3 已知缺口 (Tab 1/3/4/5 由其他 worker 接手)

---

## §6 依赖

- **依赖 worker-1**: `ContextPanel.tsx` 5 slot 约定 (从 worker-1 wt 拉分支 → merge 5 slot API 后再实装 Tab 2)
- **不依赖**: domain-comment Service 8 文件 (已存在,直接通过 port trait 调用)

---

## §7 Risk

| 风险 | 缓解 |
|---|---|
| worker-1 5 slot API 跟本 worker 期望不一致 | 协调机制: 本 worker 等 worker-1 commit 后,从 worker-1 wt (`feat/cp-tab-property`) `git fetch` + cherry-pick ContextPanel.tsx 到本 wt 再实装 |
| MSW handler 跟 worker-3 relation handler `_index.ts` 注册冲突 | 注册顺序按字母: comment → relation → activity → ai |
| domain-comment port trait 字段类型跟前端 mock 不匹配 | 必先读 `crates/domain-comment/src/port.rs` + `entity.rs` 校对字段名 |
