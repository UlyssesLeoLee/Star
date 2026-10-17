# Phase 9F4H — Run BI read UI

版本 v0.1 · 2026-10-17 · Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

在 Project → Engineering Run → Worktree → Branch 导航下，为 Run 提供有界的 Task event 只读界面，并修复正式编译发现的 BI 接口问题。本片不代表全部 phases 或生产 BI 完成。

## §1 改动矩阵

| 范围 | 实施与边界 |
|---|---|
| RunDirectoryApiClient | 真实授权 API、固定 50 条、scope/字段/排序/游标校验；沿用有界请求队列、响应上限与取消 |
| RunBiPanel / Shell | 选中 Worktree 后启用 Run BI；单页驻留、最多 100 个游标、无轮询；unknown coverage、诚实空态、503 自然提示、关联 tabpanel |
| Rust | 修复 API 调用不存在的 reply helper，改用 Json；补齐 receiver 42 处公开 rustdoc；定向 rustfmt |
| 文档 | requirements v6.85、basic v6.84、Task SRS v0.25 / BD v0.18 / DD v1.39、plan v7.13，AC-RUN-OUTBOX-11 / AC-25 |

## §2 验证摘要

- Rust 1.99.0 由 `E:/DevCache/rustup/toolchains/1.99.0-x86_64-pc-windows-msvc/bin` 实际运行。此前仅从 PATH/WSL 判断工具链不可用不充分；旧报告是当时未运行事实，本次补充新证据。
- `cargo check -p star-api-rest --all-targets --locked --offline -j 4`：exit 0，有依赖及既有 warning。移除本片未使用 import 并格式化后再次复验 exit 0；六个 Rust 文件定向 rustfmt --check exit 0。
- `cargo test -p bi-owner-runtime --all-targets --locked --offline -j 4`：12/12，exit 0；仍有 4 条公开 async trait warning。
- Vitest：`runDirectoryApi.bi.test.ts` 与 `RunBiPanel.test.tsx`，11/11。完整 `tsc --noEmit --incremental false`：exit 0。子代理执行，未安装或修改共享依赖。
- PostgreSQL：本片无 schema 改动；9F4G 隔离 PostgreSQL 18 结果沿用，不冒充本次重跑。
- `cargo clippy -p bi-owner-runtime --all-targets --no-deps --locked --offline -j 4`：exit 0，4 条公开 async trait warning；不是 strict `-D warnings` 通过。
- Cypher Guardian：11 个 source files PASS；CodeGraph sync：11 changed files 完成。coverage MCP 不可用，结论限于定向源码审查。`git diff --check`：通过。目标 dev 已复核 inactive / 0 live terminals / 无 attached PTY / 0 unverifiable；仅在 HEAD 与 clean 状态仍匹配时快进，实际结果以 Git ancestry 为准。
- CodeRabbit CLI 已知 Permission denied；未获得外部独立 review。当前为人工源码审查和定向自动检查。

## §3 已知缺口

生产身份、可信 receiver/route registry 接线、目标 DB/grants、可权威判断完整性和新鲜度的 checkpoint、聚合 BI、Benchmark、crash/recovery、RSS/latency/fairness 与完整 CLI/Agent 执行链仍未验收。组件测试不等于真实宿主/桌面视觉验收。Ingress、delivery acceptance 和 execution 继续 fail closed。未推送远端。

## §4 子代理失败接手清单

| 子代理 | 实际结果与接手 |
|---|---|
| bi_rust_docs_fix | 补公开 rustdoc；主代理实际运行编译和 12 个测试，不用完成状态代替验证 |
| bi_frontend_review | 先只读审查/测试，获得写 owner 后补 503 和 tabpanel，再跑 11 个测试与 tsc；完成后释放 owner |
| 集成 | 主代理串行检查 diff、更新文档与执行 dev 快进；未授权覆盖其他工作树内容 |

## §5 守门规则

1. Project → Run → Worktree → Branch 不变。
2. BI 归属 Run，Worktree 是焦点。
3. 每次请求使用当前授权。
4. 不填充 mock/seed。
5. 每页固定 50 条。
6. 请求/队列/响应有界。
7. 仅保留当前页。
8. 游标历史最多 100。
9. 不轮询。
10. scope 切换取消旧请求。
11. 旧响应不展示于新 scope。
12. 空态不等于完整零值。
13. coverage/freshness 不推断为已知。
14. UI 不泄漏 provider/数据库配置。
15. execution 保持 fail closed。
16. 目标 owner、HEAD、clean 与 in-progress 状态复验后才快进。

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) |
| SRE Lead | Mavis 接手审核：生产门未关闭 |
| 平台 | Mavis 接手审核：有界读取切片 |
| 评审主持 | Mavis 接手审核：定向审查，无外部 review |
| PM | Mavis 接手审核：9F4H 切片，全部 phases 未完成 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-17 | Ulysses — Mavis 接手审核 | 登记真实 API UI、编译修复、验证与生产缺口 | 用户要求子代理分头处理合并 |
