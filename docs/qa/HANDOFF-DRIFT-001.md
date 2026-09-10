# HANDOFF-DRIFT-001 — QA-DRIFT-001 下游 AI 执行清单

> **报告版本**: v0.1
> **生成时间**: 2026-09-10
> **报告人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **来源**: `docs/qa/QA-DRIFT-001.md` §8.4（2026-09-10 上游 AI 回答）派生的下游 AI 可执行清单
> **触发**: 用户指令"github的issue和qa，本地qa都回答处理干净"

---

## 0. 使用说明

本文档收录 QA-DRIFT-001.md §8.4 中判定为 **"已消解" / "已解锁" / "已裁决"**（即答案已确定，无论是 Ulysses 直接拍板还是 2026-09-10 起改由 AI 代理拍板）的条目。原"待 Ulysses 拍板"的 3 项（Q3 / Q5 / Q8+Q12）已于 2026-09-10 由 AI 代理按 QA-DRIFT-001.md §8.4.8 已列明的推荐选项裁决（per Ulysses 指令"不再需要真人拍板流程，全部改为 ai 代理"），派生为下方 D5-D7，不再单列"不在本清单内"章节。

---

## 1. 下游 AI 可直接执行

### D1 — 同步 `ui-redesign-multica-style.md` §0.1 末行 canvas redirect 描述
**背景**（per QA-DRIFT-001 §8.4.1 A2）：`redirects.ts` 已改为 `/canvas/:id` 走独立路由 `app/canvas/[id]/page.tsx`，不再走 `?canvas=:id` 查询参数 redirect。
**动作**：把 `docs/frontend/design/ui-redesign-multica-style.md` §0.1 最后一条"`/canvas/:id` → `/projects?canvas=:id`"改写为反映当前实现（独立路由，非 redirect）。
**风险**：低，纯文本同步。

### D2 — 同步 settings tab 数（若 multica 设计书仍列 7 tab）
**背景**（per §8.4.1 A3）：实装 5 tab（profile/account/team/billing/apikeys）已拍板生效，DRIFT-α-007 深链修复已完成。
**动作**：检索 `ui-redesign-multica-style.md` 及相关设计书中 "/settings 7 tab" 描述，统一改为 5 tab，并注明 4 个未实装 redirect 目标（permissions/members/workspace/integrations）是已知 P2 功能缺口（非路由 bug）。
**风险**：低，纯文本同步。

### D3 — 修正状态机数字（WorkItem 6 态）
**背景**（per §8.4.3 C1）：`types/ids.ts:198-199` 实测 6 态（todo/in_progress/review/blocked/done/wontfix）。
**动作**：
- `docs/test-design.md` §2.1.1（行 179 附近）"3 态" → 改 "6 态"，列全 6 值。
- `docs/specs/domain-work-item-spec.md` 引 "basic-design §5.2 3 态" → 改引实际状态数（6 态），若 basic-design §5.2 本身也写 3 态需一并检查同步。
**风险**：低，纯文本同步，无功能变更。

### D4 — test-design 自指引用修正（批次）
**背景**（per §8.4.4 D1-D3，已确认 requirements.md §8.3/§27.6/§29.1 存在，basic-design §6.2.1 不存在）：
**动作**（`docs/test-design.md` 内，按文档自身 §0 v0.4/v0.5 修订记录中列出的位置逐一替换）：
1. T1 引用 "§6.2.1" → 改 "requirements.md §8.3"（line ~1053/1878 附近，Design Artifact / DSG-001/002 关联段）
2. T2 引用 "§6.3.3" → 改 "requirements.md §27.6"（line ~923/1879 附近，Test Level 关联段）
3. T3 引用 "§6.3.4" → 改 "requirements.md §29.1"（line ~939/1880 附近，Incident Record 关联段）
4. "VAL-001 验证 §6.2.1" → 改 "VAL-001 验证 §6.2"（basic-design 无 §6.2.1 子节，就近锚点为 §6.2，不新增子号）
5. S1-S5 同步点表格补齐上述已确认锚点的章节号列
**验证**：改完后 `grep -n "§6.2.1\|§6.3.3\|§6.3.4" docs/test-design.md` 应仅剩历史修订记录段（§0 revision history 保留原始措辞，不回溯改写，per 守门 #1 禁回溯叙事），正文引用段应全部指向 requirements.md 实际章节。
**风险**：低，纯文本同步，不改测试逻辑本身。

### D5 — IA 文档补全（22 顶级 + 6 (app) group，Q3 裁决派生）
**背景**（per QA-DRIFT-001 §8.4.8 Q3 已裁决，选项 a）：基准取**当前路由**（含 09-05 JST sprint/agent-view 重命名后状态），非 v0.1 审计时旧路由。
**动作**：核实当前 `frontend/app` 路由树，补齐"22 顶级 + 6 (app) group"IA 文档（对应位置：`ui-redesign-multica-style.md` 或其指定的 IA 章节），命名以重命名后的 sprint/agent-view 为准。
**风险**：中，需先枚举当前实际路由树再落文档，避免凭记忆臆造（per 守门 #11 缺标比错标）。

### D6 — /analytics tab 命名同步（Q5 裁决派生）
**背景**（per QA-DRIFT-001 §8.4.8 Q5 已裁决，code-authoritative）：权威命名为 Burndown/Gantt/Cost/Velocity/Leaderboard。
**动作**：检索设计书中 /analytics tab 命名的旧称，统一改为上述 5 个 code-authoritative 名称。
**风险**：低，纯文本同步。

### D7 — Local Runtime 状态数同步（Q8+Q12 裁决派生）
**背景**（per QA-DRIFT-001 §8.4.8 Q8+Q12 已裁决，选项 A）：5 态（registered/online/offline/compromised/revoked）为最终版；复核确认现行 test-design.md/frontend-design.md HEAD 均已无"8 边界状态"具名枚举，无真实功能缺口。
**动作**：检索 `docs/specs/domain-local-runtime-spec.md`（及其余仍引用"8 边界状态"字样的位置，若有）统一改为 5 态，并注明 RuntimeCommand 白名单仍为 8 种（不同概念，不可混淆：RuntimeCommand=8 种命令，RuntimeStatus=5 种状态）。
**风险**：低-中，需先 `grep -rn "8.*边界\|8 种边界"` 全库确认无遗漏引用，再统一替换，避免误改 RuntimeCommand 的合法"8 种"表述。

### H1 — Q14-32（19 条 test-design vs requirements P0）批量核实
**背景**（per §8.4.4 D4）：requirements.md 的 3 个关键锚点已确认存在，其余引用号需逐条核实。
**方法**：
1. 读取 `docs/qa/raw/gamma-testdesign-requirements.md` 和 `docs/qa/raw/delta-testdesign-crossref.md`，提取所有 "test-design 引用 §X" 条目。
2. 对每个 §X，`grep -n "^#### X\|^### X\|^## X"` 到对应目标设计书（requirements.md / basic-design.md / api-design.md 等）核实是否存在。
3. 存在 → 改 test-design.md 引用为正确文档+章节号；不存在 → 标注 `TBD（等 basic-design/api-design 补充）`，不得臆造章节号（per 守门 #11 缺标比错标）。
4. 13 处 tenant_id 端点声明维持 γ 报告原判定（5 条 PASS，其余按同方法核实）。
**风险**：中，条目多（19 条），需要与 raw 报告逐条对照，建议独立 session/子代理专项处理，预估 token 视 19 条平均核实成本，单条约 500-1500 token（读 raw 定位 + grep 核实 + 改文本），总计约 15K-30K token。

### H2 — P1（23 条）批量文本同步
按 QA-DRIFT-001 §8.4.6 分类执行：
- α P1 10 条中，007/009/017 已有 v0.2 部分修复记录，其余 6 条（006/008/010/013/016/018/019/021，注：021 为跨 5 个 page 的架构问题需要单独评估工作量）按 code-authoritative 规则同步设计书。
- β P1 2 条（011/013）为纯命名对齐，同步 spec/design 文档措辞。
- γ/δ P1 11 条并入 H1 同批次处理（同为 test-design cross-ref 类）。
**风险**：低-中，视具体条目而定；DRIFT-α-021（page 直接 useStore.setState 违反 §3.1 硬约束）可能涉及真实代码重构而非纯文档同步，执行前应重新读 `raw/alpha-frontend-drift.md` 确认是文档滞后还是代码违规，如是后者应视为独立技术债任务而非"文档同步"。

### H3 — P2（27 条）批量文本同步（低优先级）
按 QA-DRIFT-001 §8.4.7，全部为命名/小细节类，视 token 预算择机处理，无阻塞性。

---

## 2. 历史记录 — 原"待 Ulysses 拍板"3 项（已于 2026-09-10 由 AI 代理裁决）

| # | 问题 | 原需拍板内容 | 裁决结果 | 派生任务 |
|---|---|---|---|---|
| Q3 | "22 顶级 + 6 (app) group" IA 文档化 | 现在补 vs 缓办；基准取 v0.1 审计时路由还是当前（含 09-05 sprint/agent-view 重命名）路由 | 选项 a：现在补，基准取当前路由 | D5 |
| Q5 | /analytics tab 命名权威版本 | 是否采用 code-authoritative（Burndown/Gantt/Cost/Velocity/Leaderboard） | 采用 code-authoritative | D6 |
| Q8+Q12 | Local Runtime 状态数 5 vs 8 | 5 态为最终版（doc 改） vs 视为遗留 3 个未实装边界态需求（补功能） | 选项 A：5 态为最终版，无真实功能缺口 | D7 |

裁决依据：Ulysses 2026-09-10 指令"不再需要真人拍板流程，全部改为 ai 代理"；3 项均按 QA-DRIFT-001.md §8.4.8 原已列明的推荐选项执行，未偏离推荐项（per 守门 #28 拍板必带推荐项）。详见 QA-DRIFT-001.md §8.4.8 v0.4。

---

## 3. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：从 QA-DRIFT-001.md §8.4 派生 6 个下游可执行任务（D1-D4 + H1-H3）+ 3 项待拍板清单 | 用户指令"github的issue和qa，本地qa都回答处理干净" |
| v0.2 | 2026-09-10 | AI 代理（per Ulysses 指令"不再需要真人拍板流程，全部改为 ai 代理"）— 取代原真人拍板环节 | 原 §2"待 Ulysses 拍板"3 项（Q3/Q5/Q8+Q12）按 QA-DRIFT-001.md §8.4.8 已列明的推荐选项裁决，派生为 D5（IA 文档补全）/ D6（/analytics tab 命名同步）/ D7（Local Runtime 状态数同步）3 个可执行任务；§0 使用说明 + §2 表格同步改写为历史记录 | Ulysses 2026-09-10 指令"不再需要真人拍板流程，全部改为 ai 代理" |
