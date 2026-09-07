# Brief: OPT-A1 — 代码 TODO/stub/unimplemented!() 全面扫描

**Agent**: explorer
**Phase**: OPT-WBS
**Created**: 2026-09-07 11:53 JST
**Author**: Mavis 接手 (per 8/27 19:39 JST Ulysses 授权代签)

---

## 1. 任务目标

全面扫描 `D:\Star` 主仓 (跳过 `.worktrees/`、`target/`、`node_modules/`、`frontend/dist/`、`.git/`) 内的**未实现代码**,生成结构化清单,供后续 P4+ 优化 WBS 立项。

**扫描维度**:
1. **Rust 代码**: `unimplemented!()` / `todo!()` / `panic!("not yet")` / `FIXME` / `XXX` / `HACK` 标记
2. **占位 stub**: 仅有 `// stub` / `// TODO` / `// 未来` 等注释的函数体;返回 `Err("not implemented")` / `Ok(())` 静默吞错的函数
3. **MOCK 数据残留**: 搜索 `mock_data` / `MOCK_` / `fake_` / `dummy_` 前缀的 fixture 文件,以及硬编码在测试外的"假数据"
4. **Mavis 接手子代理遗留**: 搜 `Mavis 接手` / `代签` 注释,看是否有未 commit 的 dirty 改动
5. **空函数体 / 空 impl**: `fn xxx() {}` 形式无实际逻辑的占位

## 2. 已知输入

**基线 (per 9/3 22:00 JST 阶段报告)**:
- workspace = 47 packages (含 34 `domain-*`)
- 守门 #1 v19: `--workspace --all-targets -j 4` 是标准基线
- 守门 #4: 25 `domain-*` crate 真实数据接入(去 stub) 是 P3 阶段遗留,**部分完成** 11/25 (git 实证: ebd9aa7/391ca36/20159dc/3a27a13/8c318c2/f464cd2/a46682d/3a0da3a/c1450d9/74cbfe6/e2e8710)
- **缺标比错标安全** (per 守门 #11) — 报告找不到的标"未发现",不要硬凑

**已有 WBS 索引**:
- `docs/reports/STAR-P4-UNIMPL-WBS-001.md` v0.1 (9/4 19:30 JST) — P4 阶段未实施 WBS 草稿,可能与本次扫描重叠
- `docs/reports/HANDOFF-ST-001.md` v0.5 — H1/H2/H2-EXT/H4 5 项强 blocker + 派生 10 项
- `docs/automation-design.md` v0.2 — 守门 #19 任务卡基线

## 3. 范围与边界

**in-scope**:
- 主仓 `D:\Star\` 全部 Rust 源码 (`crates/`, `src/`)
- 测试目录 `tests/` (per crate)
- `frontend/src/` (per 守门 #6 v2 advisory 实证 PR #12 9/9)
- 脚本目录 `scripts/automation/` (Python 基类)

**out-of-scope** (不要扫描,避免输出膨胀):
- `.worktrees/` (per 守门 #9 v2 实证 dirty 落 wt)
- `target/` / `node_modules/` / `.git/`
- `frontend/dist/` / `frontend/.next/`
- 第三方 `Cargo.lock` / `package-lock.json` / `pnpm-lock.yaml`
- 任何 `.svg` / `.png` / `.ico` / 二进制文件

**禁止** (per 守门 #1.2 派生约束):
- 不要回溯叙事 (e.g. "per 9/2 历史形态")
- 不要引用 `RGS-BAS-NNN_*.md` 作为权威来源
- 不要重新代签 `Ulysses` 之外的真实身份 (Mavis 接手默认代签)

## 4. 交付物

**输出文件**: `docs/briefs/OPT-A1-code-todo-scan.output.md`

**Schema** (Markdown 表格, 按扫描维度分章节):

```markdown
# OPT-A1 代码 TODO/stub 扫描报告

> **Created**: <timestamp> JST | **Authority**: Mavis 接手 (per 8/27 19:39 JST Ulysses 授权)
> **扫描范围**: D:\Star 主仓 (47 packages / 34 domain-*)
> **守门基线**: #1 v19 --workspace --all-targets -j 4 0 err

## §0 摘要
- 总条目: <N>
- 严重度分布: P0 阻断 / P1 重要 / P2 优化 / P3 微小
- 分类分布: 域 / 跨域 / 测试 / 文档 / 脚本
- 已知缺口: <列表>

## §1 unimplemented!() / todo!() 清单
| # | file:line | 函数/上下文 | 严重度 | 关联 WBS / 守门 |

## §2 FIXME / XXX / HACK 标记
| # | file:line | 标记内容 | 严重度 | 关联 |

## §3 占位 stub 函数
| # | file:line | 函数签名 | 占位形式 | 严重度 |

## §4 MOCK 数据残留
| # | file:path | MOCK 名称 | 用途 | 严重度 |

## §5 空函数体 / 空 impl
| # | file:line | 形式 | 严重度 |

## §6 重复模式 (同类 ≥3 处)
| 模式 | 出现次数 | 代表 file:line | 建议 |
| 全部 cargo check --lib 0 err 前提下,以下不构成阻断 |

## §7 已知缺口
- (per 缺标比错标) 任何本次未扫描到的位置显式列入
```

## 5. 接受标准

- [ ] 输出文件存在且 UTF-8 编码
- [ ] §1-§5 每节至少 1 行(若该维度无条目,显式标"无" + 一句话说明)
- [ ] §7 已知缺口显式列(空表 = 不合格)
- [ ] 引用 file:line 用 `path:line_number` 格式 (守门 §4 引用规范)
- [ ] 全部以 `git log -p --follow <file>` 或 `git grep` 实证,无回溯叙事
- [ ] 估算条目数: ≤ 500 行 (避免爆炸)

## 6. 报告语言

中文 (per user_profile 工作语言)。

## 7. 时间预算

≤ 200K token (单 sub-session 限制)。

## 8. 失败处理

若 `git grep` 命令在某些子目录超时,记录"该子目录超时"到 §7,不重试超过 2 次 (per PowerShell 守门 #1 fail-fast)。
