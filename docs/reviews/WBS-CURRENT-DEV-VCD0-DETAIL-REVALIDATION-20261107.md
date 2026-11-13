# VCD-0 详细设计与测试设计候选当前基线复核

- 复核日期：2026-11-07
- 目标基线：dev@e2393618d5d9cf71181bdadf78b5b4663850e4cf
- 来源：codex/vcd0-canonical-detail-test-20261024@4f0ceae9818b7bd2b3b25d9164d6e43303121689
- 处置：不合并、不 cherry-pick；保留部分结论，候选正文已过时

## 复核结果

候选包含 DD 与 TD 两份设计及两份修复 brief。候选 DD/TD 标头写 v0.1，修订记录却分别到 v0.2/v0.3；目标 dev 的 DD 已为 v0.3、TD 为 v0.4。目标版本另已纳入 noVNC、当前 Owner/路由边界与更强的跨站负向验收。

候选将 chat SSE 路由固定为 Gateway（候选 DD 第 66 行），而目标 SRS FR-VC-013 允许 Next 路由透传，目标 ADR 仍将最终路由留作决策门；候选不能取代目标版本。候选 TD 的 T-049 缺少 noVNC，且对禁止 Origin 的负向测试附加 Gateway 契约条件，可能跳过安全负向检查。目标 TD v0.4 要求将获准/不获准 Origin 或获批等效跨站防护列为验收前置，不满足时保持 NOT_RUN。

候选 FR-VC-009..012 的 Owner、数据面、迁移和 fail-closed 映射有参考价值，但目标 dev 已吸收并继续更新。候选提出的额外 FR/AC/NFR 编号超出当前 SRS 的 FR-VC-001..021、AC-VC-001..013、NFR-VC-01..08；不得把候选编号当成已批准需求。

## 继续保留的限制

- VCD-0 保持 OPEN；TD 测试保持 NOT_RUN。
- 不采用候选的第二 Owner/DB/worker/scheduler 平面或额外 VM 执行提供方。
- 不以静态设计替代真实 Vercel、Neon、Owner/RLS、Hosted Build、协作和 rollback 证据。
- 本文件不把候选分支内容覆盖回当前设计。
