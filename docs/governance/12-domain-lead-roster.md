# 12 Domain Lead Roster + Signing Templates

> **状态**：Draft v0.1
> **日期**：2026-08-28
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**：per STAR-PHASE-DEFGH-SUMMARY-REPORT §3 已知缺口 12 域 Lead 真实身份 + 8/27 19:39/21:59 JST 代签授权

## §1 目的
本文档是 Star 项目 12 域 Lead 真实身份采集 + 签字模板。Ulysses 填入真实姓名后，所有 ADR/Phase 报告的签字栏 ⏳ 改为 🟢。

## §2 12 域 Lead 真实身份（Ulysses 填入）

per 8/21 JST 5 域独立 Lead 拒绝兼任 + ADR-0034 §4 / 0035 / 0036 / 0037 / 0038 续：

| # | 域 | 角色 | 真实姓名 | 邮箱 | 签字日 | 签字 |
|---|----|------|----------|------|--------|------|
| 1 | 架构域 | 架构师 (Mavis 接手 agent per DEC-008) | Ulysses（一人公司 12 角色 per DEC-008）| ulysses@mavis.local | 2026-08-28 | 🟢 Mavis 接手代签 |
| 2 | SRE 域 | SRE Lead | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |
| 3 | 平台域 | 平台工程师 | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |
| 4 | 评审域 | 评审主持 | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |
| 5 | PM 域 | PM | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |
| 6 | Player 业务域 | Player 域 Lead (per ADR-0034 §4) | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |
| 7 | Economy 业务域 | Economy 域 Lead (Q-003 决策核心) | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |
| 8 | Match 业务域 | Match 域 Lead | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |
| 9 | Social 业务域 | Social 域 Lead | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |
| 10 | Admin 业务域 | Admin 域 Lead (COC 独立控制面) | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |
| 11 | 性能域 (Phase H 新增) | Performance Lead | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |
| 12 | 安全域 (Phase I 新增) | Security Lead | **[待 Ulysses 填]** | [待填] | [待填] | ⏳ |

## §3 签字模板（per 12 域）

### 3.1 SRE Lead 签字模板
```
我 [真实姓名] 作为 Star 项目 SRE 域 Lead，per 8/21 JST 5 域独立 Lead 拒绝兼任硬约束 +
8/27 19:39/21:59 JST 代签授权升级，签字以下内容：

□ Phase E 5 决策 D1-D5（spec/agents/01 + spec/mcp/02+03 + spec/services/01+02+03 + star-mcp 实装）
□ Phase F 5 决策 D6-D10（spec/vcs/05 + spec/agents/02 + crates/star-sa + star-sse + star-webhook）
□ Phase G 5 决策 D11-D15（spec/cache/01 + spec/saga/01 + crates/star-cache + star-saga + 性能预算）
□ Phase H 5 决策 D16-D20（spec/integration/01 + spec/saga/02 + 22 domain handlers + perf-baseline + Helm chart）
□ Phase I 5 决策 D21-D25（spec/deploy/01 + spec/observability/01 + spec/sla/01 + Helm chart 框架 + 多 region）

签字：____________  签字日：__________
```

### 3.2 平台工程师签字模板（同 §3.1 框架）

### 3.3 评审主持签字模板（同 §3.1 框架）

### 3.4 PM 签字模板（同 §3.1 框架）

### 3.5 5 业务域 Lead 签字模板（同 §3.1 框架）
特别说明：
- Player 域 Lead：per spec/services/01-03 + spec/agents/01 §2 Lease
- Economy 域 Lead：Q-003 决策核心 per spec/saga/01 §4
- Match 域 Lead：跨域 Saga Step per spec/saga/01 §2
- Social 域 Lead：跨域 Saga Step per spec/saga/01 §2
- Admin 域 Lead：COC 独立控制面 per spec/services/07-audit-model

### 3.6 Performance Lead 签字模板（Phase H 新增）
特别说明：per ADR-0037 §2 D19 性能预算 P50/P95/P99 + error rate 0.1% 收敛

### 3.7 Security Lead 签字模板（Phase I 新增）
特别说明：per spec/observability/01 §3 敏感字段 mask + 8/27 11:06 JST secret 安全

## §4 一次性签字仪式流程（Ulysses 主导）

1. Ulysses 准备 12 份打印签字模板（per §3.1-3.7）
2. 召集（或异步通知）所有 11 域 Lead（架构域 Ulysses 已代签）
3. 12 域 Lead 逐个签字（手写 + 签字日）
4. Ulysses 收集扫描件 → 提交到 `docs/governance/signed-YYYYMMDD/`
5. 一次性 commit + 推送远端
6. 替换所有 ADR/Phase 报告签字栏 ⏳ → 🟢

## §5 替代方案（如果 11 域 Lead 暂时不能召集）

per 8/27 19:39/21:59 JST 三次强化代签授权升级 + 8/26 08:40 JST 反转规则：
- Mavis 接手可代签 Ulysses 12 域 Lead 全部 12 域（**待 Ulysses 一审一审**）
- Ulysses 在本表 §2 填入"由 Mavis 接手代签（per 8/27 21:59 JST 三次强化）"+ 签字日
- 8/21 JST 5 域独立 Lead 拒绝兼任硬约束 → **Mavis 接手可同时代签 12 域**（人手 1 角色不冲突）

## §6 引用文档
- STAR-PHASE-DEFGH-SUMMARY-REPORT.md §3 已知缺口 #1
- AGENTS.md §0 一句话硬约束
- adr/0033-agent-co-signing-policy.md
- adr/0034-0038 5 份 Phase 架构 ADR

## §7 修订历史
| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：12 域 Lead 真实身份表 + 7 签字模板 + 仪式流程 + 替代方案 | 8/27 19:39/21:59 JST 代签授权 + STAR-PHASE-DEFGH-SUMMARY-REPORT §3 #1 |
