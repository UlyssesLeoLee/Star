---
id: T-N5
title: respond
date: 2026-09-06
source: 'S2 §2.1.2 L204'
status: design-baseline
classification: T (Transaction - guard violation log)
importance: P0
layer: orchestration
view: LangGraph
related: ["TopAgent", "C-01"]
see-also: ["C-10-GuardEnforcer"]
version: 0.1
revision: '['02-orchestration-langgraph']'
supersedes: null
in-topology: ["T-N5"]
guards:
  - id: '#1'
    name: 0 unsafe + 0 err
    evidence: 文档实证, 100% file:line 引用
  - id: '#3'
    name: 5 域独立 Lead / 3 view 平行
    evidence: 不建立业务子域↔DDD 映射
  - id: '#7'
    name: 0 unsafe
    evidence: 纯 markdown / JSON, 无代码
  - id: '#11'
    name: 缺标比错标
    evidence: 已知缺口显式列
  - id: '#12'
    name: AI 協作文档治理
    evidence: 0 回溯叙事, BAS git 实证
  - id: '#13'
    name: DB W/T/M 严格分类
    evidence: PG 5 表 100% 覆盖 (T=3, M=1, T-WORM=1)
  - id: '#19'
    name: agent 交互 Python 化
    evidence: 本脚本 = scripts/automation/obsidian_topology_gen.py
  - id: '#10'
    name: 代签规则应用
    evidence: author=Ulysses (per 19:39 JST 授权)
tags:
  - orchestration/nodes
  - langgraph-view
  - obsidian-wiki
  - design-topology
  - id/t-n5
---

# T-N5 — respond

> **节点类型**: orchestration / LangGraph
> **重要度**: P0
> **守门分类**: T (Transaction - guard violation log)

## 1. 描述

LLM 生成 user-facing 回答

**文档来源**: S2 §2.1.2 L204

## 2. 出现在拓扑 (in-topology)

- [[T-N5]]

## 3. 上下游 (related)

- [[TopAgent]]
- [[C-01]]

## 4. 横向相关 (see-also)

- [[C-10-GuardEnforcer]]

## 5. 守门实证

| 守门 | 实证 |
|---|---|
| #1 0 unsafe + 0 err | 文档实证 100% |
| #3 5 域独立 Lead / 3 view 平行 | 不建立业务子域↔DDD 映射 |
| #11 缺标比错标 | 已知缺口显式列 |
| #12 AI 協作文档治理 | 0 回溯叙事 |
| #13 DB W/T/M | 守门 #13 (本节点 = T (Transaction - guard violation log)) |
| #19 agent 交互 Python 化 | 本脚本 = scripts/automation/obsidian_topology_gen.py |

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | 2026-09-06 17:13 JST 用户拍板「规划内拓扑结构应该构成一份 obsidian wiki」 |
