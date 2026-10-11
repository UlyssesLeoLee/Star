/* =====================================================================
 * arch-live — Star 架构动态文档 共享脚本
 * 经典脚本 (非 ES module) + 无 fetch + 无外链 → file:// 双击可跑
 * ===================================================================== */
(function () {
  'use strict';

  var reduce = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  /* --- 1. 数字滚动 ---------------------------------------------------- */
  function countUp(el) {
    var target = parseFloat(el.getAttribute('data-count'));
    if (isNaN(target)) return;
    if (reduce) { el.textContent = fmt(target); return; }
    var dur = 1100, t0 = null;
    function frame(ts) {
      if (t0 === null) t0 = ts;
      var p = Math.min((ts - t0) / dur, 1);
      var e = 1 - Math.pow(1 - p, 3);          // easeOutCubic
      el.textContent = fmt(target * e);
      if (p < 1) requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
  }
  function fmt(n) {
    return (Math.abs(n - Math.round(n)) < 0.5 ? String(Math.round(n)) : n.toFixed(1));
  }

  /* --- 2. 滚动揭示 ---------------------------------------------------- */
  function reveal() {
    var items = document.querySelectorAll('.rv:not(.in), .stat:not(.in)');
    function show(n) {
      n.classList.add('in');
      /* data-count 通常挂在内层 .n 上, 而不是被观察的 .stat 上 —— 必须往里找一层,
         否则数字永远停在 0 (实测踩过)。 */
      var t = n.hasAttribute('data-count') ? n : n.querySelector('[data-count]');
      if (t) countUp(t);
    }
    if (!('IntersectionObserver' in window)) {
      Array.prototype.forEach.call(items, show);
      return;
    }
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) {
        if (!e.isIntersecting) return;
        show(e.target);
        io.unobserve(e.target);
      });
    }, { threshold: 0.05, rootMargin: '0px 0px -30px 0px' });
    Array.prototype.forEach.call(items, function (n) { io.observe(n); });

    /* 兜底: 入场动画只是装饰, 绝不能让内容永久不可见。
       整页截图 / 用户直接跳锚点 / IO 被禁用等场景下, 1.2s 后全部显示。 */
    setTimeout(function () {
      Array.prototype.forEach.call(document.querySelectorAll('.rv:not(.in), .stat:not(.in)'), show);
    }, 1200);
  }

  /* --- 3. 进度条填充 -------------------------------------------------- */
  function bars() {
    function fill(n) {
      var f = n.getAttribute('data-fill');
      if (f) n.querySelector('i').style.width = f + '%';
    }
    var all = document.querySelectorAll('.bar[data-fill]');
    if (!('IntersectionObserver' in window)) {
      Array.prototype.forEach.call(all, fill);
      return;
    }
    var io = new IntersectionObserver(function (es) {
      es.forEach(function (e) {
        if (!e.isIntersecting) return;
        fill(e.target);
        io.unobserve(e.target);
      });
    }, { threshold: 0.05 });
    Array.prototype.forEach.call(all, function (n) { io.observe(n); });
    setTimeout(function () {
      Array.prototype.forEach.call(document.querySelectorAll('.bar[data-fill]'), fill);
    }, 1200);
  }

  /* --- 4. 导航高亮 ---------------------------------------------------- */
  function nav() {
    var here = (location.pathname.split('/').pop() || 'index.html').toLowerCase();
    Array.prototype.forEach.call(document.querySelectorAll('.navlinks a'), function (a) {
      var href = (a.getAttribute('href') || '').split('/').pop().toLowerCase();
      if (href === here) a.classList.add('on');
    });
  }

  /* --- 5. 主题切换 ---------------------------------------------------- */
  function theme() {
    var btn = document.getElementById('themeBtn');
    if (!btn) return;
    function apply(t) { document.documentElement.setAttribute('data-theme', t); btn.textContent = t === 'light' ? '🌙 暗色' : '☀ 亮色'; }
    var saved = null;
    try { saved = localStorage.getItem('archlive-theme'); } catch (e) { /* file:// 可能禁用 */ }
    /* 默认暗色: 这套图是按暗色设计的 (渐变标题 / 霓虹描边)。
       不跟随 prefers-color-scheme —— 否则在浅色系统上整页失去对比度层次。
       浅色是显式选择, 不是默认结果。 */
    apply(saved === 'light' ? 'light' : 'dark');
    btn.addEventListener('click', function () {
      var now = document.documentElement.getAttribute('data-theme') === 'light' ? 'dark' : 'light';
      apply(now);
      try { localStorage.setItem('archlive-theme', now); } catch (e) { /* 忽略 */ }
    });
  }

  /* --- 6. 标签页 ------------------------------------------------------ */
  function tabs() {
    Array.prototype.forEach.call(document.querySelectorAll('.tabs'), function (bar) {
      var btns = bar.querySelectorAll('button[data-tab]');
      var group = bar.getAttribute('data-tabs');
      Array.prototype.forEach.call(btns, function (b) {
        b.addEventListener('click', function () {
          Array.prototype.forEach.call(btns, function (o) { o.setAttribute('aria-selected', o === b ? 'true' : 'false'); });
          /* 选择器必须落在"面板"上, 不能是 [data-tabgroup] 那个包裹层 ——
             包裹层自己没有 data-tab 属性, 于是 p.getAttribute('data-tab') 是 null,
             判 !== 恒为真, 结果是把整个容器藏起来, 子面板一个都没切换 (实测踩过:
             按钮 aria-selected 变对了, 面板纹丝不动)。
             用直接子元素 > 而非后代, 面板里若再嵌一组 tab 也不会被误伤。 */
          var all = document.querySelectorAll('[data-tabgroup="' + group + '"] > [data-tab]');
          Array.prototype.forEach.call(all, function (p) { p.hidden = p.getAttribute('data-tab') !== b.getAttribute('data-tab'); });
        });
      });
    });
  }

  /* --- 7. 步进器引擎 (请求生命周期 / Agent 闭环 / 部署) ---------------- */
  function steppers() {
    Array.prototype.forEach.call(document.querySelectorAll('[data-stepper]'), function (box) {
      var steps = box.querySelectorAll('.step');
      var total = steps.length;
      if (!total) return;
      /* 控件可以放在容器内, 也可以放在同级/上级的 .stepbar 里 —— 两处都找,
         避免"按钮写了但点不动"这种静默失效。 */
      function ctl(sel) {
        return box.querySelector(sel) ||
          (box.closest('.stepwrap') && box.closest('.stepwrap').querySelector(sel)) ||
          (box.parentElement && box.parentElement.querySelector(sel));
      }
      var prev = ctl('[data-prev]');
      var next = ctl('[data-next]');
      var play = ctl('[data-play]');
      var bar = ctl('[data-progress] > i');
      var label = ctl('[data-label]');
      var i = 0, timer = null;

      function paint() {
        Array.prototype.forEach.call(steps, function (s, k) {
          s.classList.toggle('act', k === i);
          s.classList.toggle('done', k < i);
        });
        if (bar) bar.style.width = Math.round(((i + 1) / total) * 100) + '%';
        if (label) label.textContent = (i + 1) + ' / ' + total;
        if (prev) prev.disabled = i === 0;
        if (next) next.disabled = i === total - 1;
        var hot = box.getAttribute('data-edge');
        if (hot) {
          var e = document.querySelectorAll('[data-edge-ref="' + hot + '"]');
          Array.prototype.forEach.call(e, function (el, k) { el.classList.toggle('hot', k === i); });
        }
      }
      function go(n) {
        i = Math.max(0, Math.min(total - 1, n));
        paint();
      }
      function stop() { if (timer) { clearInterval(timer); timer = null; } if (play) play.textContent = '▶ 自动播放'; }
      if (prev) prev.addEventListener('click', function () { stop(); go(i - 1); });
      if (next) next.addEventListener('click', function () { stop(); go(i + 1); });
      if (play) play.addEventListener('click', function () {
        if (timer) { stop(); return; }
        if (i === total - 1) go(0);
        play.textContent = '⏸ 暂停';
        timer = setInterval(function () {
          if (i >= total - 1) { stop(); return; }
          go(i + 1);
        }, parseInt(box.getAttribute('data-interval') || '2200', 10));
      });
      box.addEventListener('keydown', function (e) {
        if (e.key === 'ArrowRight') { stop(); go(i + 1); }
        if (e.key === 'ArrowLeft') { stop(); go(i - 1); }
      });
      paint();
    });
  }

  /* --- 8. crate 过滤 -------------------------------------------------- */
  function crates() {
    var wall = document.querySelector('[data-cratewall]');
    if (!wall) return;
    var input = document.getElementById('crateFilter');
    var chips = wall.querySelectorAll('.chip');
    function apply(q) {
      q = (q || '').trim().toLowerCase();
      Array.prototype.forEach.call(chips, function (c) {
        var name = c.getAttribute('data-name') || c.textContent;
        var hit = !q || name.toLowerCase().indexOf(q) !== -1;
        c.classList.toggle('dim', !hit);
        c.classList.toggle('hi', !!q && hit);
      });
      /* 计数器在 crate 墙"外面"(兄弟节点), 所以不能 wall.querySelector ——
         那样永远拿到 null, 筛选时"共 N 个"不跟着变 (实测踩过, 只有真点一次才发现)。 */
      var n = wall.querySelector('[data-shown]') || document.querySelector('[data-shown]');
      if (n) {
        var shown = wall.querySelectorAll('.chip:not(.dim)').length;
        n.textContent = shown;
      }
    }
    if (input) {
      input.addEventListener('input', function () { apply(input.value); });
      input.addEventListener('keydown', function (e) { if (e.key === 'Escape') { input.value = ''; apply(''); } });
    }
    Array.prototype.forEach.call(document.querySelectorAll('[data-family]'), function (b) {
      b.addEventListener('click', function () {
        var f = b.getAttribute('data-family');
        var on = b.getAttribute('aria-pressed') === 'true';
        Array.prototype.forEach.call(document.querySelectorAll('[data-family]'), function (o) { o.setAttribute('aria-pressed', 'false'); });
        if (on) { apply(''); if (input) input.value = ''; return; }
        b.setAttribute('aria-pressed', 'true');
        /* 搜索框要跟着家族按钮走: 否则点"DDD 域"时框里还留着旧词(如 worktree),
           chips 已经按 domain- 过滤, 框与结果对不上, 用户会以为按钮没生效
           (取消分支本来就会清空输入框, 这里保持同一语义)。 */
        if (input) input.value = f;
        apply(f);
      });
    });
    apply('');
  }

  /* --- 9. 复制代码 ---------------------------------------------------- */
  function copy() {
    Array.prototype.forEach.call(document.querySelectorAll('pre[data-copy]'), function (pre) {
      /* 原文必须在挂按钮之前抓: 按钮是 appendChild 到末尾的, innerText 会把它
         的"复制"两个字算进去。原来想用行首锚点去剥那两个字的, 剥的却是开头 ——
         顺序反了, 复出来的代码尾部会多一行"复制" (实测踩过)。
         直接在绑定时存一份原文, 比事后正则清理可靠。

         写这条注释时也踩过: 正文里不能出现连续星号加斜杠, 那会当场结束注释,
         中文直接被当成 JS 解析 -> 整份 arch.js 语法错误 -> 所有交互静默失效,
         而数字断言还因为"种子值已是终值"照样变绿。 */
      var orig = pre.innerText;
      var b = document.createElement('button');
      b.className = 'btn ghost';
      b.textContent = '复制';
      b.setAttribute('data-copybtn', '');
      b.style.cssText = 'position:absolute;top:8px;right:8px;font-size:11px;padding:3px 9px';
      b.addEventListener('click', function () {
        /* file:// 下 clipboard 常被拒, 且 writeText 返回的是 promise:
           不接住就是一个 unhandled rejection, 控制台报错像页面坏了。
           按钮反馈与是否真的写进剪贴板是两件事, 失败也要如实回话。 */
        var done = function () { b.textContent = '已复制'; };
        var fail = function () { b.textContent = '复制失败'; };
        if (!navigator.clipboard || !navigator.clipboard.writeText) { fail(); }
        else { navigator.clipboard.writeText(orig).then(done, fail); }
        setTimeout(function () { b.textContent = '复制'; }, 1400);
      });
      pre.style.position = 'relative';
      pre.appendChild(b);
    });
  }

  /* --- 10. SVG 路径动画（SMIL）----------------------------------------- */
  function motion() {
    /* prefers-reduced-motion 下移除 SMIL 动画节点, 保证动效可关 */
    if (!reduce) return;
    Array.prototype.forEach.call(document.querySelectorAll('animateMotion'), function (n) {
      n.parentNode.removeChild(n);
    });
  }

  /* --- 11. 启动 -------------------------------------------------------- */
  function boot() {
    document.documentElement.classList.add('anim');
    reveal(); bars(); nav(); theme(); tabs(); steppers(); crates(); copy(); motion();
    /* 动画是装饰: 兜底已由 reveal() 处理, 这里只保证极端情况下不残留隐藏态 */
    setTimeout(function () {
      Array.prototype.forEach.call(document.querySelectorAll('.rv'), function (n) { n.classList.add('in'); });
    }, 2000);
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot);
  else boot();
})();
