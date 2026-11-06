# PHASE-NAMING-STATIC-REMEDIATION-20261106

> 状态：候选已验证；本 lane 只覆盖静态演示、交付页与非发布包描述。
> 基线：`dev@a16b77c9`（lane 创建时）。

## §0 目的

将仍可能作为用户入口打开的静态 demo、交付页和包元数据说明改为 Star
中性术语，移除旧对标标签与游戏域遗留展示，同时保留兼容键、数据库身份
和历史审计资料。

## §1 改动矩阵

| 范围 | 改动 | 约束 |
|---|---|---|
| `demo/index.html` | 视图、schema、里程碑和 Agent 面板改为 Task Workflow、Canvas、Schedule 与 Agent Runtime；移除旧对标/游戏可见文案 | 仅静态展示和演示数据，未改变后端契约 |
| `deliverables/kanban-vmodel-jp/{index.html,app.js,README.md}` | 标题、Sprint 提示、确认框和注释改为 Star workflow rule | 保留 Backlog/Sprint 状态机与任务回流行为 |
| `scripts/automation/kanban_sprint_gen.py` | 断言由旧对标提示改为中性 workflow rule | 仍验证 97 项功能标记 |
| 七个 `Cargo.toml` | 仅更新 `description` 字段 | package 名称、依赖、features 与 `publish=false` 不变 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `git diff --check` | 通过 |
| `kanban_sprint_gen.py --strict` | 97/97（100%）通过 |
| 用户面静态品牌扫描 | `demo/index.html`、交付页、README、生成器无旧对标/游戏域命中；Cargo 注释中的历史追踪号不属于用户描述，保留在兼容边界 |
| `cargo +1.99.1 metadata --no-deps --format-version 1 --locked` | 通过；109 workspace members / 109 packages |
| 七个包发布标志 | `publish=[]` 全部保持为空 |
| Git 范围 | 仅 brief、报告、声明的 12 个实现文件被修改；无生成物或额外文件 |

## §3 已知缺口

1. 未运行浏览器级交付页 E2E；本 lane 只完成静态文本/脚本断言。
2. `worktree-shared-dir/Cargo.toml` 的历史注释仍含内部追踪标识；它不是包
   description，也不进入用户帮助或发布页面，后续若要统一内部注释需另立
   lane。
3. 真实连接器名称、兼容 URL/ID、数据库表名和历史调研资料未改动。
4. 未执行镜像构建、SBOM、签名、远端发布或 Multipass 部署。

## §4 子代理失败接手清单

本 lane 由 root 调度、单一 worker 在隔离 worktree 实施；没有未审查的代码
产出或失败接手项。若后续验证发现交付页行为差异，应从本 lane 的 manifest
和脚本测试重新核对，不把静态检查等同于运行时验收。

## §5 守门规则

| # | 守门 | 结果 |
|---|---|---|
| 1 | 只修改 MAGOS manifest 文件 | 通过 |
| 2 | 用户面中性命名 | 通过；旧对标/游戏展示已移除 |
| 3 | Sprint/Backlog 行为契约 | 97/97 生成器检查通过 |
| 4 | Cargo metadata 与发布标志 | 109/109 metadata；七包 `publish=[]` |
| 5 | Rust 1.99.1 基线 | metadata 以 1.99.1 执行 |
| 6 | 不触碰兼容/历史边界 | 通过 |
| 7 | 差异与生成物 | `git diff --check` 通过；无额外生成文件 |

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008)，确认静态层与业务契约分离 |
| SRE / 平台 | Mavis 接手审核，确认未声称浏览器、发布或部署通过 |
| 安全 | Mavis 接手审核，确认兼容键、数据库身份和用户脏内容未触碰 |
| 评审主持 | Mavis 接手审核，确认 manifest 范围与扫描结果一致 |
| PM | Mavis 接手审核，确认可进入 dev 集成但仍非稳定版发布批准 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 完成静态演示/交付页和七个非发布包描述的中性化，并记录 97/97 脚本与锁定 metadata 验证 | 产品命名审计 A5–A7 |
