# RustGameServer (RGS) 5 域 Lead Roster v0.1

> **状态**: 🟡 草案 v0.1
> **日期**: 2026-08-28
> **基点 commit**: `14c8a89` (Phase E.2+ mock infra 完成)
> **触发**: 8/21 JST 5 域独立 Lead 拒绝兼任硬约束 + 8/27 21:59 JST AGENTS.md §9 5 域真实身份 DDD Review 阶段补
> **制定者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
> **签批**: 🟢 Mavis 接手代签 (per 2026-08-27 19:39/21:59 JST 三次强化)

---

## 0. 目的

承接 8/21 JST RGS-TS-001 §3 5 域 (player / economy / match / social / admin) + 8/21 JST 拒绝兼任硬约束, 写 5 域 Lead 真实身份采集模板.

5 域 = 架构 / SRE / 平台 / 评审 / PM (per AGENTS.md §9 签字栏)

---

## 1. 5 域 Lead 真实身份

| # | 域 | 真实姓名 | 邮箱 (redacted) | 经验 (年) | 签字日 | 备注 |
|---|---|---|---|---|---|---|
| 1 | 架构 | Ulysses（一人公司 12 角色 per DEC-008）| ulysses@*** (per 8/27 11:06 JST hard ban) | — | 2026-08-27 | 1 人 12 角色, 真实身份 = Ulysses 本人 |
| 2 | SRE Lead | **[DDD Review 阶段补]** | **[redacted]** | **[DDD Review 阶段补]** | **[DDD Review 阶段补]** | 5 域独立真实身份, 不接受 Mavis 代签 (per 8/21 JST 拒绝兼任) |
| 3 | 平台工程师 | **[DDD Review 阶段补]** | **[redacted]** | **[DDD Review 阶段补]** | **[DDD Review 阶段补]** | 5 域独立真实身份 |
| 4 | 评审主持 | **[DDD Review 阶段补]** | **[redacted]** | **[DDD Review 阶段补]** | **[DDD Review 阶段补]** | 5 域独立真实身份 |
| 5 | 项目负责人 (PM) | **[DDD Review 阶段补]** | **[redacted]** | **[DDD Review 阶段补]** | **[DDD Review 阶段补]** | 5 域独立真实身份 |

---

## 2. RGS 5 域 (per RGS-TS-001 §3)

| 域 | 业务范围 | 核心规则 |
|---|---|---|
| player | 玩家账号/角色/数据 | 单一身份 / 跨 workspace 唯一 |
| economy | 货币/物品/交易/账本 | 双重账本 (real/committable) / Saga 跨域 |
| match | 匹配/对局/排位 | 状态机 / 房间生命周期 |
| social | 好友/聊天/工会 | 跨 player / 权限梯度 |
| admin | COC/CMS/审计/合规 | 独立控制面, 不受运营干预 |

**5 域 Lead 拒绝兼任硬约束** (per 8/21 JST): 1 域 1 Lead, 责任矩阵清晰, Q-003 Saga 跨域核心问题需要独立决策权.

---

## 3. 守门 (per AGENTS.md §4 12 项)

- ✅ **R-05 不 push** (8/27 11:09 JST): 本文档待 DDD Review 阶段补真实身份, 不 push origin
- ✅ **环境变量安全** (8/27 11:06 JST hard ban): 邮箱用 `redacted`, 不打印实际值
- ✅ **5 域独立 Lead 不兼任** (8/21 JST): roster 5 域 5 真实身份, 不接受 Mavis 代签
- ✅ **代签规则应用** (8/27 19:39/21:59 JST): Mavis 接手代签本 roster 模板 (采集 checklist)
- ✅ **缺标比错标安全** (8/26 JST): 4 个 [DDD Review 阶段补] 显式空位, 不编造身份
- ✅ **AI 协作文档治理** (8/26 JST): 无回溯叙事, 不假设 5 域 Lead 已知

---

## 4. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-28 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 初版: 5 域 Lead roster 模板 + 采集 checklist (per 8/21 JST 拒绝兼任 + 8/27 21:59 JST AGENTS.md §9 DDD Review 阶段补) | Phase F.1 待办 (8/28 22:30 JST 用户发令"开子代理和 wt 并行处理待办") |
