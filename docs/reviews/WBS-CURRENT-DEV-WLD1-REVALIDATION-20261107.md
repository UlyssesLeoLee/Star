# WLD1 Repository Boundary 候选当前基线复核

- 复核日期：2026-11-07
- 目标基线：dev@e2393618d5d9cf71181bdadf78b5b4663850e4cf
- 来源：codex/wbs-wld1-boundary-review-20261025@de2efaad3389c37073af4e6b8282d968d22eb876
- 处置：ADAPT selectively；不 cherry-pick 候选提交或整份文件

## 可保留的结论

- 用户已确认 Project→Repository 为 0..N、解绑前先迁移、Project 事实由 Star 原生 Project owner 持有；未决的是 writer/API/schema、目标引用清点、owner 事务边界、跨 owner 协调和 PostgreSQL/RLS 运行证据。
- 候选指出的未完成实现边界仍有效：当前 Group Worktree lifecycle provider 接口要求重验当前授权和 Run repository binding（crates/star-api-rest/src/group_api/worktree_lifecycle.rs:327-358），但缺省 provider 返回 worktree_lifecycle_unavailable（同文件 :528-532），GroupApiState::new 将其设为 None（crates/star-api-rest/src/group_api.rs:361-384），启动接线没有安装该 provider（crates/star-api-rest/src/main.rs:95-101）。Worktree 写入必须继续 fail closed。
- 候选关于文档错位的发现仍值得处理：当前 Group WBS §13 的 WLD1-R0b 行仍称“具体关联状态与事务边界待设计确认”（目标 WBS 第 420 行），而 SRS §13、BD §13、DD §13 已定义 active/retiring/revoked、revision/CAS、迁移先于解绑及 owner-local event 边界。候选也指出 owner 已确认但旧 review 仍写“等待确认”（当前 review 第 122-124 行，对照 WBS 第 405-412、419-422 行）。

## 必须修订后才能复用的内容

候选是旧基线快照，不能原样合并。其 WBS 交叉引用至少有以下偏移：当前 SCM Repository master 行在 WBS 第 372 行，历史“没有 master”说明在第 328 行；当前待同步设计状态行在第 420 行，而不是候选所引第 416 行；当前主要状态证据位于第 405-412、419-422 行。目标 WBS 第 324 行已说明后续状态应以 §10 为准；不得改写第 328 行的历史叙述。候选内关于“当前 WBS”的段落是历史快照，必须标注其时间或移除。

候选提出的权威引用源、eligible target、迁移期间 admission fence、proof completeness/revision、跨 owner coordination 与数据库行为问题继续 OPEN；不得用候选文本将它们写成已解决。候选关于 CodeGraph generation 的陈述不是本次新验证结果，不在本复核中重述为当前事实。

## 阻断与后续

该 WLD1 review 分支本身仅可择取内容；本次新复核保留本文件作为更新证据。Group WBS 主文件当前被受保护 Rust 候选登记，MAGOS 拒绝本 lane 修改该文件，因此本次不回写主 WBS。待该冲突解除后，需同步 WBS 的 WLD1 状态与当前 design revision；在 owner API、数据库/RLS 和运行验收通过前继续 OPEN。
