# Brief: STAR-STABLE-W3-WBS-CONTEXT-20261103

**Agent**: codex-worker-w3
**Phase**: W3-WBS-CONTEXT-20261103
**Created**: 2026-11-03 03:35:39

---

任务：校正 Star Stable WBS 中 k3s-local Dockerfile 的过时描述，保持 W7 和发行门禁的范围准确。

基线：dev@234ea49eafa9f8a6acf466b32f134f9518aeebec。WBS 顶部目前仍称 deploy/k3s-local/Dockerfile 使用 COPY crates ./crates；当前源码已变成标记模板（@star-local-crates-begin/end），scripts/automation/star_k3s_context.py 在生成 context 时展开显式 crate COPY，并拒绝 broad COPY / game 路径。现有 scripts/automation/test_star_k3s_context.py 有生成 context 的 allowlist 测试。较早 W7 阶段报告基于旧提交，当前更新还包括 2026-11-03 W7 junction 修复的验收记录。

独占范围：docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md、docs/briefs/STAR-STABLE-W3-WBS-CONTEXT-20261103.md、docs/reports/PHASE-STAR-STABLE-W3-WBS-CONTEXT-20261103.md、docs/reviews/STAR-STABLE-W3-WBS-CONTEXT-REVIEW-20261103.md、docs/reports/WBS-DISPATCH-STAR-STABLE-W3-WBS-CONTEXT-20261103.jsonl。W4 lane 不编辑 WBS，也不编辑这些文件。

要求：逐项核对当前 Dockerfile、context generator、相关测试和 WBS 已有 W7 摘要。只修正与当前 dev 事实矛盾的高层描述，并新增修订记录；写清楚生成的过滤 context 与模板行为已经有静态实现/测试证据，但这不能代替实际 BuildKit context archive、最终镜像/digest、SBOM/provenance、Multipass/K3s、consumer 和 rollback 验收。W7 / W3 / v1.0.6 保持 OPEN/BLOCKED 或 NO-GO，不要把静态测试扩写成发布通过。不要改 Dockerfile、生成器、D2 owner 文件、main、远端或 D:\Star 用户文件。

验证：对照 Git 基线、源码、已有 focused test evidence；仅运行文档 diff 检查。报告需明确证据来源和未运行的验证，不重复无依据地声称此前测试本轮通过。文档报告按仓库报告结构记录目的、改动矩阵、验证、缺口、失败/接手清单、守门规则、签批栏和修订历史。提交 lane commit 并回报 hash、实际路径和验证结果。

CodeGraph 当前仓库未索引（无 .codegraph）；对 workflow/docs/config 的有限核对已由协调者完成，若需代码验证请只读精确查看相关生成函数与测试。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
