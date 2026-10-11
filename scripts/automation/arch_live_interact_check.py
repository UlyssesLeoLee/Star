#!/usr/bin/env python3
"""arch-live 页面交互自检 (Playwright).

为什么需要
    整页截图只能证明"渲染出来了", 证明不了"按钮点了有反应"。
    步进器 / 标签页 / crate 过滤 / 主题切换 / 降级路径这几类, 必须真点一次并断言 DOM,
    否则会出现"控件写好了但点不动"的静默失效。
    价值实证: 本脚本第一次跑就抓出两个真 bug ——
      (1) 计数器 [data-shown] 在 crate 墙外面, wall.querySelector 拿不到, 筛选时数字不变;
      (2) data-count 初始文本写死 0, 无 JS 时数字墙全是 0。

断言怎么写才有意义 (三条, 都是被自己的弱断言坑过之后定的)
    a) 往返, 不测初始态。fresh load 时步进器就在第 1 步、prev 是 disabled,
       直接点 prev 测不出任何东西 —— 断言会白通过。必须 next -> prev 回到起点。
    b) 跨节点交叉核对。同一个 handler 同时改两个节点时, 断言它们互相一致
       (如 .chip.hi 数量 == [data-shown] 文本), 单看一个会漏。
    c) 边界要断言 disabled/钳位, 不只断言"变了"。不点第 11 次是因为 next 已 disabled,
       Playwright 会等它变为可点而超时 —— 这是"钳位生效"的证据, 不是脚本 bug。

失败时必须能区分三件事 (探针 + 异常 + 返回值)
    页面没加载 / 动作抛异常 / 动作成功但状态没变。
    只看"ok=false"会把"断言写错"和"功能坏了"混成一类, 定位成本极高。

退出码 (fail-closed)
    0  全部交互通过
    1  有交互失效
    2  环境/页面缺失 (不是"交互坏了")

用法
    python scripts/automation/arch_live_interact_check.py
    python scripts/automation/arch_live_interact_check.py --headed --slow 200
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import subprocess
import sys
import tempfile

REPO = pathlib.Path(__file__).resolve().parents[2]
DOCS = REPO / "docs" / "arch-live"

STEP_NEXT = "[data-stepper] [data-next]"
STEP_PREV = "[data-stepper] [data-prev]"
STEP_PLAY = "[data-stepper] [data-play]"

# 断言片段: 数字墙 —— 动画结束后每个数字都应精确落在 data-count 上。
# 以前只断言"第一个 > 0", 漏掉了"停在中间值/停在中途"这类失败。
#
# 必须同时证明 JS 真的执行过, 否则这条断言是废的: --sync-seed 把初始文本写成
# 真实值之后, textContent === data-count 在"一个 JS 都没跑"的页面上同样成立。
# 实测踩过: arch.js 里一个注释写了行首锚点正则, 其中的星号斜杠序列当场结束注释,
# 整份脚本语法错误、所有交互静默失效, 而这条数字断言依然全绿。
# html.anim 由 boot() 添加, 是"脚本确实跑了"的独立证据, 放在最前面当守门。
ALL_COUNTS_LANDED = """
  if (!document.documentElement.classList.contains('anim')) return 'JS-NOT-RUN';
  const ns = Array.from(document.querySelectorAll('[data-count]'));
  if (ns.length === 0) return 'NO-COUNTS';
  const bad = ns.filter(n =>
    n.textContent.trim() !== n.getAttribute('data-count').trim())
    .map(n => n.getAttribute('data-count') + '->' + n.textContent.trim());
  return bad.length === 0 ? true : ('MISMATCH ' + bad.join(','));
"""

# 无 JS: html.anim 这个类由 JS 加, 没有它 .rv 就没有 opacity:0 门控 —— 这正是
# "动画是装饰, 内容默认可见"这条约定的实现方式, 所以直接断言它不存在 + 元素可见。
NO_JS = """
  const root = document.documentElement;
  const rv = document.querySelector('.rv');
  const ns = Array.from(document.querySelectorAll('[data-count]'));
  return !root.classList.contains('anim')
    && !!rv && Number(getComputedStyle(rv).opacity) === 1
    && ns.length > 0 && ns.every(n =>
         n.textContent.trim() === n.getAttribute('data-count').trim())
    && document.body.innerText.length > 3000;
"""

# 步进器内部状态读取器。必须整段贴进断言函数体, 不能写成
# "() => { const s = <这段>; }" —— 那是在给 const 赋一个以 const 开头的值,
# 必然 SyntaxError: Unexpected token 'const' (实测踩过, 5 个用例一起挂)。
STEP_STATE = """
  function st() {
    const box = document.querySelector('[data-stepper]');
    const steps = Array.from(box.querySelectorAll('.step'));
    const act = steps.filter(s => s.classList.contains('act'));
    return { i: steps.indexOf(act[0]), n: act.length, total: steps.length,
             label: box.querySelector('[data-label]').textContent.trim(),
             prevOff: box.querySelector('[data-prev]').disabled,
             nextOff: box.querySelector('[data-next]').disabled,
             playTxt: box.querySelector('[data-play]').textContent };
  }
"""

CASES = [
    # --- 步进器 (04) ---
    dict(page="04-agent-runtime.html", name="步进器-下一步",
         steps=[("click", STEP_NEXT, 1)],
         assert_js="() => { %s const s = st(); return s.i === 1 && s.n === 1"
                   " && s.label.indexOf('2 /') === 0 && s.prevOff === false"
                   " && document.querySelectorAll('[data-stepper] .step')[0]"
                   "     .classList.contains('done'); }" % STEP_STATE),
    dict(page="04-agent-runtime.html", name="步进器-上一步(往返)",
         steps=[("click", STEP_NEXT, 1), ("click", STEP_PREV, 1)],
         assert_js="() => { %s const s = st(); return s.i === 0 && s.n === 1"
                   " && s.label.indexOf('1 /') === 0 && s.prevOff === true; }" % STEP_STATE),
    dict(page="04-agent-runtime.html", name="步进器-末尾钳位",
         steps=[("click", STEP_NEXT, 9)],
         assert_js="() => { %s const s = st(); return s.i === s.total - 1"
                   " && s.label === s.total + ' / ' + s.total && s.nextOff === true; }" % STEP_STATE),
    dict(page="04-agent-runtime.html", name="步进器-自动播放推进",
         steps=[("click", STEP_PLAY, 1), ("wait", 3200, 0)],
         assert_js="() => { %s const s = st();"
                   " return s.i >= 1 && /\\u6682\\u505c/.test(s.playTxt); }" % STEP_STATE),
    dict(page="04-agent-runtime.html", name="步进器-键盘左右",
         steps=[("focus", "[data-stepper]", 0), ("key", "ArrowRight", 1), ("key", "ArrowLeft", 1)],
         assert_js="() => { %s const s = st();"
                   " return s.i === 0 && s.label.indexOf('1 /') === 0; }" % STEP_STATE),
    # --- crate 墙 (01) ---
    dict(page="01-crate-map.html", name="crate-搜索过滤 + 计数器联动",
         steps=[("fill", "#crateFilter", "worktree")],
         assert_js="""() => {
           const wall = document.querySelector('[data-cratewall]');
           const all = wall.querySelectorAll('.chip');
           const hi = wall.querySelectorAll('.chip.hi');
           const dim = wall.querySelectorAll('.chip.dim');
           const shown = document.querySelector('[data-shown]').textContent.trim();
           /* 交叉核对: 同一个 handler 改的两个节点必须一致 (见文件头 b) */
           return hi.length > 0 && dim.length > 0
             && hi.length + dim.length === all.length
             && Number(shown) === hi.length;
         }"""),
    dict(page="01-crate-map.html", name="crate-家族按钮过滤",
         steps=[("click", "[data-family='domain-']", 1)],
         assert_js="""() => {
           const b = document.querySelector("[data-family='domain-']");
           const hi = document.querySelectorAll('[data-cratewall] .chip.hi');
           const shown = document.querySelector('[data-shown]').textContent.trim();
           return b.getAttribute('aria-pressed') === 'true' && hi.length >= 30
             && Number(shown) === hi.length
             && document.querySelector("#crateFilter").value === 'domain-';
         }"""),
    dict(page="01-crate-map.html", name="crate-家族按钮再点取消",
         steps=[("click", "[data-family='domain-']", 2)],
         assert_js="""() => {
           const b = document.querySelector("[data-family='domain-']");
           return b.getAttribute('aria-pressed') === 'false'
             && document.querySelectorAll('[data-cratewall] .chip.dim').length === 0
             && document.querySelector('#crateFilter').value === '';
         }"""),
    dict(page="01-crate-map.html", name="crate-Esc 清空筛选",
         steps=[("fill", "#crateFilter", "hook"), ("key", "Escape", 1)],
         assert_js="""() => document.querySelector('#crateFilter').value === ''
             && document.querySelectorAll('[data-cratewall] .chip.dim').length === 0"""),
    # --- 标签页 (02) ---
    # 失败时返回诊断字符串而不是 false: 门禁的 val 字段会原样打出来,
    # 免得"红了但看不出哪一项不对" (val=false 不含任何信息量)。
    dict(page="02-navigation-model.html", name="标签页切换",
         steps=[("click", ".tabs[data-tabs='routes'] [data-tab='l2']", 1)],
         assert_js="""() => {
           const b = document.querySelector(".tabs[data-tabs='routes'] [data-tab='l2']");
           const bars = document.querySelectorAll('.tabs').length;
           const groups = document.querySelectorAll('[data-tabgroup]').length;
           const p2 = document.querySelector("[data-tabgroup='routes'] [data-tab='l2']");
           const p1 = document.querySelector("[data-tabgroup='routes'] [data-tab='l1']");
           const ok = !!b && b.getAttribute('aria-selected') === 'true'
             && !!p2 && !!p1 && p2.hidden === false && p1.hidden === true;
           return ok ? true : JSON.stringify({
             bars, groups, btn: !!b, sel: b && b.getAttribute('aria-selected'),
             p2: !!p2, p2hidden: p2 && p2.hidden,
             p1: !!p1, p1hidden: p1 && p1.hidden,
             allSel: Array.from(document.querySelectorAll('.tabs button[data-tab]'))
                        .map(x => x.getAttribute('data-tab') + '=' + x.getAttribute('aria-selected')),
             hiddenAttr: Array.from(document.querySelectorAll('[data-tabgroup] > *'))
                        .map(x => x.getAttribute('data-tab') + ':' + (x.hidden ? 'h' : 'v')),
           });
         }"""),
    # --- 主题 + 数字墙 + 降级 (index) ---
    dict(page="index.html", name="主题切换(往返)",
         steps=[("note", "theme", 0), ("click", "#themeBtn", 1)],
         assert_js="""() => {
           const now = document.documentElement.getAttribute('data-theme');
           return (now === 'light' || now === 'dark') && now !== window.__note.theme;
         }"""),
    dict(page="index.html", name="主题切换可逆",
         steps=[("note", "theme", 0), ("click", "#themeBtn", 1), ("wait", 150, 0),
                ("note", "theme2", 0), ("click", "#themeBtn", 1)],
         # 两次 note 之间必须真的翻转过, 否则"点了没反应"也会通过 ——
         # 只断言"最后回到原值"的话, 从没点动过同样是"等于原值" (实测踩过)。
         assert_js="""() => {
           const now = document.documentElement.getAttribute('data-theme');
           const n = window.__note;
           return n.theme2 !== n.theme && now === n.theme;
         }"""),
    dict(page="index.html", name="数字墙动画落点精确",
         steps=[("wait", 2500, 0)],
         assert_js="() => {%s}" % ALL_COUNTS_LANDED),
    dict(page="index.html", name="降级-无JS仍可读", nojs=True,
         steps=[],
         assert_js="() => {%s}" % NO_JS),
    dict(page="index.html", name="降级-reduced-motion 直达终值",
         steps=[("wait", 400, 0)], reduced=True,
         assert_js="() => {%s}" % ALL_COUNTS_LANDED),

    # --- 其余页面的步进器 / 数字 / 复制 (原先 9 页里只有 4 页有断言) ---
    dict(page="03-request-lifecycle.html", name="步进器-13步往返",
         steps=[("click", STEP_NEXT, 1), ("click", STEP_PREV, 1)],
         assert_js="() => { %s const s = st(); return s.total === 13 && s.i === 0"
                   " && s.prevOff === true && s.n === 1; }" % STEP_STATE),
    dict(page="03-request-lifecycle.html", name="步进器-13步末尾钳位",
         steps=[("clickuntil", STEP_NEXT, 20)],
         assert_js="() => { %s const s = st(); return s.i === s.total - 1"
                   " && s.nextOff === true && s.label === '13 / 13'; }" % STEP_STATE),
    dict(page="03-request-lifecycle.html", name="步进器-自动播放推进",
         steps=[("click", STEP_PLAY, 1), ("waitact", 1, 9000)],
         assert_js="() => { %s const s = st();"
                   " return s.i >= 1 && /\\u6682\\u505c/.test(s.playTxt); }" % STEP_STATE),
    dict(page="07-deploy-topology.html", name="步进器-6步往返",
         steps=[("click", STEP_NEXT, 1), ("click", STEP_PREV, 1)],
         assert_js="() => { %s const s = st(); return s.total === 6 && s.i === 0"
                   " && s.prevOff === true; }" % STEP_STATE),
    dict(page="07-deploy-topology.html", name="步进器-6步末尾钳位",
         steps=[("clickuntil", STEP_NEXT, 12)],
         assert_js="() => { %s const s = st(); return s.i === s.total - 1"
                   " && s.nextOff === true && s.label === '6 / 6'; }" % STEP_STATE),
    dict(page="index.html", name="步进器-7步往返",
         steps=[("click", STEP_NEXT, 1), ("click", STEP_PREV, 1)],
         assert_js="() => { %s const s = st(); return s.total === 7 && s.i === 0"
                   " && s.prevOff === true; }" % STEP_STATE),
    dict(page="06-data-model.html", name="数字墙动画落点精确",
         steps=[("wait", 2500, 0)],
         assert_js="() => {%s}" % ALL_COUNTS_LANDED),
    dict(page="04-agent-runtime.html", name="步进器-常数落点精确",
         steps=[("wait", 2500, 0)],
         assert_js="() => {%s}" % ALL_COUNTS_LANDED),
    # --- SMIL 降级 (README 明写"prefers-reduced-motion 下 SMIL 动画会被移除") ---
    # 必须配一条反向对照: 只测"降级后没有动画"的话, 页面本来就没写动画也一样通过。
    dict(page="03-request-lifecycle.html", name="SMIL-降级后动画节点已移除",
         steps=[], reduced=True,
         assert_js="() => document.querySelectorAll('animateMotion').length === 0"),
    dict(page="03-request-lifecycle.html", name="SMIL-未降级时动画节点存在(对照)",
         steps=[],
         assert_js="() => document.querySelectorAll('animateMotion').length >= 1"),
    dict(page="05-worktree-canvas.html", name="SMIL-降级后动画节点已移除",
         steps=[], reduced=True,
         assert_js="() => document.querySelectorAll('animateMotion').length === 0"),
    dict(page="05-worktree-canvas.html", name="SMIL-未降级时动画节点存在(对照)",
         steps=[],
         assert_js="() => document.querySelectorAll('animateMotion').length >= 1"),
    # 复制按钮: 断言"复制到的是原文, 不含按钮自己那两个字"。
    # 曾经用 /^复制\s*/ 清理, 但按钮是 appendChild 到末尾的, 顺序反了 ——
    # 复出来的代码尾部会多一行"复制"。
    dict(page="03-request-lifecycle.html", name="复制按钮-原文不含按钮文字",
         steps=[("click", "pre[data-copy] [data-copybtn]", 1), ("clipread", "", 0)],
         perms=["clipboard-read", "clipboard-write"],
         assert_js="""() => {
           const c = window.__note.clip;
           const btn = document.querySelector('pre[data-copy] [data-copybtn]');
           if (typeof c !== 'string') return 'NO_CLIP_NOTE';
           if (c.indexOf('CLIPBOARD-DENIED') === 0) {
             /* 读不到剪贴板就不能判"内容对不对", 但按钮反馈必须给出,
                也不能是 unhandled rejection 导致的静默失败。 */
             return /\\u5df2\\u590d\\u5236|\\u590d\\u5236\\u5931\\u8d25/.test(btn.textContent);
           }
           return c.length > 20
             && c.indexOf('\\u590d\\u5236') === -1
             && !/\\u590d\\u5236\\s*$/.test(c);
         }"""),
]

DRIVER = r"""
const { chromium } = require(process.env.PW_PATH);
const cases = JSON.parse(process.env.PW_CASES);
const DOCS = process.env.PW_DOCS;
const HEADED = process.env.PW_HEADED === '1';
const SLOW = Number(process.env.PW_SLOW || '0');

/* 每个用例都新建 context: nojs / reducedMotion / localStorage 主题/剪贴板权限
   都是 context 级状态, 复用同一个 page 会让断言结果依赖用例顺序。 */
async function makeCtx(browser, c) {
  const opts = { viewport: { width: 1440, height: 900 } };
  if (c.nojs) opts.javaScriptEnabled = false;
  if (c.reduced) opts.reducedMotion = 'reduce';
  if (c.perms) opts.permissions = c.perms;
  const ctx = await browser.newContext(opts);
  return { ctx, page: await ctx.newPage() };
}

async function runStep(page, step) {
  const kind = step[0], arg = step[1], extra = step[2] || 1;
  if (kind === 'click') {
    for (let k = 0; k < extra; k++) {
      await page.click(arg, { timeout: 8000 });
      await page.waitForTimeout(60);
    }
  } else if (kind === 'clickuntil') {
    /* 点到按钮变 disabled 为止, 上限 arg 次。
       为什么不能写死"点 N 次": 各页步数不同 (13/10/7/6), 写死次数的用例换个页面
       就失去意义 —— 多点一次会卡在 disabled 上超时, 少点又测不到边界。
       页面无关的写法才能把同一套断言套到所有步进器上。 */
    const cap = Number(extra);
    let hit = false;
    for (let k = 0; k < cap; k++) {
      const off = await page.evaluate(s => {
        const el = document.querySelector(s);
        return !el || el.disabled;
      }, arg);
      if (off) { hit = true; break; }
      await page.click(arg, { timeout: 8000 });
      await page.waitForTimeout(50);
    }
    if (!hit) throw new Error('clickuntil: ' + arg + ' 连点 ' + cap + ' 次仍未 disabled');
  } else if (kind === 'waitact') {
    /* 等到第 N 步成为当前步, 用来测自动播放 —— 各页 data-interval 不同
       (2800/2400/…), 写死等待毫秒数会在机器慢一点时假红。 */
    const want = Number(arg), cap = Number(extra) || 9000;
    const t0 = Date.now();
    for (;;) {
      const i = await page.evaluate(() => {
        const box = document.querySelector('[data-stepper]');
        const steps = Array.from(box.querySelectorAll('.step'));
        return steps.indexOf(steps.filter(s => s.classList.contains('act'))[0]);
      });
      if (i >= want) break;
      if (Date.now() - t0 > cap) throw new Error('waitact: ' + (cap / 1000) + 's 内没走到第 ' + (want + 1) + ' 步');
      await page.waitForTimeout(120);
    }
  } else if (kind === 'fill') {
    await page.fill(arg, String(extra), { timeout: 8000 });
  } else if (kind === 'key') {
    await page.keyboard.press(arg);
  } else if (kind === 'focus') {
    await page.focus(arg);
  } else if (kind === 'wait') {
    await page.waitForTimeout(Number(arg));
  } else if (kind === 'clipread') {
    /* 读回剪贴板, 存进 window.__note.clip 供断言比对。
       用来验证"复制到的是原文, 不含按钮自己的'复制'两个字"。 */
    const t = await page.evaluate(async () => {
      try { return await navigator.clipboard.readText(); }
      catch (e) { return 'CLIPBOARD-DENIED:' + String(e).slice(0, 60); }
    });
    await page.evaluate(v => { window.__note = window.__note || {}; window.__note.clip = v; }, t);
  } else if (kind === 'note') {
    /* 断言里要比对"点击前"的值时用: 主题切换的断言必须与初值无关, 否则
       依赖 localStorage 上一次跑残留什么, 绿灯/红灯会随机。 */
    await page.evaluate(
      k => { window.__note = window.__note || {}; window.__note[k] =
             document.documentElement.getAttribute('data-theme'); }, arg);
  } else {
    throw new Error('unknown step kind: ' + kind);
  }
}

(async () => {
  const browser = await chromium.launch({ headless: !HEADED });
  const results = [];
  const errors = [];
  for (const c of cases) {
    const { ctx, page } = await makeCtx(browser, c);
    let val = '<not-run>', err = '', ok = false, probe = '';
    page.on('pageerror', e => errors.push(c.name + ' pageerror: ' + String(e)));
    page.on('console', m => {
      if (m.type() === 'error') errors.push(c.name + ' console: ' + m.text());
    });
    try {
      const url = 'file:///' + DOCS + '/' + c.page;
      await page.goto(url, { waitUntil: 'load', timeout: 15000 });
      await page.waitForTimeout(c.nojs ? 0 : 400);
      /* 探针: 断言失败时区分"页面没加载"和"加载了但状态没变"。
         必须用 IIFE 形式: page.evaluate 传字符串时按"表达式"求值,
         "() => {...}" 求值出来是个函数对象, 序列化后是 undefined ——
         探针会静默变成字符串 "undefined", 什么也区分不了 (实测踩过)。 */
      probe = String(await page.evaluate(
        "(() => JSON.stringify({href: location.href.slice(-44), title: document.title,"
        + " steps: document.querySelectorAll('[data-stepper] .step').length,"
        + " chips: document.querySelectorAll('.chip').length,"
        + " counts: document.querySelectorAll('[data-count]').length,"
        + " body: document.body ? document.body.innerHTML.length : -1}))()"));
      for (const s of c.steps) await runStep(page, s);
      await page.waitForTimeout(SLOW || 250);
      /* IIFE 字符串求值: 直接传 '() => ...' 在部分版本返回 undefined。 */
      val = await page.evaluate('(' + c.assert_js + ')()');
      ok = (val === true);
    } catch (e) {
      err = 'EXC ' + String(e).split('\n')[0].slice(0, 160);
    }
    results.push({ page: c.page, name: c.name, ok: ok === true,
                   val: JSON.stringify(val), err: err, probe: probe });
    await ctx.close();
  }
  console.log(JSON.stringify({ results, errors }));
  await browser.close();
})().catch(e => { console.error('DRIVER_FATAL ' + String(e).split('\n')[0]); process.exit(3); });
"""


def node_modules_playwright() -> pathlib.Path:
    """返回可 require 的 playwright 包目录.

    注意: 本仓装的是 @playwright/test (不是顶层 playwright), 它同样 re-export
    chromium/firefox/webkit, 因此 require 它即可。找不到就 exit 2, 不能静默跳过。
    """
    for rel in ("frontend/node_modules/@playwright/test", "frontend/node_modules/playwright",
                "node_modules/@playwright/test", "node_modules/playwright"):
        cand = REPO / rel
        if (cand / "package.json").exists():
            return cand
    return REPO / "frontend" / "node_modules" / "@playwright" / "test"


# 变异清单: 每条都是"这个门禁当初真抓出来过的 bug"。
# 为什么要固化下来: 从未失败过的门禁只是装饰。把 bug 注入回副本, 断言门禁必须变红,
# 才能证明"现在的绿"不是"门禁根本没跑"造成的假绿。
# (name, 目标文件, 变异前, 变异后, 必须变红的用例名)
MUTATIONS = [
    ("tabs-选择器落在包裹层而非面板", "assets/arch.js",
     """'[data-tabgroup="' + group + '"] > [data-tab]'""",
     """'[data-tabgroup="' + group + '"]'""",
     "标签页切换"),
    ("crate-计数器作用域太窄", "assets/arch.js",
     "wall.querySelector('[data-shown]') || document.querySelector('[data-shown]')",
     "wall.querySelector('[data-shown]')",
     "crate-搜索过滤 + 计数器联动"),
    ("家族按钮不同步搜索框", "assets/arch.js",
     "if (input) input.value = f;",
     "/* mutated */",
     "crate-家族按钮过滤"),
    ("无JS时数字种子值写死0", "index.html",
     'data-count="105" data-fact="crates_total">105<',
     'data-count="105" data-fact="crates_total">0<',
     "降级-无JS仍可读"),
    ("步进器监听器不绑定", "assets/arch.js",
     "if (next) next.addEventListener('click', function () { stop(); go(i + 1); });",
     "/* mutated: handler removed */",
     "步进器-下一步"),
    ("复制时把按钮自己的字一起复制", "assets/arch.js",
     "navigator.clipboard.writeText(orig)",
     "navigator.clipboard.writeText(pre.innerText)",
     "复制按钮-原文不含按钮文字"),
    # 这一类最阴: 整份 arch.js 语法错误 -> 脚本一行都没跑 -> 所有交互静默失效。
    # 而数字断言本来不会红 (种子值已是终值), 是"JS 真的执行了"那半句救下了它。
    ("arch.js 语法错误(注释里写了行首锚点正则)", "assets/arch.js",
     "  function copy() {",
     "  function copy( {",
     "数字墙动画落点精确"),
    ("SMIL 降级没实现", "assets/arch.js",
     "if (!reduce) return;\n    Array.prototype.forEach.call(document.querySelectorAll('animateMotion')",
     "if (true) return;\n    Array.prototype.forEach.call(document.querySelectorAll('animateMotion')",
     "SMIL-降级后动画节点已移除"),
]


def run_driver(cases: list[dict], docs: pathlib.Path, headed: bool, slow: int) -> tuple[int, dict]:
    """跑一次 node driver, 返回 (python 退出码, 解析后的结果)."""
    pkg = node_modules_playwright()
    if not (pkg / "package.json").exists():
        print(f"FAIL: playwright not found at {pkg}", file=sys.stderr)
        print("      fix: cd frontend; npm ci", file=sys.stderr)
        return 2, {}

    drv = pathlib.Path(tempfile.gettempdir()) / "arch_live_interact_driver.js"
    drv.write_text(DRIVER, encoding="utf-8")

    env = dict(os.environ)
    env["PW_PATH"] = str(pkg).replace("\\", "\\\\")
    env["PW_CASES"] = json.dumps(cases, ensure_ascii=False)
    env["PW_HEADED"] = "1" if headed else "0"
    env["PW_SLOW"] = str(slow)
    env["PW_DOCS"] = docs.as_posix()   # 直接给 file:// 可用的正斜杠路径

    proc = subprocess.run(["node", str(drv)], cwd=str(REPO), env=env,
                          capture_output=True, timeout=600,
                          # 必须显式指定 utf-8: Windows 默认用 GBK 解码, node 输出里的
                          # 中文会直接抛 UnicodeDecodeError, 表现为"脚本崩了",
                          # 而实际只是控制台编码问题 (实测踩过)。
                          encoding="utf-8", errors="replace")
    if proc.returncode != 0 or not proc.stdout.strip():
        print("FAIL: node driver failed", file=sys.stderr)
        print((proc.stderr or proc.stdout)[-2000:], file=sys.stderr)
        return 2, {}
    return 0, json.loads(proc.stdout.strip().splitlines()[-1])


def apply_mutation(root: pathlib.Path, target: str, old: str, new: str) -> bool:
    """在站点副本里注入一处变异, 返回是否真的改动了."""
    p = root / target
    src = p.read_text(encoding="utf-8")
    if old not in src:
        return False
    p.write_text(src.replace(old, new, 1), encoding="utf-8", newline="")
    return True


def self_test_mutation(headed: bool) -> int:
    """变异 + 对照: 证明这组断言真的会红, 且红的位置正确."""
    import shutil
    tmp = pathlib.Path(tempfile.mkdtemp(prefix="arch_live_mut_"))
    problems: list[str] = []
    try:
        # 对照组: 不注入任何变异, 必须全绿 —— 否则无法区分"能发现违规"和"装置永远失败"
        base = tmp / "base"
        shutil.copytree(DOCS, base)
        rc, out = run_driver(CASES, base, headed, 0)
        if rc == 2:
            return 2
        green = sum(1 for r in out["results"] if r.get("ok"))
        total = len(out["results"])
        if green != total:
            problems.append(f"对照组: 未注入变异却有 {total - green} 项红 —— 变异装置或基线有问题")
        print(f"  对照组(无变异): {green}/{total} 绿"
              + ("" if green == total else "  <-- 异常"))

        for name, target, old, new, case_name in MUTATIONS:
            root = tmp / ("mut_" + str(abs(hash(name)) % 10_000))
            shutil.copytree(DOCS, root)
            if not apply_mutation(root, target, old, new):
                problems.append(f"MUTATE-FAIL {name}: 在 {target} 里找不到待变异片段, "
                                f"变异没被应用 (页面已被手工改过, 或字符串写错了)")
                print(f"  MISS  {name}  <- 片段未命中")
                continue
            cases = [c for c in CASES if c["name"] == case_name]
            if not cases:
                problems.append(f"MUTATE-FAIL {name}: 找不到用例 {case_name!r}")
                continue
            rc, out = run_driver(cases, root, headed, 0)
            if rc == 2:
                problems.append(f"MUTATE-FAIL {name}: driver 跑挂了, 不算数 (既没抓也没漏)")
                continue
            r = out["results"][0]
            caught = not r.get("ok")
            print(f"  {'CAUGHT' if caught else 'MISS  '} {name}"
                  f"  -> 用例[{case_name}] {'变红(有效)' if caught else '仍绿(门禁没牙!)'}")
            if not caught:
                problems.append(f"MUTATE-FAIL {name}: 注入后用例 [{case_name}] 仍绿 —— "
                                f"这条断言抓不住该 bug")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    if problems:
        for p in problems:
            print(p, file=sys.stderr)
        print(f"\nSELF-TEST FAIL: {len(problems)} 项 (exit=1)", file=sys.stderr)
        return 1
    print(f"\nSELF-TEST PASS: 对照组全绿 + {len(MUTATIONS)} 处变异全部被抓到")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--headed", action="store_true")
    ap.add_argument("--slow", type=int, default=0)
    ap.add_argument("--only", default="", help="只跑名字含该子串的用例")
    ap.add_argument("--mutate", action="store_true",
                    help="变异测试: 注入已知 bug, 断言门禁必须变红 (附对照组)")
    args = ap.parse_args()

    if not DOCS.is_dir():
        print(f"FAIL: docs dir not found: {DOCS}", file=sys.stderr)
        return 2
    if not node_modules_playwright().joinpath("package.json").exists():
        print("FAIL: playwright not installed (cd frontend; npm ci)", file=sys.stderr)
        return 2

    if args.mutate:
        return self_test_mutation(args.headed)

    cases = [c for c in CASES if args.only in c["name"]]
    if not cases:
        print(f"FAIL: --only {args.only!r} 没匹配到任何用例", file=sys.stderr)
        return 2

    rc, out = run_driver(cases, DOCS, args.headed, args.slow)
    if rc == 2:
        return 2

    failed = 0
    for r in out["results"]:
        mark = "OK  " if r.get("ok") else "FAIL"
        if not r.get("ok"):
            failed += 1
        print(f"{mark} {r['page']:<26} {r['name']}")
        if not r.get("ok"):
            print(f"       val={r.get('val')}  err={r.get('err') or '-'}")
            print(f"       probe={r.get('probe')}")
    if out["errors"]:
        print("\npage errors:", file=sys.stderr)
        for e in out["errors"][:10]:
            print(f"  {e}", file=sys.stderr)

    if failed:
        print(f"\nFAIL: {failed}/{len(out['results'])} 项交互失效 (exit=1)", file=sys.stderr)
        return 1
    print(f"\nPASS: {len(out['results'])} 项交互全部生效")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
