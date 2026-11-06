# PHASE-NAMING-CLI-REMEDIATION-20261106

> 状态为候选已验证，详见 §2。
> 基线为 `788c878c71a477131022e3367115b7037326290e`，分支为 `codex/naming-cli-remediation-20261106`。

## §0 目的

处理[产品命名审计](PHASE-PRODUCT-NAMING-AUDIT-20261106.md) A3 与 A8。以 Star 能力描述 CLI 命令、Worktree 起点选择和默认任务工作流。

## §1 改动矩阵

| 文件 | 改动 | 契约核对 |
|---|---|---|
| `crates/star-cli/src/main.rs` | 根帮助以「Worktree 操作与起点选择」「管理技能注册表」说明入口 | Clap 属性、命令名和分发与基线一致 |
| `crates/star-cli/src/commands/worktree.rs` | 说明如何列出或解析起点候选，并明确位置参数是本地 Git 仓库路径 | `<REPO>`、`--resolve <KIND:VALUE>`、`--json`、候选种类和输出路径与基线一致 |
| `crates/star-workflow/src/lib.rs` | 默认描述采用 `Default task workflow (5 states, 4 transitions)`，同步中性化该库公开 Rust 文档 | `default` 名称、序列化字段、状态、迁移规则和 ID 构造与基线一致 |

对照了 [CLI 需求](../requirements/SRS-STAR-CLI-001.md)、[CLI 基本设计](../design/BD-STAR-CLI-001.md) 与 [CLI 规格](../architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md)。本次文案沿用既有命令和参数定义。[工作流模板需求](../requirements/SRS-WORKFLOW-TEMPLATE-001.md)、[基本设计](../design/BD-WORKFLOW-TEMPLATE-001.md) 与[详细设计](../design/DD-WORKFLOW-TEMPLATE-001.md) 已将模板管理和事项状态机分开记录。本次修订覆盖后者的公开描述，验收记录集中于本报告。

## §2 验证摘要

| 命令或检查 | 结果 |
|---|---|
| `git diff --check` | 通过 |
| `git show HEAD:<path>` 与修改后源码的 Python 比对 | 三文件通过。剔除注释并单独归一默认工作流构造语句后，可执行源码一致。CLI 帮助范围和工作流公开文档的旧品牌扫描通过 |
| `cargo fmt --check -p star-workflow` | 通过 |
| `cargo fmt --check -p star-cli -p star-workflow`，及 `git show HEAD:crates/star-cli/src/commands/worktree.rs` 对照 `rustfmt --edition 2021 --check` | CLI 格式门有基线差异。修改前后 `worktree.rs` 的 60 行格式差异内容一致，`binary_integration.rs` 在本次差异之外 |
| `rustc +1.99.0 --version`，`rustc --version`，`cargo metadata --format-version 1 --no-deps --locked` | 本机 1.99.0 工具链缺少可用的 `rustc.exe`。两个包的清单及仓库工具链固定在 1.99.1，本次 Rust 验证使用 1.99.1 |
| `cargo test --locked -p star-cli -p star-workflow --all-targets -j 4` | 通过。CLI 单元 18、二进制集成 4、工作流单元 9，共 31 项。依赖与 CLI 既有文件仍有编译警告 |
| `star --help`、`star worktree --help`、`star worktree start-from-picker --help` | 使用本次构建的 Windows `star.exe` 验证，三条命令均退出 0，stderr 为空。核对中性中文、命令列表、`<REPO>`、`--resolve <KIND:VALUE>`、`--json` 和四种候选类型 |
| CodeGraph MCP、`codegraph explore` | MCP 调用超时，CLI 确认本工作树没有本地索引，转用源码与 Git 比对 |
| `cgg_guard.py --changed --stamp` | 守卫对三个原本缺少 Cypher header 的文件返回 2。逐文件差异确认没有新增、删除或改名代码节点及关系 |
| `coderabbit --version`、`coderabbit auth status` | WSL 包装器返回 `/root/.local/bin/coderabbit: Permission denied`。已做本地差异复核 |

## §3 已知缺口

- CLI 格式检查和 Cypher header 检查保留基线缺口，详见 §2。
- 包外消费者、既有持久化工作流的文本与 API 的对外可见性未做端到端验收。
- A4 兼容能力清单、其他命名审计项、发布制品和远端状态由各自的验收范围记录。

## §4 子代理失败接手清单

本独立工作树由 root 调度，未转派子代理，无接手项。

## §5 守门规则

| # | 门禁 | 结果 |
|---|---|---|
| 1 | 基线 | 已核对 SHA |
| 2 | 分支和工作树隔离 | 本 lane 独立工作树 |
| 3 | 修改文件范围 | 三个源码文件与本 lane 的 brief、报告 |
| 4 | 根命令帮助 | 源码和实际帮助输出通过 |
| 5 | Worktree 子命令帮助 | 源码和实际帮助输出通过 |
| 6 | Picker 参数与参数值 | 源码比对通过 |
| 7 | CLI JSON 输出契约 | 源码比对通过 |
| 8 | 内部兼容标识 | 源码比对通过 |
| 9 | 默认工作流文案 | 已核对构造值与公开文档 |
| 10 | 工作流状态和迁移 | 源码比对通过 |
| 11 | 需求与设计对照 | §1 |
| 12 | Rust 格式与工具链 | §2 |
| 13 | 包级测试与运行验收 | §2 |
| 14 | 结构关系、差异和审查 | §2 |
| 15 | 提交和集成边界 | 本 lane 仅交付独立分支候选 |

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008)，核对展示与兼容契约边界 |
| SRE Lead | Mavis 接手审核，核对 §2 验证状态 |
| 平台 | Mavis 接手审核，核对 CLI 参数和序列化声明 |
| 评审主持 | Mavis 接手审核，核对差异和已知缺口 |
| PM | Mavis 接手审核，登记 A3/A8 候选状态 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 A3/A8 最小文案修复和验收结果 | 产品命名审计 |
| v0.2 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 清除三份源码文件中残留的外部品牌/FR-ORCA 注释，仅保留中性追踪描述；行为契约未变 | 代码审查发现非用户可见品牌命中 |
