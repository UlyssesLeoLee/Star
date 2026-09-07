# Brief: OPT-NEXT-03 — Phase F 凭证 + DB + CI (per OPT-WBS-19..23, 推下 session)

**Agent**: worker
**Phase**: OPT-P4-NEXT-SESSION
**Created**: 2026-09-07 12:30 JST
**Status**: 🟡 等待凭证 + GA 管理员权限到位
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签

---

## 1. 任务目标

完成 Phase F 5 子项, 等凭证到位 (per `STAR-P4-UNIMPL-WBS-001.md` §7):

- **F.1**: B.5 OpenClaw 真实集成 e2e (凭证切真) — 5M
- **F.2**: B.6 Hermes 真实集成 e2e (凭证切真) — 5M
- **F.3**: E.4 KMS 集成 (Vault / AWS KMS 凭证) — 5M
- **F.4**: 守门 #DB-13 DB 三類横展開 (W/T/M) 跨项目 P3-D 阶段落地 — 3M
- **F.5**: D.2/D.6 CI runner 真实配置 (跨平台 e2e + markdownlint + cargo doc CI job) — 3M

**总估 token**: 21M
**mock 备选已落地**: `29692a7` (OpenClaw/Hermes), `5ea9611` (LocalMockKms), `8ace1d5` (CI stub)

## 2. 依赖

| # | 依赖 | 状态 |
|---|---|---|
| 1 | OpenClaw API key + endpoint | 🔴 Ulysses 提供 (mock 备选可维持) |
| 2 | Hermes API key + endpoint | 🔴 Ulysses 提供 (mock 备选可维持) |
| 3 | Vault / AWS KMS 凭证 | 🔴 Ulysses 提供 (LocalMockKms per `5ea9611` 可维持) |
| 4 | GitHub Actions 管理员权限 (GA runner) | 🔴 Ulysses 提供 (CI stub per `8ace1d5` 可维持) |
| 5 | 100 表 W/T/M 分类验证 (F.4) | 🟢 100/100 已落 (per `00-CLASSIFICATION-W-T-M.md` v0.1, 4 混合表待 DDD Review 拍板) |

## 3. 实施路径

### F.1 / F.2 / F.3 — 凭证切真

- 替换 mock 备选为真实 endpoint
- 安全审计 (per守门 #5 禁打印 secret)
- 集成测试覆盖 happy path + 3 类失败
- mock 备选保留 (per 9/3 11:35 JST 拍板 A) 作为 fallback

### F.4 — DB W/T/M 跨项目落地

- 100 表已分类 (per `00-CLASSIFICATION-W-T-M.md` v0.1)
- 4 混合表待 DDD Review 拍板 (per OPT-A3 §3.4):
  - `workspace.workspace` (M/T)
  - `planning.roadmap` (M/T)
  - `audit.audit_event_outbox` (T/W)
  - `local_runtime.runtime_observation` (T/W)
- 派生守门 CW-04 / CW-06 / CW-07~10 待 P3-B SRE Lead 拍板 (4/10 pending)

### F.5 — CI runner 真实配置

- 跨平台 e2e (Ubuntu / macOS / Windows)
- markdownlint (per PR #12 `81b90ee` 已落地)
- cargo doc CI job (per守门 #1 v26 advisory 派生)
- e2e-integration main only 改 PR 触发 (per 9/5 报告 §3.7)

## 4. Worktree

```bash
git worktree add -b feat/opt-phase-f D:/Star/.worktrees/wt-opt-phase-f main
cd D:/Star/.worktrees/wt-opt-phase-f
```

## 5. 守门硬约束

- 守门 #1 v19: cargo check 0 err
- 守门 #5: 禁打印 env secret (9/7 11:00 JST hard ban)
- 守门 #13 W/T/M 100 表覆盖
- 守门 #10: author=Ulysses
- 守门 #12: 禁回溯叙事

## 6. 提交

5 commit (per 5 子项) 或 1 合并

## 7. 失败处理

- 凭证未到位 → 报告阻塞, mock 备选可维持无限期 (per 9/3 11:35 JST 拍板 A)
- 仍失败: 报告具体错误, 不 commit

## 8. 状态

🟡 **推下 session** (凭证到位触发, 时间不固定 per Ulysses 决定)

## 9. 引用

- 触发源: `docs/reports/STAR-P4-OPT-WBS-001.md` §4.2 #3
- 基线: `docs/reports/STAR-P4-UNIMPL-WBS-001.md` §7
- W/T/M 基线: `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1
- 混合表: OPT-A3 §3.4
