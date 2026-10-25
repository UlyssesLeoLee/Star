# Brief: w14-desktop-performance-audit-20261025

**Agent**: explorer
**Phase**: WBS-PARALLEL-20261025-W14
**Created**: 2026-10-25 15:57:18

---

## 目标
依据当前 dev 的 WBS §1/W14，对 Star Desktop 的资源占用与性能验证现状做证据审计，产出可供下一实现 lane 直接执行的验收拆解。

## 当前证据与边界
- 父协调器在本轮调用 CodeGraph MCP 时收到 `Transport closed`；已对 dev 做定向源码回退，候选范围包括 `crates/star-desktop/src-tauri/**`、`crates/star-desktop/frontend/**`、`crates/canvas-renderer/**` 及其现有测试。
- 初步源证据显示 Tauri 2.0 PoC 的页面层使用 React 19/TypeScript/Vite；必须明确区分当前事实与用户要求的 Rust 桌面性能目标，不得把现有 Tauri shell、React 页面或 demo 说成 Rust 原生 UI 已实现。
- WBS W14 当前要求有界缓存/队列、虚拟化、日志、IPC、公平调度、背压、取消/drain、RSS 与 p95 benchmark；并发档位 1/2/4/8/16 只是待测候选扫描，不是容量承诺。不得自行发明 RSS、延迟或并发阈值。

## 任务
逐项检查当前实现、测试和已有性能报告，建立证据矩阵，至少覆盖：UI 列表/画布渲染与分页/虚拟化；前后端 IPC 与 Tokio 队列；日志/事件驻留和缓存上界；慢消费者、背压、取消/drain；RSS、CPU、p95 的现有采样方法；当前 Tauri 前端实现语言与用户 Rust 桌面方向之间的缺口。引用真实文件路径和行号，并将每项判为 implemented / partial / missing / unknown。

基于现有 fixture 和本机 Windows Tauri 运行条件，给出可复现的 benchmark workload/profile 记录模板：设备/OS/Rust/前端构建/数据规模/并发负载/采样时长/采样方法/结果字段/置信与限制。区分已测事实、候选 workload、待用户目标环境确认项。建议 1–3 个按 WBS 依赖可执行的后续实现切片及明确验收，不要扩大为整仓重构。

## 严格交付范围
- 只新增并提交 `docs/reviews/WBS-W14-DESKTOP-PERFORMANCE-AUDIT-20261025.md`。
- 不修改源码、WBS、基本/详细设计、其他报告或脚本；不把 mock/fixture/PoC 当成生产证明；不运行测试或基准；不提出未测量的容量保证。
- 文档中文，证据与推断分开；未知项明确写 unknown / NOT_RUN。完成后在报告中列出检查过的路径、结论和最小后续任务。


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
