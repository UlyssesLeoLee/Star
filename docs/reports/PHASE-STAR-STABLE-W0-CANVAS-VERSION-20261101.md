# PHASE-STAR-STABLE-W0-CANVAS-VERSION-20261101 实装报告 v0.1

> **状态：🟡 W0 静态实现完成，稳定发行仍未放行。**
> **基线：** `dev@6c394465635d86894312700ed79eb583c79bd804`。
> **工作分支：** `codex/star-stable-w0-canvas-version-20261101`。
> **修订人：** Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。
> **签批：** 🟡 本 lane 自审；由协调者独立审查和集成。

## 0. 目的

把三份 Canvas 服务的运行时版本、OCI 版本标记、来源 commit 和活动 K3s 部署文件绑定到同一个严格解析的 Git stable tag。下一目标版本仍为 `v1.0.6`；本报告不表示该 tag 已存在或已创建。

## 1. 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `.github/workflows/publish-star-canvas-services.yml` | 新增受现有人工审批门和测试 job 约束的 resolver job；各镜像 job 核对 checkout SHA，按 resolver version 改写临时隔离 Cargo workspace package 和 lock root，再走既有每服务过滤上下文构建；OCI version/source labels、版本与 SHA tags、digest evidence 都使用同一 resolver 输出；汇集三个 digest 后渲染并上传活动 K3s manifest artifact | ✅ 静态接线完成；hosted build 未执行 |
| `deploy/star-canvas-services-k3s.yaml` | 六处 Deployment/pod version label 从 `v0.1` 改为必需 tag placeholder；三份 Canvas image digest placeholder 保留，文件明确为不可直接部署的模板 | ✅ |
| `scripts/release/render_star_canvas_manifest.py` | 新增标准库 renderer：校验 resolver JSON、canonical tag/version/SHA、三个 allowlisted service 记录、规范 OCI version/source refs、不可变 sha256 image refs、全部模板 markers；一致性不满足时失败关闭，输出固定换行的确定性 YAML | ✅ |
| `scripts/release/tests/test_render_star_canvas_manifest.py` | 新增 renderer 确定性、CLI、真实活动模板、错误输入、缺失/重复服务、可变引用、marker drift 和 workflow release control 的 focused tests | ✅ |
| 三份 Canvas `src/lib.rs` 与 `tests/smoke.rs` | 未改动。三份 ServiceMetadata 已从 `CARGO_PKG_VERSION` 取版本，smoke tests 已断言其与包版本相等；release-only 隔离构建 context 将继承的 workspace version 改为 resolver 输出，普通本地 workspace 构建仍用既有 `0.1.0` | ✅ 通过构建输入绑定；未完成 Rust 编译验收 |
| 本报告、dispatcher brief、dispatcher audit JSONL | 记录本 lane 改动、验证与阻断 | ✅ |

### 绑定链路

`resolve_star_version.py` 先确认 stable tag、peeled commit、当前 HEAD 与 clean checkout。Canvas 构建按 resolver 的 `version` 设置临时生成的 `[workspace.package].version`，并同步临时过滤后的本地 Canvas root Cargo.lock package 版本；各服务沿用已有 `env!("CARGO_PKG_VERSION")` 产生 runtime metadata。OCI tags 使用 resolver 输出的 normalized version 和 source SHA，OCI labels 显式固定 `org.opencontainers.image.version` 与 `org.opencontainers.image.revision`。每个 Buildx digest record 同时保存 resolver 的 tag/version/source SHA、OCI version/source refs 及 immutable digest ref。最终渲染器只接受与 resolver JSON 完全一致的三份服务记录。

正常开发 crate 的 package 版本仍来自根 workspace 当前 `0.1.0`；本 lane 未修改任何 Cargo manifest 或锁文件。三份 Canvas crate 通过工作流生成的临时 release manifest 才得到 tag 版本。

### 没有执行的交付操作

本 lane 没有创建 tag、构建/push image、触发真实 hosted workflow、部署 Kubernetes、修改远端、rebase 或集成任何分支。sample `v1.0.6` 数据只用于验证渲染器和隔离上下文版本注入，不是实际 release resolver 输出或 hosted 制品证明。

## 2. 验证摘要

| 核验 | 结果 |
|---|---|
| `rtk proxy python -m unittest discover -s scripts/release/tests -p test_render_star_canvas_manifest.py -v` | ✅ 8 tests passed |
| `rtk proxy python -m unittest discover -s scripts/release/tests -p test_resolve_star_version.py -v` | ✅ 10 tests passed |
| Python `py_compile`（renderer 与新增 test） | ✅ 通过 |
| PyYAML workflow/template parse | ✅ workflow YAML 解析成功；活动 K3s 模板解析为 15 个 YAML documents |
| workflow 内两段 inline Python `compile(...)` | ✅ 过滤 Cargo context 和 digest record 段语法通过 |
| 三服务 filtered-context 仿真 | ✅ 用合成 `v1.0.6` resolver 值分别生成 119 / 119 / 136 个锁定包的隔离 context；每份 context 为 9 个文件；生成 workspace 与本地 lock root 都为 `1.0.6`；未包含非允许 crate/game 来源 |
| CodeGraph / coverage | ⚠️ 本 lane 没有可用的 coverage API；之前对已索引的 dev project 查询超时。按要求对候选文件做了定向 RTK 源码读取，知识图覆盖完整性未知 |
| `rtk cargo test -p canvas-engine -p domain-canvas -p canvas-realtime --all-targets --locked -j 4` | ⛔ 本机 Rust `1.99.0-x86_64-pc-windows-msvc` 缺失适用的 `rustc.exe` 组件，命令在编译启动前失败；未用其他 toolchain 冒充 release proof |
| `rtk actionlint -version` | ⛔ `actionlint` 不在 PATH；workflow YAML 与 inline Python 做了静态解析，未声称 actionlint 通过 |
| `rtk git diff --check` / Cypher Graph Guardian | ✅ diff whitespace 通过；`cgg_guard.py --changed --stamp` 检查并刷新两个新增 Python source headers |

隔离上下文仿真只执行了 workflow 已有的上下文生成 Python 片段并解析生成的 Cargo TOML/lock；它没有调用 resolver、Cargo、Docker Buildx、容器运行时或 GHCR，也不证明 tag 或 image 实际存在。

## 3. 已知缺口与 W0/W3/W7 状态

1. **W0 静态绑定完成，端到端发行仍 partial。** 尚未在真实 `v1.0.6` tag commit 上跑 hosted workflow；没有实际 resolver artifact、三份 OCI digest、runtime `/version` 读取结果或最终 manifest artifact。不得将本阶段称为稳定发布通过。
2. **W1 MIT license gate 保持阻断。** 当前根 Cargo workspace license metadata 与根 `LICENSE` 正文不一致；本 lane 按 brief 未改 root Cargo/license 内容，既有稳定发行 workflow gate 应拒绝构建。
3. **W3 artifact proof 未完成。** 静态过滤-context 仿真通过，但没有实际 Buildx 输入/输出、镜像/SBOM/provenance 下载验证、manifest artifact consumer 验证或运行时 image digest 证明。另有独立 `deploy/k3s-local/Dockerfile` 使用宽泛 `COPY crates ./crates` 且 `.dockerignore` 仅排除有限旧目录；它需要单独做 payload 分类或上下文过滤，不能据此宣称所有 Docker 制品仅含 Star 允许载荷。本 lane 未触碰该文件。
4. **W7 hosted Buildx 未验收。** 本 lane保留 pinned Buildx、人工环境审批、`queue: max`、`cancel-in-progress: false`、license gate、per-service allowlist/过滤上下文、SBOM、provenance 与 attestation 配置；没有 hosted workflow 执行证据，也没有 actionlint 可用证明 GitHub runner 接受全部配置。
5. 本地版本映射审计文件仍描述了此前 metadata action 的 semver tag 输入形态；本 lane 将版本与 source SHA 显式取自 resolver。协调者集成时应同步该 W0 version-map 叙述，避免文档落后于工作流实现。

## 4. 子代理失败接手清单

本 lane 未另行派发子代理；没有 child RPC/worker 失败或接手项。协调者需独立审查本 lane 的 workflow diff、集成冲突和后续 hosted 验收，再决定是否接受。

## 5. 守门规则

| 守门 | 结果 |
|---|---|
| 单一版本源为 canonical Git stable tag；resolver 检查精确 tag ref、peeled commit、HEAD 与 clean checkout | ✅ workflow 集成已有 resolver；真实 tag 尚未执行 |
| 每服务运行时 package version、OCI tags/labels、digest evidence 都从 resolver 输出派生 | ✅ 静态接线；真实构建未执行 |
| Canvas release context 保持 allowlist、过滤 Cargo 闭包与只复制服务 crate | ✅ 已有控制保留；三服务仿真通过 |
| Cargo.lock 仅在 runner temp 过滤 context 中重写 Canvas root package 版本 | ✅ 仿真通过；仓库 Cargo.lock 未修改 |
| 活动 K3s template 必须提供全部 tag 与 digest，不可直接部署 | ✅ 六个 version placeholders、三份 digest placeholders |
| renderer 校验精确三服务、同 resolver identity 和 immutable references；无未渲染 placeholder | ✅ 8 个 focused tests 通过 |
| release workflow 保留人工审批、环境保护、并发排队、SBOM/provenance 和 pinned Buildx | ✅ 静态 assertion 通过；hosted 未运行 |
| root license 争议必须 fail closed，不由本 lane 修改 | ✅ 保留原 gate；W1 blocker 仍存在 |
| Rust release validation 只用 1.99.0；工具链缺件不以其他版本替代 | ✅ 明确记录 Cargo blocker |
| 不创建版本 tag、OCI 镜像或 Kubernetes rollout | ✅ 未执行 |
| 不改 main/dev、其他 worktree、remote、Dockerfiles、Cargo manifests/lock、`deploy/k3s-local` | ✅ |
| dispatcher brief 路径包含在 lane commit message；新增源码 header SHA 已由 Cypher Graph Guardian 刷新 | ✅ commit `fc7cbe3e5b002328910a54b713d7a0996f5dc7f8` 与 source headers 已复核 |

## 6. 签字栏

| 角色 | 审核者 | 结论 |
|---|---|---|
| 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 版本绑定设计自审；W0 hosted proof 开放 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 未执行 Buildx、镜像运行与部署验收 |
| 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 根 license gate/W1 未关闭 |
| 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 🟢 coordinator 独立审查通过；renderer/resolver focused tests 通过；Rust 1.99.0 编译与 hosted Buildx 验收仍开放 |
| 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 不构成稳定发行批准 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Canvas runtime/tag/digest/manifest 静态绑定、focused tests、工具链阻塞与 W0/W3/W7 已知缺口 | dispatcher brief `docs/briefs/STAR-STABLE-W0-CANVAS-VERSION-20261101.md` |
| v0.2 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录协调者对 renderer/workflow 的独立审查和 rebase 后 focused tests；保留 Rust 1.99.0 与 hosted Buildx 发布阻断 | 协调者集成前审查 |
