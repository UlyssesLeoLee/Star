# Brief: STABLE-HARDENING-DESKTOP-PACKAGING-20261031

**Agent**: worker
**Phase**: STABLE-HARDENING
**Created**: 2026-10-31 08:18:58

---

读取仓库 AGENTS.md 与本 brief。只处理本 lane manifest：新增 .github/workflows/star-stable-desktop.yml、crates/star-desktop/frontend/package.json 与新 package-lock.json、crates/star-desktop/src-tauri/tauri.conf.json 与 Cargo.toml、scripts/build-desktop.sh，以及本 lane 报告/dispatcher 文件。基线 dev@04d8b7a84514dcb454874eb852651778e1452dea。避免修改被 Dependabot owner 占用的 .github/workflows/star-desktop-build.yml。为 Windows/macOS/Linux 设计确定性锁驱动的 build/artifact 流程，淘汰 Node 20；任何 artifact 缺失都应使 job 失败。仓库当前 updater 公钥是 placeholder，不能编造密钥或宣称签名完成；不得发布、创建 tag 或部署。可运行本 lane 必要的 lock/YAML/shell 检查，并在报告中如实列出真实 runner、签名密钥和安装/更新验收仍缺失。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)

## 补充范围 v0.2（MAGOS stable-hardening revision 16）

为关闭 Desktop 发布路径的本地构建阻塞，本 lane 增加 `crates/star-desktop/frontend/tsconfig.json`、`src/constants.ts`、`src/hooks/useTauriKeyboardLayout.ts`、`src/pages/WorktreeVizPage.tsx` 与测试文件 `tests/BoardView.test.tsx`、`tests/setup.ts`、`tests/KanbanBoard.test.tsx`、`tests/WorktreePage.test.tsx`、`tests/WorktreeVizPage.test.tsx`。同时将 Vite/Vitest/官方 React 插件升级到通过安全审计的兼容版本。验收增加 frontend production build、完整 Vitest suite 与 npm audit；不扩大到旧桌面工作流 owner 文件，不授权发布、签名或部署。
