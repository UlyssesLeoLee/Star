#!/usr/bin/env python3
"""
IPA SEC 合规性修复脚本 (per 守门 #9 v19 Mavis 自驱 + 守门 #1 v19 批量改 1 commit 收官).

触发: 2026-09-10 19:06 JST Ulysses "确保这里的文档符合日本IPA标准" 指令.
范围: P3-D.5 10 份新写文档, 5 处 IPA SEC 不合规修复.
落地: 1 commit 5 文件 + 1 任务卡同步 commit (per 守门 #12 v21).
"""
import sys
from pathlib import Path

REPO_ROOT = Path("D:/Star")


# ===== Fix 1: SRS-CANVAS-001.md =====
# 当前: §0-§9 (10 段), 缺独立 §10 5 角色签字栏 段, §9 修订履历 需改 §11
def fix_srs_canvas_001() -> int:
    """Add §10 5 角色签字栏 段, renumber §9 修订履历 → §11 修订履历."""
    path = REPO_ROOT / "docs/requirements/SRS-CANVAS-001.md"
    content = path.read_text(encoding="utf-8")
    print(f"Read {len(content)} bytes from {path.name}")

    # Check if already fixed (idempotent)
    if "## §10 5 角色签字栏" in content:
        print(f"  {path.name}: §10 already exists, skipping")
        return 0

    # Add §10 5 角色签字栏 before §9 修订履历, and renumber §9 → §11
    section_10_content = """## §10 5 角色签字栏 (per AGENTS.md §3 7 段结构, per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D + 9/10 12:45 JST v0.62 反转)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-10 JST | 5 域 Lead 真人未到位, Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 真人到位后追溯签字 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

**5 域 Lead (player / economy / match / social / admin)** 真人未到位, Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)

---

"""

    # Renumber §9 → §11 修订履历 (the "## §9 修订履历" line)
    content = content.replace(
        "## §9 修订履历\n\n### 9.1 本总册",
        f"{section_10_content}## §11 修订履历\n\n### 11.1 本总册",
        1,
    )
    # Update sub-headings 9.1, 9.2, 9.3 → 11.1, 11.2, 11.3
    content = content.replace("### 9.1 本总册 (SRS-CANVAS-001) 修订履历", "### 11.1 本总册 (SRS-CANVAS-001) 修订履历")
    content = content.replace("### 9.2 平行 2 专题 SRS 修订履历", "### 11.2 平行 2 专题 SRS 修订履历")
    content = content.replace("### 9.3 跨拍板派生", "### 11.3 跨拍板派生")

    path.write_text(content, encoding="utf-8")
    print(f"  Wrote {len(content)} bytes to {path.name}")
    return 1


# ===== Fix 2: SRS-CANVAS-GAMIFY-001.md =====
# 当前: §0-§9 (10 段), 缺独立 §10 5 角色签字栏 段 (在附录 D)
def fix_srs_canvas_gamify_001() -> int:
    """Add §10 5 角色签字栏 段, renumber §9 → §11 修订履历."""
    path = REPO_ROOT / "docs/requirements/SRS-CANVAS-GAMIFY-001.md"
    content = path.read_text(encoding="utf-8")
    print(f"Read {len(content)} bytes from {path.name}")

    if "## §10 5 角色签字栏" in content:
        print(f"  {path.name}: §10 already exists, skipping")
        return 0

    section_10_content = """## §10 5 角色签字栏 (per AGENTS.md §3 7 段结构, per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D + 9/10 12:45 JST v0.62 反转)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-10 JST | 5 域 Lead 真人未到位, Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 真人到位后追溯签字 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

**5 域 Lead (player / economy / match / social / admin)** 真人未到位, Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)

---

"""

    # Insert §10 before §9 修订履历, renumber §9 → §11
    # SRS-GAMIFY-001 has simple table without sub-headings (per grep)
    content = content.replace(
        "## §9 修订履历\n\n| 版本",
        f"{section_10_content}## §11 修订履历\n\n| 版本",
        1,
    )
    # Update sub-headings 9.1 → 11.1 (and 9.2, 9.3 if any)
    content = content.replace("### 9.1", "### 11.1")
    content = content.replace("### 9.2", "### 11.2")
    content = content.replace("### 9.3", "### 11.3")

    path.write_text(content, encoding="utf-8")
    print(f"  Wrote {len(content)} bytes to {path.name}")
    return 1


# ===== Fix 3: DD-CANVAS-001.md =====
# 当前: §0-§10 (11 段), 缺 §11 NFR + §12 5 角色签字栏 + §13 修订历史 段
def fix_dd_canvas_001() -> int:
    """Add §11 NFR + §12 5 角色签字栏 + §13 修订历史 段 (after §10 测试用例)."""
    path = REPO_ROOT / "docs/design/DD-CANVAS-001.md"
    content = path.read_text(encoding="utf-8")
    print(f"Read {len(content)} bytes from {path.name}")

    if "## §11 NFR 详细" in content:
        print(f"  {path.name}: §11 already exists, skipping")
        return 0

    new_sections = """## §11 NFR 详细 (Non-Functional Requirements, 6 类, per BD §7 + AGENTS.md §3 守门 19/19 跨域)

### 11.1 NFR-CANVAS-PERF: 性能 (跨域 ≥ 6 项, per 守门 #1 v15)

| ID | 名称 | 指标 | 验证方式 | 优先级 |
|---|---|---|---|---|
| NFR-CANVAS-PERF-01 | 画布首次渲染 | ≤ 500ms (mock 12 agent + 12 worktree + 30 wi + 50 ARG 边 + 10 多人并发) | FCP / LCP | P0 |
| NFR-CANVAS-PERF-02 | 状态同步延迟 | ≤ 200ms (P95) 反映到 agent_node 色码 | dev tools + P95 | P0 |
| NFR-CANVAS-PERF-03 | ARG 边创建 latency | P95 < 200ms (Memgraph Bolt) | Prometheus | P0 |
| NFR-CANVAS-PERF-04 | 多人同时编辑 Realtime 同步 | 10 用户并发, 元素增删改 ≤ 200ms (P95) | dev tools + WSS | P0 |
| NFR-CANVAS-PERF-05 | 实时 Cursor 同步 | cursor 移动 ≤ 100ms (P95, throttle 50ms) | dev tools + P95 | P0 |
| NFR-CANVAS-PERF-06 | Follow mode 同步 | viewport 同步 ≤ 100ms (P95) | dev tools + P95 | P1 |

### 11.2 NFR-CANVAS-RELIABILITY: 可靠性 (3 项, per 守门 #1 累积规)

- **NFR-CANVAS-RELI-01**: 元素增删改 100% 持久化 (PostgreSQL `canvas_elements_backend` + audit `canvas_multi_user_audit` 双写, per 守门 #13 Transaction 100% audit)
- **NFR-CANVAS-RELI-02**: WSS 断线自动 retry 3 次 (exponential backoff 1s/2s/4s), 重连后自动 sync, 不丢操作 (per UC-A14)
- **NFR-CANVAS-RELI-03**: Memgraph 不可达时 in-process 缓存继续工作 (派生自 `BD-AGENT-RELATIONSHIP-001` §7.2, per §9.2 离线降级)

### 11.3 NFR-CANVAS-SECURITY: 安全 (per 守门 #5 + 守门 #13 + 9/1 13:03+13:05 JST envoy 偏好)

- **NFR-CANVAS-SEC-01**: 13 租户隔离 (RLS 13 类必携, per 守门 #13), audit log 必记 (per 守门 #13 Transaction 100% audit)
- **NFR-CANVAS-SEC-02**: A12 BFF 3 级权限校验走 envoy middleware (per A12.7 + 9/1 13:03+13:05 JST envoy 独立 deployment 偏好), 不依赖前端隐藏
- **NFR-CANVAS-SEC-03**: A12 WSS connection 走 TLS 1.3+ (per NFR-CANVAS-MU-CONS-01)
- **NFR-CANVAS-SEC-04**: A12.8 `canvas_multi_user_audit` 表 100% RLS 13 类必携 (per 守门 #13 + A12.8)
- **NFR-CANVAS-SEC-05**: Memgraph 连接字符串走 env (per 守门 #5), 不打印
- **NFR-CANVAS-SEC-06**: 代签规则 (per 守门 #10 + 9/8 15:19 第 6 次强化), author = Ulysses, 真人到位后追溯签字

### 11.4 NFR-CANVAS-USABILITY: 易用 (5 项, per 守门 #1 v15)

- **NFR-CANVAS-UI-01**: 视觉一致性 (跟 StatusPill 60+ 配色一致 / 跟 V0.1 `frontend-canvas-design.md` 一致 / 跟 ARG 10 类关系颜色规范一致 / dark mode 优先)
- **NFR-CANVAS-A11Y-01**: 键盘可达 (顶部 dropdown 满足 WAI-ARIA listbox 模式 / 右键菜单满足 menu 模式 / 多人 cursor 颜色不能仅靠颜色区分, 12 色调色板 + 形状/字母辅助, color-blind 友好)
- **NFR-CANVAS-DET-01**: 派生确定性 (同样输入永远出同样输出, 排序稳定 `[status_order ASC, due_date ASC, id ASC]`)
- **NFR-CANVAS-STATE-01**: 派生只读 + 跨块接口 (画布不进 zustand store 派生数据, 跨块接口走 URL 参数 + 跳路由, 不直接调其他 view 的 LangGraph node; **A12.3 多人编辑元素增删改走 backend 持久化 + WSS 广播, 不进 zustand persist**)
- **NFR-CANVAS-I18N-01**: 国际化 (3 语言 zh-CN / en / ja 友好, 至少 4 项 i18n key 落 `dictionary.ts`: agent status / role / kind / 关系 type 10 类 / **多人 cursor 名字 + 权限 view-comment-edit + 评论 + @ 提示**)

### 11.5 NFR-CANVAS-OBSERVABILITY: 可观测 (3 项, per 守门 #23 v2 调试控制台 + Prometheus)

- **NFR-CANVAS-OBS-01**: agent 状态变化 / ARG 边创建 / 状态联动 / **多人 cursor 移动 / 元素增删改 / 评论 / @** 100% audit + Prometheus 导出
- **NFR-CANVAS-OBS-02**: tracing 日志 (关系增删改 INFO / Memgraph 不可达 WARN / 成就解锁 INFO / trust_score 变化 DEBUG / 多人操作 INFO)
- **NFR-CANVAS-OBS-03**: OpenTelemetry 链路追踪 (元素创建 → 同步桥 → effect reload / 多人编辑 → BFF → audit 全链路)

### 11.6 NFR-CANVAS-ARG: ARG 特定 (派生自 `BD-AGENT-RELATIONSHIP-001` §7.4)

- **NFR-CANVAS-ARG-01**: 20 成就 3 维度分布 (8 拓扑 + 7 行为 + 5 产出, 稀有度 8 COMMON + 7 RARE + 3 EPIC + 2 LEGENDARY)
- **NFR-CANVAS-ARG-02**: 4 维度 effect reload ≤ 50ms (in-process state diff)
- **NFR-CANVAS-ARG-03**: Period flush 30s 周期, 0 阻塞 (独立 tokio task)
- **NFR-CANVAS-ARG-04**: WebSocket 推送 < 100ms (10 并发客户端, SSE fan-out)
- **NFR-CANVAS-ARG-05**: 5 模板 1-click 部署 ≤ 30s (1 事务 Cypher)
- **NFR-CANVAS-ARG-06**: 信任度动态 (成功 +0.01 / 失败 -0.05)

### 11.7 NFR-CANVAS-MU-CONS-01 (A12 多人编辑守门合规, 跨域)

- **5 域 Lead 真人未到位 disclaimer**: A12.7 3 级权限矩阵 + A12.5 评论 @ + A12.3 多人编辑元素增删改 + A12.1 多人同时编辑 + A12.2 实时 cursor + A12.4 Follow mode + A12.6 CRDT 选型 + A12.8 audit log, 共 8 项中 5 项 (A12.1/A12.6/A12.7) 需 5 域 Lead 真人到位后拍板, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)

---

## §12 5 角色签字栏 (per AGENTS.md §3 7 段结构, per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D + 9/10 12:45 JST v0.62 反转)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-10 JST | 5 域 Lead 真人未到位, Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 真人到位后追溯签字 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

**5 域 Lead (player / economy / match / social / admin)** 真人未到位, Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)

---

## §13 修订历史 (per AGENTS.md §3 7 段结构, per 守门 #1 禁回溯叙事不重写 v0.1 + 修订历史表 1 行)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-10 18:25 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **初始版本: 双核心 (管理 agent + 游戏化) + 多人编辑 (per 17:34 JST v0.63 反转) + ARG (per 17:21 JST) 总册 DD 跨域汇总, 11 段 + 5 附录, 15 章节 IPA SEC 模板, 5 view 跨域 + 14 张表 SQL DDL W/T/M 100% 覆盖 + 32 API 端点 OpenAPI spec + 5 WebSocket 协议 + 13 关键 class + 5 状态机 + 11 共享类型 + 4 关键时序图 + 6 类 NFR + 19 守门 + 12 已知缺口 + 5 角色签字栏** | **2026-09-10 18:25 JST Ulysses 拍板"完善详细设计文档" + 18:00 JST BD 拍板 + 17:34 JST v0.63 反转多人编辑 + 17:21 JST ARG 图论构造 + 17:08 JST 双核心 + 17:00 JST 旧方向撤回 (per 守门 #1 禁回溯叙事)** |
| **v0.1.1** | **2026-09-10 18:35 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **协调性检查修复: 3 处 — (1) §4 C-16 `CanvasBackend` → C-25 `CanvasElementsBackend` (本批新增 A12.3 顺延, 跟表名 `canvas_elements_backend` 一致) + §4 C-21 `MultiUserAudit` → C-26 `CanvasMultiUserAudit` (本批新增 A12.8 顺延, 跟表名 `canvas_multi_user_audit` 一致); (2) §5.3 Trust Score 5 档 命名/阈值统一为 ARG 源 `Untrusted (0-0.2) / Low (0.2-0.4) / Medium (0.4-0.7) / High (0.7-0.9) / VeryHigh (0.9-1.0)`; (3) §1.1 / §4 列表头同步更新 (C-21/C-16 → C-25/C-26). 不重写 ARG 4 份 commit (per 守门 #1 禁回溯叙事). BD-CANVAS-AGENT-001 v0.1 无需修复 (5.4.4 已用 ARG 源命名/阈值). DD-CANVAS-AGENT-001 v0.1 子代理 1 已用 C-16=RelationshipEditor / C-21=ARGController (匹配 ARG 源), A12 走 §4.14.x sub-class 模式 (与本总册 C-25/C-26 不同但等价, 跨 DD 一致性在 附录 A 派生源说明).** | **2026-09-10 18:30 JST Ulysses 拍板"确保本设计和 agent 图关系设计妥善协调不冲突" + 协调性检查报告 `COORDINATION-CHECK-001.md` v0.1 §3 修复方案** |
| **v0.1.2** | **2026-09-10 19:10 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **IPA SEC 合规性修复: 加 §11 NFR 详细 (6 类, 29 项, 跨域) + §12 5 角色签字栏 (per AGENTS.md §3 7 段结构, per 守门 #14 v2/v3/v4) + §13 修订历史 (从原 §0 修订履历 + 附录 E.1 跨拍板派生 抽出). 不重写 §0-§10 + 附录 A-E (per 守门 #1 禁回溯叙事, v0.1.2 反转行显式标).** | **2026-09-10 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准" + 守门 #9 v19 Mavis 自驱 + 守门 #1 v19 批量改 1 commit 收官** |

---

"""

    # Insert new sections after §10 测试用例 段
    # §10 测试用例 段 ends with the table, then 附录 A starts
    # Find: ## 附录 A:
    content = content.replace(
        "## 附录 A: 跨专题引用清单",
        f"{new_sections}## 附录 A: 跨专题引用清单",
        1,
    )

    path.write_text(content, encoding="utf-8")
    print(f"  Wrote {len(content)} bytes to {path.name}")
    return 1


# ===== Fix 4: DD-CANVAS-AGENT-001.md =====
# 当前: §0-§12 (13 段), 缺 §13 5 角色签字栏 + §14 修订历史 段
def fix_dd_canvas_agent_001() -> int:
    """Add §13 5 角色签字栏 + §14 修订历史 段 (after §12 守门合规)."""
    path = REPO_ROOT / "docs/design/DD-CANVAS-AGENT-001.md"
    content = path.read_text(encoding="utf-8")
    print(f"Read {len(content)} bytes from {path.name}")

    if "## §13 5 角色签字栏" in content:
        print(f"  {path.name}: §13 already exists, skipping")
        return 0

    new_sections = """## §13 5 角色签字栏 (per AGENTS.md §3 7 段结构, per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D + 9/10 12:45 JST v0.62 反转)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-10 JST | 5 域 Lead 真人未到位, Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 真人到位后追溯签字 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

**5 域 Lead (player / economy / match / social / admin)** 真人未到位, Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)

---

## §14 修订历史 (per AGENTS.md §3 7 段结构, per 守门 #1 禁回溯叙事不重写 v0.1 + 修订历史表 1 行)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-10 18:32 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **初版落档 — 子代理 1 bg_9ce1be6e 撰写, 13 段 (含 §11 NFR + §12 守门) + 5 附录, A1-A12 12 子能力 46 项 (A1-A10 28 + A11 10 + A12 8), 5 view 跨域, 5-tier 架构, 24 组件 + 6 crate + 1 BFF, 14 张表 W/T/M 100% 覆盖, 23 API + 5 WebSocket, 5 状态机 + 11 共享类型 + 13 关键 class (C-16=RelationshipEditor + C-21=ARGController 1:1 派生 ARG 源) + 6 时序图 (4 必含 + 2 补充 A12), NFR 6 类, 守门 19/19 + 26 派生规跨域全过, 19 已知缺口含 3 P0 阻塞 (#11 A12 WSS + #13 localStorage + #15 CRDT) + 1 跨 session 总结 (#17), 15 子代理失败接手; A11 1:1 派生自 `DD-AGENT-RELATIONSHIP-001.md` v0.1 模板; A12 1:1 派生自 `frontend-canvas-design.md` v0.1 §4.1+§4.6+§3.4** | **2026-09-10 18:25 JST Ulysses 拍板"完善详细设计文档"** |
| **v0.1.1** | **2026-09-10 19:10 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **IPA SEC 合规性修复: 加 §13 5 角色签字栏 (per AGENTS.md §3 7 段结构) + §14 修订历史 (从原附录 E 抽出). 不重写 §0-§12 + 附录 A-D (per 守门 #1 禁回溯叙事, v0.1.1 反转行显式标).** | **2026-09-10 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准"** |

---

"""

    # Insert new sections after §12 守门合规 (before 附录 A 跨专题引用)
    content = content.replace(
        "## 附录 A: 跨专题引用清单",
        f"{new_sections}## 附录 A: 跨专题引用清单",
        1,
    )

    path.write_text(content, encoding="utf-8")
    print(f"  Wrote {len(content)} bytes to {path.name}")
    return 1


# ===== Fix 5: COORDINATION-CHECK-001.md =====
# 当前: §1-§7 (7 段), 缺 §0 文档信息 / 修订履历 段
def fix_coordination_check_001() -> int:
    """Add §0 文档信息 / 修订履历 段 (before §1 检查范围)."""
    path = REPO_ROOT / "docs/reports/COORDINATION-CHECK-001.md"
    content = path.read_text(encoding="utf-8")
    print(f"Read {len(content)} bytes from {path.name}")

    if "## §0 文档信息 / 修订履历" in content:
        print(f"  {path.name}: §0 already exists, skipping")
        return 0

    section_0_content = """## §0 文档信息 / 修订履历 (per AGENTS.md §3 7 段结构 + IPA SEC 模板)

### 0.1 文档信息

| 项目 | 内容 |
|---|---|
| 文书 ID | COORDINATION-CHECK-001 |
| 文书名 | 无限画布设计 vs Agent Relationship Graph (ARG) 设计 协调性检查报告 |
| 版本 | v0.1 (协调性检查初始版) |
| 作成日 | 2026-09-10 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手) (per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses) |
| 关联 commit | `fb89e4a` (10 files / 8,160 insertions, 第 74 次新事件) + `153441a` (§4.29 + registry v0.14, 第 75 次新事件) + `34fa7e7` (DD-001 v0.1.1 自审 fix 2 处, 第 76 次新事件) |
| 触发 | 2026-09-10 18:30 JST Ulysses 拍板"确保本设计和 agent 图关系设计妥善协调不冲突" |

### 0.2 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-10 18:33 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **初始版本: 协调性检查报告, 10 项检查 7 通过 + 3 冲突发现 (C-16 + C-21 关键 class 编号冲突 + Trust Score 5 档 命名/阈值不匹配), 修复方案: 关键 class 顺延 (C-25 + C-26 新增) + Trust Score 5 档统一为 ARG 源, 7 段结构 (§0 文档信息 + §1-§5 检查 + §6 5 角色签字栏 + §7 修订履历, per AGENTS.md §3 模板)** | **2026-09-10 18:30 JST Ulysses 拍板"确保本设计和 agent 图关系设计妥善协调不冲突"** |
| **v0.1.1** | **2026-09-10 19:10 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **IPA SEC 合规性修复: 加 §0 文档信息 / 修订履历 段 (跟其他 9 份 P3-D.5 文档结构对齐, 之前 §1-§7 缺 §0 文档信息) + §6 5 角色签字栏 / §7 修订履历 已存在无需重写. 不重写 §1-§5 检查内容 (per 守门 #1 禁回溯叙事, v0.1.1 反转行显式标).** | **2026-09-10 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准"** |

---

"""

    # Insert §0 before §1 检查范围
    content = content.replace(
        "## §1 检查范围",
        f"{section_0_content}## §1 检查范围",
        1,
    )

    path.write_text(content, encoding="utf-8")
    print(f"  Wrote {len(content)} bytes to {path.name}")
    return 1


def main() -> int:
    """Execute all 5 IPA SEC compliance fixes."""
    print("=" * 70)
    print("IPA SEC 合规性修复 - 5 文档")
    print("=" * 70)

    fixes = [
        fix_srs_canvas_001,
        fix_srs_canvas_gamify_001,
        fix_dd_canvas_001,
        fix_dd_canvas_agent_001,
        fix_coordination_check_001,
    ]

    modified_count = 0
    for fix_fn in fixes:
        result = fix_fn()
        modified_count += result

    print("=" * 70)
    print(f"Done. {modified_count}/5 documents modified.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
