# Dedupe 2026-11-13 Lane B: 文档整合审计

方法: `git hash-object` 全量比对 plan/plans/implementation-plans/frontend/design 无字节相同文件；按版本头、日期、`git grep` 入站引用(排除 reports/briefs)判定。结论: 所列"重叠"主要是 主文件 + 专题增量目录 的分层结构，**无字节级重复**，故未删除任何文件。

| 组 | canonical | stale/次要 | 已执行 | 暂缓及原因 |
|---|---|---|---|---|
| basic-design | basic-design.md (v7.x, 274 入站) | basic-design/ (专题增量) | 无需 | 非重复，增量文档 |
| detailed-design | detailed-design/ + 各 *-design.md | detailed-design-feedback.md | 无 | feedback 为审计证据 |
| frontend-design | frontend-design.md (18 入站) + frontend/design/ | frontend-design-feedback.md, frontend-internal-01..04 (2026-08-26 v0.1) | 无 | internal-04 零入站但属 frontend-design 子文档，内容是否已被吸收需人工确认，故不加 banner |
| data-design | data-design.md (v2.31, 169 入站) | db-design/ (1 份), data-design/ (专题 WTM) | 无 | 分工不同，非重复 |
| requirements | requirements.md (v7.09) | requirements/ (各 SRS) | 无 | 分工不同 |
| plan vs plans vs implementation-plans | implementation-plans/ (2026-10 现行 WBS) | plan/ (2026-08-25 v0.1 草案)；plans/ | plans/post-merge-development-plan-2026-09-02.md 加 superseded banner(被 PHASE-EXEC-PLAN-2026-09-03 取代，仅后者引用它) | plan/*.md 与 plans/plan-016..032 与 rfcs/rfc-016.. 命名成对但内容不同(哈希不同)，需人工逐份确认是否被取代；plan/master-implementation-plan.md 2026-10-15 仍被更新，故不标 stale |

## 建议归档策略（未执行）
- docs/reports (746) 与 docs/briefs (538) 为审计证据，AGENTS.md 禁止改写历史：**不删除、不改内容**。
- 提议: 按季度只增不改；新增 `docs/reports/INDEX.md`(自动生成: 日期、主题、关联 commit)，超过 90 天且无入站引用的报告在索引中标 archive，物理上仍保留或经显式拍板后整体移动到 `docs/archive/<yyyy-qN>/` 并保留旧路径跳转说明。
- 移动前须: `git grep` 全仓入站引用清单 + 批量链接修复由单独 lane 执行 + 人工拍板。
