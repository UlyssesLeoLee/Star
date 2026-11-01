# Brief: WBS-STAR-STABLE-GATE-SYNC-20261101

**Agent**: codex
**Phase**: WBS-STAR-STABLE-GATE-SYNC-20261101
**Created**: 2026-11-01 13:11:02

---

## 本任务目标
基于固定基线 dev@4ee0a7670dcba4e24b5df23eef18e90fa353e8e0 和已独立完成的 W1/W4 审阅，更新稳定版 WBS 当前证据、阻塞状态与执行顺序，并新增本阶段报告。仅在当前登记 worktree 修改 4 个 manifest 文件；不得修改源码、Cargo/lock、workflow、release 配置、main、D:\Star 用户脏文件或任何旧 lane/worktree。

## 已完成审阅证据
- W1: commit 80be912a51c28b3e839dee56f4ee97d9077cb218；review: docs/reviews/W1-CARGO-OVERLAP-AUDIT-20261101.md；phase: docs/reports/PHASE-W1-CARGO-OVERLAP-AUDIT-20261101.md。结论 W1 BLOCKED：用户选 MIT；13 个包 manifest 已 MIT，但 root Cargo.toml 仍 Apache-2.0，LICENSE 是 MIT，Canvas release gate 明确拒绝不一致。W8 / Dependabot source refs 和 detached dirty worktree 保持未改未读。仅静态审核，无 cargo build 或 PostgreSQL 重跑。
- W4: commit e3075b2e4408bd87e674cb413c30ce5f5dd01bd4（含 da28c5a5a56488f8aab5aa2f1cc65484e631137e）；review: docs/reviews/W4-TOOLCHAIN-DEPENDABOT-AUDIT-20261101.md；phase: docs/reports/PHASE-W4-TOOLCHAIN-DEPENDABOT-AUDIT-20261101.md。结论候选 HOLD、W4 未关闭：固定 candidate 将 8 个 workflow 的 19 个 action refs 从 1.99.0 改成 1.120.0；项目基线 Rust 1.99.0。只核对本地 refs 和官方 release/action 文档，未 fetch、未看 live PR/hosted CI。
- W6: 用户针对四份 W6 review 文档的 MAGOS override 成功登记。本次在本地 dev 检查确认四份文档（brief、phase、dispatcher audit、review）均已在 dev 历史中，无需再 cherry-pick，不能重复合并或改写。仅记录 ancestry 结果。
- W0 focused tests 在固定 dev 基线上曾新鲜执行并通过：scripts/release/tests/test_resolve_star_version.py 10/10，scripts/release/tests/test_render_star_canvas_manifest.py 8/8；git diff --check 通过。它们不证明真实 tag build、hosted/native artifact、Canvas image、Vercel 或 rollback 通过。

## 必须保留的发布结论
稳定版本仍 NO-GO。不要创建 v1.0.6 tag/release，不做 build/publish/deploy/push。本 WBS update 只记录静态审查和已执行的 focused unit tests，不能把它们写成完整 release gate。W0 PARTIAL；W1 BLOCKED；W4 OPEN/HOLD；W2/W3/W5/W6/W7/W8/W9 依 WBS 现有证据保持开放/阻塞，不能因为本次报告变更而关闭。

## Docker/镜像及环境边界
Canvas workflow 的 allowlist/filter 仅为静态规则，不等于已构建/检查镜像。deploy/k3s-local 使用生成 context；本轮没有执行生成器或检查最终 context/image，不能声称所有镜像已经证明只含 Star。W7 的 Multipass/K3s/image digest/SBOM/provenance/rollout/rollback 真实证据仍需执行。测试阶段 Multipass VM inventory 上限为 2；不要创建实例、启动/停止/删除/purge 任一 VM。保留老 D2 和 W8 dirty worktree。

## 下一步顺序
1. 单独新 lane 解决 root Cargo workspace MIT 元数据与 LICENSE、所有包/构建产物清单的一致性；执行 Cargo metadata、锁文件与实际 license gate。先处理 MAGOS overlap/ownership，绝不覆盖现有用户文件或 active lane。
2. 在 Rust 1.99.0 可用环境完成 W2 的 locked metadata/all-targets 和 PostgreSQL/RLS 门禁，区分当前 1.99.1 通过证据与最低版本目标。
3. 为 W3/W5/W6/W7 补齐实际产物、attestation、安装/签名、Vercel/Neon、Canvas 镜像和 Multipass/K3s consumer rollout 证据；如静态设计更优先更新对应文档。
4. 对 W4 的 action SHA pin 或 MSRV 变更单独立项；不得接受当前 1.120.0 候选。
5. W8 先修复 PG Inbox decode（source_sequence 查询缺列）并通过 DB/RLS/concurrency/ACK 证据后才能关闭。
6. 所有 W0-W8 release gates 满足并审核后才进入 W9，由 accountable release owner 决定是否创建 tag v1.0.6。

## 验收
- 更新 docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md：状态、当前基线/报告链接、已核实与未验证分离、下一步顺序、revision history 均准确；不擦除既有历史。
- 新增 docs/reports/PHASE-STAR-STABLE-GATE-SYNC-20261101.md，遵守报告 7 段结构，§5 至少 15 条守门规则。
- 保留 docs/briefs/WBS-STAR-STABLE-GATE-SYNC-20261101.md 与 JSONL dispatch audit。
- 仅限登记 4 文件；运行 rtk proxy git diff --check；提交前回传准确 HEAD、manifest 和验证结果。
- 不执行任何 merge/tag/release/push/deploy；协调者在独立审查后使用 MAGOS set convergence 合并本组文档到本地 dev.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
