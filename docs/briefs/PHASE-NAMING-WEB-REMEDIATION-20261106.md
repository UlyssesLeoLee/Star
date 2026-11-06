# Brief: PHASE-NAMING-WEB-REMEDIATION-20261106

**Agent**: worker
**Phase**: PHASE-NAMING-WEB-REMEDIATION-20261106
**Created**: 2026-11-05 16:29:43

---

在当前 dev 基线上完成命名审计 A1 的最小修复：将 GanttChart 的 linkCountTitle 三语言 tooltip 从品牌/对标文案改为中性的依赖链接计数提示，保留插值、语言切换和现有行为。只修改 GanttChart.tsx 与 en/ja/zh-CN 三份语言字典；不要修改其他 UI、seed、连接器、历史文档或协议标识。运行可用的前端静态检查/测试，并在报告中按 AGENTS.md §0–§7 记录修改矩阵、验证、缺口和下一组待办；不得把未运行的浏览器验收写成通过。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
