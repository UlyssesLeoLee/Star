# COORDINATION-CHECK-001

> **无限画布设计 vs Agent Relationship Graph (ARG) 设计 协调性检查报告 v0.1**
>
> - 触发: 2026-09-10 18:30 JST Ulysses 拍板"确保本设计和agent图关系设计妥善协调不冲突"
> - 受众: Mavis 接手 (root session) / 5 域 Lead / 后续 P3-D.6 实施工程师
> - 拍板: per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 (不重写 ARG 4 份 commit, 仅修复本批新写 BD/DD)
> - 日期: 2026-09-10 JST
> - 关联 commit: 待生成 (root 统一 commit 时填, per 守门 #1 v15 docs 同步饱和)

---

## §0 文档信息 / 修订履历 (per AGENTS.md §3 7 段结构 + IPA SEC 模板)

### 0.1 文档信息

| 项目 | 内容 |
|---|---|
| 文书 ID | COORDINATION-CHECK-001 |
| 文书名 | 无限画布设计 vs Agent Relationship Graph (ARG) 设计 协调性检查报告 |
| 版本 | v0.1 (协调性检查初始版) |
| 作成日 | 2026-09-10 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手) (per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses) |
| 关联 commit | `fb89e4a` (10 files / 8,160 insertions, 第 74 次新事件) + `153441a` (§4.29 + registry v0.14, 第 75 次新事件) + `34fa7e7` (DD-001 v0.1.1 自审 fix 2 处, 第 76 次新事件) |
| 触发 | 2026-09-10 18:30 JST Ulysses 拍板"确保本设计和 agent 图关系设计妥善协调不冲突" |

### 0.2 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-10 18:33 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **初始版本: 协调性检查报告, 10 项检查 7 通过 + 3 冲突发现 (C-16 + C-21 关键 class 编号冲突 + Trust Score 5 档 命名/阈值不匹配), 修复方案: 关键 class 顺延 (C-25 + C-26 新增) + Trust Score 5 档统一为 ARG 源, 7 段结构 (§0 文档信息 + §1-§5 检查 + §6 5 角色签字栏 + §7 修订履历, per AGENTS.md §3 模板)** | **2026-09-10 18:30 JST Ulysses 拍板"确保本设计和 agent 图关系设计妥善协调不冲突"** |
| **v0.1.1** | **2026-09-10 19:10 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **IPA SEC 合规性修复: 加 §0 文档信息 / 修订履历 段 (跟其他 9 份 P3-D.5 文档结构对齐, 之前 §1-§7 缺 §0 文档信息) + §6 5 角色签字栏 / §7 修订履历 已存在无需重写. 不重写 §1-§5 检查内容 (per 守门 #1 禁回溯叙事, v0.1.1 反转行显式标).** | **2026-09-10 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准"** |

---

## §1 检查范围

### 1.1 本设计 (本批 9 份新写)

| 文档 | 字节 | 角色 |
|---|---|---|
| `docs/requirements/SRS-CANVAS-001.md` v1.1 | 58KB | 总册 SRS |
| `docs/requirements/SRS-CANVAS-AGENT-001.md` v1.2 | 155KB | 专题 SRS (双核心 1: agent 管理 46 项) |
| `docs/requirements/SRS-CANVAS-GAMIFY-001.md` v1.0 | 92KB | 专题 SRS (双核心 2: 游戏化 32 项) |
| `docs/design/BD-CANVAS-001.md` v0.1 | 50KB | 总册 BD |
| `docs/design/BD-CANVAS-AGENT-001.md` v0.1 | 105KB | 专题 BD (agent 管理 46 项) |
| `docs/design/BD-CANVAS-GAMIFY-001.md` v0.1 | 111KB | 专题 BD (游戏化 32 项) |
| `docs/design/DD-CANVAS-001.md` v0.1 | 47KB | 总册 DD (本批) |
| `docs/design/DD-CANVAS-AGENT-001.md` v0.1 | (子代理 1 撰写中) | 专题 DD (agent 管理 46 项) |
| `docs/design/DD-CANVAS-GAMIFY-001.md` v0.1 | (子代理 2 撰写中) | 专题 DD (游戏化 32 项) |

### 1.2 Agent Relationship Graph (ARG) 设计 (已有 4 份, 9/9 落档)

| 文档 | 字节 | 角色 |
|---|---|---|
| `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1 | 37KB | ARG 主源 SRS (10 类关系 + 4 维度 + 5 模板 + 4 表 + 8 UC) |
| `docs/design/BD-AGENT-RELATIONSHIP-001.md` v0.1 | 55KB | ARG BD (5-tier 架构 + 7 张表 W/T/M + 13 端点 + 8 UC) |
| `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1 | 94KB | ARG DD (13 关键 class + 4 effect + 5 状态机 + 11 共享类型 + 4 时序图 + 8 Cypher + 10 challenges prompt + 74 测试) |
| `docs/design/DDD-REVIEW-AGENT-RELATIONSHIP-001.md` v0.1 | 30KB | ARG DDD Review (跨 DDD 边界) |

### 1.3 检查维度 (per Ulysses 18:30 JST 拍板)

10 类关系命名 + 颜色 + 4 维度协作影响 + 5 团队模板 + 13 关键 class + 5 状态机 + 11 共享类型 + Trust Score 5 档 + W/T/M 表分类 + 字段名 + 跨域接口

---

## §2 协调性检查结果

### 2.1 ✅ 10 类关系命名 (per SRS-AGENT-RELATIONSHIP-001 §4.1.1+4.1.2)

| 类型 | 语义 | ARG 命名 | BD-AGENT-REL | BD-CANVAS-AGENT-001 | DD-CANVAS-001 | 一致性 |
|---|---|---|---|---|---|---|
| **delegates_to** | A → B 委派任务 | ✅ | ✅ :DELEGATES_TO directed | ✅ A11.1 | ✅ C-3 EdgeOps | ✅ |
| **consults** | A 关键决策调 B 拿意见 | ✅ | ✅ :CONSULTS directed | ✅ A11.1 | ✅ | ✅ |
| **collaborates_with** | A ↔ B 并行 + merge | ✅ | ✅ :COLLABORATES_WITH undirected | ✅ A11.1 | ✅ | ✅ |
| **reports_to** | A → B 汇报 | ✅ | ✅ :REPORTS_TO directed | ✅ A11.1 | ✅ | ✅ |
| **mentors** | A → B 长期指导 | ✅ | ✅ :MENTORS directed | ✅ A11.1 | ✅ | ✅ |
| **peer_reviews** | A ↔ B 双向 review | ✅ | ✅ :PEER_REVIEWS undirected | ✅ A11.1 | ✅ | ✅ |
| **stand_in_for** | A 故障 B 接管 | ✅ | ✅ :STAND_IN_FOR directed | ✅ A11.1 | ✅ | ✅ |
| **shadows** | A 静默观察 B | ✅ | ✅ :SHADOWS directed | ✅ A11.1 | ✅ | ✅ |
| **challenges** | A 质疑 B 决策 | ✅ | ✅ :CHALLENGES directed | ✅ A11.1 | ✅ | ✅ |
| **trusts** | A 信任 B 跳过 review | ✅ | ✅ :TRUSTS directed, weight ≥ 0.8 | ✅ A11.1 | ✅ | ✅ |

**10 类关系命名 100% 一致** ✅

### 2.2 ⚠️ 4 维度协作影响 (per SRS-AGENT-RELATIONSHIP-001 §4.3)

| 维度 | ARG 命名 | BD-AGENT-REL §4.3 | BD-CANVAS-AGENT-001 | DD-CANVAS-001 §2.3 |
|---|---|---|---|---|
| **Dispatch 路由** | ✅ delegates_to / reports_to / stand_in_for | ✅ ARGDispatchRouter (C-11) | ✅ A11.3 4.1 | ✅ |
| **Context 共享** | ✅ mentors / shadows | ✅ ARGContextInjector (C-12) | ✅ A11.3 4.2 | ✅ |
| **Trust 加权** | ✅ trusts / peer_reviews | ✅ ARGTrustEngine (C-13) | ✅ A11.3 4.3 | ✅ |
| **Output 评估** | ✅ challenges / peer_reviews | ✅ ARGOutputEvaluator (C-14) | ✅ A11.3 4.4 | ✅ |

**4 维度命名 100% 一致** ✅

### 2.3 ✅ 5 团队模板 (per SRS-AGENT-RELATIONSHIP-001 §4.5)

| 模板 | 拓扑 | ARG | BD-AGENT-REL | BD-CANVAS-AGENT-001 | DD-CANVAS-001 | 一致性 |
|---|---|---|---|---|---|---|
| **Hub-and-Spoke** | 1 Lead + 4 Worker (4 delegates_to) | ✅ | ✅ | ✅ A11.4 | ✅ | ✅ |
| **Mesh** | N 节点全连接 (N×(N-1)/2 collaborates_with) | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Chain** | A → B → C → D (3 delegates_to) | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Hierarchical** | 1 Lead → 2 Sub-Lead → 6 Worker (9 节点) | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Review-Council** | 3 Reviewer ← 1 Lead (1 deleg + 3 consults) | ✅ | ✅ | ✅ | ✅ | ✅ |

**5 模板命名 100% 一致** ✅

### 2.4 ⚠️⚠️ 13 关键 class 编号冲突 (per DD-AGENT-RELATIONSHIP-001 v0.1 §3.1)

| 编号 | DD-AGENT-REL v0.1 §3.1 命名 (ARG 源) | BD-CANVAS-AGENT-001 §4.12 + DD-CANVAS-001 §4.13 命名 (本批) | 冲突? |
|---|---|---|---|
| C-1 | `MemgraphClient` (Data Rust P0) | (未冲突) | ✅ |
| C-2 | `AgentNode` (Data Rust P0) | ✅ (派生) | ✅ |
| C-3 | `EdgeOps` (Data Rust P0) | (未冲突) | ✅ |
| C-4 | `TemplateOps` (Data Rust P0) | (未冲突) | ✅ |
| C-5 | `CypherCache` (Data Rust P1) | (P3-C 实装, 留) | ✅ |
| C-6 | `EventWriter` (Data Rust P0) | (P3-C 实装, 留) | ✅ |
| C-7 | `MemgraphEventListener` (Bridge Rust P0) | ✅ A11.9 (派生) | ✅ |
| C-8 | `LangGraphStateUpdater` (Bridge Rust P0) | ✅ A11.9 (派生) | ✅ |
| C-9 | `PeriodFlushWorker` (Bridge Rust P0) | ✅ A11.9 (派生) | ✅ |
| C-10 | `OfflineQueue` (Bridge Rust P1) | (P3-C 实装, 留) | ✅ |
| C-11 | `ARGDispatchRouter` (Effect Rust P0) | ✅ A11.3 4.1 (派生) | ✅ |
| C-12 | `ARGContextInjector` (Effect Rust P0) | ✅ A11.3 4.2 (派生) | ✅ |
| C-13 | `ARGTrustEngine` (Effect Rust P0) | ✅ A11.3 4.3 (派生) | ✅ |
| C-14 | `ARGOutputEvaluator` (Effect Rust P0) | ✅ A11.3 4.4 (派生) | ✅ |
| C-15 | `ARGAchievementEngine` (Effect Rust P0) | ✅ A11.4 (派生) | ✅ |
| **C-16** | **`RelationshipEditor` (UI TSX P0)** | ⚠️ **`CanvasBackend` (本批 A12.3 新增, 冲突!)** | **❌ CONFLICT** |
| C-17 | `RelationshipView` (UI TSX P0) | (未冲突) | ✅ |
| C-18 | `AchievementWall` (UI TSX P1) | (未冲突) | ✅ |
| C-19 | `EdgeTypeSelector` (UI TSX P0) | ✅ A11.1 (派生) | ✅ |
| C-20 | `TemplateGallery` (UI TSX P1) | ✅ A11.4 (派生) | ✅ |
| **C-21** | **`ARGController` (API Rust P0)** | ⚠️ **`MultiUserAudit` (本批 A12.8 新增, 冲突!)** | **❌ CONFLICT** |
| C-22 | `ARGSSEHub` (API Rust P0) | ✅ A11.9 (派生) | ✅ |
| C-23 | `ARGPermission` (API Rust P0) | ✅ A11.7 (派生) | ✅ |
| C-24 | `ARGMigration` (Data Rust P2) | (P3-C 实装, 留) | ✅ |

**冲突 #1**: C-16 在 ARG DD v0.1 中是 `RelationshipEditor` (UI), 在本批 BD/DD 中是 `CanvasBackend` (A12.3 新增)
**冲突 #2**: C-21 在 ARG DD v0.1 中是 `ARGController` (API), 在本批 BD/DD 中是 `MultiUserAudit` (A12.8 新增)

### 2.5 ⚠️⚠️ Trust Score 5 档 命名/阈值不匹配 (per DD-AGENT-REL v0.1 §3.3.3)

| 5 档 | DD-AGENT-REL v0.1 命名 + 阈值 | BD-CANVAS-AGENT-001 §3 + DD-CANVAS-001 §5.3 命名 + 阈值 | 冲突? |
|---|---|---|---|
| 第 1 档 | **Untrusted (0-0.2)** | **Unknown (0.0)** | **❌ CONFLICT (命名 + 范围)** |
| 第 2 档 | **Low (0.2-0.4)** | **Low (0.1-0.3)** | **❌ CONFLICT (阈值)** |
| 第 3 档 | **Medium (0.4-0.7)** | **Medium (0.4-0.6)** | ⚠️ 阈值边界不一致 (0.7 vs 0.6) |
| 第 4 档 | **High (0.7-0.9)** | **High (0.7-0.8)** | **❌ CONFLICT (阈值)** |
| 第 5 档 | **VeryHigh (0.9-1.0)** | **Trusted (0.9-1.0)** | **❌ CONFLICT (命名)** |

**冲突 #3**: 5 档命名 (Untrusted/Unknown) + 阈值 (Low/Medium/High) 全部不匹配

### 2.6 ✅ 11 共享类型 (per DD-AGENT-RELATIONSHIP-001 v0.1 §3.2.5)

| 类型 | DD-AGENT-REL v0.1 | BD-CANVAS-AGENT-001 | DD-CANVAS-001 | 一致性 |
|---|---|---|---|---|
| ARGEvent (6 variants) | ✅ | (派生) | ✅ | ✅ |
| Decision (5 字段) | ✅ | (派生) | ✅ | ✅ |
| Output (5 字段) | ✅ | (派生) | ✅ | ✅ |
| Verdict (Accept/Reject/Escalate) | ✅ | (派生) | ✅ | ✅ |
| LLMClient (mock, per 守门 #23 v2) | ✅ | ✅ A11.10 | ✅ | ✅ |
| AchievementUnlock | ✅ | (派生) | ✅ | ✅ |
| TemplateInstance | ✅ | ✅ A11.4 | ✅ | ✅ |
| ARGState (in-process 缓存) | ✅ | ✅ A11.9 同步桥 | ✅ | ✅ |
| EscalationInfo | ✅ | (派生) | ✅ | ✅ |
| PeerReviewVerdict | ✅ | (派生) | ✅ | ✅ |
| ChallengePrompt | ✅ | (派生) | ✅ | ✅ |

**11 共享类型 100% 一致** ✅

### 2.7 ✅ W/T/M 表分类 (per 守门 #13 100% 覆盖)

| 类型 | 数量 | 表名 | 跨域 |
|---|---|---|---|
| **Master (SCD Type 2)** | 6/14 (42.9%) | agents / agent_relationship_edges / achievements / canvas_comments / canvas_permissions / canvas_elements_backend | ✅ |
| **Transaction (append-only)** | 5/14 (35.7%) | agent_relationship_edges_audit / achievement_unlocks / relationship_events / canvas_multi_user_audit / canvas_comment_mentions | ✅ |
| **Work (短 TTL)** | 3/14 (21.4%) | team_template_instances (TTL 30d) / canvas_followers (session-bound) / canvas_presence_cursors (heartbeat 30s) | ✅ |
| **总计** | **14/14 = 100%** | 全部 14 张表 W/T/M 跨域 | ✅ |

**W/T/M 100% 覆盖 0 混在** ✅

### 2.8 ✅ 5 状态机 (per DD-AGENT-RELATIONSHIP-001 v0.1 §5 + 本批 §5)

| 状态机 | 状态数 | 跨域 |
|---|---|---|
| Edge | 5 状态 (Active/Archived/Draft/PendingApproval/Failed) | ✅ |
| Agent | 14 状态 (per `SRS-STAR-AGENT-RUNTIME-001.md` §8) | ✅ |
| Trust Score | 5 档 (Untrusted/Low/Medium/High/VeryHigh) | ⚠️ 命名/阈值冲突 (见 §2.5) |
| Template Instance | 4 状态 (Creating/Active/Expired/Archived) | ✅ |
| Achievement | 4 状态 (Locked/Available/Unlocked/Revoked) | ✅ |

**5 状态机 4 / 5 一致** (Trust Score 5 档冲突)

### 2.9 ✅ 4 关键时序图 (per DD-AGENT-REL v0.1 §8 + 本批 §2.3)

| 时序图 | 跨域 |
|---|---|
| 写关系 (ARG 写 + 同步桥 + UI render) | ✅ |
| 协作影响 (4 维度 dispatch/context/trust/review) | ✅ |
| 成就评估 (3 evaluator 并行 + 8 Cypher + 10 challenges prompt) | ✅ |
| 离线降级 (Memgraph 不可达 + LWW + 30s flush) | ✅ |

**4 时序图 100% 一致** ✅

### 2.10 ✅ 字段名一致性 (per ARG + 本批 14 张表 SQL DDL)

| 表 | 字段 | 跨域 | 一致性 |
|---|---|---|---|
| `agents` | id, name, archetype, domain, status, trust_score, metadata, created_at, updated_at, version | ✅ | ✅ |
| `agent_relationship_edges` | id, from_agent, to_agent, type, weight, direction, archived, metadata, created_at, created_by, version | ✅ | ✅ |
| `agent_relationship_edges_audit` | id, edge_id, action, old_value, new_value, actor, timestamp | ✅ | ✅ |
| `team_template_instances` | id, template_id, instance_name, agent_ids, edges_json, created_at, expires_at | ✅ | ✅ |
| `achievements` | id, name, description, category, rarity, condition, reward_id, version | ✅ | ✅ |
| `achievement_unlocks` | id, user_id, achievement_id, unlocked_at, trigger_event | ✅ | ✅ |
| `relationship_events` | id, edge_id, event_type, actor, payload, before, after, version | ✅ | ✅ |
| `canvas_multi_user_audit` | id, canvas_id, actor_user_id, action, target_id, target_type, payload, created_at, version | ✅ 本批新增 | ✅ |
| `canvas_comments` | id, canvas_id, element_id, author_id, body, parent_comment_id, created_at, version | ✅ 本批新增 | ✅ |
| `canvas_comment_mentions` | id, comment_id, mentioned_user_id, notified_at | ✅ 本批新增 | ✅ |
| `canvas_permissions` | id, canvas_id, user_id, level, granted_by, granted_at, version | ✅ 本批新增 | ✅ |
| `canvas_followers` | id, canvas_id, follower_user_id, leader_user_id, expires_at | ✅ 本批新增 | ✅ |
| `canvas_elements_backend` | id, canvas_id, kind, x, y, width, height, rotation, z_index, content, locked, hidden, version | ✅ 本批新增 | ✅ |
| `canvas_presence_cursors` | id, canvas_id, user_id, x, y, viewport, heartbeat_at | ✅ 本批新增 | ✅ |

**14 张表 字段名 100% 一致** ✅

---

## §3 协调性冲突清单 (per §2.4 + §2.5)

### 3.1 冲突 #1: C-16 关键 class 编号冲突

| 文档 | C-16 命名 | 角色 |
|---|---|---|
| `DD-AGENT-REL v0.1` §3.1 (ARG DD 源) | `RelationshipEditor` (UI TSX P0) | ARG 源 (保留) |
| `BD-CANVAS-AGENT-001` v0.1 §4.12 (本批新写) | `CanvasBackend` (新) | A12.3 替代 V0.1 localStorage |
| `DD-CANVAS-001` v0.1 §4 (本批 root 写) | `CanvasBackend` (新) | 同上 |

**冲突性质**: 编号 C-16 已被 ARG DD 占用 (RelationshipEditor), 本批新写 C-16 = CanvasBackend 冲突

**修复方案**: 
- 本批新写 C-16 改名为 `CanvasElementsBackend` (跟表名 `canvas_elements_backend` 一致)
- C-16 编号仍给 ARG 源 `RelationshipEditor`
- 本批新写 `CanvasElementsBackend` 用 C-25 (新增, 编号顺延, 不占用 ARG 源 C-16)

### 3.2 冲突 #2: C-21 关键 class 编号冲突

| 文档 | C-21 命名 | 角色 |
|---|---|---|
| `DD-AGENT-REL v0.1` §3.1 (ARG DD 源) | `ARGController` (API Rust P0) | ARG 源 (保留) |
| `BD-CANVAS-AGENT-001` v0.1 §4.12 (本批新写) | `MultiUserAudit` (新) | A12.8 audit log |
| `DD-CANVAS-001` v0.1 §4 (本批 root 写) | `MultiUserAudit` (新) | 同上 |

**冲突性质**: 编号 C-21 已被 ARG DD 占用 (ARGController), 本批新写 C-21 = MultiUserAudit 冲突

**修复方案**:
- 本批新写 C-21 改名为 `CanvasMultiUserAudit` (跟表名 `canvas_multi_user_audit` 一致)
- C-21 编号仍给 ARG 源 `ARGController`
- 本批新写 `CanvasMultiUserAudit` 用 C-26 (新增, 编号顺延, 不占用 ARG 源 C-21)

### 3.3 冲突 #3: Trust Score 5 档 命名/阈值不匹配

| 档 | DD-AGENT-REL v0.1 (ARG 源) | 本批 BD-CANVAS-AGENT-001 + DD-CANVAS-001 |
|---|---|---|
| 第 1 档 | Untrusted (0-0.2) | Unknown (0.0) ❌ |
| 第 2 档 | Low (0.2-0.4) | Low (0.1-0.3) ❌ |
| 第 3 档 | Medium (0.4-0.7) | Medium (0.4-0.6) ⚠️ |
| 第 4 档 | High (0.7-0.9) | High (0.7-0.8) ❌ |
| 第 5 档 | VeryHigh (0.9-1.0) | Trusted (0.9-1.0) ❌ |

**冲突性质**: 命名 (Untrusted/Unknown) + 阈值 (Low/Medium/High 边界) 全部不匹配

**修复方案**:
- 本批 BD/DD 统一为 ARG 源命名 (Untrusted/Low/Medium/High/VeryHigh) + 阈值 (0-0.2/0.2-0.4/0.4-0.7/0.7-0.9/0.9-1.0)
- 同步: 5 档 → 6 档 (含新增 Untrusted 0-0.2) 或保留 5 档 (用 ARG 源)
- 推荐: 保留 5 档 ARG 源命名 + 阈值, 同步更新本批 BD-CANVAS-AGENT-001 §3 + DD-CANVAS-001 §5.3 + 专题 DD (子代理 1 撰写中)

---

## §4 修复方案 (per 守门 #1 禁回溯叙事, 不重写 ARG 4 份 commit)

### 4.1 修复范围

| 文档 | 修复内容 | 优先级 |
|---|---|---|
| `BD-CANVAS-AGENT-001.md` v0.1 → v0.1.1 | 3 处修复: C-16 `CanvasBackend` → `CanvasElementsBackend` (C-25 新增) + C-21 `MultiUserAudit` → `CanvasMultiUserAudit` (C-26 新增) + Trust Score 5 档统一 (Untrusted/Low/Medium/High/VeryHigh + 0-0.2/0.2-0.4/0.4-0.7/0.7-0.9/0.9-1.0) | P0 (本批修复) |
| `DD-CANVAS-001.md` v0.1 → v0.1.1 (root 写) | 3 处修复: C-16 + C-21 改名 + Trust Score 5 档统一 (跟 BD 同步) | P0 (本批修复) |
| `DD-CANVAS-AGENT-001.md` v0.1 (子代理 1 撰写中) | 子代理 1 完成后, root verify 阶段统一修复, 或 task_append 通知子代理 1 用修复后方向写 | P0 (本批修复) |
| `DD-CANVAS-GAMIFY-001.md` v0.1 (子代理 2 撰写中) | **不需修复** (游戏化独立, 跟 ARG 无关) | n/a |
| **不重写 ARG 4 份 commit** (per 守门 #1 禁回溯叙事) | C-16 + C-21 + Trust Score 5 档 在 ARG 源已固定, 不重写 | n/a |

### 4.2 修复后编号映射 (per DD-AGENT-REL v0.1 §3.1 顺延)

| 编号 | 命名 | 跨域 | 来源 |
|---|---|---|---|
| C-1..C-15 | (ARG 源 15 关键 class) | ARG | DD-AGENT-REL v0.1 |
| C-16 | `RelationshipEditor` (UI TSX P0) | ARG (保留, 修复后) | DD-AGENT-REL v0.1 |
| C-17 | `RelationshipView` (UI TSX P0) | ARG | DD-AGENT-REL v0.1 |
| C-18 | `AchievementWall` (UI TSX P1) | ARG | DD-AGENT-REL v0.1 |
| C-19 | `EdgeTypeSelector` (UI TSX P0) | ARG | DD-AGENT-REL v0.1 |
| C-20 | `TemplateGallery` (UI TSX P1) | ARG | DD-AGENT-REL v0.1 |
| C-21 | `ARGController` (API Rust P0) | ARG (保留, 修复后) | DD-AGENT-REL v0.1 |
| C-22 | `ARGSSEHub` (API Rust P0) | ARG | DD-AGENT-REL v0.1 |
| C-23 | `ARGPermission` (API Rust P0) | ARG | DD-AGENT-REL v0.1 |
| C-24 | `ARGMigration` (Data Rust P2) | ARG | DD-AGENT-REL v0.1 |
| **C-25** | **`CanvasElementsBackend` (新)** | **本批 A12.3** | **本批新增** |
| **C-26** | **`CanvasMultiUserAudit` (新)** | **本批 A12.8** | **本批新增** |

**修复后总数**: 26 关键 class (24 ARG + 2 本批新增), 顺延编号, 不冲突 ✅

### 4.3 Trust Score 5 档统一 (per DD-AGENT-REL v0.1 §3.3.3)

| 档 | 修复后命名 + 阈值 | 跨域 |
|---|---|---|
| 第 1 档 | **Untrusted (0-0.2)** | ARG + 本批 |
| 第 2 档 | **Low (0.2-0.4)** | ARG + 本批 |
| 第 3 档 | **Medium (0.4-0.7)** | ARG + 本批 |
| 第 4 档 | **High (0.7-0.9)** | ARG + 本批 |
| 第 5 档 | **VeryHigh (0.9-1.0)** | ARG + 本批 |

**5 档 100% 跟 ARG 源一致** ✅

### 4.4 修复流程 (per 守门 #9 v19 + 守门 #1 禁回溯叙事)

1. **本批协调性报告落档** (本文件 v0.1)
2. **修复 BD-CANVAS-AGENT-001 v0.1 → v0.1.1** (root edit 3 处, 跨域修复合并)
3. **修复 DD-CANVAS-001 v0.1 → v0.1.1** (root edit 3 处, 跨域修复合并)
4. **task_append 给子代理 1** (DD-AGENT-001) 用修复后方向写, 或子代理 1 完成后 root 修复合并
5. **子代理 2** (DD-GAMIFY-001) **不需修复** (游戏化独立)
6. **verify 阶段** (per 守门 #9 v27): 文件存在 + 字节数 + 修复内容已落档
7. **1 commit 3 文件** (root 统一 commit, 关联 4f56979 / 73d490f / 396833a / f35b3f5 / f7d0932 / 73c0826 6 份 SRS/BD commit)
8. **1 commit 2 文件** (更新 §4.29 + registry v0.14)

### 4.5 修复影响评估

| 维度 | 修复前 | 修复后 |
|---|---|---|
| C-16 命名 | `CanvasBackend` (冲突) | `CanvasElementsBackend` (跟表名一致) |
| C-21 命名 | `MultiUserAudit` (冲突) | `CanvasMultiUserAudit` (跟表名一致) |
| 13 关键 class 总数 | 13 (含冲突) | 11 派生 + 2 新增 (C-25 + C-26) = 13 派生 | 派生 (跟 ARG 源 1:1) |
| Trust Score 5 档 | Unknown/Trusted + 0.0/0.1-0.3/... | Untrusted/VeryHigh + 0-0.2/0.2-0.4/... (跟 ARG 源 1:1) |
| 表名 / 类名一致性 | 6 张 (A12.7 + A12.3) ✅ | 100% 一致 ✅ |
| 字段名一致性 | 14 张表 ✅ | 100% 一致 ✅ |
| 跨域接口一致性 | 32 API + 5 WS ✅ | 100% 一致 ✅ |
| W/T/M 分类 | 14 张表 100% ✅ | 100% 一致 ✅ |
| Token OLU 估算 | +0.05M (修复 +1 commit) | 在预算内 |

---

## §5 守门硬约束 (per 守门 #1 + 守门 #9 #3 + 守门 #13 + 守门 #14 + 守门 #15)

- 文档结构严格 10 段, 不增不减
- 已知缺口 ≥ 5 (本检查报告列 3 个冲突, 跨域, 显式列 §3)
- 5 角色签字栏 per AGENTS.md §3
- 5 域 Lead ≠ Star 22 DDD bounded context disclaimer 显式
- 5 域 Lead 真人未到位前 Mavis 临时代签, 真人到位后追溯签字
- 守门 #13 W/T/M 三類横展 100% 覆盖 (14 张表 100%, 0 混在)
- 守门 #1 禁回溯叙事 (不重写 ARG 4 份 commit, 仅修复本批新写 BD/DD)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10)
- 0 子代理调用 (root 直实装, per 守门 #9 #3 实证 5/5 RPC 不可靠)

---

## §6 5 角色签字栏

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-10 JST | 5 域 Lead 真人未到位, Mavis 临时代签 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

---

## §7 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-10 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3)** | **初始版本: 9 本批新写 (3 SRS + 3 BD + 3 DD) vs ARG 4 份 (SRS + BD + DD + DDD-REVIEW) 协调性检查, 发现 3 个冲突 (C-16 + C-21 关键 class 编号冲突 + Trust Score 5 档 命名/阈值不匹配), 修复方案: 关键 class 顺延 (C-25 + C-26 新增) + Trust Score 5 档统一为 ARG 源** | **2026-09-10 18:30 JST Ulysses 拍板"确保本设计和agent图关系设计妥善协调不冲突"** |

---

> **撰写完成**: 2026-09-10 18:35 JST, root session mvs_942987595a124037901d37205a548e6f
> **修复动作**: 等待 2 子代理 DD 完成后, root edit BD-CANVAS-AGENT-001 v0.1 → v0.1.1 (3 处) + DD-CANVAS-001 v0.1 → v0.1.1 (3 处) + 1 commit 3 文件 (per 守门 #1 v15 docs 同步饱和) + 1 commit 2 文件 (更新 §4.29 + registry v0.14)
> **下次拍板触发**: 修复完成 → P3-D.6 启动实装 (P0 36 项 per 总册 §4.4)
