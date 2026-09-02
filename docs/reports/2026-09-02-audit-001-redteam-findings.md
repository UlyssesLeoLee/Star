# AUDIT-001 红方全文档对抗式挑刺 — 结果记录

> **来源**: 后台 agent "红方:全文档对抗式挑刺" (task-id `a71e563fff85484a1`)
> **完成时间**: 2026-09-02
> **范围**: `docs/requirements.md`, `docs/basic-design.md`, `docs/specs/domain-*-spec.md`(32 files), `docs/specs/workflow-templates-spec.md`。`internal-design.md` / `external-design.md` 排除。
> **状态**: 未经蓝方复核，未拍板，仅存档原始输出，供后续 blue-team 跟进使用。

## 总计

高: 13　中: 8　低: 5　共 26 项

(已知问题——裸 ID `WT-001`/`FBK-001`/`CTX-001`/`CTX-002`、scm↔worktree 事件订阅误标——按指示未重复上报)

## 分类摘要

### Category 1 — REQ-ID 双向交叉检查
- **F1(高)**: 13 个 `requirements.md` 中定义的 REQ-ID 在 `basic-design.md`/33 个 spec 中零引用: `REQ-COLLAB-003/004`, `REQ-DATA-001`, `REQ-OPS-001/002/003`, `REQ-PLAN-002/007`, `REQ-RT-002`, `REQ-TST-001/002`, `REQ-TWP-002`, `REQ-WI-001`
- 反向检查干净（未定义引用均归约为已知裸 ID 遗留）
- 低置信: `REQ-PLAN-008` 只是 REQ-PLAN-007 文本内的假设性未来 ID，非真实定义

### Category 2 — §引用完整性(全部 33 个 header)
- **F-missing7(高)**: 7 个 spec (`ai/cli/dashboard/form/kms/report/theme`) 描述的域完全不在 `basic-design.md` 25 行 §2.1 表中
- **F4(高)**: `[basic-design §6 22 logical domain + 7 supporting crate]` 双重错误引用
- **F3(高)**: 7 个 supporting-crate spec 相对路径链接全部损坏(`test -f` 验证)
- **F2(中)**: `domain-identity/local-runtime-spec.md` 误将 requirements.md 自身章节号(§23.x)当作 basic-design.md 的章节引用
- **F5(高)**: 5 个文件内部自相矛盾，同一文档 "7 supporting crate" vs "6 supporting crate"
- **F6(高)**: 5 个文件遗留未替换模板占位符 `{display}`
- **F7(中)**: `basic-design.md:509-511` 声称补 14 个域接触面，实际枚举 15 个
- **F8(中)**: 声称 §3.2.1-§3.2.8 覆盖 11 个域，实际可数 12 个
- **F9(中)**: 声称全部 22 domain 覆盖，实际 tally 得 25（22 vs 25 两套域清单混用）

### Category 3 — 依赖方向(全部 25 行 §2.1 逐行检查)
- **Finding-Worktree/WorkItem(高)**: §2.3 禁线 vs §2.1/spec Appendix B 三方矛盾
- **Finding-SCM(高)**: §2.3 禁线 vs spec"下游调用"+§3.2.7 矛盾
- **Finding-Workflow(高)**: §2.1 依赖方向与 spec 自述、§3.2.1、work-item spec 三方反向
- **Finding-Context(高)**: spec 自身 Appendix B 内部矛盾，且其中一支违反 §2.3 禁线
- **Finding-Validation(高)**: §2.1 声称依赖 agent，spec 自述及 §3.2.6 显示方向相反
- **Finding-Agent(高)**: §2.1 依赖集合与 spec 上/下游列表方向和成员均不符
- **Finding-Board/Planning(高)**: §2.1 row 10/11 与两个 spec 自身 Appendix B 均零重叠或方向相反
- **Finding-Identity/Permission(中)**: §2.1 列 tenant 依赖，两个 spec 自述"无核心依赖"未做限定说明
- **Finding-Integration(中)**: §2.1 列 work-item 依赖，spec 自身列表中不存在
- **Finding-Relation(中，spec 内部)**: 自身上游/下游依赖列表不一致，且与 §3.2.9 方向相反
- **Finding-Comment(低)**: §2.1 遗漏 spec 自述的 scm 依赖，非矛盾，是欠记录
- **Category-7 扩展(中)**: search/audit/notification/collaboration 四行 §2.1 未标注"订阅/投影"性质，重复 domain-automation(row17) 已知误标模式
- **Arrow-notation meta-finding(中)**: §2.3 ASCII 依赖图相邻两行箭头方向读法不一致

### Category 4/5/6 — 抽查基本干净
- 状态机交叉检查(Worktree 17-state / AgentSession 14-state / WorkItem 3-state / 13 类 tenant_id 对象)一致，无差异
- 低置信观察: P0-P4(§4.4.4) 与 P0-P5(§4.10.7) 命名重叠可能引发混淆；7 个 supporting-crate spec 均缺"附录 B:边界清单"章节，结构性不一致

## 下一步（待用户确认后再执行）
- [ ] 蓝方复核 26 项，逐项定性："文档笔误" vs "真实架构矛盾需拍板"
- [ ] Finding-Context / Finding-SCM / Finding-Worktree-WorkItem 涉及 §2.3 硬禁线冲突，优先级最高，可能需要 Ulysses 直接拍板
- [ ] F3/F5/F6（链接损坏/自相矛盾/占位符未替换）是纯文档缺陷，可批量修复，风险低
