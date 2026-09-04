# docs/README.md - Star Mock Project 回归测试报告目录

> **状态**: 初版 (per 2026-09-05 06:50 JST user 拍板 "全栈覆盖 v0.7" + 新建 tools/star-flash-mock/)
> **触发**: scripts/run-all.sh 收尾自动生成 (per 守门 #12 commit-time docs 同步)
> **守门**: 守门 #1+#9+#12+#13

---

## 报告清单

| 文件 | 日期 | 范围 | 状态 |
|---|---|---|---|
| `regression-report-2026-09-05.md` | 2026-09-05 | 全栈 (langgraph + agent-runtime + mcp + streamable-http + db-wtm + 5 域 + openclaw) | 初版 |

## 报告格式

每份报告含 3 段:
1. **跑脚本结果**: 8 份脚本 PASS/FAIL 表
2. **mock_data fixture 统计**: 12 类目录各自 fixture 数量
3. **已知缺口**: 7 个缺标项 (per 守门 #11 缺标比错标安全)

## 跨项目引用 (per 守门 #12)

- **不**引用 RGS 仓 (`D:\RustGameServer\tools\rgs-flash-mock`) 报告格式, 镜像治理但不依赖
- **不**引用 docs/qa/QA-DRIFT-001.md 103 乖离条目 (本项目是测试设计, 不是乖离对账)
- **不**引用 docs/architecture/2026-09-03-langgraph/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md 实装报告 (本项目是回归测试, 不是实装计划)
