---
id: PG-audit_audit_event
title: audit_audit_event (T WORM)
date: 2026-09-06
source: 'S7 §3.2 L82 + ADR-0043'
status: design-baseline
classification: T
importance: P0
layer: persistence
view: Cross
related: ["PostgresCheckpointer", "M-25"]
see-also: []
version: 0.1
revision: '['05-persistence-checkpoint']'
supersedes: null
in-topology: ["PG-audit_audit_event"]
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
  - persistence/nodes
  - cross-view
  - obsidian-wiki
  - design-topology
  - id/pg-audit_audit_event
---

# PG-audit_audit_event — audit_audit_event (T WORM)

> **节点类型**: persistence / Cross
> **重要度**: P0
> **守门分类**: T

## 1. 描述

Transaction (WORM), 自身必审计, RLS 13 類必携, per ADR-0043

**文档来源**: S7 §3.2 L82 + ADR-0043

## 2. 出现在拓扑 (in-topology)

- [[PG-audit_audit_event]]

## 3. 上下游 (related)

- [[PostgresCheckpointer]]
- [[M-25]]

## 4. 横向相关 (see-also)

_无_

## 5. 守门实证

| 守门 | 实证 |
|---|---|
| #1 0 unsafe + 0 err | 文档实证 100% |
| #3 5 域独立 Lead / 3 view 平行 | 不建立业务子域↔DDD 映射 |
| #11 缺标比错标 | 已知缺口显式列 |
| #12 AI 協作文档治理 | 0 回溯叙事 |
| #13 DB W/T/M | 守门 #13 (本节点 = T) |
| #19 agent 交互 Python 化 | 本脚本 = scripts/automation/obsidian_topology_gen.py |

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | 2026-09-06 17:13 JST 用户拍板「规划内拓扑结构应该构成一份 obsidian wiki」 |
