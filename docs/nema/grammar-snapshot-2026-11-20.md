# NEMA 语法快照 — 2026-11-20 (结构化摘要)

- 原始页面: `https://nema-sigma.vercel.app/website/grammar.html`
- 抓取时间: 2026-11-20 (Asia/Shanghai), HTTP 200, 5281 字节
- 原文存储: `docs/nema/grammar-snapshot-2026-11-20.html`
- SHA-256 (sha256sum): `67110ac289dfd520c43e6c6055c042f9d5a0c18049471d066395a92177f94cde`
- 与既有 pin 一致性: 该哈希与 `crates/nema-codec/NEMA_INTERPRETER_PLUGIN_CONTRACT.md`
  中已 pin 的本地 `grammar.html` 快照 (`67110ac2…94cde`) **字节级一致** —
  本次实网抓取确认线上页面与已接受的本地快照未漂移。

## 页面内容概要

页面标题「语法规范 · .NEMA」, 主体是「TABULA CANONICA · 规范矩阵」:
全景七维矩阵 (语法 / 释义 / 金石范式 / 设计理由 / 防范异常 / 写法变体 / 解析结果),
覆盖表格 (table) / 5W1H 六维 / Node–Edge 拓扑图 (graph) 三类, 另有共通典章
(转义规范与字典压缩)。页面自称收录 **139 条严密语法案例**; 规范中的前导空格均为
标准 ASCII U+0020。矩阵数据由 JS 加载 (`rule-matrix-data.js`), 语法解析由
`strict-parser.js` 提供 — HTML 本体是壳页, 矩阵数据不在本快照文件内。

> 诚实标注: 本任务仅抓取了 HTML 壳页。矩阵数据与参考解析器未从线上二次抓取,
> 沿用仓库内已 pin 的本地输入 (见 manifest.json 的 provenance 节):
> - 2026-11-15 矩阵: `ec274eda324c8fc88edc88a4bbc5afe3b508d4b4ca953ead173809aca6ae058a`
> - 2026-11-15 参考解析器: `d2f2d2fead79c4f3127a695e8ef5906880c42f5811e7c14c4be5aed5cd851fe8`
> - v1 矩阵: `0e1ae91167227f595f7586fa2174b19cb0eab4458c72e00c5790de770130f7b3`
> - 本地 Rust 解析器源码制品 (strict.rs, LF 规范化): `4ae855e1da4c595d0d146183c304bee966c5fb2f34a846fd0da3d224d7311707`

## 语法规则 (Rust strict 解析器镜像, 来源: `crates/nema-codec/src/strict.rs`)

### 域头 (domain header)

- 形式: ` \MODE<id> <revision>`, 行首一个空格 + 反斜杠, MODE 缺省为表格:
  `\` (table)、`?` (5W1H)、`>` (graph)。
- `<id>` / `<revision>` 为正整数 (禁前导 0); 同一文档内 id 不可重复。
- 表格域可带显式表头行 `\# Name1 Name2 …` (名称互不重复) 与声明宽度行 `\%<width>`。

### 单元格与值类型 (四个互斥状态)

| 书写 | 语义 | Cell AST | 说明 |
|---|---|---|---|
| `~` | Null 空值 | `{"kind":"null"}` | 单独成格; 混入文本则 `~` 视为字面 |
| `\e` | Empty 空字符串 | `{"kind":"empty"}` | 与其他特殊值混用报错 |
| `\_` | Missing 缺失值 | `{"kind":"missing"}` | 显式缺失; 行尾不足列数时隐式补 Missing |
| 其他 | 普通值 | `{"kind":"value","value":…}` | 转义: `\ ` `\\` `\n` `\r` `\t` `\~` `\?` `\>` `\<` `\=` `\^` `\#` `\%` |

- 全角空格 U+3000、制表符、回车在 strict 源码中一律禁止。
- 值不可为长度 0 的 `Value` (空串必须写 `\e`)。
- 资源预算: 源 ≤ 16 MiB, 列 ≤ 4096, 域 ≤ 4096, 记录 ≤ 100 000, 总单元格 ≤ 262 144。

### 续行语义 (v1 与 v2 的差异点, 2026-11-15 矩阵)

- **v1 (缩进续行, `matrix-2026-11-10`)**: 行首空格数为 n, 续写列
  `n-1` (graph 域再 +1); 缩进即列索引, 每列只能追加一次。
- **v2 (显式列索引续行, `matrix-2026-11-15`)**: 续行形如 ` N <value>` —
  单一空格 + 显式列索引 N (0 起始; graph 的 X0 禁止续写) + 值; 单字段形式
  ` <value>` 隐含列 (graph → X2, 其他 → X1)。
- 续写规则: 追加到 `Value`; `Empty` 接非空值升级为 `Value`;
  v2 下隐式 Missing 可被续写 (显式 `\_` 不可); 向 Null/显式 Missing 追加报错。
- 6 轴 5W1H: 行 ≤ 6 字段; 表格: 行 ≤ 声明宽度; 记录总数受预算约束。

### Graph 完整性

- 节点 ID: `A..Z 开头` + `[A-Za-z0-9_.:-]`; 边表达式 `A>Z>B` / `B<Z<C` / `A=Z=B`
  (方向符左右一致); 节点与边 ID 集合互斥; 边端点必须已声明为节点; 行至少 2 格。

### 规范化输出 (canonical format)

- 由 `format_strict_v1/v2` 生成: 域头 + `\%width` + `\# header` + 每行全格转义
  join; 输出必须能被同一 profile 重新解析且 AST 相等 (含四值状态), 否则拒绝输出。
- v2 与 v1 的规范化文本形态一致 (v2 复用 v1 发射器并做 v2 重解析校验)。

## 样例文件

见 `docs/nema/examples/*.nema` (v1/v2 × Value/Null/Empty/Missing + 续行差异样例),
每份样例的期望 AST 存于 `docs/nema/examples/expected/<同名>.ast.json`,
由 `crates/nema-codec/tests/plugin_runtime.rs::examples_conform_to_pinned_snapshot`
在 CI 中逐一断言。
