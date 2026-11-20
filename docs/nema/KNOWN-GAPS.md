# NEMA 隔离解释器运行时 — 已知缺口 (诚实标注)

本文按 `crates/nema-codec/NEMA_INTERPRETER_PLUGIN_CONTRACT.md` 的风格记录
当前 lane 交付物中**尚未接通**的部分。任何自报为「已可用」的第三方插件执行
都必须以本文被修订为证据前提。

## 本 lane 已交付 (真跑验证)

- `nema-codec::plugin_runtime`: 固定 JSON request/response 协议 (v1)、制品
  sha256 校验、资源限制 (输入/输出/AST 预算 + 宿主内存/CPU 预算传递)、
  `register_isolated` / `parse_isolated` / `format_isolated` (未知 profile
  fail-closed, 绝不回落 built-in)、独立安装生命周期 (install→activate→drain→
  uninstall, ABI 不兼容拒绝, 同 id 重复安装明确报错, upgrade 保留旧版本)、
  文件级 profile 绑定 + 迁移预览/显式迁移。
- `nema-codec/src/sha256.rs`: 纯 Rust SHA-256 (0 unsafe, NIST 向量测试) —
  Cargo.lock 冻结期无法引入 sha2 crate 的替代实现, 仅用于制品完整性校验。
- `sandboxd::nema_worker`: NEMA 协议请求 → sandboxd 会话的适配
  (4 维策略冻结快照 + digest 校验 + 受限 worker 启动 + 响应大小上限),
  e2e 测试用 sandboxd 自带 `nema-worker-fixture` 参考 worker 真跑。
- `nema-codec-wasm`: `previewNemaMigration` 导出。

## 缺口 1: 跨 crate 胶水 (阻塞点: Cargo.lock 冻结)

`nema-codec` 的 `NemaPluginHost` trait 与 `sandboxd::nema_worker` 的受限启动
分属两个 crate, 而两 crate 之间建立 path 依赖需要改动 `Cargo.toml` +
`Cargo.lock` (本 lane 红线文件)。当前两端:

- nema-codec 侧: `NemaPluginHost` trait seam + `WorkerLaunch` 描述
  (program/args/env/cwd — 与 sandboxd `CommandSpec` 1:1 映射, 见文档注释);
  e2e 协议路径用进程内测试宿主 (scripted host) 证明, 非真实第三方执行。
- sandboxd 侧: `run_nema_worker` 真实后端 e2e (本机 backend + 参考 worker)。

**接通方式 (父代理待办)**: 允许 workspace 级依赖后, 在 nema-codec (或独立
胶水 crate) 实现 `SandboxdNemaPluginHost: NemaPluginHost`, 内部调用
sandboxd 后端; 删除两 crate 中重复的 sha256 实现之一。

## 缺口 2: 各维强制力 (Windows v0.1)

sandboxd 的 network/fs/capability 三维在 Windows 上是 v0.1
(PolicyAuditOnly / spawn 前拦截), 仅 resource 维为 Job Objects 内核级。
NEMA worker 会话策略默认 network=BLOCK_ALL + capability=DropAll + fs
allowlist, 但 Windows 上网络/文件系统维的实际强制依赖 sandboxd v0.2
(AppContainer / WFP)。审计记录中的 hit_violations 是策略层证据, 不是内核强制。

## 缺口 3: WASM 目标构建

`nema-codec-wasm` 新导出 (`previewNemaMigration`) 已写 + 宿主侧对照测试
(Rust 与 WASM 输出一致性的证明), 但本 lane 未跑 `wasm32-unknown-unknown`
目标构建 (无 wasm-pack 基建)。构建验证命令与前端接线列父代理待办。

## 缺口 4: 持久化

`NemaFileBinding` / 安装记录目前是进程内结构; 与 Star 持久层 (任务条目、
工作区绑定表) 的落库由父代理在 star-api-rest 侧接线。旧文件保留原解释器
(现状), 迁移只在显式调用时发生 — 持久层必须复刻该语义。
