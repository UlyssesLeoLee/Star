# STAR-STABLE-W3-CONSUMER-VERIFY-REVIEW-20261103

- 审查对象：`codex/star-stable-w3-consumer-verify-20261103`
- 代码提交：`4ed2dc2725de57213d5271ff853bd2cc909c7c11`、`75cc9b72e8fad3500d80176e22d97e5542246e90`
- 审查结论：通过本地实现与聚焦测试门，可进入本地 dev 集成；不能据此关闭 W3 或稳定发行门。

## 范围

- `scripts/release/verify_attested_release.py`
- `scripts/release/tests/test_verify_attested_release.py`
- 与 WBS W3“独立 consumer 验证”要求的对应关系及测试证据。

本地仓库在 `D:\\Star` 的 main worktree 存在用户未提交文件，本审查与本 lane 写入均在独立 Orca worktree 执行；main 与这些用户文件未改动。目标 dev checkout 基线为 `fedc3dc12ec16235394a7c46d2603507231f0ee6`，检查时干净且 Orca 没有活动 terminal/agent。

## 发现及处理

| 严重度 | 发现 | 处理 | 结果 |
|---|---|---|---|
| Warning | 首版将 GitHub CLI stdout 全部收集后才检查 4 MiB，实际上没有限制峰值缓冲。 | 提交 `75cc9b72` 改为并发读取、固定大小缓冲，超过上限就终止并回收子进程；新增 8 KiB 输入/32 byte 上限测试。 | 关闭；测试确认读取不超过上限加一个固定 8 byte 读块并调用 kill。 |
| Warning | 本地文件可能在初始 SHA-256 与 CLI 检查之间发生内容变化。 | 成功返回前再次流式计算本地文件 SHA-256；新增验证中变更测试。 | 一般持久内容变化关闭；同一用户恶意改写后恢复原字节不在此工具的本地并发威胁模型内。 |
| Info | GitHub CLI 成功 JSON 文档保证顶层为非空数组且每项有 `attestation`、`verificationResult` 对象，但没有稳定嵌套 schema。 | 工具只检查文档保证的顶层形状，并显式传入来源 digest/ref、批准 signer 与 SLSA predicate 约束。 | 接受；不读取未文档化的 bool/字段。 |
| Info | 当前无真实 v1.0.6 artifact、hosted run 或 attestation bundle 可供端到端检查。 | 测试模拟子进程并将该限制保留在阶段报告/WBS 开放项。 | 未关闭，不能标记 W3 完成。 |

## 验证

- 独立运行 `rtk proxy py -3 -m unittest scripts.release.tests.test_verify_attested_release -v`：11 项通过。
- 独立运行 `rtk proxy py -3 -m py_compile scripts/release/verify_attested_release.py scripts/release/tests/test_verify_attested_release.py`：通过。
- `rtk proxy git diff --check dev...HEAD`：通过。
- CypherGraph Guardian：worker 报告 `PASS (2 source file(s) checked)`；本轮未独立重复。
- GitHub CLI hosted verification、真实发布制品与 v1.0.6 端到端消费：未运行。

## 与当前 W3 验收要求的差距

此 lane 只交付一个可复用的 consumer verifier。它没有构建 Desktop/Web/Canvas 制品，没有生成或绑定实际 SPDX BOM，没有归档 bundle，也没有通过 hosted workflow 或真实 release 候选消费。当前 dev WBS 对 W3 的整体状态必须保持 OPEN/BLOCKED；稳定版 v1.0.6 仍 NO-GO。

## 结论

批准仅限把上述已审查代码与本 lane 记录合入本地 dev。WBS 主文件当前由 D2 lane 持有，需等该 claim 释放后补充 W3 evidence，并维持未完成状态，直至真实制品链的全部验收证据到位。
