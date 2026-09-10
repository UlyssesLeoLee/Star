# SANDBOX-IMPL-PLAN-002 — Sandbox-as-a-Service (sandboxd) 实施计划

> **版本**: v0.1
> **作者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手
> **日期**: 2026-09-10 JST
> **状态**: 🟡 Draft v0.1 (2026-09-10 JST 初版落档)

---

## §0 文档目的

本文档定义 **Sandbox-as-a-Service (sandboxd)** 的实施计划，覆盖 5 阶段 × 16 子项 × ~3.5M token 总预算的落地表。

**核心定位**：

- 5 阶段: P0 骨架 (sandboxd binary + 1 平台) → P1 三平台 (Win+Linux+macOS) → P2 client 集成 (dispatcher + mavis) → P3 fail-open 兼容性 → P4 收官 (报告 + UAT)
- 16 子项: 估 ~3.5M token, ~3.5-4 周 (per STAR-OLU-001 1 SRE·周 = 1.2M)
- 跟既有 `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` 模式对齐
- 引用上游 3 份需求/设计文档 (SRS-002 v0.1.1 + BD-002 v0.1.1 + DD-002 v0.1 + TDD-002 v0.1)
- 9 已知缺口显式标注（per 守门 #11 缺标比错标）

---

## §1 范围

### 1.1 In-Scope（5 阶段 × 16 子项）

| 阶段 | 子项数 | Token 预算 | 周期 | 范围 |
|---|---|---|---|---|
| **P0 骨架** | 4 | ~0.5M | 1 周 | sandboxd binary + Linux 单平台 + Resource 维 + gRPC 4 RPC stub + fail-open 降级 |
| **P1 三平台** | 4 | ~1.0M | 1-1.5 周 | Windows + macOS 后端 + 3 维 (network/fs/capability) 落地 |
| **P2 client 集成** | 3 | ~0.8M | 1 周 | dispatcher.py 切 gRPC client + mavis desktop 集成 + E2E 测试 |
| **P3 fail-open 兼容性** | 3 | ~0.6M | 0.5-1 周 | SANDBOX-001 v0.2 降级路径 + 故障注入 + 性能 bench |
| **P4 收官** | 2 | ~0.6M | 0.5 周 | PHASE-SANDBOX-002-IMPL-REPORT + UAT 8 AC 验收 + 推 origin |
| **合計** | **16** | **~3.5M** | **3.5-4 周** | |

### 1.2 Out-of-Scope

- **TLS / mTLS 双向认证** → v0.2 拍摄
- **真实 K8s / Helm 部署** → v0.2 拍摄
- **多租户隔离** → v0.2 拍摄
- **AI 行为审计 (LLM 决策录屏)** → v2.x
- **加密 / 凭据管理 (mavis 内置 vault)** → 跨项目需求
- **跨域编排 (5 域 Lead DDD Review / Saga orchestrator)** → 跨 P3-B 子项

---

## §2 P0 骨架阶段 (4 子项 / ~0.5M token / 1 周)

### P0.1 sandboxd crate 骨架

| 项 | 详情 |
|---|---|
| 标题 | sandboxd crate 骨架 + Cargo.toml + 6 模块占位 + lib.rs |
| Token 估 | ~0.1M |
| 估时 | 1-2 天 |
| 关联 DD §1.1 | 18 源 + 6 测试 + 1 binary |
| 验收 | `cargo check --workspace --lib -j 4` 0 err (per 守门 #1 v25 + 守门 #1 v19) |
| 触发 | 2026-09-10 22:11 JST Ulysses 拍板"各级文档完善好" |
| 守门 | #1+#1 v19+#1 v25+#5+#6+#14 v3+#14 v4+#28 |

### P0.2 proto schema + 4 RPC stub

| 项 | 详情 |
|---|---|
| 标题 | `proto/sandboxd.proto` 落地 + tonic-build 自动生成 + SandboxServiceImpl 4 RPC stub |
| Token 估 | ~0.1M |
| 估时 | 1-2 天 |
| 关联 DD §1.1 + §2.2 | proto/ + server.rs |
| 验收 | `cargo build -p sandboxd` 0 err + 4 RPC stub 可调 |
| 触发 | P0.1 完成 |
| 守门 | #1+#1 v19+#1 v25+#14 v3+#14 v4 |

### P0.3 Linux Resource 维 (cgroups v2) 落地

| 项 | 详情 |
|---|---|
| 标题 | `backend/linux.rs` cgroups v2 落地 + SandboxLimits 应用 + 跟 SANDBOX-001 v0.2 100% 兼容 |
| Token 估 | ~0.2M |
| 估时 | 2-3 天 |
| 关联 DD §1.1 + §2.1 + §5.1 | backend/linux.rs + SandboxLimits |
| 验收 | `cargo test -p sandboxd --lib -j 4` 8+ tests pass (TC-RES-01~08) + 真实 cgroups v2 实测 (e.g. memory 限制 100MB, 跑 cargo build 不 OOM) |
| 触发 | P0.2 完成 |
| 守门 | #1+#1 v19+#1 v25+#5+#14 v3+#14 v4 |

### P0.4 fail-open 降级路径

| 项 | 详情 |
|---|---|
| 标题 | sandboxd 不可用时, dispatcher 走 SANDBOX-001 v0.2 subprocess.run 降级 |
| Token 估 | ~0.1M |
| 估时 | 1-2 天 |
| 关联 BD §1.3 + DD §3.3 | fail-open 降级时序 |
| 验收 | IT TC-IT-FO-01~04 pass + root session 通知 < 500ms |
| 触发 | P0.3 完成 |
| 守门 | #1+#1 v19+#1 v25+#9+#9 v3+#14 v3+#14 v4+#22 |

---

## §3 P1 三平台阶段 (4 子项 / ~1.0M token / 1-1.5 周)

### P1.1 Windows Resource + Network (Job Objects + WFP)

| 项 | 详情 |
|---|---|
| 标题 | `backend/windows.rs` Job Objects + WFP 落地 + 跟 SANDBOX-001 v0.2 Resource 维 100% 复用 |
| Token 估 | ~0.3M |
| 估时 | 3-4 天 |
| 关联 BD §5.1 + DD §5.1 | windows crate 0.58 + Job Objects + WFP |
| 验收 | `cargo test -p sandboxd --lib -j 4` 8 tests pass (TC-RES-08 + TC-NET-08) + Windows CI pass |
| 触发 | P0 完成 |
| 守门 | #1+#1 v19+#1 v25+#5+#6+#14 v3+#14 v4+#28 |

### P1.2 Linux Network + FS + Capability (netns + mount ns + libcap)

| 项 | 详情 |
|---|---|
| 标题 | `backend/linux.rs` 扩展: netns (iptables) + mount namespace + libcap |
| Token 估 | ~0.3M |
| 估时 | 3-4 天 |
| 关联 BD §5.2 + DD §5.1 | nix 0.28 + caps 0.5 |
| 验收 | `cargo test -p sandboxd --lib -j 4` 8+7+5 = 20 tests pass + Linux CI pass |
| 触发 | P1.1 完成 |
| 守门 | #1+#1 v19+#1 v25+#5+#6+#14 v3+#14 v4 |

### P1.3 macOS 后端 (sandbox-exec)

| 项 | 详情 |
|---|---|
| 标题 | `backend/macos.rs` sandbox-exec profile 落地 + 4 维隔离 (resource + network + fs + capability implicit) |
| Token 估 | ~0.3M |
| 估时 | 3-4 天 |
| 关联 BD §5.3 + DD §5.1 | sandbox 0.5 crate |
| 验收 | `cargo test -p sandboxd --lib -j 4` 4 tests pass (TC-NET/MET/CAP/AUD) + macOS CI pass |
| 触发 | P1.2 完成 |
| 守门 | #1+#1 v19+#1 v25+#5+#6+#14 v3+#14 v4 |

### P1.4 跨平台 backend factory + UT 56 测全跑

| 项 | 详情 |
|---|---|
| 标题 | `backend/mod.rs` DefaultBackendFactory 三平台 + 56 UT 全部 pass |
| Token 估 | ~0.1M |
| 估时 | 1-2 天 |
| 关联 DD §1.1 + §5.1 | backend/mod.rs + DefaultBackendFactory |
| 验收 | `cargo test -p sandboxd --lib -j 4` 56/56 pass (per 守门 #1 v25 单 crate) |
| 触发 | P1.3 完成 |
| 守门 | #1+#1 v19+#1 v25+#5+#6+#9+#14 v3+#14 v4+#28 |

---

## §4 P2 client 集成阶段 (3 子项 / ~0.8M token / 1 周)

### P2.1 dispatcher.py gRPC client 集成

| 项 | 详情 |
|---|---|
| 标题 | `scripts/automation/dispatcher.py` v0.2: SandboxdClient 替代 subprocess.run + fail-open 降级 |
| Token 估 | ~0.3M |
| 估时 | 3-4 天 |
| 关联 DD §1.3 + §3.3 | dispatcher.py 集成点 |
| 验收 | E2E TC-E2E-PY-01~04 pass + 跟 SANDBOX-001 v0.2 兼容 (IT TC-IT-FO-01~04 pass) |
| 触发 | P1 完成 |
| 守门 | #1+#5+#6+#9+#9 v3+#11+#14 v3+#14 v4+#22+#28 |

### P2.2 mavis desktop (Rust) gRPC client 集成

| 项 | 详情 |
|---|---|
| 标题 | `crates/star-mcp/src/sandboxd_client.rs` 新建 + SandboxdClient 跟 Python client 对称 |
| Token 估 | ~0.3M |
| 估时 | 3-4 天 |
| 关联 DD §1.3 + §1.3 | star-mcp 集成点 |
| 验收 | E2E TC-E2E-RS-01~04 pass + cargo test 4/4 pass |
| 触发 | P2.1 完成 |
| 守门 | #1+#5+#6+#9+#9 v3+#14 v3+#14 v4+#22+#28 |

### P2.3 17 IT + 11 E2E 全部 pass

| 项 | 详情 |
|---|---|
| 标题 | 跨模块 17 IT + dispatcher + mavis 11 E2E 全部 pass (含 4 RPC 端到端 + 4 fail-open + 5 PG audit + 4 dispatcher + 4 mavis + 3 跨平台) |
| Token 估 | ~0.2M |
| 估时 | 2-3 天 |
| 关联 TDD §3 + §4 | IT + E2E 全部 pass |
| 验收 | `cargo test -p sandboxd --tests -j 4` 17/17 IT pass + E2E 11/11 pass (pytest + cargo test) |
| 触发 | P2.2 完成 |
| 守门 | #1+#1 v19+#1 v25+#5+#6+#9+#9 v3+#14 v3+#14 v4+#22+#28 |

---

## §5 P3 fail-open 兼容性 + 性能阶段 (3 子项 / ~0.6M token / 0.5-1 周)

### P3.1 SANDBOX-001 v0.2 降级路径端到端验证

| 项 | 详情 |
|---|---|
| 标题 | sandboxd 不可用时, 走 SANDBOX-001 v0.2 subprocess.run 降级端到端验证 |
| Token 估 | ~0.2M |
| 估时 | 2-3 天 |
| 关联 BD §1.3 + DD §3.3 | fail-open 降级时序 + 集成点 |
| 验收 | IT TC-IT-FO-01~04 pass + E2E TC-E2E-PY-03 pass (dispatcher 降级) |
| 触发 | P2 完成 |
| 守门 | #1+#1 v19+#1 v25+#9+#9 v3+#14 v3+#14 v4+#22+#28 |

### P3.2 5 bench 全部 P95 < 阈值

| 项 | 详情 |
|---|---|
| 标题 | CreateSession p50/p99 + RunCommand 端到端 + 100 session 并发 + gRPC IPC 序列化 5 bench 全部 P95 < 阈值 |
| Token 估 | ~0.2M |
| 估时 | 2-3 天 |
| 关联 TDD §5 | 5 bench 实证 |
| 验收 | `cargo bench -p sandboxd -- --quick` 5/5 bench P95 < 阈值 (per 守门 #1 v25) |
| 触发 | P3.1 完成 |
| 守门 | #1+#1 v19+#1 v25+#5+#14 v3+#14 v4+#28 |

### P3.3 故障注入测试 (fail-open / fail-closed)

| 项 | 详情 |
|---|---|
| 标题 | sandboxd 不可用 / 启动中 / 健康检查失败 / PG 不可用 4 类故障注入 + fail-open / fail-closed 二分验证 |
| Token 估 | ~0.2M |
| 估时 | 2-3 天 |
| 关联 TDD §3.2.2 + §3.2.3 + §6.1 AC-3 | 故障注入测试矩阵 |
| 验收 | AC-3 100% fail-open + 100% fail-closed 实证 |
| 触发 | P3.2 完成 |
| 守门 | #1+#1 v19+#1 v25+#5+#6+#9+#9 v3+#14 v3+#14 v4+#22+#28 |

---

## §6 P4 收官阶段 (2 子项 / ~0.6M token / 0.5 周)

### P4.1 PHASE-SANDBOX-002-IMPL-REPORT 落档

| 项 | 详情 |
|---|---|
| 标题 | `docs/reports/PHASE-SANDBOX-002-IMPL-REPORT.md` 落档 (跟现有 6 份 PHASE-*-IMPL-REPORT 模式一致) |
| Token 估 | ~0.3M |
| 估时 | 2-3 天 |
| 关联 SRS §7 + DD §11 + TDD §6.2 | 报告 7 段结构 (目的/进度/验证/缺口/失败接手/守门/签字+修订) |
| 验收 | 报告 commit author=Ulysses + 守门 12/12 实证 + WBS row 落档 |
| 触发 | P3 完成 |
| 守门 | #1+#1 v15+#1 v19+#1 v25+#5+#6+#9+#11+#12 v21+#14 v3+#14 v4+#22+#28 |

### P4.2 UAT 8 AC 验收 + 推 origin

| 项 | 详情 |
|---|---|
| 标题 | UAT 8 AC 全部 100% pass + 5 域 Lead 签字 (Mavis 临时代签) + 推 origin main |
| Token 估 | ~0.3M |
| 估时 | 2-3 天 |
| 关联 TDD §6.2 + SRS §7 | UAT 验收 + 5 域 Lead 签字 (Mavis 代签) + 推 origin (per 守门 #1 反转 2026-08-30 07:09 JST 推 origin 已落地) |
| 验收 | UAT 8 AC 100% + 5 域 Lead 签字 + `git push origin main` 成功 |
| 触发 | P4.1 完成 |
| 守门 | #1+#1 v15+#1 v19+#1 v25+#5+#6+#9+#11+#14 v3+#14 v4+#22+#28 |

---

## §7 Token 预算 + 风险矩阵

### 7.1 Token 预算汇总 (per STAR-OLU-001 1 SRE·周 = 1.2M)

| 阶段 | 软预算 | 比例 | 风险 |
|---|---|---|---|
| P0 骨架 | 0.5M | 14% | 🟢 低 (Linux 单平台 + Resource 维) |
| P1 三平台 | 1.0M | 29% | 🟡 中 (macOS 第一次集成, 可能有 sandbox-exec profile 调试) |
| P2 client 集成 | 0.8M | 23% | 🟡 中 (跟 dispatcher + mavis 集成, 跨语言 RPC 调试) |
| P3 fail-open + 性能 | 0.6M | 17% | 🟢 低 (降级路径已落地, 性能 bench 标准) |
| P4 收官 | 0.6M | 17% | 🟢 低 (报告模板 + 推 origin) |
| **合計** | **3.5M** | **100%** | |

### 7.2 风险矩阵

| 风险 | 严重度 | 触发 | 缓解 |
|---|---|---|---|
| macOS sandbox-exec 调试超 0.5M | P1 | profile 语法错 / 权限问题 | 走 Mavis 跨 session 续做 + Ulysses 内推 macOS 工程师 |
| gRPC schema breaking change 致 mavis 集成失败 | P1 | mavis 升级 | 走 buf 兼容 + 关注 changelog |
| cgroups v2 在老 Linux 不可用 | P1 | 容器内 / 老 kernel | 走 cgroups v1 fallback (per 守门 #11 缺标比错标显式标) |
| 5 域 Lead 真人到位延迟 | P2 | 真人未到位 | 维持 Mavis 临时代签 (per 守门 #14 v3) |
| 推 origin 网络故障 | P1 | Recv failure / Connect failed / timeout | 走守门 #1 推 origin 重试细则 (max 2 retries) |

---

## §8 跟守门 / WBS 联动

### 8.1 跟守门联动 (per 守门 #11 缺标比错标)

| 守门 | 跟 IMPL-PLAN 联动 |
|---|---|
| #1 fail-open | P3.1 fail-open 端到端验证 |
| #1 v15 docs 同步饱和 | P4.1 报告落档 |
| #1 v19 -j 4 修正 | 所有 cargo test 命令加 `-j 4` |
| #1 v25 cargo test 单 crate | P1.4 + P2.3 + P3.2 + P4.2 走单 crate cargo test |
| #5 env 安全 | 所有 P 阶段不读 env 内容 |
| #6 PowerShell + 跨平台 | P1 三平台 (Win + Linux + macOS) |
| #9 子代理 dispatch | P2.1 dispatcher.py 集成 + P3.1 降级 |
| #9 v3 调试控制台 subprocess | P3.1 走 subprocess.run 降级 |
| #11 缺标比错标 | §7 风险矩阵 + §1.2 Out-of-Scope + 9 已知缺口 |
| #12 v21 [P] docs 同步 | P4.1 报告落档 + WBS row |
| #13 T/M 100% 覆盖 | P0.3 + P1.1 + P1.2 + P1.3 4 表 DDL |
| #14 v3 Mavis 永久代签 | 5 域 Lead 签字栏 (Mavis 临时代签) |
| #22 mavis desktop 集成 | P2.2 mavis desktop 集成 |
| #28 拍板必带推荐项 | 6 决策点 D-1~D-6 全部已拍板 per Ulysses A 选项 |

### 8.2 跟 WBS 联动

- 新增 §14.6 "SANDBOX-002 sandboxd 实装" 5 阶段 × 16 子项任务表 (per §9 WBS 更新格式)
- 跟既有 §14.1 (行业预设) + §14.2 (H2 强类型重构) + §14.3 (DB W/T/M) + §14.4 (跨 Phase 阻塞项) 并列
- 不破坏 P3-B 占位表 (§1 §2 §3 §4 §5) + 守门基线 (§12.6)

---

## §9 修订履歴 (詳細)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 22:11 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 5 阶段 × 16 子项 × ~3.5M token 总预算, P0 骨架 0.5M / P1 三平台 1.0M / P2 client 集成 0.8M / P3 fail-open + 性能 0.6M / P4 收官 0.6M, 跟 SRS-002 + BD-002 + DD-002 + TDD-002 100% 对齐, 风险矩阵 5 项显式标, 跟守门 14 联动 + WBS §14.6 联动 | 2026-09-10 22:11 JST Ulysses 拍板"各级文档完善好, 更新后续任务到 wbs" + DD-SANDBOX-002 v0.1 + TDD-SANDBOX-002 v0.1 派生 |

---

## §10 签字栏 (Sign-off)

| 角色 | 氏名 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | ✅ 2026-09-10 | 2026-09-10 JST |
| SRE Lead | SRE Lead (Mavis 临时代签 per 9/3 11:35 JST 拍板 B, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 平台 Lead | 平台 Lead (Mavis 临时代签 per 守门 #14 v3, 真人到位后追溯) | ✅ 2026-09-10 | 2026-09-10 JST |
| 评审主持 | 评审主持 (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |
| PM | PM (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-10 | 2026-09-10 JST |

(per 守门 #14 v4 反转 v0.62, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)
