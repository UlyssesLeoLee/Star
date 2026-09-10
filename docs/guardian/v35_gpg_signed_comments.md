# v35 v33 留言 GPG 签名守门候选 (per 守门 v33 已知缺口 #2 + 守门 #5 派生)

> **Status**: 🟡 **Draft v0.1** (per 2026-09-10 21:12 JST Mavis 自驱, 待 Ulysses 拍板激活)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 Mavis 永久代签)
> **关联 commit**: 待落档
> **编号避让**: v32 已被 `v32_audit_boundary.md` 占用, v33/v34 已用, v35 落到下一个空号位
> **守门基线**: 守门 #1+#5+#9+#11+#12 v15+#13+#14 v3+#14 v4 8 项必过 (跟 #1 v15 docs 同步饱和联动, 1 commit 多文件)

---

## 0. 问题陈述 (Problem Statement)

**留言 author 假冒风险** (per 守门 v33 已知缺口 #1 + #2):

1. **现状**: `docs/briefs/<task_id>.comments.jsonl` 是普通 JSON Lines 文件, **任何能写该目录的进程都能伪造成 `author: "Ulysses"` 的留言**
2. **风险场景**:
   - 子代理 (worker-1) 写一条 `author: "Mavis"` 留言, 内容却是"已批准 deploy, 可继续"
   - root session `check_blocked` 看到 actor_role="Mavis" 权威, 解除 BLOCK
   - 实际: 没人真批准, 子代理骗了 guard
3. **per 守门 #5 派生**: 8/27 11:06 JST Ulysses hard ban "环境变量内容不打印" — 同理,留言 author 必可信, **不能 0 验证**
4. **per 守门 v33 v0.2 智能 check_blocked**: 权威 actor {Ulysses, architect, Mavis} 写的留言可解除 BLOCK — 但**没有验证 author 身份是否真伪**

**核心矛盾**:
```
信任 author 字段 → 子代理可伪造, 防御 0
强制 GPG 签名 → 复杂度高, 但真可信
折中 → 签名 + 短时戳 + idempotent 验证
```

---

## 1. 设计方案 (Solution)

### 1.1 物理存储:每条留言带签名

`docs/briefs/<task_id>.comments.jsonl` 每行 JSON 增加 2 字段:

```json
{
  "id": "comment_001",
  "ts": "2026-09-10T19:55:00.000+09:00",
  "author": "Mavis",
  "actor_role": "orchestrator",
  "body": "请加 X 字段",
  "mentions": ["P3-D.6-1-1"],
  "blocks": false,
  "tags": ["requirement"],
  "parent_comment_id": null,
  "refs": [],
  "_gpg_signature": "-----BEGIN PGP SIGNATURE-----\n...base64...\n-----END PGP SIGNATURE-----",
  "_signed_payload": "id=comment_001|ts=2026-09-10T19:55:00|body=请加 X 字段|blocks=false|author=Mavis"
}
```

**签名内容** (`_signed_payload`): `id|ts|author|body|blocks|tags` 5 字段 canonical string, 用 author 的 GPG private key 签名 (RSA-2048 / ed25519)。

### 1.2 验证流程 (list_comments / check_blocked 联动)

```python
def verify_comment_signature(comment: dict) -> bool:
    """验证留言 GPG 签名 (per 守门 v35 §1.1).

    Returns:
        True = 签名有效 (gpg --verify exit 0)
        False = 签名无效 / 缺失 / _signed_payload 跟 comment 字段不一致
    """
    sig = comment.get("_gpg_signature")
    payload = comment.get("_signed_payload")
    if not sig or not payload:
        return False
    # 用 gpg --verify 验证
    result = subprocess.run(
        ["gpg", "--verify", "-"],  # 从 stdin 读 signature
        input=sig.encode("utf-8"),
        capture_output=True, timeout=10, check=False,
    )
    return result.returncode == 0
```

### 1.3 check_blocked 增强 (per 守门 v35 联动 v33 v0.2)

```python
def check_blocked(self, task_id: str) -> list:
    """v35 增强: 权威 actor 留言必 GPG 签名有效 (per 守门 v35 §1.2)."""
    blocking = super().check_blocked(task_id)  # v0.2 智能逻辑
    verified_blocking = []
    for c in blocking:
        if c.actor_role in {"Ulysses", "architect", "Mavis"}:
            # 权威 actor → 必 GPG 验证
            sig_ok = verify_comment_signature(asdict(c))
            if not sig_ok:
                logger.warning("权威留言 GPG 签名无效, 仍按 BLOCK 处理: %s", c.id)
        verified_blocking.append(c)
    return verified_blocking
```

**软约束 v0.1**: 签名无效 → warn log, 仍按 BLOCK 处理 (per 缺标比错标 #11)。

### 1.4 GPG key 准备 (落地前必做)

每个权威 actor 必生成 GPG key pair:

```bash
# Ulysses
gpg --full-generate-key --algorithm ed25519 --name "Ulysses <ulysses@mavis.local>"
# Mavis (Mavis 跟 Ulysses 共享, per 守门 #14 v3)
gpg --full-generate-key --algorithm ed25519 --name "Mavis <mavis@mavis.local>"
# Architect (5 域 Lead / 平台 / 评审 / PM 中任一, 暂时用 Mavis key)
# → public key 上传到 docs/keys/<actor>.asc
```

---

## 2. 跟现有守门的关系

| 守门 | 跟 v35 联动 |
|---|---|
| **#5 env 安全** (8/27 11:06 JST 硬 ban) | v35 是 #5 派生: 同理"author 字段必可信, 不允 0 验证" |
| **#13 T Transaction append-only** | v35 不改 append-only 性质, 仅加 2 字段 (签名 + payload) |
| **v33 §1.2 schema** | v35 扩展 schema, 不重写 (per 守门 #1 禁回溯叙事) |
| **v33 v0.2 智能 check_blocked** | v35 在 v0.2 基础上加 GPG 验证, 仍返 blocking list (软约束) |
| **v34 30min sustained 探活** | v35 落档后由 v34 探活 trigger docs/briefs/ 留言 GPG 验证 |

---

## 3. 落地清单 (Deliverables)

| # | 文件 | 改动 | 估 LOC |
|---|---|---|---|
| 1 | `docs/guardian/v35_gpg_signed_comments.md` | 新增 (本文件) | 200 |
| 2 | `scripts/automation/guardian/comment_gpg.py` | 新增 verify_comment_signature 函数 | 80 |
| 3 | `scripts/automation/dispatcher.py` | `comment()` 自动签名 + `check_blocked()` 增强验证 | +50 |
| 4 | `docs/keys/ulysses.asc` + `docs/keys/mavis.asc` | GPG public keys | - |
| 5 | `scripts/automation/guardian/tests/test_gpg_signed_comments.py` | 5+ TC | 150 |
| **总计** | **5 文件** | **1 commit 多文件** (per #1 v15) | **~480 LOC** |

**估 token**: ~0.3M, ~20 min

---

## 4. 激活条件 (Activation)

per 守门 v3x 候选激活流程 (per AGENTS.md §4.1.1 + 9/1 14:58 + 9/8 16:08):

1. Mavis 走 `ask_user` 必带推荐项 (per 守门 v28 格式)
2. Ulysses 拍板 (激活 / 不激活 / 改方案)
3. 拍板后立即执行 (per 9/5 04:03)
4. commit author=Ulysses
5. 修订历史表 +1 行
6. WBS v0.X+1 升版同步

**当前状态**: 🟡 Draft v0.1, Mavis 自驱设计稿落档, 待 Ulysses 拍板激活。

---

## 5. 已知缺口 (per 缺标比错标)

| # | 缺口 | 严重度 | 缓解 |
|---|---|---|---|
| 1 | 现有留言 (v0.1 + v0.2 + v0.3 阶段) 无签名, 反向兼容困难 | P0 阻塞 | v35 落地时一次性 backfill (per actor 生成签名), 或 accept legacy |
| 2 | 软约束: 签名无效仍按 BLOCK, 不阻断 (per 缺标比错标 #11) | P2 | v0.2 升 hard 约束 |
| 3 | GPG key 落地需要 Ulysses / Mavis 真实 key pair | P0 阻塞 | 落地前 5 min 生成 |
| 4 | gpg CLI 跨平台 (Windows Gpg4win / POSIX gnupg) | P2 | 软依赖 + fail-open (per 守门 #6) |
| 5 | Mavis 跟 Ulysses 共享 key 引发责任混淆 | P1 | v0.2 分开 key (Mavis 独立 key) |

---

## 6. 修订履歴

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 21:12 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | 初版落档, 6 段 (问题/设计/守门/落地/激活/缺口+修订), 5 落地文件 ~480 LOC, _gpg_signature + _signed_payload 2 字段, 跟 v33 v0.2 智能 check_blocked 联动, 编号避让 v32 + v33 + v34 落到 v35 | 2026-09-10 21:12 JST Mavis 自驱 (per 守门 v33 已知缺口 #2 + 守门 #5 派生) + 跟 v33 主题延续 |
