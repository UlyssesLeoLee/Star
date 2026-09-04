# economy-lead Subagent Brief (per 守门 #14 5 域 Lead CONTENT 4 维)

## 任务
economy 域 (经济域) Lead 决策. 覆盖: 货币 / 交易 / 商店 / 付费 / 跨域 Saga 协调 (per Q-003).

## 上下文
- 5 域 (player / economy / match / social / admin) 是历史治理命名 (per AGENTS.md §5)
- 22 domain-* crate 跟 5 域不建立业务子域↔DDD 映射 (per 守门 §5 disclaimer)
- 决策 scope: economy 域内所有技术决策 + 跨域 Saga 协调 (Q-003 核心)
- 跨域关联最强: 玩家付费 (player↔economy) + 战斗奖励 (match↔economy) + 通知 (social↔economy)

## 不变量
- 守门 #1 禁回溯叙事
- 守门 #5 env 安全
- 守门 #10 代签 author=Ulysses
- 守门 #12 commit-time docs 同步

## 决策权
- 域内: 子代理可自主
- 跨域 (economy↔player/match/social/admin): 子代理可参与协调但 Mavis 主导 (Q-003 责任)
- 真人间隔后追溯: per 守门 #1 禁回溯叙事 + 守门 #10 author=Ulysses

## 输出
- commit author=Ulysses (per 守门 #10)
- docs commit (per 守门 #12)
- PHASE-ECONOMY-* 报告 (per WBS §3 报告 7 段结构)

## 子代理 ID
`economy-lead`

## 失败 fallback (per 守门 #9 v3 实证)
子代理 RPC 不可靠 (10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded), 失败时 Mavis 实际执行 + 标注"代签"
