# Brief: ContextPanel Tab 4 — 活动 (worker-4 / cp-tab-activity, 批 2)

> **per AGENTS.md 守门 #9 v20**: 子代理 dispatch 必先落 brief
> **author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签
> **触发**: 2026-09-06 07:34 JST Ulysses 拍板 Q1=C / Q2=B / Q4=B (2 批 3+2)

---

## §1 Objective & Why

**实装 ContextPanel Tab 4 "活动"** —— per `ui-3pane-arch.md` §1.4 line 194,接 `domain-audit` InMemory Service,展示 work-item 的状态变更 / 字段修改 / 通知已读 记录。

**Why**: 当前 0 实装,审计是协作可追溯性核心;批 2 启动原因 — 等 worker-1 合并 + worker-2/3 commit 落 main 链后,Tab 4 才能引用已稳定的 ContextPanel 5 slot API。

---

## §2 已知事实

- `crates/domain-audit/src/lib.rs` (48KB) 已含 8 层架构 (entity/error/event/invariants/macros/port/service/value_object + lib)。
- `domain-audit::port::AuditPort` 暴露: `record_event` / `list_events_by_target` / `list_events_by_actor`。
- Event 类型 5 种: WorkItemCreated / WorkItemStatusChanged / WorkItemFieldUpdated / NotificationRead / CommentCreated (跨域事件)。
- 5 wt 已建,本 worker wt = `feat/cp-tab-activity`,**批 2 启动** (等批 1 合并)。

---

## §3 Scope

**必做**:
1. MSW handler: `frontend/src/mocks/handlers/audit.ts` — 2 端点 (GET /api/audit/events?target_type=work_item&target_id=X / GET /api/audit/events?actor_id=X)。
2. Mock data: 至少 20 条样例 (覆盖 5 种 event_type,时间跨度 7 天)。
3. `frontend/src/components/contextpanel/TabActivity.tsx`:
   - 时间线倒序列表
   - 按 event_type 分组筛选 (5 chips: 全部/创建/状态变更/字段修改/通知/评论)
   - 按 actor 筛选
   - 详情展开: 显示 diff (old → new)
4. i18n 4 语言 × 5 key = 20 行。
5. 测试: `TabActivity.test.tsx` ≥ 4 测试 (render / filter / diff-expand / empty-state)。
6. 报告: `PHASE-CONTEXTPANEL-TAB4-ACTIVITY-REPORT.md` 7 段。

**不做**:
- ❌ 不实装 Tab 1/2/3/5。
- ❌ 不实现 export (PDF/CSV) — Phase 2+ 缺口。
- ❌ 不动 WorkItemDetailDrawer。

---

## §4 Deliverable

| 文件 | 估行 |
|---|---|
| `frontend/src/mocks/handlers/audit.ts` | ~80 |
| `frontend/src/mocks/data/audit.ts` | ~120 |
| `frontend/src/components/contextpanel/TabActivity.tsx` | ~300 |
| `frontend/src/components/contextpanel/__tests__/TabActivity.test.tsx` | ~120 |
| `frontend/src/lib/i18n/{zh-CN,ja,en,ts}.ts` | +20 行 |
| `docs/reports/PASE-CONTEXTPANEL-TAB4-ACTIVITY-REPORT.md` | 7 段 |

---

## §5 Acceptance Criteria

- [ ] 2 MSW 端点全 200
- [ ] 时间线倒序正常
- [ ] 5 chips 筛选正常
- [ ] diff 展开显示 old → new
- [ ] i18n 4 语言覆盖
- [ ] `pnpm test TabActivity` 4/4 pass
- [ ] 守门 4 项 0 err
- [ ] commit author = Ulysses, 引用 brief + 自动化档
- [ ] 7 段报告含 §3 已知缺口 (Tab 1/2/3/5 已由其他 worker 接手,本 wt 不重做)

---

## §6 启动条件 (批 2)

- ✅ worker-1 (cp-tab-property) commit 已 merge 到 main 链
- ✅ worker-2 (cp-tab-comment) commit 已 merge 到 main 链
- ✅ worker-3 (cp-tab-relation) commit 已 merge 到 main 链
- ✅ ContextPanel.tsx 5 slot API 稳定

**启动机制**: Mavis 接手在批 1 验证通过后,cherry-pick 批 1 三 commit 到本 wt base 后再派 worker-4。

---

## §7 Risk

| 风险 | 缓解 |
|---|---|
| 批 1 三 wt 合并冲突 | 5 wt 互不重叠文件 (per 守门 §3 + §4.6 互不通信规则) |
| domain-audit 5 种 event_type 字段差异 | 必先 `grep` `crates/domain-audit/src/lib.rs` 列全 event_type 列表,前端 type 用 discriminated union |
| 时间线性能 (> 1000 条) | MSW 分页 50/页,前端 virtualized list (react-window 或自实现) |
