# P1 阻断项汇总 — Phase C 第 2 轮 3 子代理 cross-validate

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— 自审报告
> **签批**：⏳ 待 Ulysses 终审

---

## 0. 报告目的

3 子代理（A / B / C）独立审查完 Phase C 54 份 spec 后，**共识 4 项 P1 阻断项 + 子代理独立 4 项 P1**。本汇总报告是 Mavis 自审后产物，给 Ulysses 终审时**一目了然**的 P1 优先级列表 + 推荐修法。

## 1. P1 阻断项（Phase D 实施前必关）

### 1.1 共识 P1（3 子代理都发现）

| P1 | 描述 | 修法 | 来源 |
|---|---|---|---|
| **P1-A** | CLI "17 命令" vs cli/01 §2 表 23 命令 + arch/03 §2.2 bash 块 18 命令 — **3 处数字不一致** | cli/01 §2 加 "MVP 17 子集边界" 标注 + 拆"扩展命令"附表 | 子代理 A 🔴 #1 + 子代理 C P1-3 |
| **P1-B** | Universal Submit 11 vs 12 步（flows/05 §2 文字 + 列表矛盾，列表多"12. 回写 IDE Session 状态"）| 文字 + 列表统一到 12（建议保留 IDE Session 回写步）| 子代理 A 🔴 #7 + 子代理 C P1-6 |
| **P1-C** | `agent-api/v1#Workspace` 实际是 IDE 视角（agent-api 引用到 `ide-api` 的 schema，含 open_files / diagnostics / ide_client）—— **跨层数据泄漏** | 拆 `WorkspaceSummary`（agent 视角，agent-api）+ `WorkspaceState`（IDE 视角，ide-api），CLI `star workspace current` 引用改 WorkspaceSummary | 子代理 A 🔴 #5 + 子代理 B 18 份 spec 多次引 |
| **P1-D** | `agent-api/v1` 21 个 schema 引用，§3 主体只展开 3 个（Task / Worktree / SubmitResult），**18 个 schema 黑盒** | §3 扩展为 3.1-3.15（15 个核心 schema 各 3-5 字段定义）+ 落盘 `crates/star-cli/src/schemas/agent-api-v1/*.json` | 子代理 A 🔴 #4 + 子代理 B 7 处引 + 子代理 C 2 处引 |

### 1.2 子代理独 flag P1（3 子代理部分发现）

| P1 | 描述 | 修法 | 来源 |
|---|---|---|---|
| **P1-E** | MCP spec 漏 2026-07-28 关键变更 6 项（stateless / Header routing / ttlMs / Feature Lifecycle / RFC 9207 / MRTR）| mcp/01 §1 加 6 项变更符合度表 + 工具表 metadata 列 ttlMs | 子代理 A 🔴 #2 |
| **P1-F** | MCP §7 说"必须能 invoke star submit"但 §2 工具表**无 submit tool** | 加 `submit` tool（domain 语义），或 §7 删验证项 | 子代理 A 🔴 #3 |
| **P1-G** | 错误模型 4 套并存（CLI 5 字段 / Submit 4 字段 / MCP 无 / REST 无）| 统一为 `agent-api/v1#Error` 单 schema，CLI/MCP/REST/Submit 全部引用 | 子代理 A 🔴 #6 |
| **P1-H** | Universal Submit 5 步无独立 CLI 命令（diff / policy / commit / push / link）| 加 5 个命令（`star diff` / `star policy check` / `star commit` / `star push` / `star mr link`）| 子代理 A 🔴 #7 |

### 1.3 MCP / REST 数字矛盾（子代理 C 独 flag）

| P1 | 描述 | 修法 | 来源 |
|---|---|---|---|
| **P1-I** | MCP 13 tools vs arch/03 列 14（多 `request_review`） | arch/03 §2.3 加"MVP 13 子集边界"，14 = 13+1 扩展 | 子代理 C P1-4 |
| **P1-J** | REST 12 endpoints vs arch/05 列 14 | arch/05 §5 加"MVP 12 子集边界"，14 = 12+2 扩展 | 子代理 C P1-5 |

### 1.4 arch/03 vs acceptance 冲突（子代理 C 独 flag）

| P1 | 描述 | 修法 | 来源 |
|---|---|---|---|
| **P1-K** | arch/03 §7 主张"Unknown Agent Test 必须 Level 4 通过"，但 acceptance/01 §3 实际 16 步用 star CLI = Level 1，**不可同时成立** | arch/03 §7 改"Unknown Agent Test 跑 Level 1，Level 2/3/4 单独跑 conformance" | 子代理 C P1-1 |

### 1.5 测试位置不一致（子代理 C 独 flag）

| P1 | 描述 | 修法 | 来源 |
|---|---|---|---|
| **P1-L** | vcs/04 §5 说 `crates/star-cli/tests/`；acceptance/01-03 说 `tests/unknown-*/` | vcs/04 §5 改 `tests/` 而非 `crates/star-cli/tests/` | 子代理 C P1-2 |

### 1.6 子代理 B 独 flag P1

| P1 | 描述 | 修法 | 来源 |
|---|---|---|---|
| **P1-M** | flows/03 Resume JSON `current_state: "Implementing"` vs flows/01 §1 `IMPLEMENTING`（大小写不一致）| 统一为 Rust PascalCase（Implementing），flows/01 §1 改 PascalCase + 文档加注"以 Rust enum 为准" | 子代理 B 🔴 B-01 |
| **P1-N** | flows/01 实际是 9+5=14 状态，任务摘要写 9+4 | 保留 5 个异常状态（业务语义完整）| 子代理 B 🔴 B-02 |
| **P1-O** | flows/03 Resume JSON 11 字段未在 agent-api/v1 schema 定义（本任务范围外）| Phase C 第 3 轮把 agent-api/01-schema.md 纳入审查范围 | 子代理 B 🔴 B-19 + P3 #18 |

---

## 2. P0 修补（已修）

| P0 | 描述 | 修法 | commit |
|---|---|---|---|
| **P0-1** | 18 份 spec 引用 `../../adr/...` 解析到 architecture/adr/，但 5 份 ADR 在 `docs/adr/` 顶层，**11 个引用断链** | 复制 5 份 ADR 到 `docs/architecture/2026-08-26-upgrade/adr/`（方案 A，零改动 spec 文本）| `245cf56` |

## 3. 优先级建议

| 优先级 | 修 P1-A / B / C / D | 修 P1-E / F / G / H | 修 P1-I / J / K / L | 修 P1-M / N / O |
|---|---|---|---|---|
| **P0**（开工前必关）| ✅ 4 项 | ✅ 4 项 | ✅ 4 项 | 🟡 Phase D 同步 |
| **P1**（实施同期）| — | — | — | ✅ 3 项 |

**结论**：**15 项 P1 全部 P0 优先**，Phase D 开工前必关。

## 4. 不阻塞 P1 但仍需注意的 P2（按子代理汇总）

- 子代理 A P2 #8-#20：13 项（如 CLI ↔ MCP 命名映射 / REST 端点覆盖度 / OpenAPI 3.1 关键字段 / agent-api info.version 关系等）
- 子代理 B P2 #5-#17：13 项（如 flows/02 Agent Lost 恢复 / Audit Service 缺定义 / 4 类权限主体不全 / 4 处 spec 引用 agent-api 但本任务范围外）
- 子代理 C P2 #1-#12：12 项（如 arch/03 §7 "真实 Agent 4 款" vs acceptance/01 "自实现 minimal agent" 职责切分 / acceptance/03 §3 10 步未消费 OpenAPI / NFR-OP-001 单位 "SRE·周/周" 跟 token-OLU 偏好不对齐 / R-003 工具链断言无版本号 / R-007 cache 层无指等）

**合计 38 项 P2**——不阻塞 Phase D 实施，但落地时需逐条消解。

## 5. P3 已知缺口（per "缺标比错标安全"）

- 子代理 B §3 P3 #18-#25：8 项（agent-api/ide-api/cli/rest/mcp/vcs/acceptance/arch 等 27 份 spec 未在审查范围）
- 子代理 C §9 弱信号：12 项
- 5 份基础文件 §6 已知缺口：3 项

**合计 23 项 P3**——给后续 Phase C 第 3 轮 / Phase D 实施时当 backlog 备查。

---

## 6. 守门规则遵循度（per 3 子代理自报）

| 子代理 | 不沿用 bc23d6c 叙事 | 不编造 commit hash | 缺标比错标 | 不写代码 | 不改 spec | 不 commit | 签字栏完整 |
|---|---|---|---|---|---|---|---|
| A (接口) | ✅ 显式拒绝 3 处 | ✅ 无 | ✅ 8+ 处缺口 | ✅ | ✅ | ✅ | ✅ |
| B (流程) | ✅ 0 处提到 | ✅ 12 个 "待补" | ✅ 8+ 处缺口 | ✅ | ✅ | ✅ | ✅ |
| C (验收) | ✅ 0 处提到 | ✅ 无 | ✅ 8+ 处缺口 | ✅ | ✅ | ✅ | ✅ |

3 子代理全部通过自审 + 5 项硬约束（Mavis 自审 2026-08-26 22:38 JST 报告）。

---

## 7. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008）| 2026-08-26 | 🟡 草案 v0.1；P1 阻断项 15 项 + P0 修补 1 项 + P2 警告 38 项 + P3 已知缺口 23 项；待 Ulysses 终审 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM）| ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008）| 初版：P1 汇总 + P0 修补记录 + P2/P3 链接 | 3 子代理完成后 Mavis 自审 |
