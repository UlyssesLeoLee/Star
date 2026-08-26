# 41. MVP

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md)

## 1. 范围

不实现所有 AI / IDE / Code Intelligence 功能。MVP 必先证明：

```text
Unknown Coding Agent / Unknown IDE
   ↓
Clone GitGit Repository
   ↓
读 Repository Instructions
   ↓
发现 STAR CLI
   ↓
Capability Discovery
   ↓
获取 Assigned Issue
   ↓
获取 Context
   ↓
搜索相关代码
   ↓
定位相关符号
   ↓
创建 Workspace
   ↓
创建 Worktree
   ↓
修改代码
   ↓
运行测试
   ↓
Commit
   ↓
star submit
   ↓
创建 MR
   ↓
STAR 更新 Issue 状态
```

## 2. 关键约束

- 整个过程**不**修改 STAR Core / GitGit Core
- **不**写 AI 厂商适配器
- **不**等 IDE 厂商适配

## 3. MVP 退出条件

- [ ] `star` CLI 17 个核心命令
- [ ] `star --json` 稳定 schema (`agent-api/v1`)
- [ ] MCP server 13 tools
- [ ] REST API 12 endpoints + OpenAPI 3.1
- [ ] AGENTS.md 自动生成器
- [ ] Universal Submit 11 步
- [ ] Agent Task Lifecycle 9 状态 + 4 异常
- [ ] Agent Lease / Heartbeat / Resume
- [ ] Version Control Provider 4 实现
- [ ] Unknown Agent Test 通过
- [ ] Zero-Knowledge Agent Test 通过
- [ ] Unknown IDE Test 通过
- [ ] GitGit 标准 Git 兼容 100%
- [ ] Fallback Ladder 4 级全部跑通

## 4. 实施位置

- `crates/star-cli/` (主)
- `crates/star-mcp/`
- `crates/star-rest/`
- `crates/star-domain/`
- `crates/star-application/`
- `crates/star-context/`
- `crates/star-workspace/`
- `crates/star-agent/`
- `crates/star-vcs/`
- `crates/star-audit/`
- `crates/star-policy/`
- `tests/unknown-agent/` + `tests/zero-knowledge-agent/` + `tests/unknown-ide/`

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
