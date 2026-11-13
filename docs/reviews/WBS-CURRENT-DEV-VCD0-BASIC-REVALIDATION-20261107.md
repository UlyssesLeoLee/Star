# VCD-0 基本设计候选当前基线复核

- 复核日期：2026-11-07
- 目标基线：dev@e2393618d5d9cf71181bdadf78b5b4663850e4cf
- 来源：codex/wbs-vcd0-basic-review-20261024@98c1c7eb
- 处置：已登记内容已在 dev；不 cherry-pick，不合并来源分支

## 复核结果

该 lane 登记的 compatibility review 与 brief 已存在于目标 dev，review Git blob ID 为 34673af9e9db56764b2cbe975b5df89705a01fe2。登记范围没有可再次摘取的内容。来源分支另包含 docs/briefs/WBS-VCD0-BD-REVIEW-R2-20261024.md 和 docs/reports/PHASE-VCD0-BD-REVIEW-R2-20261024.md；它们不在该 lane 的登记清单内，本次不纳入。

当前 SRS v0.4 仍标记为 Draft、0 实现（docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md:3-6）。FR-VC-013 允许 chat SSE 经 Next 路由透传，同时把 terminal/PTY/noVNC/Canvas 执行连接留在 Star 执行面（同文件 §4.4）；FR-VC-014 将 Cron 限定为 best-effort wake-up，并要求每次无人值守执行复验 run_as_actor_id 授权。通用函数 WebSocket 与 Rust Runtime 能力仍分开描述；不得把通用平台 GA 推导为 Rust Runtime 已支持。

稳定版 WBS 的 W6 保持 ADAPT/OPEN，没有 Vercel production deployment、Neon/Owner runtime、consumer、协作推送或 rollback 证据（docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md:43）。

## 继续保留的限制

- Vercel 不拥有 Star 的 Run 授权、Task/Canvas 事实、调度 worker 或 Worktree/Agent 执行。
- 目标身份、数据、owner API、RLS 与运行时验收未完成前保持 OPEN/NOT_RUN。
- 不从来源分支合入登记范围外的文件；先单独登记与复核。
- 本文件只记录文档复核，不构成部署、运行或发布证明。
