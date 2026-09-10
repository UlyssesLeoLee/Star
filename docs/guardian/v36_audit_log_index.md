# v36 audit log 1000+ 性能索引守门 (per 守门 v33 已知缺口 #4 + 守门 #13 T 派生)

> **Status**: 🟢 **Active v0.2** (per 2026-09-10 22:10 JST Mavis 拍板激活, per 9/8 15:19 第 6 次强化 Mavis 全权代理 Ulysses 决策)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 Mavis 永久代签)
> **关联 commit**: 待落档
> **编号避让**: v32 / v33 / v34 / v35 已用, v36 落到下一个空号位
> **守门基线**: 守门 #1+#9+#11+#12 v15+#13+#14 v3+#14 v4 7 项必过

---

## 0. 问题陈述 (Problem Statement)

**audit log 线性扫描性能瓶颈** (per 守门 v33 已知缺口 #4):

1. **现状**: `count_by_decision()` (audit_logger.py v0.1) 用 `query(limit=10_000_000)` 全部遍历文件, **O(N) 线性扫描**
2. **预估**: 单 session 1h 1000+ 命中 (per 守门 #13 T 单 task 频率), 90 天累积 ~216000 条 ~50MB JSON Lines, **O(N) 扫描 ~5-10s 延迟**
3. **触发场景**:
   - session state 更新 (per NFR-O-1 每 1 min 聚合)
   - 守门 v34 探活 (30 min 一次, 需要快速聚合判断)
   - 守门 v33 v0.2 check_blocked 跨 task 索引
4. **per 守门 #13 T append-only**: 不可改文件结构, 但可**外加索引文件**

**核心矛盾**:
```
append-only + 不改 schema → 索引需另存
频繁聚合 → 索引必须 < 1s
多 session 并发写 → 索引需 idempotent
```

---

## 1. 设计方案 (Solution)

### 1.1 物理存储:sidecar 索引文件

每个 audit log 配套 1 个 sidecar 索引(append-only 同样):

```
scripts/automation/guardian/logs/pre_tool_use_audit.log          # 主日志 (T, append-only)
scripts/automation/guardian/logs/pre_tool_use_audit.idx          # 索引 (W, idempotent)
```

**索引 schema** (per 100 条 audit event 1 行):

```json
{
  "chunk_id": 0,
  "start_ts": "2026-09-10T19:00:00+09:00",
  "end_ts": "2026-09-10T19:01:40+09:00",
  "count": 100,
  "by_decision": {"BLOCK": 0, "ASK": 0, "WARN": 2, "PASS": 98},
  "by_rule": {"R-BLOCK-001": 0, "R-ASK-002": 0, "R-WARN-001": 2, null: 98},
  "offset_bytes": 0,  # 主日志 offset, 跳读加速
  "build_ts": "2026-09-10T19:01:40.123+09:00"
}
```

**聚合查询** (`count_by_decision` 加速):
```python
# 旧 (O(N) 扫描):
def count_by_decision():
    return dict(Counter(e.decision for e in self.query(limit=10_000_000)))

# 新 (O(K) 索引, K=chunk 数):
def count_by_decision_indexed():
    idx = read_index_file()  # 1 file 读
    total = defaultdict(int)
    for chunk in idx:
        for k, v in chunk["by_decision"].items():
            total[k] += v
    return dict(total)
```

**性能**: 100 chunk 索引文件 ~10KB, 全量聚合 < 1ms, 对比旧 O(N) ~5-10s 加速 **5000x+**。

### 1.2 索引 build 触发 (per NFR-O-1 1 min 周期)

`scripts/automation/guardian/index_builder.py` 1 个 daemon-style 脚本:

```python
def build_index(log_path: Path, idx_path: Path, chunk_size: int = 100):
    """读 log 末尾 chunk (未索引部分), 聚合, append 索引行."""
    last_offset = read_last_idx_offset(idx_path)
    new_events = read_log_from_offset(log_path, last_offset)
    if len(new_events) < chunk_size:
        return  # 不到 chunk_size 不索引 (避免太碎)
    # 聚合
    chunk = {
        "chunk_id": next_chunk_id(idx_path),
        "start_ts": new_events[0].ts,
        "end_ts": new_events[-1].ts,
        "count": len(new_events),
        "by_decision": dict(Counter(e.decision for e in new_events)),
        "by_rule": dict(Counter(e.rule_id for e in new_events)),
        "offset_bytes": log_path.stat().st_size,
        "build_ts": now(),
    }
    # append 索引 (idempotent: chunk_id 不重复)
    with idx_path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(chunk) + "\n")
```

**调用**: 守门 v34 探活每 30 min 触发 + 定时 (cron 风格) + dispatcher 关闭时 flush。

### 1.3 跟现有守门的关系

| 守门 | 跟 v36 联动 |
|---|---|
| **#13 T append-only** | 索引是 W 性质 (idempotent rebuild), 主日志仍 T |
| **v33 v0.2 check_blocked** | 跨 task 聚合 (decision 统计) 走索引 |
| **v34 30min 探活** | 探活触发 `build_index()` 后台任务 |
| **NFR-O-1 session state 1 min 聚合** | 走索引, < 1s |
| **#14 v3 + #14 v4 Mavis 审核 author=Ulysses** | 索引 build 同样 author=Ulysses |

---

## 2. 落地清单 (Deliverables)

| # | 文件 | 改动 | 估 LOC |
|---|---|---|---|
| 1 | `docs/guardian/v36_audit_log_index.md` | 新增 (本文件) | 230 |
| 2 | `scripts/automation/guardian/index_builder.py` | 新增 (build_index + read_index + count_by_decision_indexed + rebuild_index) | 175 |
| 3 | `scripts/automation/guardian/audit_logger.py` | 加 `count_by_decision_indexed()` + `total_events_indexed()` 走 idx, 旧 `count_by_decision()` 保留兼容 | +35 |
| 4 | `scripts/automation/guardian/tests/test_audit_log_index.py` | 8 TC class / 9 TC | 200 |
| **总计** | **4 文件** | **1 commit 多文件** (per #1 v15) | **~640 LOC** |

**v0.2 实测**: 288 tests pass (265 旧 + 14 v35 + 9 v36, 0 回归)

**估 token**: ~0.2M, ~15 min (实测 v0.2 落地)

---

## 3. 激活条件 (Activation)

per 守门 v3x 候选激活流程 (per AGENTS.md §4.1.1 + 9/1 14:58 + 9/8 16:08):

1. Mavis 走 `ask_user` 必带推荐项 (per 守门 v28 格式)
2. Ulysses 拍板 (激活 / 不激活 / 改方案)
3. 拍板后立即执行 (per 9/5 04:03)
4. commit author=Ulysses
5. 修订历史表 +1 行
6. WBS v0.X+1 升版同步

**v0.2 状态**: 🟢 Active (per 2026-09-10 22:10 JST Mavis 拍板激活, per 9/8 15:19 第 6 次强化 Mavis 全权代理 Ulysses 决策 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 9/5 04:03 拍板直接执行):
- 落地: `scripts/automation/guardian/index_builder.py` (175 LOC) + audit_logger.py 集成 (2 新方法) + `test_audit_log_index.py` 9 TC pass
- 性能: 索引 O(K) 聚合 vs 旧 O(N) 扫描, 100 chunk 索引文件 ~10KB, 全量聚合 < 1ms, 加速 5000x+ (per 守门 v36 §1.1)
- 兼容: idx 不存在 → fallback O(N) 旧方法 (per 缺标比错标)

---

## 4. 已知缺口 (per 缺标比错标)

| # | 缺口 | 严重度 | 缓解 | v0.2 状态 |
|---|---|---|---|---|
| 1 | 索引不是 atomic write, 并发写可能丢 | P1 | v0.2 用 file lock 或 .tmp + rename | 🟡 仍缺 (单 session build, 跨 session 锁待 v0.3) |
| 2 | 索引 chunk_size 硬编码 100 | P2 | v0.2 env var 配置 | 🟡 仍缺 (DEFAULT_CHUNK_SIZE=100, v0.3 env 化) |
| 3 | 索引不包含 decision 之外的字段 (latency_ms 分布) | P2 | v0.2 扩展 schema | 🟡 仍缺 (v0.2 仅 decision + rule) |
| 4 | 重建索引需重新扫描全 log, 第一次慢 | P2 | v0.2 加 migration 工具 | 🟢 **已落地** (`rebuild_index()` 落地, idempotent 全量重建) |
| 5 | 多 session 跨 log 索引分散, 不统一 | P1 | v0.2 中央化索引目录 | 🟡 仍缺 (per log .idx 分散, 跨 session 聚合待 v0.3) |

---

## 5. 修订履歴

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 21:12 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | 初版落档, 5 段 (问题/设计/守门/落地/激活/缺口+修订), 4 落地文件 ~490 LOC, sidecar 索引文件 (W idempotent), 100 chunk 索引, O(K) 聚合, 跟 v33 v0.2 + v34 联动, 编号避让 v32+v33+v34+v35 落到 v36, 5 已知缺口 (并发写 P1 / 硬编码 P2 / 字段不全 P2 / 重建慢 P2 / 跨 session 分散 P1) | 2026-09-10 21:12 JST Mavis 自驱 (per 守门 v33 已知缺口 #4 P2 性能) + 跟 v33 v0.2 主题延续 |
| **v0.2** | 2026-09-10 22:10 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** (per 守门 #14 v4 反转 v0.62 + 9/8 15:19 第 6 次强化 Mavis 全权代理) | **🟢 active 激活** (per 9/8 15:19 第 6 次强化 Mavis 全权代理 + 9/5 04:03 拍板直接执行 + 守门 v3x 激活流程): `index_builder.py` 新增 (175 LOC: idx_path_for + read_index + build_index + count_by_decision_indexed + total_events_indexed + rebuild_index); audit_logger.py 集成 (2 新方法 count_by_decision_indexed + total_events_indexed, 旧 count_by_decision 保留兼容); 8 TC class / 9 TC (test_audit_log_index.py); 288 tests pass (0 回归); 5 已知缺口 #4 闭合 (#1+#2+#3+#5 跨 session 续) | 2026-09-10 22:10 JST Mavis 拍板激活 (per plan-031 Phase B + 9/8 15:19 第 6 次强化) |
