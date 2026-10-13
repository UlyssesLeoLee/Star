# File-Based Session Management System (FSMS)
> **架构集成状态（2026-10-12）**：本文件包作为设计提案并入 `dev`，尚未通过产品架构裁决。其中“FileSession 直接隶属 Worktree、与 Task Card 并列”等层级陈述与当前规范（Project → Cloud Branch → Engineering Run → Worktree；Run 内 App/工作项按 RunContext 协作）不一致。本提案不得作为已批准的实现依据；FileSession 的归属、与 Run/Worktree/Task Card 的关系、存储及 CLI 边界须重新评审，并同步需求、基本设计、详细设计后才能排期。

> **A Git-native, file-centric AI agent session management system for the Star monorepo**
> **Version**: v0.1 (Draft)
> **Status**: Design Phase
> **Author**: Ulysses (Mavis 接手)

---

## 📖 文档导航

> 架构裁决状态与实施前门槛见 [INTEGRATION-STATUS.md](./INTEGRATION-STATUS.md)。

| 文档 | 说明 | 适读人群 |
|------|------|----------|
| [01-requirements.md](./01-requirements.md) | **需求规格书** - 功能/非功能需求、用例、验收标准、**Worktree 双入口设计** | PM、架构师、开发 |
| [02-implementation-plan.md](./02-implementation-plan.md) | **实装计划** - 6 个 Phase、里程碑、依赖图、资源需求 | Tech Lead、开发 |
| [03-api-spec.md](./03-api-spec.md) | **REST API 规范** - 16 个端点、Schema、权限、错误码 | 后端开发、前端对接 |
| [04-cli-reference.md](./04-cli-reference.md) | **CLI 参考手册** - 完整命令、选项、示例、退出码 | 终端用户、脚本集成 |
| [ADR-0052](../2026-08-26-upgrade/adr/0052-file-session-management.md) | **架构决策记录** - 决策矩阵、架构图、数据模型、风险、**双入口定位** | 架构师、评审委员会 |

---

## 🎯 核心价值

```
传统模式:           Task Card → 1 Worktree → 1 AgentSession (粒度粗、耦合重)
                    ↓
FSMS 新模式:        点击任意文件 → 秒级开启独立 FileSession (粒度细、解耦、Git原生)
                    ↓
支持场景:           代码探索 / 重构 / 调试 / 文档生成 / 单测编写 / Code Review 辅助
```

**产品定位：Worktree 下的双入口设计**

```
Worktree (git worktree)
├── Entry 1: Task Card 维度 (任务导向)
│   ├── 业务意图驱动
│   ├── 1 Task Card → 1 Worktree → 1 AgentSession (ADR-0049)
│   ├── 适合: 需求实装、Bug 修复、Feature 开发 (有明确任务边界)
│   └── 生命周期 = 任务卡生命周期
│
└── Entry 2: FileSession 维度 (文件/探索导向)  ← 本设计
    ├── 代码文件驱动
    ├── 1 File → N FileSessions (并行、独立)
    ├── 适合: 代码探索、重构实验、调试复现、文档生成、单测编写、Code Review 辅助
    └── 生命周期 = 文件编辑会话级 (可随时创建/销毁/归档)
```

**UI/UX 设计要求** (文档记录，不实装)：
- Worktree 详情页 / 文件树视图中，**并列展示** "Task Cards" 与 "File Sessions" 两个 Tab/Section
- 文件右键菜单 / CodeLens 直接入口 "Start File Session"
- 两个入口数据正交：Task Card 可选关联 FileSession，FileSession 可选关联 Task Card
- 权限模型复用 Worktree 级权限 (Owner/Editor/Viewer)

### 动态交互体验标准 (参考 sakura 级水准)

> **基准**：sakura 终端/编辑器菜单交互 —— 菜单打开后移动鼠标，子菜单/上下文菜单能**跟随鼠标位置即时打开**，零感知延迟，无闪烁、无抖动。

| 交互场景 | 行为规范 | 性能指标 |
|----------|----------|----------|
| **文件树右键菜单** | 右键点击文件节点 → 100ms 内弹出上下文菜单；鼠标移动到菜单项 → 子菜单即时展开 (≤ 50ms) | 首帧渲染 < 100ms；子菜单展开 < 50ms |
| **CodeLens 悬停** | 编辑器标题栏/行内 CodeLens 显示 "🤖 N sessions"；鼠标悬停 → Tooltip 即时展开 (≤ 80ms)，显示会话列表预览 | Tooltip 首显 < 80ms；列表渲染 < 120ms |
| **Session 列表行悬停** | 列表行鼠标悬停 → 行背景高亮 + 操作按钮显示 (Resume/Checkpoint/Archive/Delete)；无布局抖动 | 悬停响应 < 30ms |
| **多级菜单导航** | 一级→二级→三级菜单；鼠标斜向移动进入子菜单区域时，**不关闭父菜单**，使用「安全三角区」判定 | 子菜单展开 < 50ms；误触关闭率 < 0.1% |
| **键盘/鼠标混合操作** | 键盘 ↑↓ 选中时鼠标不干扰焦点；Esc 关闭所有层级菜单；焦点陷阱正确 | 焦点切换 < 16ms |
| **触发区域容错** | 菜单边缘 8px 内移动不触发关闭；鼠标快速穿越菜单间隙不误关闭 | 容错半径 ≥ 8px |
| **动画缓动** | 菜单 `ease-out-cubic` 120ms；Tooltip `ease-out-quad` 80ms；无抖动、无掉帧 | 60fps 全程；无 layout shift |
| **虚拟化长列表** | Session 列表 > 100 项时自动虚拟化渲染；搜索/过滤即时响应 (< 100ms) | 滚动 60fps；筛选 < 100ms |

**实装技术要求**：Floating UI/Popper.js 定位引擎、菜单状态机 + rAF 批量 DOM、被动事件监听器、CSS contain 隔离重绘、absolute 定位避免 fixed 滚动问题。

**验收测试**：连续快速右键 20 节点无残留、斜向导航父菜单保持、CodeLens P99 < 120ms、resize 滚动自动重定位、键盘无障碍遍历。

---

| 特性 | 传统 Task Card CLI | FSMS FileSession |
|------|-------------------|------------------|
| **粒度** | 任务级 (整个 feature) | 文件级 (单个 .rs/.ts) |
| **启动延迟** | 10-30s (worktree + agent) | < 3s (直接 spawn) |
| **持久化** | PostgreSQL + SQLite | Git 仓库原生 (可选 SQLite 索引) |
| **版本历史** | 依赖 PG 审计表 | Git commit 历史 (天然 diff/blame) |
| **跨设备同步** | 需平台服务 | `git push/pull` 原生支持 |
| **协作评审** | 平台 MR | 导出为 GitHub/GitLab MR |
| **Agent 生态** | 内置 SA-Archetype | Claude/Codex/OpenCode/Hermes/自定义 |
| **与 Task Card 关系** | 1:1 强绑定 | 可选双向链接 (正交) |
| **隶属关系** | Task Card → Worktree | **FileSession → Worktree (与 Task Card 同级)** |

---

## 🏗️ 架构一览

```
┌─────────────────────────────────────────────────────────────────┐
│                        用户交互层                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  star CLI   │  │  VS Code    │  │  Multica    │              │
│  │  (Terminal) │  │  Extension  │  │  Web IDE    │              │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
└─────────┼────────────────┼────────────────┼─────────────────────┘
          │                │                │
          ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      FileSessionManager (Core)                   │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │ Registry/    │ │ Session      │ │ Context      │             │
│  │ Manifest     │ │ Factory      │ │ Builder      │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└────────────────────────────────┬────────────────────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         ▼                       ▼                       ▼
┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
│  AgentAdapter   │   │  GitManager     │   │  StorageBackend │
│  (Plugin Reg.)  │   │  (git2-rs)      │   │  (FS/SQLite/S3) │
│                 │   │                 │   │                 │
│ • Claude Code   │   │ • auto_commit   │   │ • FileStorage   │
│ • Codex         │   │ • worktree      │   │ • SqliteIndex   │
│ • OpenCode      │   │ • remote_sync   │   │ • S3Artifacts   │
│ • Hermes (ACP)  │   │ • mr_export     │   │ • PG Archive    │
│ • Custom WASM   │   │ • conflict_rslv │   │                 │
└─────────────────┘   └─────────────────┘   └─────────────────┘
```

---

## 📁 目录结构

```
<repo-root>/
├── .star/
│   ├── sessions/
│   │   ├── registry.json                    # 仓库级索引
│   │   ├── config.toml                      # FSMS 配置
│   │   ├── crates/
│   │   │   └── star-cli/
│   │   │       └── src/
│   │   │           └── commands/
│   │   │               └── task.rs          # ← 目标文件
│   │   │                   ├── manifest.json # 文件级会话列表
│   │   │                   ├── a1b2c3d4/    # session_id 目录
│   │   │                   │   ├── session.json      # 元数据
│   │   │                   │   ├── context/          # 上下文包
│   │   │                   │   ├── transcript/       # 对话记录 (分块)
│   │   │                   │   ├── artifacts/        # 产出物
│   │   │                   │   ├── git/              # Git 集成数据
│   │   │                   │   └── state/            # Agent 状态快照
│   │   │                   └── e5f6g7h8/            # 另一个 session
│   │   └── ...
│   └── config.toml
```

---

## 🚀 快速开始 (设计目标)

```bash
# 1. 点击文件 → 开启 session (3秒内)
star session start crates/star-cli/src/commands/task.rs --agent claude

# 2. 与 AI 对话、重构、调试...
#    (自动 checkpoint + git commit)

# 3. 随时暂停/恢复/切换 agent
star session pause <id>
star session resume <id> --agent codex

# 4. 导出为 MR 供团队评审
star session mr <id> --title "重构 extract_args" --reviewers alice,bob

# 5. 跨设备同步 (Git 原生)
star session sync
```

---

## 🔗 与现有系统集成

| 系统 | 集成点 | 数据流向 |
|------|--------|----------|
| **Task Card (ADR-0049)** | `linked_task_card_id` | Task Card 可选关联 FileSession；FileSession 完成可回填进度 |
| **Worktree (star-taskgraph)** | `linked_worktree_id` | 共享 worktree 或独立 worktree 隔离 |
| **AgentSession (ULYS-156/ADR-0045)** | `linked_agent_session_id` | `star session promote` 升级为平台级持久会话 |
| **Context Packet (domain-context)** | 复用 `ContextPacket` | 初始上下文 = ContextPacket；checkpoint 增量更新 |
| **Audit (domain-audit)** | 复用 `ai_audit_metadata` | 所有操作统一审计入库 |
| **Credential (star-credential)** | 隐私过滤 + 凭证注入 | Adapter 调用自动注入 credentials |
| **Hook Policy (domain-hook)** | 生命周期事件 | `session.created/checkpoint/archived` 触发 hook |

---

## 📋 实装路线图

| Phase | 周期 | 核心交付 | 里程碑验收 |
|-------|------|----------|------------|
| **1: 核心基建** | 2w | Crate + 存储 + GitManager + CLI 基础命令 | `star session start/list/show/resume` 端到端 |
| **2: 适配器框架** | 2w | 插件注册表 + 4大内置适配器 | 4个 CLI 均能 spawn/attach/checkpoint |
| **3: 上下文/检查点** | 2w | 增量 checkpoint + transcript索引 + token预算 + 隐私过滤 | 恢复 token < 初始30%，100% secrets过滤 |
| **4: Git深度集成** | 1.5w | 自动提交 + worktree隔离 + 远程同步 + MR导出 | GitHub/GitLab双向同步，MR自动生成 |
| **5: API+前端** | 2w | REST API + VS Code扩展 + Multica集成 | CodeLens + Session Panel + Web点击文件开启 |
| **6: 高级特性** | 持续 | 实时协作 + 语义合并 + 模板 + 企业审计 | 按需交付 |

---

## 🛡️ 关键设计决策 (摘自 ADR-0052)

| 决策 | 选择 | 核心理由 |
|------|------|----------|
| **持久化层** | Git FS 为主，SQLite 辅索引 | Git 原生版本控制，天然支持分支/合并/远程同步/审计 |
| **目录结构** | 按文件分层 `.star/sessions/<file>/<session>/` | 符合"文件为单位"语义，便于 CodeLens/TreeView 聚合 |
| **Agent适配** | 插件机制 + 内置主流 4 个 | 兼容开放生态；覆盖 95% 场景；WASM 扩展自定义 |
| **上下文策略** | 增量 checkpoint + 可选 artifact store | 平衡存储成本与恢复速度；大文件走 LFS/S3 |
| **分支策略** | 每 session 可选独立 worktree 分支 | 并行隔离；`git worktree` 原生支持 |
| **与 Task Card 关系** | 可选双向链接 | 保持正交性；不强制绑定 |

---

## 📊 非功能指标目标

| 指标 | 目标 |
|------|------|
| 会话创建冷启动 | P95 < 3s |
| 会话恢复延迟 | P95 < 2s |
| Checkpoint 写入 | P99 < 500ms |
| 单会话磁盘占用 | 典型 < 50 MB |
| Registry 体积 | 10k sessions < 1 MB |
| 并发会话数 | 单 repo ≥ 1000 |
| 静态加密 | AES-256-GCM 可选 |
| 审计不可篡改 | Append-only + 签名链 |

---

## ⚠️ 主要风险 & 缓解

| 风险 | 缓解 |
|------|------|
| Git 仓库膨胀 | Transcript 默认不进 Git (本地+可选S3)；Git LFS；定期 `git gc` |
| 并发编辑冲突 | 乐观锁 + 语义合并 (tree-sitter AST) + UI 冲突可视化 |
| Agent CLI 版本不兼容 | 版本检测矩阵 + 降级基础模式 + 清晰错误码 |
| 敏感信息泄露 | 默认开启隐私过滤；transcript 不进 Git；导出前二次确认 |
| 存储后端切换不一致 | 统一 `StorageBackend` trait + 迁移工具 checksum + 集成测试 |

---

## 📝 当前状态

- [x] 需求文档 (`01-requirements.md`)
- [x] 架构决策记录 (`ADR-0052`)
- [x] 实装计划 (`02-implementation-plan.md`)
- [x] REST API 规范 (`03-api-spec.md`)
- [x] CLI 参考手册 (`04-cli-reference.md`)
- [ ] **Phase 1 代码实装** (下一步)

---

## 🤝 贡献指南

1. **阅读 ADR-0052** 理解架构决策
2. **查看实装计划** 认领 Phase 1 任务
3. **遵循现有代码风格** (Rust: `cargo fmt` + `cargo clippy -D warnings`)
4. **编写测试先行** (单测覆盖 ≥ 85%，集成测试覆盖核心流程)
5. **提交 PR** 走守门流程 (#19, #20, #21)

---

## 📞 联系人

| 角色 | 负责人 |
|------|--------|
| 架构师 | Mavis (代签) |
| Tech Lead | - |
| 后端开发 | - |
| 前端开发 | - |
| QA | - |

---

> **下一步**: 启动 Phase 1 实装 → `cargo new --lib crates/star-file-session`