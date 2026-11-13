# PHASE-STAR-STABLE-W2-BFF-LOCK-CANDIDATE-REVIEW-20261102

> **结论：ACCEPT（仅接受 BFF lockfile 工件）；W2 仍 OPEN/BLOCKED，稳定版 v1.0.6 仍 NO-GO。** 接受结论不等于 Rust 1.99.0 发布门通过。

## 0. 目的

独立判断当前 W2 BFF lockfile 候选是否具有可集成的静态一致性；将 lockfile 工件结论与 W2 总体验收分开。

## 1. 改动与范围矩阵

| 项目 | 结果 |
|---|---|
| 源候选 | `codex/star-stable-w2-bff-lock-20261031`，HEAD `2a57fa0d329be830735d6657677236e195673362`；只有未提交 `bff/Cargo.lock`。 |
| 基线 | `dev@001904298ca1f01d889beb5809e67e57853565df`；来源分支落后 101 个提交、领先 0 个。 |
| Lockfile | v4，388 个唯一 package identity；20 项 BFF 直接依赖与 manifest 完全匹配；1,237 条依赖边均可解析；28/28 本地包从 BFF 可达。 |
| 当前 dev 适配 | 根 manifest、toolchain 文件及这 28 个本地包的 `Cargo.toml` 在来源 HEAD 与当前 dev 间无字节差异。仍需 rebase 与集成复核。 |
| 差异量 | 新增 58 个 package identity、移除 4 个；包括 async-nats 0.50.0。许可证、advisory 与 runtime feature closure 不在本 review 的证明范围内。 |

## 2. 验证摘要

- `cargo +1.99.1 metadata --manifest-path bff/Cargo.toml --locked --offline --format-version 1`：exit 0；执行前后 lock blob id 均为 `928a8f79d9a1a2efbb187ca1840418492f52f324`。
- Rust 1.99.0 `cargo metadata --locked --offline`：exit 1；原因是本机 `rustc.exe` 不适用于 `1.99.0-x86_64-pc-windows-msvc`，故正式基线门未验证。
- 静态解析：TOML 成功；0 重复 identity、0 未解析/歧义依赖、0 registry checksum 缺失；`git diff --check -- bff/Cargo.lock` exit 0。
- 未运行 all-target build/test、clippy、PostgreSQL、license/advisory、hosted CI 或运行时验证。

## 3. 已知缺口

- Cargo 1.99.1 是诊断证据，不替代规定的 Rust 1.99.0 基线。
- 源候选仍有未提交 lockfile；分支需 rebase 到最新 dev 后再提交/集成。
- W2 根/BFF/ACI 全目标 metadata、features、MSRV 与 release dependency closure 尚未闭合。
- 依赖许可/NOTICE/advisory 审查、真实构建与发行制品验证仍开放。
- W2 继续 OPEN/BLOCKED；整体发行继续 NO-GO。

## 4. 子代理失败接手清单

未发生子代理 RPC 失败。Rust 1.99.0 metadata 进程在执行 `rustc -vV` 时退出，归类为本机 toolchain 阻断；1.99.1 结果单列为诊断证据。

## 5. 守门规则

1. 只审阅注册的 BFF lockfile 工件。
2. 源 W2 checkout 全程只读。
3. 不 stage、reset、stash、commit 或重生成源 lockfile。
4. 不改动其他 worktree/lane。
5. 不读取或改动 `D:\Star` 用户工作树内容。
6. 不移动/修改 main。
7. 不 fetch 或修改远端 refs。
8. 不 push。
9. 不创建 tag/release。
10. 不 build/publish/deploy 制品。
11. 不把 1.99.1 诊断结果写成 1.99.0 通过。
12. 不把 metadata 通过扩大为 all-targets/test 通过。
13. 不把 Cargo.lock 当作许可证/advisory 证明。
14. 记录 locked metadata 前后的 lock hash。
15. 保留 Cargo 1.99.0 本机错误原意，不猜测根因。
16. W2 的全部验收关闭前继续保持 OPEN/BLOCKED。
17. 只接受本 lockfile 候选，不接受整条稳定发行门禁。

## 6. 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 无架构变更；Mavis review 仅覆盖 lockfile 一致性。 |
| SRE Lead | 未评估；无部署/运维就绪结论。 |
| 平台 | 未评估；无 hosted build/runtime 结论。 |
| 评审主持 | 有限范围候选审查完成；rebase/集成由协调者复核。 |
| PM / Release Owner | W2 OPEN/BLOCKED；v1.0.6 NO-GO。 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | W2 candidate review agent — Mavis review | 记录 lockfile 静态一致性、1.99.1 诊断通过与 1.99.0 toolchain 阻断。 | 对精确 W2 BFF lockfile 候选进行独立审查。 |