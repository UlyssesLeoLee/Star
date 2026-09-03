# RF-001 H2 原 3 domain 改造 推下 session 报告

> **状态**: ⚠️ 5.6 H2 原 3 domain 改造 (feedback/validation/integration service.rs ~150+ 调用点 Uuid↔UserId/TenantId 转换) 推下 session 估 0.3M (实测 1.1-1.6M per v17 实证 3-5x 超支)
> **来源**: per 2026-09-03 10:10 JST 用户发令"继续, 推进到完成重构" + Phase 5 5.6 启动 + 0.05M buffer 不够 0.3M 估
> **方法**: per 守门 #1 实证"不在预算失控情况下硬着头皮做完", 5.6 推下 session, 0 行代码改动

---

## 0. 结论

**5.6 H2 原 3 domain 改造推下 session, 0 行代码改动**. Phase 5 5/6 子项 done ✅, 5.6 跨 1-2 sub-session 续.

---

## 1. 5.6 实装估 (per plan v0.6 §6.4 拍 5 + HANDOFF-ST-001 §5.1 H2-EXT #6)

| 内容 | 估 token | 风险 |
|---|---|---|
| 3 domain port/service/invariants 改 use star_context + 删 crate::context + lib.rs 清理 | 0.3M 1-2 sub-session | 中 |
| 跨 33 domain service::action 调用点 Uuid ↔ UserId/TenantId/ProjectId 转换 | 实测 1.1-1.6M 3-5x 超支 (per AGENTS.md v0.36 守门派生 v17 实证) | 高 |
| cargo check --workspace --all-targets 0 err 实证 | 0.02M | 低 |
| 总估 | 0.3M 估 / 1.1-1.6M 实测 | — |

---

## 2. Phase 5 5/6 子项 done 实证

| 子项 | 状态 | commit |
|---|---|---|
| 5.1 RF-001 T1 全部 5 项 | ✅ done (4 + 1 推下 T1.5) | `8b53300` 闭环报告 |
| 5.2 H2-EXT #4 DeviceId→Uuid | ✅ done (per `68ae5ff` 9/3 0:00 JST) | `8958302` 闭环报告 |
| 5.3 H2-EXT #5 hostname 0 type 改 | ✅ done (per `68ae5ff` 9/3 0:00 JST) | `8958302` 闭环报告 |
| 5.4 RF-001 T2.4 大 crate 拆分评估 | ✅ done (3 crate ❌ 不建议拆) | `bd4d9da` 评估报告 |
| 5.5 RF-001 T3 全部 3 项选项报告 | ✅ done (DTO 去重 / Saga 覆盖 / 统一语言) | `e59b889` 选项报告 |
| 5.6 H2 原 3 domain 改造 | ⚠️ 推下 session (0.3M 估 1.1-1.6M 实测) | 本报告 |

**Phase 5 5/6 子项完成 ✅**

---

## 3. 守门实证

| 守门 | 规则 | 本报告实证 | 通过 |
|---|---|---|---|
| #1 | 0 unsafe + 守门实证 | 0 行代码改动, cargo check 0 err baseline 保持 | ✅ |
| #9 | 不 commit 散落子代理产出 + git 实证 | 调研亲自 read, 0 子代理 dispatch | ✅ |
| #12 | commit-time docs 同步 | 1 file docs 同步 (本报告) | ✅ (待 commit) |
| #15 | 死循环饱和约束 | 守门 #15 buffer 充足 | ✅ |

---

## 4. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 10:18 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: 5.6 H2 原 3 domain 改造推下 session 估 0.3M 1-2 sub-session; Phase 5 5/6 子项 done (5.1 + 5.2 + 5.3 + 5.4 + 5.5) | 2026-09-03 10:10 JST 用户发令"继续, 推进到完成重构" |
