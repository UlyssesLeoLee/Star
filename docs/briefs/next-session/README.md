# Next-Session Briefs 索引 (per 9/7 12:30 JST 用户发令"abc都做")

> **Status**: 🟡 8 brief 模板落档, 等依赖触发 (5 域 Lead 真人 / 凭证切真 / DDD Review 拍板)
> **Created**: 2026-09-07 12:30 JST
> **Authority**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签

---

## 1. 总览

8 份推下 session brief 模板, 覆盖 P4+ 阶段剩余工作 (per `docs/reports/STAR-P4-OPT-WBS-001.md` §4.2 + §3.2):

| # | brief | 内容 | token 估 | 触发 |
|---|---|---|---|---|
| 1 | `OPT-NEXT-01-phase-d.md` | Phase D T3.2/5.6/G-10 3 子项 | 0.5-1.7M | 5 域 Lead T3 到位 (~ 9/26) |
| 2 | `OPT-NEXT-02-phase-e.md` | Phase E 5 子项 (Saga + DDD + 5 角色) | 4.5M | 5 域 + 4 角色到位 (~ 10/17) |
| 3 | `OPT-NEXT-03-phase-f.md` | Phase F 凭证 + DB + CI 5 子项 | 21M | 凭证 + GA 权限到位 |
| 4 | `OPT-NEXT-04-phase-g.md` | Phase G G-1~G-9 9 子项 | 12M | ECS 选型 + 22 domain-identity |
| 5 | `OPT-NEXT-05-phase-h.md` | Phase H 3 套新架构 + 终审 8 子项 | 7.5M | E.3 + G ECS 选型 + 16 tool |
| 6 | `OPT-NEXT-06-code-stub.md` | OPT-CODE-01..80 47 stub 实施 | 1.5-3.0M | Phase D + H 推进 |
| 7 | `OPT-NEXT-07-g-dep.md` | G-DEP-01/02 P0/P1 tool 实装 | 0.7-1.1M | P3-F #5 拍板 |
| 8 | `OPT-NEXT-08-adr-promote.md` | 23 ADR 升 v0.2 + 4 crate 命名 + 4 混合表 | 0.5-0.9M | DDD Review 拍板 |

**总估 token**: 48-77M (per守门 #1 v18 H2 实证 3-5x 超支风险)

## 2. 触发依赖矩阵

| 依赖 | 触发方 | 等待方 |
|---|---|---|
| 5 域 Lead 真人到位 (T3 ~ 9/26) | Ulysses 内推 (per `5-business-domain-lead-referral.md` v0.1) | NEXT-01, NEXT-02 |
| 4 角色真人到位 (T4 ~ 10/17) | 同上 | NEXT-02, NEXT-05 |
| 凭证切真 (B.5/B.6/E.4) | Ulysses 填入 (mock 备选 per `29692a7` + `5ea9611` 可维持) | NEXT-03 |
| GA 管理员权限 | Ulysses 提供 | NEXT-03 |
| ECS 选型 (G.2) | DDD Review 拍板 (per OPT-A3 §1.5) | NEXT-04, NEXT-05 |
| ADR 升 v0.2 启动 | 拍板 | NEXT-08 |
| 4 crate 命名 ADR | DDD Review 拍板 | NEXT-08 |
| 4 混合表分类 | DDD Review 拍板 | NEXT-08 |

## 3. 派发协议 (per守门 #9 v20)

```powershell
# 1. 选 brief
$brief = Get-Content "docs/briefs/next-session/OPT-NEXT-01-phase-d.md"

# 2. dispatcher.brief
python scripts/automation/dispatcher.py `
    --task-id "OPT-NEXT-01" `
    --phase "OPT-P4-NEXT" `
    --agent "worker" `
    --content $brief `
    --audit-log "docs/reports/OPT-NEXT-01.log"

# 3. dispatcher.invoke (subprocess, per 守门 #9 v3)
# 4. parent 实证 (git log --follow)
# 5. parent merge --no-ff
```

## 4. 守门硬约束 (per AGENTS §4 + §4.1 + §4.2)

- 守门 #1 v19: `cargo check --workspace --all-targets -j 4` 0 err
- 守门 #3: 真人到位 + 追溯签字
- 守门 #10: author=Ulysses
- 守门 #12: 禁回溯叙事, BAS git 实证, commit-time docs 同步触发链
- 守门 #13: W/T/M 派生
- 守门 #19: 优先 Python 化 (per `docs/automation-design.md` v0.1)
- 守门 #5: 禁打印 env secret

## 5. 引用源

- WBS 主体: `docs/reports/STAR-P4-OPT-WBS-001.md`
- 4 维度扫描: `docs/briefs/OPT-A{1,2,3,4}-*.output.md`
- P3-WBS: `docs/reports/STAR-P3-WBS-001.md` v0.6
- P4-UNIMPL-WBS: `docs/reports/STAR-P4-UNIMPL-WBS-001.md` v0.1
- HANDOFF: `docs/reports/HANDOFF-ST-001.md` v1.4
- 5 域 Lead 寻访: `docs/recruitment/5-business-domain-lead-referral.md` v0.1
- 守门基线: `AGENTS.md` §4 + §4.1 + §4.2

## 6. 修订历史

| v | 时间 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-07 12:30 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 首版 8 份推下 session brief 模板 + master README 索引 |
