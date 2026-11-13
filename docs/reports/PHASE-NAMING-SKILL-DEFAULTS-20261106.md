# PHASE-NAMING-SKILL-DEFAULTS-20261106

> 状态：候选已验证；本 lane 只覆盖 Skill Registry 的默认兼容集合。
> 基线：`dev@ae674db7`（lane 创建时）。

## §0 目的

关闭命名审计 A4 的默认输出问题：未指定兼容列表时，CLI JSON 只显示 Star
中性 runtime 身份；显式传入的 adapter ID 仍原样保留，供迁移和兼容层使用。

## §1 改动矩阵

| 文件 | 改动 | 契约 |
|---|---|---|
| `crates/star-cli/src/skill_registry.rs` | 默认 `agent_compat` 改为 `claude-code`、`codex`、`star-agent-runtime`；显式列表不改写；测试名和公开注释中性化 | Skill JSON 字段、列表过滤、显式 adapter round-trip 保持 |
| `docs/briefs/PHASE-NAMING-SKILL-DEFAULTS-20261106.md` | 固定范围与验收 | 仅本 lane 三文件 |
| `docs/reports/PHASE-NAMING-SKILL-DEFAULTS-20261106.md` | 记录验证、兼容边界和缺口 | 本文 §2–§7 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `cargo +1.99.1 test --locked -p star-cli --all-targets -j 4` | 通过：19 个 CLI 单元测试 + 4 个二进制集成测试，共 23/23；首次尝试因共享目标目录链接文件锁 `LNK1104` 失败，释放锁后重跑通过 |
| `rustfmt +1.99.1 --edition 2021 --check crates/star-cli/src/skill_registry.rs` | 通过 |
| `git diff --check` | 通过 |
| 静态命名扫描 | 本文件无旧产品品牌命中；默认 JSON 断言不含 `legacy-provider`，显式 adapter 测试证明 round-trip |
| CLI help/JSON | 既有 `star skill` help 契约保持；本 lane 未改变命令参数或 JSON 字段名 |

## §3 已知缺口

1. Skill 的下载、签名、持久化和沙箱执行仍是既有 P1 缺口，本 lane 不把内存 registry 视为生产闭环。
2. 真实连接器/adapter 的授权与版本化身份需要后续运行时集成验收；显式旧 ID 仅证明序列化兼容，不证明 provider 可用。
3. Workspace 全量 fmt 仍有既有基线差异；本 lane 的受影响文件已单独通过 rustfmt。

## §4 子代理失败接手清单

本 lane 由单一 worker 在独立 worktree 实施，没有未审查的代码产出或失败接手项。

## §5 守门规则

| # | 守门 | 结果 |
|---|---|---|
| 1 | 默认输出不含旧 provider 名称 | 通过，默认集合为 Star 中性 runtime 身份 |
| 2 | 显式 adapter ID 兼容 | 通过，显式列表原样序列化并有单测 |
| 3 | Skill JSON schema | 通过，未改字段、命令和序列化形状 |
| 4 | Rust 1.99.1 | 通过，测试与 rustfmt 使用 1.99.1 |
| 5 | 测试与格式 | 通过，23/23 focused tests、rustfmt、diff check |
| 6 | 修改范围 | 通过，仅三份 manifest 文件 |
| 7 | 用户数据与分支边界 | 通过，未触碰 main、远端或其他 worktree |

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008)，确认默认与兼容边界分离 |
| SRE / 平台 | Mavis 接手审核，确认未把内存 registry 写成生产完成 |
| 安全 | Mavis 接手审核，确认显式 adapter ID 不改变授权边界 |
| 评审主持 | Mavis 接手审核，确认测试覆盖默认和显式两条路径 |
| PM | Mavis 接手审核，确认 A4 可合入但不解除整体发布门 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 中性化 Skill Registry 默认兼容集合并记录 23/23 focused tests 与兼容边界 | 产品命名审计 A4 |
