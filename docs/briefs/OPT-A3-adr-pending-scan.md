# Brief: OPT-A3 — ADR / 架构 / IPA 文档未实装行动扫描

**Agent**: explorer
**Phase**: OPT-WBS
**Created**: 2026-09-07 11:53 JST
**Author**: Mavis 接手 (per 8/27 19:39 JST Ulysses 授权代签)

---

## 1. 任务目标

扫描 `D:\Star\docs\architecture\` 全部 ADR / IPA 文档,识别:
1. **ADR 决策未实装行动** (e.g. ADR-0046 LangGraph TMO 7 节点已文档化, 部分实装)
2. **IPA 3 文档 v0.1 草稿** (per AGENTS §4 #1 v2 当前 `unreachable_pub = deny` + 待 T1.5 step 2/3)
3. **守门 #13 DB W/T/M 强制分类** (per 9/1 18:30 JST 拍板) - 缺表/混合分类
4. **5 域 Lead RACI / CONTENT 4 维** (per 9/3 19:43 JST 拍板) - 待 DDD Review 落地
5. **Runtime 概念→物理 crate 映射门禁** (per AGENTS §4.2) - 未完成
6. **架构 view 平行关系** (per AGENTS §6.1) - LangGraph vs Agent Runtime 接口边界

## 2. 已知输入

**ADR 索引 (per AGENTS §6)**:
- 0021-0032 早期 ADR (zero vendor / IDE / VCS / session / MCP stdio 等)
- 0033 agent co-signing policy
- 0034 jira-ification
- 0035-0042 phase F-I architecture
- 0043 audit_onboarding failed event (守门 #13 W/T/M)
- 0044 STAR Agent Runtime SRS (113 节, 1M logical agents)
- 0045 STAR Agent Runtime Basic + Detailed Design (40K + 52K + 14K)
- 0046 LangGraph TMO 任务卡管理操作
- 0047 PostgreSQL Checkpointer Tier 3 (5 域 RACI + 5 阶段 E-1..E-5)

**IPA 3 文档 v0.1 草稿** (per AGENTS §4.2 实装前一致性门):
- 缺表/混合分类 (DB W/T/M 100% 覆盖待补)
- 47 packages 但部分仍 v0.1 草稿

**守门 #13 W/T/M (per 9/1 18:30 JST 拍板)**:
- 引用基线: `D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-W-T-M.md` v0.1 + `00-CLASSIFICATION-RULES.md` v0.1
- 派生守门 10 条 CW-01~CW-10
- 100 表覆盖目标

## 3. 范围与边界

**in-scope**:
- `docs/architecture/2026-08-26-upgrade/adr/00*.md` (47 份 ADR)
- `docs/architecture/2026-09-03-langgraph/0*-requirements/basic/detailed.md` (3 份 IPA v0.2)
- `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` + `03-detailed-design.md` (2 份)
- `docs/data-design/ipa-detail/00-CLASSIFICATION-*.md` (守门 #13 基线)
- `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` (113 节)
- `docs/recruitment/5-business-domain-lead-referral.md` (5 域 Lead 内推)

**out-of-scope**:
- 旧版 `docs/architecture/legacy/` (如有)
- 任何 .rs 源码 (那是 A1 范围)
- AGENTS.md 守门规则文本本身(那是 A2 范围)

## 4. 交付物

**输出文件**: `docs/briefs/OPT-A3-adr-pending-scan.output.md`

**Schema**:

```markdown
# OPT-A3 ADR / 架构 / IPA 文档未实装行动扫描报告

> **Created**: <timestamp> JST | **Authority**: Mavis 接手
> **扫描范围**: 47 份 ADR + 5 份 IPA 文档 + 守门 #13 基线 2 份
> **守门基线**: 守门 #13 W/T/M + 守门 #1 v19

## §0 摘要
- ADR pending 实装: <N> / 47
- IPA v0.1 草稿: <N> 份
- 守门 #13 表覆盖: <N>/100 (按 docs/data-design 索引)
- Runtime 概念→物理 crate 映射门禁: <状态>
- 5 域 Lead RACI 落地: <状态>

## §1 ADR pending 实装矩阵
| ADR | 标题 | 落地状态 | 缺口 | 依赖 |
|---|---|---|---|---|

## §2 IPA 3 文档版本矩阵
| 文档 | 当前版本 | 草稿/P0/P1 | 缺口章节 | 下一拍板 |
|---|---|---|---|---|

## §3 守门 #13 W/T/M 100 表覆盖矩阵
| 表名 | 当前分类 | 应分类 | 缺口 |
|---|---|---|---|

## §4 Runtime 概念→物理 crate 映射门禁
| 概念 | 物理 crate (当前) | 缺口 | 决策依据 |
|---|---|---|---|

## §5 5 域 Lead RACI / CONTENT 4 维 落地状态
| 域 | RACI | CONTENT | 真人到位 | Mavis 代签 |
|---|---|---|---|---|

## §6 架构 view 平行关系边界
| View | 接口 | 实装 | 缺口 |
|---|---|---|---|

## §7 已知缺口
- 任何未量化的"per 守门"标"未量化"
- 跨仓引用 / 旧版本未实装等显式列入
```

## 5. 接受标准

- [ ] §1 ADR 矩阵覆盖 47 份 (per AGENTS §6)
- [ ] §3 守门 #13 表覆盖引用 `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` 实证
- [ ] 全部 commit 引用 7 字符 hash + 实证
- [ ] 估算条目数: ≤ 500 行

## 6. 报告语言

中文。

## 7. 时间预算

≤ 200K token (单 sub-session)。

## 8. 失败处理

若 ADR 列表超长, §1 拆 sub-table 列出, 不省略。
