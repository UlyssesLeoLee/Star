# INTEGRATION-MAP-GITHUB-REPOS-001 — 自有 GitHub 仓库集成地图

> 状态：✅ v0.1（2026-11-14 用户授权：所有自有库可直接从 GitHub 拉取更新）
> 范围：把用户 GitHub (UlyssesLeoLee) 下的自有仓库内核/插件/MCP 深度集成进 Star 界面与后端
> 依据：AGENTS.md §0「本机 Rust 服务端按模块复用」+ 用户连续发令（GemArc / MAGOS / GitGit / 其他）

## §0 授权与同步策略

- **授权**：UlyssesLeoLee 名下全部仓库，允许 `git pull` 拉取更新，按各库许可证移植/集成。
- **同步**：每次集成前先 `git -C <本地仓库> pull --ff-only`；无 `.git` 的目录（如 `E:\GitGit`）按 GitHub 源 clone 或拷贝（以 hash 固定引用版本）。
- **落点原则**：前端优先复用（UI 页面对接真实内核/MCP），后端按 REUSE-MAP-COLLAB-SANDBOX-001 §4 的 MIT 授权移植 Rust 模块。

## §1 仓库清单与集成映射

| 仓库 | 语言 | 本地 | 状态 | Star 落点 | 集成方式 |
|---|---|---|---|---|---|
| **GemArc** | Python | E:\GemArc (2557eb6) | MVP 可跑 | /prototype 原型页 + MCP | Streamable HTTP MCP 服务接入 /api/gemarc 代理; 前端 1D-5D 生成流水线对接 init→validate→simulate→build |
| **MAGOS** | Python | E:\MAGOS (b484fcd) | 可跑 | LANE 树 + /worktree 管理 | 深度集成: 侧栏 ProjectRunTree 读 magos_guard 车道登记; worktree 页显 lane 计划/锁域/审批记录; 前端调 magos_guard.py 只读子命令 |
| **GitGit** | Rust | D:\GitGit (69770e6, dev) | V0 已实现 | Git 管理 (设置→集成→代码托管) | 内核功能移植进 Star crates/star-git-core; 前端 SCM 页接其对象模型 (commit/tree/branch) |
| **SandTree** | Rust | E:\SandTree (3f6eda2) | v1.1 | /sandbox 沙盒 | resource-graph/kernel/observation 移植进 crates/sandboxd (见 REUSE-MAP §3) |
| **Argo** | Rust | E:\Argo (2941596) | v0.4 | /sandbox 实体机组 + /agents | Controller/Worker 多机接入 (参考为主) |
| **IDE1.0** | Rust | E:\IDE1.0 (a200e5d) | v0.6.7 | /nema 协同 | nema-codec 已移植 ✅; ide-agent-router (collaboration/coordinator/nema_pipeline) 下一批 |
| **IME1.0** | Rust | 未检出 | — | .nema 编辑器输入 | 与 nema-codec 同源; 编辑器内核按需移植 |
| **.NEMA** | JS | 未检出 | — | .nema 格式规范 | 规范文档对齐 (nema-sigma.vercel.app 同源) |
| **IM1.0** | Rust | 未检出 | — | 聊天/Team (kanban 右 rail + /worktree/chat) | IM 通信内核 → 协同聊天频道 |
| **CATs** | Rust | 未检出 | — | i18n 三语 | AI 辅助翻译接入设置→扩展 |
| **XINGTIAN** | — | 未检出 | — | 浏览器自动化探针 | 沙盒观测面浏览器探针 (与 SandTree 探针互补) |
| **Ada** | Rust | 未检出 | — | /remote 远程控制 | 跨平台 adapter |
| **Physis** | Rust | 未检出 | — | GemArc 3D 物理原型 | 3D 原型物理引擎 |
| **Rigin** | Python | 未检出 | — | 待设计 | 待设计 |
| **PANTHEON / ICHOR / Modatrix / RAGit / GOSKernel** | — | 未检出 | — | 待设计 | 待设计 (先读 README 再定) |

## §2 执行顺序（按价值与依赖）

1. **GemArc → /prototype**（用户已点名, 且 mockup 已按 GemArc 1D-5D 布局）
   - 本地 GemArc 起 Streamable HTTP; /api/gemarc 代理; /prototype 接真实 tools (brief/inspect/validate/build/verify)
2. **MAGOS → LANE 树 + /worktree**（用户已点名）
   - ProjectRunTree 接 lane 登记; worktree 页加 magos 面板 (车道/计划/审批)
3. **GitGit → Git 管理** (本地已 pull: D:\GitGit @ 69770e6)
   - 内核模块映射 (src/ 实测): `repo/` (冲突/对象模型) → star-git-core; `remote/sync` → 双向同步; `server/` (Git HTTP) → 本地 git 服务; `ai/` (provider/diff/prompt) → 与 domain-ai 对齐; `objectstore` (S3 sigv4) → 快照存储
   - 移植方式: 新 crate crates/star-git-core (MIT 授权同 REUSE-MAP §4); 先 `repo/conflict.rs` + `remote/sync.rs` 两个纯内核模块, 编译+测试移植
4. **其余**：IM1.0(聊天) > CATs(i18n) > XINGTIAN(探针) > Ada(远程) > Physis(3D) — 按需后续轮次

## §3 修订历史

| 版本 | 日期 | 内容 |
|---|---|---|
| v0.1 | 2026-11-14 | 初版: 仓库全景 + 集成映射 + 执行顺序 (用户授权直接 pull) |
