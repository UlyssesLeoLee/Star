# Brief: ContextPanel Tab 3 — 关联 + AI 助手 (worker-3 / cp-tab-relation)

> **per AGENTS.md 守门 #9 v20**: 子代理 dispatch 必先落 brief
> **author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签
> **触发**: 2026-09-06 07:34 JST Ulysses 拍板 Q1=C / Q2=B / Q3=A1 (接 domain-relation Graph 真实)

---

## §1 Objective & Why

**实装 ContextPanel Tab 3 "关联" + Tab 5 "AI 助手"** —— 本 worker 负责 2 个 tab,合并原因: AI 助手 tab 按 Q3=A1 拍板接 domain-relation 的 GraphNode/GraphEdge 真实数据,跟关联 tab 数据源同源,合 1 worker 减少跨 wt 协调成本。

**Why**:
- Tab 3 关联: per `ui-3pane-arch.md` §1.4 line 194 + `/relation` 路由并入项 (per §1.2.1 line 149),**完全没实装**。
- Tab 5 AI 助手: per ADR 0031 ContextGraph + §1.4 line 195,**0 实装**;Q3=A1 拍板接 `domain-relation` 的 GraphNode/GraphEdge (实测 `crates/domain-relation/src/lib.rs:511-540` 完整实体)。

---

## §2 已知事实

- `crates/domain-relation/src/lib.rs` 是单文件 57KB,含完整 8 层架构 + GraphNode/GraphEdge (line 511-540) + graph 子图查询 (line 1114+)。
- `GraphNode { id, node_type, label, properties }` + `GraphEdge { from, to, edge_type, weight }`。
- `Subgraph { root, nodes, edges }` 是查询结果类型。
- 5 wt 已建,本 worker wt = `feat/cp-tab-relation`。

---

## §3 Scope

**必做 (Tab 3 关联)**:
1. MSW handler: `frontend/src/mocks/handlers/relation.ts` — 4 端点 (GET /api/relations?work_item_id=X / POST /api/relations / DELETE /api/relations/:id / GET /api/relations/subgraph?root_id=X&depth=2)。
2. Mock data: 至少 3 种关系 (parent-child / blocks / relates-to) + 5 个 work-item 的子图样例。
3. `frontend/src/components/contextpanel/TabRelation.tsx`:
   - 关联列表 (按 edge_type 分组: 子项 / 依赖 / 阻塞 / 相关)
   - 添加关联 (类型下拉 + 目标 work-item 搜索)
   - 关联图谱小视图 (svg 渲染 Subgraph, ≤ 20 节点)
4. i18n 4 语言 × 6 key = 24 行。
5. 测试: `TabRelation.test.tsx` ≥ 4 测试 (render / add / delete / subgraph)。
6. 报告: `PHASE-CONTEXTPANEL-TAB3-RELATION-REPORT.md` 7 段。

**必做 (Tab 5 AI 助手)**:
1. `frontend/src/components/contextpanel/TabAi.tsx`:
   - 接 `/api/relations/subgraph?root_id={work_item_id}&depth=2` 拉关联子图
   - 展示"相似项 Top 5" (按子图节点数排序)
   - 展示"风险评估" (子图中有 blocks 关系 → 红色徽章)
   - 展示"建议" (基于 edge_type 推荐 3 个下一步操作, mock 静态模板)
2. i18n 4 语言 × 4 key = 16 行。
3. 测试: `TabAi.test.tsx` ≥ 3 测试 (render / load-subgraph / show-suggestion)。
4. 报告: `PHASE-CONTEXTPANEL-TAB5-AI-REPORT.md` 7 段 (或合并到 TAB3 报告)。

**不做**:
- ❌ 不实装 Tab 1/2/4。
- ❌ 不接真实 LLM (per Q3=A1 拍板只接 Graph,不开 OpenAI)。
- ❌ 不动 WorkItemDetailDrawer 主路径。

---

## §4 Deliverable

| 文件 | 估行 |
|---|---|
| `frontend/src/mocks/handlers/relation.ts` | ~150 |
| `frontend/src/mocks/data/relation.ts` | ~100 |
| `frontend/src/components/contextpanel/TabRelation.tsx` | ~400 |
| `frontend/src/components/contextpanel/TabAi.tsx` | ~250 |
| `frontend/src/components/contextpanel/__tests__/TabRelation.test.tsx` | ~120 |
| `frontend/src/components/contextpanel/__tests__/TabAi.test.tsx` | ~100 |
| `frontend/src/lib/i18n/{zh-CN,ja,en,ts}.ts` | +40 行 |
| `docs/reports/PHASE-CONTEXTPANEL-TAB3-RELATION-REPORT.md` | 7 段 |
| `docs/reports/PHASE-CONTEXTPANEL-TAB5-AI-REPORT.md` | 7 段 |

---

## §5 Acceptance Criteria

**Tab 3**:
- [ ] 4 MSW 端点全 200
- [ ] 关联列表按 edge_type 分组正常
- [ ] 添加 / 删除关联正常
- [ ] 关联图谱 SVG 渲染 ≤ 20 节点
- [ ] i18n 4 语言覆盖
- [ ] `pnpm test TabRelation` 4/4 pass
- [ ] 守门 4 项 0 err

**Tab 5**:
- [ ] Subgraph API 调用正常 (mock 数据)
- [ ] 相似项 Top 5 + 风险徽章 + 建议模板 3 块全显示
- [ ] 失败回退: API 504 → 显示"AI 助手暂不可用"占位
- [ ] `pnpm test TabAi` 3/3 pass

**通用**:
- [ ] commit author = Ulysses
- [ ] 2 commit (Tab 3 + Tab 5),message 各自引用 brief
- [ ] 7 段报告 × 2,含 §3 已知缺口 (Tab 1/2/4 由其他 worker 接手)

---

## §6 依赖 + 协调

- **依赖 worker-1**: `ContextPanel.tsx` 5 slot 约定
- **协调 worker-5 (批 2)**: worker-5 做 5 页面 2-Pane 改造,会复用本 wt 的 TabRelation 组件 — worker-5 brief 显式引用本 wt `TabRelation.tsx`

---

## §7 Risk

| 风险 | 缓解 |
|---|---|
| GraphNode/GraphEdge 字段跟前端 type 不一致 | 必先 `grep` `crates/domain-relation/src/lib.rs` 511-540 段校对,前端 type 用 GraphNode interface mirror |
| SVG 渲染 > 20 节点性能 | 子图查询默认 depth=2,前端 limit 20,超出折叠 |
| worker-5 复用 TabRelation 时的 prop 兼容 | 本 wt TabRelation 暴露 `workItemId` + `onAdd` + `onDelete` 3 个标准 prop |
| 2 commit 跟 worker-1/2 在 main 链上冲突 | 5 wt 互不重叠,本 wt 只动 `contextpanel/TabRelation.tsx` + `contextpanel/TabAi.tsx` + `mocks/handlers/relation.ts` + `mocks/data/relation.ts` |
