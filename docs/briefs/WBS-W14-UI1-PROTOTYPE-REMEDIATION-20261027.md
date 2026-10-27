# Brief: WBS-W14-UI1-PROTOTYPE-REMEDIATION-20261027

**Agent**: worker
**Phase**: WBS-W14-UI1-PROTOTYPE-REMEDIATION-20261027
**Created**: 2026-10-27 05:16:46

---

任务：依据已合入 dev 的 WBS-W14-UI1-INDEPENDENT-REVIEW-20261027.md 修正静态 UI-1 原型并同步 WBS。

基线：dev@4f629ce5103fb4a9fb67a775341475ef66bbf2ea。

唯一可编辑文件：docs/prototypes/w14-ui1-flow-20261026/index.html、docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md，以及本 lane 的 brief 与 PHASE-WBS-W14-UI1-PROTOTYPE-REMEDIATION-20261027.md 报告。不要改生产代码、API、数据库、其他设计文档或既有复核报告。

修复复核报告 I-1 至 I-4：1) 冷启动默认无 Worktree 焦点、无已通过授权；Run Apps 与 Branch 值必须等显式模拟授权通过后显示。2) Project Index 列表授权与 Worktree 焦点授权分开建模；列表未授权/检查中/拒绝/撤权时隐藏受保护样例行，授权成功后才显示；点击 Worktree 后重新进入 checking，授权拒绝/撤权清除所选焦点和旧 Branch/Run App/WORKTREE scope。Index 复验/拒绝/撤权只影响列表，不撤销仍有效的 Worktree 焦点或其聊天 scope；Index 复验开始时保留当前焦点页面，取消/拒绝/撤权后恢复仍有效焦点，检查中提供“返回当前 Worktree”并取消本地模拟检查。若焦点授权已失效则回到 workspace。不得让迟到或旧状态闪现。3) 变体 B 的 Project Index 视图不能重复绘制索引。4) Task Card 归属 Run；Worktree 和本地 Branch 只作为 CLI 执行上下文；CLI 保持禁用，因为真实受控 Runtime 未接入。

保留：Project → Engineering Run → Worktree → Branch 顺序；Canvas 与 Task Cards 同级；GLOBAL/WORKTREE 草稿隔离；演示数据和非生产说明；无任意 Shell。文案不得引用竞品品牌。

同步 WBS §16 和 §17：记录独立复核报告已完成、四项发现，以及本 lane 实际修复内容；W14 保持 OPEN，浏览器/键盘/读屏/产品 API/性能与 production 验证均不得标成通过。

验证：不新增或运行测试、构建或产品运行时。对改动文件做 diff/self-review、检查 HTML/脚本静态结构与 WBS 描述一致性，并在报告中逐项记录结果与未验证限制。任何无法安全区分的授权状态按 fail-closed 处理。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
