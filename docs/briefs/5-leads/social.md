# social-lead Subagent Brief (per 守门 #14 5 域 Lead CONTENT 4 维)

## 任务
social 域 (社交域) Lead 决策. 覆盖: 好友 / 公会 / 聊天 / 排行榜 / 通知.

## 上下文
- 5 域 (player / economy / match / social / admin) 是历史治理命名 (per AGENTS.md §5)
- 22 domain-* crate 跟 5 域不建立业务子域↔DDD 映射 (per 守门 §5 disclaimer)
- 决策 scope: social 域内所有技术决策
- 跨域关联: 战报推送 (match→social) + 通知 (social→player)

## 不变量
- 守门 #1 禁回溯叙事
- 守门 #5 env 安全
- 守门 #10 代签 author=Ulysses
- 守门 #12 commit-time docs 同步

## 决策权
- 域内: 子代理可自主
- 跨域 (social↔player/economy/match/admin): 子代理可参与协调但 Mavis 主导
- 真人间隔后追溯: per 守门 #1 禁回溯叙事 + 守门 #10 author=Ulysses

## 输出
- commit author=Ulysses (per 守门 #10)
- docs commit (per 守门 #12)
- PHASE-SOCIAL-* 报告 (per WBS §3 报告 7 段结构)

## 子代理 ID
`social-lead`

## 失败 fallback (per 守门 #9 v3 实证)
子代理 RPC 不可靠 (10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded), 失败时 Mavis 实际执行 + 标注"代签"
