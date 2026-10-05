# arch-live · Star 架构动态图谱

一套**纯静态 HTML 动态文档**，用来演示"Star 这个程序现在是怎么被构造出来的"。
图文并茂、带动画、可交互，全部内容对着仓库真实代码写成。

## 怎么打开

直接双击 `index.html` 即可（`file://` 协议可用，零外部依赖、无需起服务、不联网）。

```
docs/arch-live/index.html
```

## 分册索引

| 页面 | 内容 | 回答什么问题 |
|---|---|---|
| [index.html](index.html) | 规模基线 + 六层系统全景 + 7 步构造链 | 这个系统整体长什么样？ |
| [01-crate-map.html](01-crate-map.html) | 105 个 crate 分 8 家族，可搜索/过滤 + 实装度 Top-15 | 我这段逻辑该写进哪个 crate？ |
| [02-navigation-model.html](02-navigation-model.html) | Project → Branch → Run → Worktree 四级模型 + 57 个前端路由 + 50 条 REST 路由 | 一个页面/接口该挂在哪一层？ |
| [03-request-lifecycle.html](03-request-lifecycle.html) | 13 步请求生命周期（可自动播放）+ 错误码语义 + 真实 SQL | 一次写操作在运行时怎么跑？ |
| [04-agent-runtime.html](04-agent-runtime.html) | Agent 执行闭环 10 步 + Task 状态机 + Hook 判定器 + 资源预算 | Agent 到底被什么约束住？ |
| [05-worktree-canvas.html](05-worktree-canvas.html) | worktree-canvas 14 crate + 3 WASM 桥 + ARG 三层 + Canvas 四进程 | 图谱 / Canvas / Agent 关系各由谁实现？ |
| [06-data-model.html](06-data-model.html) | 6 schema 150 表 + W/T/M 三类横展 + SCD2 + 幂等表族 + 525 RLS | 这张表为什么长这样？ |
| [07-deploy-topology.html](07-deploy-topology.html) | k3s 拓扑 + envoy 5 前缀路由 + 端口全表 + 拉起顺序 | 服务跑在哪、流量怎么走？ |
| [08-guardrails.html](08-guardrails.html) | 编译器 lint + CI 8 job + AGENTS 守门 + 门禁自身的三个陷阱 | 哪些规则会真的拦住我？ |

## 三条设计约定

1. **只写代码里真实存在的东西。** 每个数字都带 `data-fact` 声明，可被脚本核对到仓库实测值；
   凡是"已实装 vs 仅骨架"有差别的地方，页面上用 `no-op stub` 之类的标签显式标出，并给出证据文件路径。
2. **证据可追溯。** 每页底部都有 `证据` 折叠块，列到 `file:line` 级别。看到可疑描述可以立刻回源码核对。
3. **动画是装饰，不是门禁。** 入场动画失败、内容默认可见；`prefers-reduced-motion` 下 SMIL 动画会被移除。

> 第 3 条不是口号，它由两条门禁实测守着：
> `data-count` 数字的**初始文本就写成真实值**（`--sync-seed` 强制），所以禁用 JS 时数字墙照常显示 105 / 741 / 251311，
> 而不是一片 `0`；`html.anim` 这个透明度门控类**只由 JS 添加**，无 JS 时 `.rv` 一律不透明。

## 配套脚本

三个脚本都在 `scripts/automation/`（按仓库守门 #19，agent 与外部交互必须走脚本）：

```powershell
# 1) 数字漂移 + 资产静态门禁：重扫仓库比对 data-fact / 种子值, 并 node --check arch.js
python scripts/automation/arch_live_doc_check.py
python scripts/automation/arch_live_doc_check.py --self-test    # 变异 + 对照 + 全零反例
python scripts/automation/arch_live_doc_check.py --sync-seed    # 把数字初始文本同步为真实值
python scripts/automation/arch_live_doc_check.py --rebase-baseline  # 把基线 commit 串更新到当前 HEAD

# 2) 渲染验证：用仓库自带 Playwright 为每页生成整页 PNG 到 .preview/
python scripts/automation/arch_live_shot.py
python scripts/automation/arch_live_shot.py --pages index.html

# 3) 交互自检：真点按钮 + 断言 DOM，含无 JS / reduced-motion 降级路径
python scripts/automation/arch_live_interact_check.py
python scripts/automation/arch_live_interact_check.py --mutate   # 变异测试：注入已知 bug，门禁必须变红
```

> 这三条纪律已升为**仓库级守门 `AGENTS.md` §4.1 派生规 v30**（2026-10-05，Ulysses 拍板），
> 落地脚本 `scripts/automation/agents_guard_v30.py`：① 门禁落库前必做变异测试（含对照组）；
> ② 退出码区分「确实有问题」与「没测到」；③ 只比较最终状态的断言必须配「机器真的启动过」的独立证据。
> 本套文档自己的三个脚本就是按 v30 写的，`--self-test` / `--mutate` 不是可选项。

### 退出码语义（三份脚本统一）

| 码 | 含义 |
|---|---|
| 0 | 全部通过 |
| 1 | **确实有问题**（数字漂移 / 交互失效 / 变异未被抓到） |
| 2 | **没测到**——无法解析、页面缺失、Playwright 未安装。必须与 1 区分 |
| 3 | 仓库或页面路径缺失 |

### 「数字一致」与「基线新鲜」是两件事

页脚写着"数据基线 `<commit>`"，但这个串**不参与退出码**，只发 `NOTICE`。原因是：

- 数字对得上，才是这套图谱还准确的**真信号**——哪怕 HEAD 已经前进，只要 105 crate / 150 表
  这些计数在 HEAD 上依然成立，图谱就没过期。
- 若把 commit 串也做成硬门禁，仓库每来一个无关提交门禁就红一次，久了就没人看它了。

所以 `NOTICE` 只提醒"这句 provenance 过期了"，修法是 `--rebase-baseline`；
只有**页脚里扫不出基线串**（格式被改坏）才 exit 2——那是"没测到"，不是"没问题"。

### 为什么交互自检不能省

整页截图只能证明"渲染出来了"，证明不了"按钮点了有反应"。
`arch_live_interact_check.py` 上线后陆续抓出**五个 bug**——它们在截图里完全看不出来，
因为截图拍的永远是控件的初始状态：

| # | 症状 | 根因 | 位置 | 怎么发现的 |
|---|---|---|---|---|
| 1 | 筛选 crate 时"共 N 个"计数从不变化 | `[data-shown]` 是 crate 墙的**兄弟**节点，`wall.querySelector` 永远拿不到 | `assets/arch.js` `crates()` | 交互自检 |
| 2 | 点标签页只变高亮，面板纹丝不动 | 选择器落在 `[data-tabgroup]` **包裹层**上，而它自己没有 `data-tab`，`null !== 'l2'` 恒真 → 整个容器被藏起来 | `assets/arch.js` `tabs()` | 交互自检 |
| 3 | 关掉 JS 后数字墙全是 `0` | `data-count` 元素的初始文本写死 `0`，只靠 `countUp()` 覆写 | 9 个页面的 `.n[data-count]` | 读代码 |
| 4 | 复制的代码尾部多一行"复制" | 按钮是 `appendChild` 到末尾的，清理正则却按**行首**锚定 | `assets/arch.js` `copy()` | 读代码 → 变异测试确认 |
| 5 | **整站交互全部静默失效** | 块注释里写了行首锚点正则，其中的星号斜杠序列当场结束注释 → 整份 `arch.js` SyntaxError | `assets/arch.js` | 交互自检（我改坏的） |

「怎么发现的」这一列不是装饰：③ 和 ④ 是**读代码**发现的，机器当时并没有覆盖到那两点。
把它们混在一起记账，会让人误以为门禁已经能拦住全部同类问题——实际上第 3 条的补法
（`--sync-seed`）本身就是 ③ 的修复。

第 2、5 个最值得反复看：

- **#2**：按钮的 `aria-selected` 更新是**对的**，只看「高亮变了」会判定通过。必须断言子面板的 `hidden` 真的翻转。
- **#5**：`node --check` 一秒就能判的语法错误，我却是在交互断言集体变红之后才查出来的。
  更糟的是**数字断言当时依然是绿的**——#3 的修复让种子值已经等于终值，
  于是那条断言根本区分不了「JS 跑了」和「JS 一行都没跑」。
  现在数字断言开头第一句就是"确认 `html.anim` 在"（由 `boot()` 添加），
  静态层也补了 `node --check` 门禁，不必等 Playwright。

`--mutate` 把上述 bug 注入回**站点副本**（`shutil.copytree` 到临时目录，绝不动真文件），
断言对应用例必须变红，另加一组不注入的对照组必须全绿——没有这一步，
"28/28 通过"和"门禁根本没跑"是同一种输出。实跑：对照组 28/28 绿，8/8 变异全部被抓到。

> 断言写成什么样才不是废的，本轮也交了学费：
> 「主题切换可逆」原本只断言"最后回到原值"，而**一次都没点动**同样满足这个条件——
> 现在改成两次点击之间必须真的翻转过。SMIL 降级也配了反向对照，
> 否则"页面本来就没写动画"一样能让降级断言通过。

> **已知局限（如实记录）**：Mavis 内置浏览器（FilePanel，1280×720）里点首屏以下的控件不生效——
> 04 页的"下一步/自动播放"、02 页的标签页都点不动，而顶部 `#themeBtn` 正常。
> 排查结论是**内置浏览器的问题，不是页面缺陷**，依据有三条：
> ① `html[data-theme]` 与 `class="anim"` 都在，说明 `arch.js` 确实执行了；
> ② 视口只有 720px 高，这些控件在 y≈1128，**本就在首屏之外**，而视口内的主题按钮正常；
> ③ 屏幕上能直接看到大片白色覆盖块（合成层伪影），`End` 键与滚动同样推不动页面。
>
> 但要说清楚：**这个结论的依据是 Playwright 的真实点击断言（15/15 通过）+ 上述现象观察，
> 不是我在内置浏览器里成功复现了交互**。若你用内置浏览器打开本套文档，
> 请把首屏以下的控件理解为"需要先滚动到视口内"，而不是"功能坏了"。

## 数据基线

| 项 | 值 |
|---|---|
| commit | 见各页页脚（随 `--rebase-baseline` 刷新） |
| 快照时间 | 2026-10-05 |
| 数字校验 | `python scripts/automation/arch_live_doc_check.py`（23 个 `data-fact` + 29 处种子值 + `node --check`） |
| 交互校验 | `python scripts/automation/arch_live_interact_check.py`（28 项，覆盖 9 页 + 8 处变异） |
| 渲染校验 | `python scripts/automation/arch_live_shot.py`（9 页整页 PNG） |

> 架构会变，数字会漂移。跑一次上面的门禁就能知道这套文档是否还跟得上代码——
> 变红就按提示更新页面对应数字（`data-fact` + `data-count`），别直接改门禁下限去迁就。
> 基线 commit 过期只会发 `NOTICE`（理由见上），用 `--rebase-baseline` 刷新即可。

## 文件结构

```
docs/arch-live/
├── index.html                 总览
├── 01..08-*.html              8 个分册
├── assets/
│   ├── arch.css               共享样式（暗/亮双主题、响应式、reduced-motion 降级）
│   └── arch.js                共享脚本（揭示动画、数字滚动、步进器、标签页、crate 过滤、SMIL 降级）
└── .preview/                  Playwright 生成的整页 PNG（git 忽略，可随时重建）
```

## 与既有文档的关系

本套**不替代**仓库已有的设计文档，而是提供"一眼看全 + 可交互"的上层视图：

- 需求 / 基本设计 / 详细设计 → `docs/requirements/` `docs/basic-design.md` `docs/detailed-design.md`
- 阶段实施报告 → `docs/reports/`
- DB 分类规则手册 → `docs/data-design/ipa-detail/`
- 治理与守门原文 → `AGENTS.md`

发现两处说法冲突时，**以代码和 `AGENTS.md` 为准**，并顺手修掉过时的那一份。
