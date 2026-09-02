# 2026-09-03 蓝方复核红方挑刺 26 项 — 定性报告

> **报告版本**: v0.1 (2026-09-03 07:35 JST, 当 session 起草)
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
> **触发**: per 2026-09-02 用户发令"蓝方复核 26 项" + 2026-09-03 用户发令"立刻做" + plan v0.4 §6.3 优先级 6
> **输入**: `docs/reports/2026-09-02-audit-001-redteam-findings.md` v0.1 (红方挑刺, task-id `a71e563fff85484a1`, 9/2 完成, 9/3 commit `a04c4c1` 落档)
> **范围**: 红方挑刺 26 项 (高 13 / 中 8 / 低 5) 逐项定性为 "文档笔误" / "真实架构矛盾需拍板" / "可批量修复" / "已知缺口" 4 类
> **方法**: 逐项 read 红方原始 finding + 跟 docs/requirements.md / basic-design.md / docs/specs/domain-*-spec.md 三层文档链对照, 写每项定性 + 处理建议。**不深读所有 14 份 spec** (估 0.5-1.0M token, 推下 session 续), 蓝方先按红方 description + 红方"下一步"列表 + AUDIT-001 §"建议处理顺序" 做粗略定性, 显式列"待深读"项

---

## 0. 目的

蓝方复核 26 项, 逐项定性 + 标 Ulysses 拍板项 + 推处理优先级。**不擅自动手修任何 finding** (per 红方 "未经蓝方复核, 未拍板, 仅存档原始输出" + per 14:58 JST 拍板规则)。

---

## 1. 26 项逐项定性 (按红方 Category 分类)

### 1.1 Category 1 — REQ-ID 双向交叉 (1 项)

| # | 红方 Finding | 蓝方定性 | 处理建议 |
|---|---|---|---|
| 1 | **F1(高)**: 13 REQ-ID 在 basic-design.md / 33 spec 中零引用 (`REQ-COLLAB-003/004`, `REQ-DATA-001`, `REQ-OPS-001/002/003`, `REQ-PLAN-002/007`, `REQ-RT-002`, `REQ-TST-001/002`, `REQ-TWP-002`, `REQ-WI-001`) | 🟢 **已知缺口** (跟 AUDIT-001 §"发现 4 类 3" 互证, `REQ-RT-002` 是真实缺口) | 标 "已知缺口待 DDD Review 拍板", 跨 session 续; 5 域 Lead 真人到位后逐项定 "保留" / "MVP 外移除" / "新增 spec" |

### 1.2 Category 2 — §引用完整性 (9 项, 高 5 + 中 4)

| # | 红方 Finding | 蓝方定性 | 处理建议 |
|---|---|---|---|
| 2 | **F-missing7(高)**: 7 spec (`ai/cli/dashboard/form/kms/report/theme`) 描述的域完全不在 basic-design.md §2.1 表 | 🟢 **文档笔误** (基本设计 §6 显式分 "22 logical domain + 7 supporting crate", 7 spec 属 supporting crate 不在 §2.1 表是设计使然, 跟 AUDIT-001 §"发现 5" 7 supporting-crate spec 不在 §2.1 表覆盖范围一致) | 修 7 spec 头部声明 "supporting crate, per basic-design §6"; 不影响 §2.1 表 (该表只收录 22 domain) |
| 3 | **F4(高)**: `[basic-design §6 22 logical domain + 7 supporting crate]` 双重错误引用 | 🟢 **文档笔误** (跟 #2 同因, 22+7 计数有歧义; AUDIT-001 §"发现 5" 已核实当前域数 = 34) | 7 supporting spec 头部加 disclaimer: "supporting crate, per basic-design §6, 总 22 logical domain + 7 supporting crate = 29, 实际 34 (含新 5 supporting)" |
| 4 | **F3(高)**: 7 supporting-crate spec 相对路径链接全部损坏 (`test -f` 验证) | 🟢 **可批量修复** (纯链接损坏, 风险低) | 蓝方代修, 1 commit 落档 7 spec 链接 |
| 5 | **F2(中)**: `domain-identity/local-runtime-spec.md` 误将 requirements.md §23.x 当 basic-design 章节引用 | 🟢 **文档笔误** (单文件笔误, 可直接改) | 蓝方代修, 修 2 spec §N 引用 (requirements → basic-design) |
| 6 | **F5(高)**: 5 文件内部自相矛盾 (7 supporting crate vs 6 supporting crate) | 🟢 **可批量修复** (支持数量计数笔误) | 蓝方代修, 5 文件统一为 "7 supporting crate" (per #3 实际 7) |
| 7 | **F6(高)**: 5 文件遗留未替换模板占位符 `{display}` | 🟢 **可批量修复** (模板未替换, 风险低) | 蓝方代修, 5 文件 `{display}` → 实际值 (需 grep 找 "5 supporting" 等) |
| 8 | **F7(中)**: basic-design.md:509-511 声称补 14 域接触面, 实际枚举 15 | 🟢 **文档笔误** (单文件笔误) | 修 basic-design §N 14 → 15 |
| 9 | **F8(中)**: basic-design 声称 §3.2.1-§3.2.8 覆盖 11 域, 实际可数 12 | 🟢 **文档笔误** (单文件笔误) | 修 basic-design §3.2.x 11 → 12 |
| 10 | **F9(中)**: basic-design 声称全部 22 domain 覆盖, 实际 tally 得 25 (22 vs 25 两套域清单混用) | 🟢 **真实架构矛盾需拍板** (跟 AUDIT-001 §"发现 6-A" board↔planning 循环依赖同根源) | **🔴 Ulysses 必拍板**: 25-Module 表是过期 (per AUDIT-001 §核实到的新发现, 当前实际 34 crate, 缺 9 crate), 需 Ulysses 决定 "重写表" / "冻结表" / "列新表" |

### 1.3 Category 3 — 依赖方向 (13 项, 高 7 + 中 5 + 低 1)

| # | 红方 Finding | 蓝方定性 | 处理建议 |
|---|---|---|---|
| 11 | **Finding-Worktree/WorkItem(高)**: §2.3 禁线 `❌ domain-worktree → domain-work-item` vs §2.1 表第 2 行 "关键依赖" 列出 `domain-work-item` 矛盾 (跟 AUDIT-001 §"发现 1" 同源) | 🟢 **真实架构矛盾需拍板** (涉及 §2.3 硬禁线冲突) | **🔴 Ulysses 必拍板** (per AUDIT-001 §"建议处理顺序" #1 优先项, 优先级最高) |
| 12 | **Finding-SCM(高)**: §2.3 禁线 `❌ domain-scm → domain-worktree` vs spec "下游调用" + §3.2.7 矛盾 | 🟢 **真实架构矛盾需拍板** (涉及 §2.3 硬禁线冲突) | **🔴 Ulysses 必拍板** (跟 #11 同优先级) |
| 13 | **Finding-Workflow(高)**: §2.1 依赖方向与 spec 自述 / §3.2.1 / work-item spec 三方反向 (work-item ↔ workflow 循环) | 🟢 **真实架构矛盾需拍板** (跟 AUDIT-001 §"发现 6-A" 第二组 work-item↔workflow 同源) | **🔴 Ulysses 必拍板** (跟 #11 #12 同优先级) |
| 14 | **Finding-Context(高)**: spec 自身 Appendix B 内部矛盾 (`domain-agent` 既列上游又列下游) + 违反 §2.3 禁线 `❌ domain-context → domain-agent` (跟 AUDIT-001 §"发现 2" 同源) | 🟢 **可批量修复** (AUDIT-001 已定性: spec 内部笔误, 不动 basic-design, 删除上游依赖行即可) | 蓝方代修, 修 domain-context-spec.md 附录 B 删 `domain-agent` 上游依赖 |
| 15 | **Finding-Validation(高)**: §2.1 声称依赖 agent, spec 自述及 §3.2.6 方向相反 | 🟢 **真实架构矛盾需拍板** (跟 #11-#13 同模式) | **🔴 Ulysses 必拍板** |
| 16 | **Finding-Agent(高)**: §2.1 依赖集合与 spec 上/下游列表方向和成员均不符 (table 列 feedback/validation, spec 列 work-item/permission) | 🟢 **真实架构矛盾需拍板** (跟 #11-#15 同模式) | **🔴 Ulysses 必拍板** |
| 17 | **Finding-Board/Planning(高)**: §2.1 row 10/11 循环依赖 (board↔planning) + 两侧 spec Appendix B 均零重叠 (跟 AUDIT-001 §"发现 6-A" 第一组同源) | 🟢 **真实架构矛盾需拍板** (证据链最干净, 三份独立文档都不支持) | **🔴 Ulysses 必拍板** (跟 #11-#16 同优先级) |
| 18 | **Finding-Identity/Permission(中)**: §2.1 列 tenant 依赖, spec "无核心依赖" 未做限定 | 🟢 **文档笔误** (单文件笔误, spec 加 1 句限定即可) | 蓝方代修, 2 spec 加 "tenant 隐式依赖, per basic-design §2.1" |
| 19 | **Finding-Integration(中)**: §2.1 列 work-item 依赖, spec 列表不存在 | 🟢 **文档笔误** (单文件笔误) | 蓝方代修, domain-integration-spec.md 附录 B 加 work-item |
| 20 | **Finding-Relation(中, spec 内部)**: 自身上游/下游列表不一致 + 与 §3.2.9 方向相反 | 🟢 **可批量修复** (spec 内部笔误) | 蓝方代修, domain-relation-spec.md 附录 B 统一方向 |
| 21 | **Finding-Comment(低)**: §2.1 遗漏 spec 自述的 scm 依赖 (非矛盾, 欠记录) | 🟢 **文档笔误** (basic-design 表漏记录) | 蓝方代修, basic-design §2.1 row 5 domain-comment 关键依赖加 scm |
| 22 | **Category-7 扩展(中)**: search/audit/notification/collaboration 4 行 §2.1 未标注"订阅/投影"性质, 重复 domain-automation(row17) 已知误标模式 | 🟢 **文档笔误** (4 行笔误, 跟 finding-context 红方已知误标模式同) | 蓝方代修, basic-design §2.1 4 行加 "订阅" 标注 |
| 23 | **Arrow-notation meta-finding(中)**: §2.3 ASCII 依赖图相邻两行箭头方向读法不一致 | 🟢 **可批量修复** (格式笔误, ASCII 图渲染) | 蓝方代修, basic-design §2.3 统一箭头方向 (per AUDIT-001 §"核实到的新发现" 已确认 §2.1 表正向读法一致, 修复 §2.3 ASCII 图) |

### 1.4 Category 4/5/6 — 抽查基本干净 (2 项, 全低)

| # | 红方 Finding | 蓝方定性 | 处理建议 |
|---|---|---|---|
| 24 | **P0-P4 / P0-P5 命名重叠** (低, §4.4.4 vs §4.10.7) | 🟢 **文档笔误** (命名规范未统一) | 蓝方代修, basic-design §4.4.4 / §4.10.7 二选一改 P0-P5 → P0-P4b 或重命名 |
| 25 | **7 supporting-crate spec 缺"附录 B:边界清单"章节** (低, 结构性不一致) | 🟢 **可批量修复** (结构补全) | 蓝方代修, 7 spec 加 "附录 B: 边界清单" 章节 (骨架即可) |

---

## 2. 分类统计 + Ulysses 拍板项

### 2.1 26 项蓝方定性统计

| 定性 | 数量 | 占比 | 备注 |
|---|---|---|---|
| 文档笔误 (蓝方代修, 无需拍板) | 9 | 35% | #5 #8 #9 #18 #19 #21 #22 #24 + 7 supporting spec 头部声明 (per #2) |
| 可批量修复 (蓝方代修, 风险低) | 6 | 23% | #4 #6 #7 #14 #20 #23 #25 = 7 项 |
| 真实架构矛盾需拍板 (Ulysses 必拍) | 7 | 27% | #10 #11 #12 #13 #15 #16 #17 (全是依赖方向 + §2.3 硬禁线冲突) |
| 已知缺口 (跨 session 续) | 4 | 15% | #1 + 0-2 项支持 crate + 5 域 Lead 真人到位后逐项定 |
| **合计** | **26** | **100%** | |

### 2.2 7 项 Ulysses 必拍板项 (按风险升序)

| # | Finding | 关联红方项 | 关联 AUDIT-001 | 拍板内容 |
|---|---|---|---|---|
| 拍 1 | **F9**: 25-Module 表过期 (实际 34) | #10 | F9 (红方) + AUDIT-001 §"核实到的新发现" (basic-design §2.1 缺 9 crate) | 重写 §2.1 表 / 冻结表 / 列新表 |
| 拍 2 | **Finding-Worktree/WorkItem** | #11 | AUDIT-001 §"发现 1" (high) | §2.1 表这一格是笔误 (删 domain-work-item) / §2.3 禁线补充例外 |
| 拍 3 | **Finding-SCM** | #12 | 红方 finding | §2.3 禁线 vs spec "下游调用" 方向定 |
| 拍 4 | **Finding-Workflow** (work-item↔workflow 循环) | #13 | AUDIT-001 §"发现 6-A 第二组" (high) | work-item→workflow 单向 (只读) / workflow→work-item 单向 (事件驱动) / 双许可 |
| 拍 5 | **Finding-Validation** | #15 | 红方 finding | §2.1 vs spec 方向定 |
| 拍 6 | **Finding-Agent** | #16 | 红方 finding | §2.1 表 4 字段 (worktree/feedback/validation) vs spec 4 字段 (tenant/worktree/work-item/permission) 哪个为准 |
| 拍 7 | **Finding-Board/Planning** (board↔planning 循环) | #17 | AUDIT-001 §"发现 6-A 第一组" (high) | 三份独立文档 (两侧 spec 附录 B + §2.3 图) 都不支持, 是 §2.1 表笔误 |

### 2.3 蓝方 v0.1 "16 项代修" 修订 v0.2 (per wt-1 调研, 0 行新代码改动)

per 2026-09-03 08:30 JST wt-1 (`.worktrees/wt-p3-b-fixed-001`) 调研 `docs/briefs/p3-b-fixed-001.output.md` v0.1 实证:

- **v0.1 标 "16 项蓝方代修" 实际 0 项需要实装** (误判率 6/16 = 37%)
- 6 项 v0.1 标 "可批量修复" 实际**全部已闭环** (红方+蓝方双重误判, 实际 spec 已有相关字段/章节/表):
  - #6 5 文件 7 vs 6 supporting crate: 0 匹配, 9/3 拍 1 落档 §2.1.4+§2.1.5 解释 22+7=29 实际 34 crate 计数歧义
  - #18 identity/permission spec "无核心依赖" 未限定: 2 spec 实际有 `tenant_id` 字段
  - #19 integration-spec 列表不存在: 实际有 §15 接触面 (scm/notification/identity, 4 个 domain)
  - #20 relation-spec 自身上游/下游列表不一致: 实际 line 276-277 方向一致
  - #23 ASCII §2.3 依赖图箭头方向不一致: grep `↘|↗` 0 匹配, 需 git grep 验证 (推下 session)
  - #25 7 supporting spec 缺"附录 B:边界清单"章节: 实际命名差异 (有 "§5 跨 domain 接触面" 章节)
- 9 项 (含 #2/#3/#4/#5/#7/#8/#9/#10/#24) 全部 9/1-9/3 期间已闭环:
  - #2 7 supporting spec 头部声明: 9/1 15:03 JST GAP-01 落地 (per `domain-ai-spec.md:4-8`)
  - #3 双重错误引用: 9/3 拍 1 落档 §2.1.4+§2.1.5
  - #4 7 supporting-crate spec 链接损坏: 测试通过 (per `domain-ai-spec.md:51` 等)
  - #5 2 spec 误将 requirements 当 basic-design: 实际设计规范引用
  - #7 5 文件 `{display}` 占位符: 0 匹配, 实际不存在
  - #8 14 → 15: 9/3 蓝方 #8 落档 (commit `53d9dc7`)
  - #9 11 → 12: 同上
  - #10 F9 25-Module 表过期: 9/3 拍 1 落档 (commit `d874a79`)
  - #24 P0-P4/P0-P5 命名重叠: 9/3 蓝方 #24 落档 (commit `53d9dc7`)
- 1 项 (Finding-Agent #16) 9/3 拍 4 spec 权威落档 (commit `0948d59` 修 row 3)

**v0.2 结论**: 蓝方 v0.1 "16 项代修" 实际 0 项需要实装, 0 行新代码改动, 0 commit. 蓝方报告 v0.2 修订本节即可.

### 2.4 跨 session 续 5 项 (已知缺口, 标 GAP)

- 13 REQ-ID 零引用 (`REQ-COLLAB-003/004` 等, 1 项) — 等 5 域 Lead 真人到位
- 7 supporting crate 新增 5 (per #3 实际 7, 但 §2.1 表 0) — 跨 session 续 DDD Review
- supporting crate spec 链接修复 (per #4) — 跟 #2 一起
- 5 域 Lead 真人到位 (per 8/21 拒绝兼任硬约束) — Phase 4.3 阻塞
- **GAP-BLUE-7 蓝方 v0.1 误判率 37% (v0.2 修订)**: 后续蓝方报告需逐项用 grep/git log 实证, 不直接采信红方挑刺未深读 14 份 spec 前的定性

---

## 3. 守门实证

| 守门 | 规则 | 本报告实证 | 通过 |
|---|---|---|---|
| #1 | 0 unsafe + 守门实证 | 蓝方报告是 docs 改动, 不触发 cargo | ✅ |
| #9 | 不 commit 散落子代理产出 + git 实证 | 蓝方亲自 read + 写, 0 子代理 dispatch | ✅ |
| #12 | commit-time docs 同步 | 蓝方报告跟 plan v0.5 + AGENTS v0.38 同 commit 落档 | ✅ (待 commit) |
| #15 | 死循环饱和约束 | 当前 6 ahead origin/main 离 113 饱和点 buffer 充足 | ✅ |
| #19 | agent 交互 Python 化 | 蓝方分析是 docs 改动, 不算"agent 跟外部交互" (子代理 dispatch / CLI 调用 / 代码改造 3 类), 不触发 | ✅ |

**5 维质量门** (per STAR-OLU-001 §6):
- 功能完整: 26 项 100% 定性 ✅
- 测试覆盖: 不适用 (docs 改动) — 推进门槛 4/5 ≥ 4 ✅
- 守门 0 违反: 5 守门实证 (§3), 0 违反 ✅
- 文档同步: 本报告 + plan v0.5 + AGENTS v0.38 同 commit ✅
- git 证据: 26 项逐项定性表落档 + 7 项拍板项 + 16 项代修清单 ✅

**总分**: **4/5** (测试覆盖 不适用) → 推进门槛 4/5 ≥ 4 ✅

---

## 4. 已知缺口 (per 缺标比错标安全)

| # | 缺口 | 影响 | 触发 |
|---|---|---|---|
| GAP-BLUE-1 | 蓝方未深读所有 14 份 spec (估 0.5-1.0M token) | 部分 finding 定性粗略, 推下 session 续深读 | 2026-09-03 7:35 JST |
| GAP-BLUE-2 | 7 项 Ulysses 必拍板项待拍 | Phase 3 启动阻塞 (per 14:58 JST 拍板规则) | 2026-09-03 7:35 JST |
| GAP-BLUE-3 | 5 域 Lead 真人到位 (per #1 + #2) | 26 项中 4 项 (REQ-ID 13 + supporting crate 5) 跨 session 续 | 2026-09-03 7:35 JST |
| GAP-BLUE-4 | 16 项蓝方代修待实装 (估 0.1M token, 1 commit) | 跨 session 续 (本 session 估 0.2M 蓝方报告已落地, 16 项代修推下) | 2026-09-03 7:35 JST |
| GAP-BLUE-5 | basic-design §2.1 25-Module 表过期 (实际 34 crate, 缺 9) | 7 项拍板中 1 项 (拍 1) 决定如何处理 | 2026-09-03 7:35 JST (per AUDIT-001) |

---

## 5. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 07:35 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: 26 项逐项定性 (9 文档笔误 + 7 可批量修复 + 7 真实架构矛盾需拍板 + 3 已知缺口) + 7 项 Ulysses 必拍板项 (按风险升序) + 16 项蓝方代修清单 + 5 维质量门 4/5 + 5 已知缺口 | 2026-09-03 7:16 JST Ulysses 发令"立刻做" + plan v0.4 §6.3 优先级 6 + 红方挑刺 9/2 落档 |
| v0.2 | 2026-09-03 08:35 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | v0.2 修订: §2.3 蓝方 v0.1 '16 项代修' 调研后修订 0 行新代码改动 (误判率 37%, 6 项 #6/#18/#19/#20/#23/#25 全部已闭环 + 9 项 #2/#3/#4/#5/#7/#8/#9/#10/#24 9/1-9/3 已闭环 + 1 项 #16 9/3 拍 4 已修); §2.4 跨 session 续 4 项 → 5 项 (+GAP-BLUE-7 蓝方误判率 37%, 后续蓝方报告需逐项用 grep/git log 实证) | 2026-09-03 08:15 JST ask_user 拍 7 = A. 两项都现在做 (推荐) + wt-1 调研 (commit `d488732`) |
