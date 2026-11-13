# PHASE: STAR-STABLE-W0-DESKTOP-VERSION

**日期**：2026-11-01
**范围**：桌面稳定版本绑定
**基线**：dev@6c394465635d86894312700ed79eb583c79bd804
**状态**：实现完成；Rust 编译验证受已安装的 1.99.0 toolchain 阻断；未发布、未签名、未合并。

## §0 目的

以 scripts/release/resolve_star_version.py 的精确 Git tag 解析结果作为桌面稳定版本唯一来源，让安装包版本、Rust get_app_version()、制品名称和随制品身份文件使用同一 tag、版本与 source SHA。非 tag 开发构建继续读取 crate 版本。

## §1 改动矩阵

| 项目 | 实施结果 | 证据 |
|---|---|---|
| 精确 tag 校验 | Linux、Windows、macOS checkout 后立即运行 resolver；它验证规范 tag、tag peeled commit 等于 HEAD 且工作树干净 | .github/workflows/star-stable-desktop.yml 三个平台 Resolve exact stable tag before build mutations |
| 安装包版本 | stable build 将 resolver 的版本作为 Tauri --config 的 version 覆盖；开发构建保留原命令 | 三个平台 stable/development build steps |
| Runtime 版本 | get_app_version() 优先返回编译时 STAR_RELEASE_VERSION；未设置或空值时回退到 CARGO_PKG_VERSION | crates/star-desktop/src-tauri/src/lib.rs |
| 制品身份 | 制品名含 tag、版本、source SHA；每个平台上传身份 JSON；attestation job 将 JSON 与 resolver 输出逐项比较 | 三个平台 upload 与 attest steps |
| 锁与发布门 | 保留 --locked、人工 APPROVED 条件、tag-only attestations、现有 SBOM 和 provenance 步骤；未执行发布 | 工作流原门禁仍存在 |
| 版本逻辑测试 | 增加稳定版本优先、开发版本回退的单元测试 | app_version_* 测试 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| rtk git diff --check | 通过 |
| rtk cargo +1.99.0 fmt --check --manifest-path crates/star-desktop/src-tauri/Cargo.toml | 通过 |
| PyYAML 解析工作流；结构断言确认三平台 resolver 在 stable build 前执行、版本/tag/SHA 传递至制品名与身份校验、三组 attestation 仍有 tag + APPROVED 门禁 | 通过 |
| CypherGraph Guardian --changed --stamp | 通过（1 个 Rust 源文件） |
| rtk cargo +1.99.0 test --manifest-path crates/star-desktop/src-tauri/Cargo.toml --lib -j 2 | 未能启动：当前安装的 1.99.0-x86_64-pc-windows-msvc 不含可用的 rustc.exe（提示 rustc component 不适用于该 toolchain）；未改用其他版本作为证明 |
| rtk actionlint -version | 未能运行：actionlint 不在 PATH |
| 多平台 Tauri bundle、真实 tag resolver、SBOM 与 attestation | 未在本地执行；需要相应 CI runner 和有效 Rust 1.99.0 安装 |

YAML 解析只证明文件结构可解析，不替代 GitHub Actions schema/lint 或远端执行。Rust 单测和三平台打包仍是 W0 集成门禁。

## §3 已知缺口

1. Rust 1.99.0 Windows toolchain 当前不可用于编译；须在受支持的 CI runner 上用目标 toolchain 复验。
2. 本机未安装 actionlint，尚无 GitHub Actions 专用静态 lint 结果。
3. 未运行带真实、干净 stable tag 的 resolver → Tauri Linux/Windows/macOS 构建闭环。
4. 尚无实际安装包，不能宣称安装、签名、公证或发布完成。
5. 稳定发布仍受独立 release metadata gate（MIT workspace metadata）和 accountable release owner 最终审批约束。

## §4 子代理失败接手清单

本 lane 未派发子代理。无子代理产出或接手项。

## §5 守门规则

1. Git tag 是稳定版本的唯一真相来源。
2. 仅接受 resolver 认可的规范 vMAJOR.MINOR.PATCH tag。
3. 稳定构建必须验证 tag peeled commit 与 checkout HEAD 相同。
4. Resolver 必须在依赖安装和构建生成文件之前确认 checkout 干净。
5. 稳定构建无 resolver 输出时应失败关闭，不回退到 Cargo 版本。
6. 非稳定开发构建继续使用 crate 版本。
7. Tauri installer 配置与 Rust runtime 使用同一个 resolver 版本。
8. 稳定制品名记录 tag、版本与 source SHA。
9. 制品随附的身份文件记录同一组 resolver 值。
10. Attestation 阶段必须校验身份文件与构建 job 输出完全一致。
11. 稳定 attestations 继续要求明确 APPROVED 输入且仅允许 tag ref。
12. Rust 构建继续使用 --locked，不得用未锁定依赖替代。
13. 保留现有 SBOM、provenance 与分平台打包控制。
14. 不以本地开发 fallback 冒充稳定版本验证。
15. 不以 YAML 解析或格式检查冒充 Rust 编译和端到端构建。
16. 不以其他 Rust toolchain 替代 1.99.0 的 release 证明。
17. 未经实际 runner 证据，不宣称已发布、签名、公证或安装验证。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | Mavis lane implementation；协调者复核待办 |
| SRE Lead | 多平台 CI runner 证据待补 |
| 平台 | Rust 1.99.0 toolchain / actionlint 验证待补 |
| 评审主持 | 协调者独立审查待办 |
| PM / Release Owner | 用户是 accountable release owner；最终稳定候选与发布审批待用户执行 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 tag-derived Desktop 版本实现、可用验证及 toolchain/actionlint 缺口 | STAR-STABLE-W0-DESKTOP-VERSION lane |
