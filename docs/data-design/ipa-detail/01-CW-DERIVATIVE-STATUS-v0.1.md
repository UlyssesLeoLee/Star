# CW 派生累积规 状态 (v0.1, 2026-09-07 18:43 JST 落档)

> **Status**: 🟡 5/10 done | 🔴 5/10 pending SRE Lead 真人到位拍板
> **Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **基线**: `00-CLASSIFICATION-RULES.md` §N.5 派生 10 条 (CW-01..CW-10)

---

## 1. 派生 10 条状态总览

| # | 派生规 | 状态 | 实证 / 阻塞 |
|---|---|---|---|
| **CW-01** | 全仓 table W/T/M 1 table 1 classification | 🟢 done | 100/100 表格 100% 业务分类 (v0.3 §3.1, per `f13f325`) |
| **CW-02** | W/T/M 主分类 1 个不允许混合 | 🟢 done | 6 混合 → 0 已知混合 (v0.3, per `f13f325`) |
| **CW-03** | W 0 audit + 不引 Master + session-bound | 🟢 done | §10.4 PASS 验证 (v0.3) |
| **CW-04** | T 0 audit, write/read 需独立 Module (audit module) | 🔴 **pending** | 等 SRE Lead 拍板 + 5 域 Lead 真人到位 |
| **CW-05** | M 13 類 tenant_id RLS 必携 | 🟢 done | 100% RLS (per守门 #13 (c) 派生) |
| **CW-06** | T > 1M rows 必 RANGE(`created_at`) 分区 | 🔴 **pending** | 等 SRE Lead 拍板 + 性能基准 |
| **CW-07** | W 必显式 `retention_period` + 自动清理 | 🔴 **pending** | 等 SRE Lead 拍板 + cron job 实装 |
| **CW-08** | 同一 Module 内 W/T/M 必独立分仓, 跨仓事务禁 | 🔴 **pending** | 等 SRE Lead 拍板 + 22 DDD bounded context 联动 |
| **CW-09** | 横开 enum/status/role/policy/permission/tag/category 全分类独立列出 | 🟢 done | 100% 业务分类 (per守门 #13 横展派生) |
| **CW-10** | 分类变更要阻断, T↔M 转换需 Migration 工具 | 🔴 **pending** | 等 SRE Lead 拍板 + Migration 工具实装 |

**完成**: 5/10 (CW-01/02/03/05/09)
**pending**: 5/10 (CW-04/06/07/08/10) — **brief "4/10" 笔误, 实际 5/10**

## 2. 阻塞详细 (per守门 #11 缺标比错标)

### CW-04: T 0 audit 独立 Module

- **派生规**: T 表 0 audit 字段, write/read 需走独立 audit Module (跟 5 域 Lead audit 域 联动)
- **阻塞**: 等 SRE Lead 真人到位 (T3 ~ 2026-09-26 JST, per `docs/recruitment/5-business-domain-lead-referral.md` v0.1)
- **依赖**: 5 域 Lead 真人到位 + audit 域 (5 域 admin) 拍板 + RLS 13 類 audit policy 落地

### CW-06: T > 1M rows RANGE 分区

- **派生规**: T 表超过 1M rows 必 RANGE(`created_at`) 分区 (避免全表扫描性能问题)
- **阻塞**: 等 SRE Lead 真人到位 + 性能基准实证 (估算 22 DDD bounded context + 100 表 T 类可能 30+ 表超过 1M)
- **依赖**: SRE Lead 拍板分区策略 (按月/按季/按年) + PostgreSQL 12+ PARTITION BY RANGE 落地

### CW-07: W retention_period + 自动清理

- **派生规**: W 表 100% 必显式 `retention_period` + 自动清理 cron job
- **阻塞**: 等 SRE Lead 真人到位 + cron job 实装 (per守门 #22 调试控制台同理)
- **依赖**: SRE Lead 拍板 + 14 Work 表 retention 默认值 (7d/30d/90d)

### CW-08: 同一 Module 内 W/T/M 独立分仓

- **派生规**: 同一 DDD bounded context 内 W/T/M 表必独立 schema 仓, 跨仓事务禁止
- **阻塞**: 等 SRE Lead 真人到位 + 22 DDD bounded context 边界确认
- **依赖**: DDD Review Lead 拍板 + 现有 12 schema (workspace/planning/audit/local_runtime/...) 重组

### CW-10: T↔M 转换 Migration 工具

- **派生规**: 分类变更要阻断, T↔M 转换需专用 Migration 工具 (per守门 #11 缺标比错标安全)
- **阻塞**: 等 SRE Lead 真人到位 + Migration 工具实装
- **依赖**: 工具 spec (DDL 生成 + 审计 + 演练 dry-run + 回滚 rollback)

## 3. 触发条件 (5 项 pending 同步)

| # | 触发 | 估计时点 |
|---|---|---|
| 1 | 5 域 Lead 真人到位 (T3 ~ 9/26 JST) | per 9/5 内推 brief `docs/recruitment/5-business-domain-lead-referral.md` v0.1 |
| 2 | SRE Lead 真人到位 | per AGENTS §1.2 不代签底线 (SRE Lead 仍 ⏳) |
| 3 | DDD Review Lead 真人到位 | per 守门 #14 v2 CONTENT 4 维 |
| 4 | 性能基准实证 (T > 1M rows 验证) | 等 T 表实际数据 > 1M |
| 5 | Migration 工具 spec 落地 | 等 SRE Lead 拍板 + DDD Review 联动 |

**最早触发时点**: 2026-09-26 JST (T3) 之后
**最晚触发时点**: 5 域 Lead + SRE Lead + DDD Review Lead 全到位 (待定)

## 4. 守门合规

- **守门 #13 W/T/M 派生**: 10 条派生 5 done + 5 pending, 全部显式列出, 0 隐性假设
- **守门 #11 缺标比错标**: 5 pending 全部显式标, 含阻塞 + 依赖 + 触发时点
- **守门 #12 禁回溯叙事**: 全部状态基于 git 实证 (per `f13f325` v0.3 + `bdd1b29` v0.30)
- **守门 #14 v2**: 5 域 Lead CONTENT 4 维 (决策 scope/RACI/到位 timeline/Mavis 代签边界) 已显式列
- **守门 #20 拆 commit 派生规**: 1 docs commit (本文) + 1 docs commit (AGENTS §7 表更新)

## 5. 已知缺口

- 本 brief "4/10" 笔误, 实际 5/10 pending (CW-04/06/07/08/10), per Worker 8/13 报告笔误
- 22 DDD bounded context 边界尚未 DDD Review 拍板, 影响 CW-08 独立分仓落地
- 100 表 T 类实际 30+ 表超过 1M rows 实证, 需性能基准测试 (SRE Lead 拍板后启动)

## 6. 引用

- 基线 派生 10 条: `00-CLASSIFICATION-RULES.md` §N.5
- v0.3 升版: `00-CLASSIFICATION-W-T-M.md` v0.3 (per `f13f325`, 2026-09-07 14:12 JST)
- 5 域 Lead 真人 timeline: `docs/recruitment/5-business-domain-lead-referral.md` v0.1
- 守门 #13 派生: `AGENTS.md` §4 #13
- 守门 #11 缺标比错标: `AGENTS.md` §4 #11
- OPT-NEXT-08 brief: `docs/briefs/next-session/OPT-NEXT-08-adr-promote.md` (本 v0.1 跟 §3 OPT-ADR-28..31 配套)

## 7. 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手代签 Ulysses | 2026-09-07 18:43 JST |
| SRE Lead | ⏳ 待真人到位 | TBD |
| DDD Review Lead | ⏳ 待真人到位 | TBD |
| 5 域 Lead (admin 域 audit 联动) | ⏳ 待真人到位 (T3 ~ 9/26 JST) | TBD |
| PM | ⏳ 待真人到位 | TBD |

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-07 18:43 JST | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签 | 初版落档, 5/10 done + 5/10 pending (CW-04/06/07/08/10), 含阻塞 + 依赖 + 触发时点 (per守门 #11 缺标比错标) |
