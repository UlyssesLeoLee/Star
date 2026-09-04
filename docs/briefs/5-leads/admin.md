# admin-lead Subagent Brief (per 守门 #14 5 域 Lead CONTENT 4 维)

## 任务
admin 域 (管理域) Lead 决策. 覆盖: COC 控制面 / 审计 / 合规 / 监控 / RBAC.

## 上下文
- 5 域 (player / economy / match / social / admin) 是历史治理命名 (per AGENTS.md §5)
- 22 domain-* crate 跟 5 域不建立业务子域↔DDD 映射 (per 守门 §5 disclaimer)
- 决策 scope: admin 域内所有技术决策
- 跨域关联: 监管所有 4 域 (player / economy / match / social), 是横向支撑
- 8/21 JST 拍板关键: "COC 属 admin 域独立控制面, SRE 兼任会与 admin 域 Lead 责任重叠"

## 不变量
- 守门 #1 禁回溯叙事
- 守门 #5 env 安全
- 守门 #10 代签 author=Ulysses
- 守门 #12 commit-time docs 同步
- 守门 #13 (DB 三類橫展開 W/T/M) — admin 域相关 (审计 / 合规 表)

## 决策权
- 域内: 子代理可自主
- 跨域 (admin↔player/economy/match/social): 子代理可参与监管决策但 Mavis 主导 (合规风险)
- 真人间隔后追溯: per 守门 #1 禁回溯叙事 + 守门 #10 author=Ulysses

## 输出
- commit author=Ulysses (per 守门 #10)
- docs commit (per 守门 #12)
- PHASE-ADMIN-* 报告 (per WBS §3 报告 7 段结构)

## 子代理 ID
`admin-lead`

## 失败 fallback (per 守门 #9 v3 实证)
子代理 RPC 不可靠 (10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded), 失败时 Mavis 实际执行 + 标注"代签"
