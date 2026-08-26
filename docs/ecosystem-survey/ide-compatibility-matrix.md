# IDE / Cloud Development Environment 兼容性矩阵（事实基线）

> **调查日期**：2026-08-26
> **范围**：6 款主流 IDE / CDE 的公开能力 + Dev Container 标准
> **目的**：给 STAR × GitGit IDE Gateway 设计提供事实基线
> **原则**：只记录厂商已公开、无需厂商为 STAR 适配的能力

---

## 1. 6 款 IDE / CDE 横向能力

| 能力 | VS Code | Cursor | JetBrains IDEs | GitHub Codespaces | Gitpod / Ona | Dev Containers 标准 |
|---|---|---|---|---|---|---|
| **Git CLI** | ✅ 集成 terminal | ✅ 内置 | ✅ 内置 | ✅ 集成（继承 host git） | ✅ 集成 | ✅ 容器内任意 git |
| **Shell / Terminal** | ✅ xterm.js + shell integration | ✅ 同 VS Code | ✅ 内置 terminal | ✅ browser VS Code + desktop | ✅ Theia + VS Code SSH | ✅ devcontainer 内任意 shell |
| **文件系统访问** | ✅ workspace API | ✅ full FS | ✅ 全 IDE | ✅ bind-mount | ✅ workspace | ✅ devcontainer 全部 |
| **LSP 客户端** | ✅ 原生 + 多 server | ✅ 继承 | ✅ 原生（最强） | ✅ 通过 devcontainer | ✅ 通过 VS Code | ✅ 标准 |
| **MCP 客户端** | ✅ GA in 1.102 (2025-07-14) | ✅ .cursor/mcp.json | ⚠️ Junie 通过 /mcp 命令 | ✅ 通过 VS Code 客户端 | ✅ 通过 VS Code | ✅ 任何 MCP-aware 客户端 |
| **MCP Server（IDE 暴露能力给 agent）** | ✅ 通过 Language Model Tools API + 扩展 | ✅ 有限（仅 tools，非 resources） | ✅ 通过 Junie | ✅ 通过 VS Code | ✅ | ✅ |
| **AGENTS.md** | ✅ .github/instructions/ + AGENTS.md 扫描 | ✅ .cursorrules + AGENTS.md | ✅ Junie 可 import | ✅ 透过 VS Code | ✅ | ✅ |
| **OpenAPI 客户端** | 通过 extensions | 通过 extensions | 通过 HTTP Client plugin | 通过 VS Code extensions | 通过 VS Code | ✅ |
| **Dev Container / 镜像** | ✅ devcontainer.json 标准 | ✅ 同（Cursor 用 Anysphere Dev Containers extension） | ✅ JetBrains Dev Containers (RustRover) | ✅ 原生 Codespace = devcontainer | ✅ .gitpod.yml | ✅ 标准 |
| **Port Forwarding** | ✅ 内置 | ✅ | ⚠️ 需配置 | ✅ 强项（https://*.github.dev） | ✅ 自动 | ✅ devcontainer ports 属性 |
| **Cloud / Remote SSH** | ✅ vscode.dev/agents + Remote-SSH | ✅ Remote-SSH | ✅ JetBrains Gateway | ✅ Browser VS Code | ✅ 浏览器 + SSH | ✅ 任意 backend |
| **工作区持久化** | ✅ workspace trust + state | ✅ | ✅ IDE 自身 | ✅ codespace 可暂停 | ✅ workspace snapshot | ✅ devcontainer 本身 |
| **License** | MIT (core) | Proprietary | Proprietary | 闭源 + devcontainer 开源标准 | 开源 Gitpod core + 闭源 Ona | 标准开放 |

---

## 2. Cloud Development Environment 关键事实

| 平台 | 计费 | 浏览器 IDE | Prebuild | 自我描述 |
|---|---|---|---|---|
| **GitHub Codespaces** | Free 120 core-h/月；2-core $0.18/h | ✅ VS Code Web | ✅ GitHub Action | GitHub 原生 CDE |
| **Gitpod / Ona** | 2025-10-15 Gitpod Classic 关停；现 Ona 主导 | ✅ Theia | ✅ 自动 prebuild | "AI agent mission control"，与 OpenAI Codex 合作 |
| **DevPod** | 完全开源免费 | ✅ VS Code + JetBrains | ❌ 客户端式 | 任意 backend，包括本地机器 |
| **CodeSandbox** | Free 400 credits/月 | ✅ 浏览器 | ✅ microVM | 强调 microVM 即时启动 |

> **重要事实（2026-08-26）**：Gitpod Classic 已停服，Ona 自我定位为 "mission control for AI agents"。这验证了"AI-first CDE" 是行业趋势，但同时也证明 **vendor 可能在不通知的情况下停服** → 强化零厂商适配。

---

## 3. Dev Container 标准（2026-08-26 状态）

| 字段 | 行为 | 用途 |
|---|---|---|
| `image` / `build.dockerfile` | 基础镜像 | 容器构建 |
| `features` | Dev Container Features 目录（OCI distribution） | 工具链（terraform / kubectl / python / 等） |
| `customizations.vscode.extensions` | VS Code 扩展自动装 | 团队统一 IDE |
| `remoteUser` | 非 root | 权限 |
| `forwardPorts` / `portsAttributes` | 端口转发 | 预览 web app |
| `postCreateCommand` | 容器创建后跑 | 装依赖 |
| `hostRequirements` (Codespaces) | 最低 CPU/mem/storage | Codespace 资源 |
| `devcontainer-lock.json` | OCI SHA-256 锁 | 供应链完整性 + 复现构建 |
| Lifecycle scripts | onCreate / updateContent / postCreate / postStart | 标准化引导 |

来源：https://containers.dev/implementors/json_reference · https://viprasol.com/blog/devcontainers · https://www.kunalganglani.com/blog/self-hosted-devcontainers-ssh-vscode · https://microsoft.github.io/hve-core/docs/customization/environment

---

## 4. STAR × GitGit 设计的 4 个直接推论

1. **Dev Container 已经是事实标准** — 6/6 款 CDE/IDE 都用同一个 `devcontainer.json` 规范；STAR 应在 `devcontainer.json` 加 `features` 入口暴露 `star` CLI / MCP Server / AGENTS.md bootstrap
2. **VS Code 客户端是最大公约数** — Codespaces / Gitpod / DevPod 都通过 VS Code 客户端远程；STAR IDE Gateway 应假设 VS Code Remote 协议为头等支持（不依赖 IDE 厂商开发 STAR 插件）
3. **JetBrains 强项在 LSP + 语义索引** — STAR Code Intelligence 服务在 v0.x 可选提供 LSP 端点，让 JetBrains Junie 通过 native LSP 直接消费（per Claude Code 2025-12 native LSP 同款路径）
4. **Port Forwarding 是 cloud IDE 必要能力** — STAR IDE Gateway 设计时要考虑 web preview 通道（但**不强制**走 STAR HTTP；用 GitHub / Codespace 的 port forwarding 即可）

---

## 5. 资料来源

- VS Code：https://code.visualstudio.com/docs/agents/overview · https://code.visualstudio.com/docs/chat/chat-tools
- Cursor：https://zairalabs.ai/guide/tools/cursor
- JetBrains：https://www.jetbrains.com/help/rust/getting-started-with-dev-containers.html
- Codespaces：https://devtune.ai/verticals/cloud-development-environments-cdes/github-codespaces · https://aicoolies.com/tools/github-codespaces
- Gitpod / Ona：https://aisotools.com/blog/gitpod-review-2026 · https://www.16idc.com/en-us/provider-detail/gitpod · https://aicoolies.com/tools/gitpod
- Dev Containers：https://viprasol.com/blog/devcontainers · https://www.kunalganglani.com/blog/self-hosted-devcontainers-ssh-vscode · https://microsoft.github.io/hve-core/docs/customization/environment

---

## 6. 已知缺口

- ⚠️ "Cursor MCP 仅 tools 非 resources" — zairalabs 2026-08-19 评测，**2026-07-28 新 MCP 规范后是否变化未实测**
- ⚠️ Replit Agent 3 / Windsurf 详细能力未在本次范围
- ⚠️ JetBrains AI Assistant 非 Junie 形态（如 inline completion / chat）未单独调研
- ⚠️ DevPod 与 Gitpod 关系、Ona 与 OpenAI 商业合作（per aisotools 报道）未深查
