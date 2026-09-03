/* =====================================================================
 * V-Model Kanban · アプリケーションロジック
 * 状態管理 / レンダリング / インタラクション / 永続化
 * =================================================================== */

(function () {
  'use strict';

  const { PHASES: DEFAULT_PHASES, AUX } = window.VMODEL;
  const STORAGE_KEY = 'vmodel-kanban-v1';
  const PHASE_STORAGE_KEY = 'vmodel-phases-v1';
  const TASK_STORAGE_KEY = 'vmodel-tasks-v1';
  const THEME_KEY = 'vmodel-theme-v1';
  const SPRINT_STORAGE_KEY = 'vmodel-sprints-v1';
  const TEAM_CONFIG_KEY = 'vmodel-team-config-v1';
  const METRICS_OPEN_KEY = 'vmodel-metrics-open-v1';
  const CEREMONIES_OPEN_KEY = 'vmodel-ceremonies-open-v1';

  /* ------------------------------------------------------------------
   * 永続化レイヤー
   * ----------------------------------------------------------------*/
  const store = {
    load(key, fallback) {
      try {
        const raw = localStorage.getItem(key);
        return raw ? JSON.parse(raw) : fallback;
      } catch { return fallback; }
    },
    save(key, val) {
      try { localStorage.setItem(key, JSON.stringify(val)); } catch {}
    }
  };

  /* ------------------------------------------------------------------
   * 状態
   * ----------------------------------------------------------------*/
  const state = {
    phases: store.load(PHASE_STORAGE_KEY, null) || deepClone(DEFAULT_PHASES),
    tasks:  store.load(TASK_STORAGE_KEY,  null) || buildInitialTasks(),
    activePhaseId: 'P1',
    view: 'kanban',          // kanban | list | timeline | sprint
    filter: '',
    industry: store.load('vmodel-industry-v1', 'all'),  // all | finance | public | ec | embedded
    theme: store.load(THEME_KEY, 'dark'),
    sprints: store.load(SPRINT_STORAGE_KEY, null) || [],  // [{id, name, goal, startDate, endDate, durationDays, status, taskIds[], dailySnapshots[], ceremonies, createdAt, completedAt, velocity}]
    teamConfig: store.load(TEAM_CONFIG_KEY, null) || { size: 3, hoursPerWeek: 40 },  // 团队规模 + 每人每周可用工时
    metricsOpen: store.load(METRICS_OPEN_KEY, false),  // Sprint 视图 metrics panel 展开状态
    ceremoniesOpen: store.load(CEREMONIES_OPEN_KEY, false),  // Sprint 视图 ceremonies panel 展开状态
    activeSprintId: null  // id of the currently active sprint (status='active')
  };

  function buildInitialTasks() {
    // Convert phases[].tasks[] into flat task map with stable ids
    const out = {};
    DEFAULT_PHASES.forEach(p => {
      p.tasks.forEach(t => { out[t.id] = { ...t, _industry: 'all' }; });
      if (p.subphases) {
        p.subphases.forEach(sp => {
          sp.tasks.forEach(t => { out[t.id] = { ...t, _subphase: sp.id, _industry: 'all' }; });
        });
      }
    });
    // 加载行业预设 (if available — VMODEL_INDUSTRIES 由 data/industries/*.js 注入)
    if (window.VMODEL_INDUSTRIES) {
      Object.values(window.VMODEL_INDUSTRIES).forEach(entry => {
        entry.tasks.forEach(t => {
          out[t.id] = { ...t, _industry: entry.industry, _subphase: entry.phase };
        });
      });
    }
    return out;
  }

  function deepClone(x) { return JSON.parse(JSON.stringify(x)); }

  /* ------------------------------------------------------------------
   * Theme
   * ----------------------------------------------------------------*/
  function applyTheme(t) {
    document.documentElement.setAttribute('data-theme', t);
    store.save(THEME_KEY, t);
  }
  applyTheme(state.theme);

  /* ------------------------------------------------------------------
   * 集計
   * ----------------------------------------------------------------*/
  function tasksForPhase(phase) {
    // 行业过滤规则: 主任务 (_industry='all') 总是显示; 行业预设任务只在选中其行业时显示。
    // 任务归属: 主任务 (P1-001 等) 用 id 起始匹配 phaseId;
    //          行业任务 (P1-FIN-001 等) 用 _subphase (buildInitialTasks 写入) 匹配 phaseId。
    const out = [];
    const includeIndustry = state.industry;
    const phaseId = phase.id;  // e.g. P1, P6.1, P6
    Object.values(state.tasks).forEach(t => {
      // 任务归属 phase: 优先用 _subphase (industry + main subphase)
      let taskPhase = t._subphase;
      if (!taskPhase) {
        // 主任务: 从 id 推断 (P1-001, P6.1-001, P6-001)
        const m = (t.id || '').match(/^(P\d+(?:\.\d+)?)-/);
        taskPhase = m ? m[1] : null;
      }
      if (taskPhase !== phaseId) return;
      if (t._industry !== 'all' && t._industry !== includeIndustry) return;
      out.push(t);
    });
    return out;
  }

  function taskCount(phaseId) {
    const phase = findPhase(phaseId);
    if (!phase) return 0;
    return tasksForPhase(phase).length;
  }

  // Compute total task count for a phase under the *current industry filter* (used by stat)
  function totalFilteredCount(phase) {
    return tasksForPhase(phase).length;
  }

  function findPhase(id) {
    for (const p of state.phases) {
      if (p.id === id) return p;
      if (p.subphases) {
        const sp = p.subphases.find(x => x.id === id);
        if (sp) return { ...sp, _parentId: p.id };
      }
    }
    return null;
  }

  function findPhaseStrict(id) {
    for (const p of state.phases) {
      if (p.id === id) return p;
    }
    return null;
  }

  function statForPhase(phase) {
    const all = tasksForPhase(phase);
    const total = all.length;
    const done = all.filter(t => t.status === 'done').length;
    const doing = all.filter(t => t.status === 'doing' || t.status === 'review').length;
    const todo = total - done - doing;
    return { total, done, doing, todo, pct: total ? Math.round(done / total * 100) : 0 };
  }

  /* ------------------------------------------------------------------
   * Renderers
   * ----------------------------------------------------------------*/

  // ----- Top v-model strip -----
  function renderVStrip() {
    const el = document.querySelector('.vmodel-strip');
    el.innerHTML = '';
    state.phases.forEach((p, i) => {
      const item = document.createElement('button');
      item.className = 'vmodel-strip__item';
      item.style.setProperty('--c', p.color);
      item.dataset.phase = p.id;
      item.innerHTML = `
        <span class="vmodel-strip__num">${p.num}</span>
        <span class="vmodel-strip__icon">${p.icon}</span>
        <span>${p.name}</span>
      `;
      if (p.id === state.activePhaseId) item.classList.add('is-active');
      item.addEventListener('click', () => switchPhase(p.id));
      el.appendChild(item);

      if (i < state.phases.length - 1) {
        const sep = document.createElement('span');
        sep.className = 'vmodel-strip__sep';
        sep.textContent = '›';
        el.appendChild(sep);
      }
    });
  }

  // ----- Right phasebar -----
  function renderPhasebar() {
    const el = document.getElementById('phaseList');
    el.innerHTML = '';
    state.phases.forEach((p, i) => {
      const li = document.createElement('li');
      li.className = 'phase-item';
      li.style.setProperty('--c', p.color);
      li.dataset.phase = p.id;
      if (p.id === state.activePhaseId) li.classList.add('is-active');
      li.innerHTML = `
        <div class="phase-item__icon">${p.icon}</div>
        <div class="phase-item__main">
          <div class="phase-item__name">${p.name}</div>
          <div class="phase-item__meta">
            <span class="phase-item__num">${p.num}</span>
            <span>·</span>
            <span>${taskCount(p.id)} 件</span>
          </div>
        </div>
        <div class="phase-item__count">${taskCount(p.id)}</div>
        <button class="phase-item__menu" data-menu="${p.id}" aria-label="メニュー">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/></svg>
        </button>
      `;
      li.addEventListener('click', (e) => {
        if (e.target.closest('.phase-item__menu')) return;
        switchPhase(p.id);
      });
      li.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        openCtxMenu(e.clientX, e.clientY, p.id);
      });
      el.appendChild(li);

      // subphases (P6 children) — show whenever P6 is visible, not only when active
      if (p.subphases) {
        const wrap = document.createElement('div');
        wrap.className = 'phasebar__sub-list';
        p.subphases.forEach(sp => {
          const sli = document.createElement('div');
          sli.className = 'phase-item phase-item--sub';
          sli.style.setProperty('--c', sp.color);
          sli.dataset.phase = sp.id;
          if (sp.id === state.activePhaseId) sli.classList.add('is-active');
          sli.innerHTML = `
            <div class="phase-item__icon">${sp.icon}</div>
            <div class="phase-item__main">
              <div class="phase-item__name">${sp.name}</div>
              <div class="phase-item__meta">
                <span class="phase-item__num">${sp.num}</span>
                <span>·</span>
                <span>${taskCount(sp.id)} 件</span>
              </div>
            </div>
            <div class="phase-item__count">${taskCount(sp.id)}</div>
          `;
          sli.addEventListener('click', () => switchPhase(sp.id));
          wrap.appendChild(sli);
        });
        el.appendChild(wrap);
      }
    });

    // bind menu buttons
    el.querySelectorAll('.phase-item__menu').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const r = btn.getBoundingClientRect();
        openCtxMenu(r.right, r.bottom, btn.dataset.menu);
      });
    });

    // auto-scroll active phase into view
    requestAnimationFrame(() => {
      const active = el.querySelector('.phase-item.is-active');
      if (active) active.scrollIntoView({ block: 'center', behavior: 'smooth' });
    });
  }

  // ----- Stage header -----
  function renderStageHeader() {
    const phase = findPhase(state.activePhaseId);
    if (!phase) return;
    const parent = findPhaseStrict(phase._parentId || phase.id);
    const display = parent || phase;
    const stat = statForPhase(phase);

    document.documentElement.style.setProperty('--c-phase', display.color || phase.color);
    document.documentElement.style.setProperty('--c-phase-2', mix(display.color || phase.color, '#818cf8', 0.5));

    document.getElementById('stageNum').textContent = display.num;
    document.getElementById('stageTotal').textContent = String(state.phases.length).padStart(2, '0');
    document.getElementById('stageKana').textContent = display.kana || '';
    document.getElementById('stageTitle').textContent = phase.name;
    document.getElementById('stageDesc').textContent = phase.desc || '';

    document.getElementById('statTotal').textContent = stat.total;
    document.getElementById('statDone').textContent  = stat.done;
    document.getElementById('statDoing').textContent = stat.doing;
    document.getElementById('statTodo').textContent  = stat.todo;

    const arc = document.getElementById('progressArc');
    const dash = 175.93;
    arc.setAttribute('stroke-dashoffset', String(dash * (1 - stat.pct / 100)));
    document.getElementById('progressPct').textContent = `${stat.pct}%`;
  }

  // ----- Kanban board -----
  function renderKanban() {
    const phase = findPhase(state.activePhaseId);
    if (!phase) return;
    const board = document.getElementById('kanban');
    board.innerHTML = '';
    const allTasks = tasksForPhase(phase);
    const cols = phase.cols || [
      { id: 'backlog', name: 'バックログ', color: '#6b7280' },
      { id: 'todo',    name: 'To Do',     color: '#3b82f6' },
      { id: 'doing',   name: '進行中',    color: '#eab308' },
      { id: 'review',  name: 'レビュー',  color: '#a855f7' },
      { id: 'done',    name: '完了',      color: '#22c55e' }
    ];

    cols.forEach(col => {
      const colEl = document.createElement('section');
      colEl.className = 'kanban-col';
      colEl.style.setProperty('--c', col.color);
      colEl.dataset.col = col.id;

      const tasks = allTasks
        .filter(t => t.status === col.id)
        .filter(t => !state.filter || (t.title + ' ' + t.desc + ' ' + t.tags.join(' ')).toLowerCase().includes(state.filter.toLowerCase()));

      const limit = col.limit;
      const limitWarn = limit && tasks.length > limit;
      const headHTML = `
        <header class="kanban-col__head">
          <div class="kanban-col__title">
            <span class="kanban-col__dot"></span>
            <span>${col.name}</span>
            ${limit ? `<span class="kanban-col__limit ${limitWarn ? 'is-warn' : ''}">/ WIP ${limit}</span>` : ''}
          </div>
          <span class="kanban-col__count">${tasks.length}</span>
        </header>
        <div class="kanban-col__body" data-dropzone="${col.id}">
          ${tasks.length ? tasks.map(cardHTML).join('') : `<div class="kanban-col__empty">タスクなし</div>`}
        </div>
        <button class="kanban-col__add" data-addcol="${col.id}">+ 追加</button>
      `;
      colEl.innerHTML = headHTML;
      board.appendChild(colEl);
    });

    // Drag & drop
    board.querySelectorAll('[data-dropzone]').forEach(zone => {
      zone.addEventListener('dragover', (e) => {
        e.preventDefault();
        zone.closest('.kanban-col').classList.add('is-drop-target');
      });
      zone.addEventListener('dragleave', () => {
        zone.closest('.kanban-col').classList.remove('is-drop-target');
      });
      zone.addEventListener('drop', (e) => {
        e.preventDefault();
        zone.closest('.kanban-col').classList.remove('is-drop-target');
        const id = e.dataTransfer.getData('text/plain');
        if (id && state.tasks[id]) {
          state.tasks[id].status = zone.dataset.dropzone;
          save();
          renderAll();
          toast(`${id} → ${zone.dataset.dropzone}`);
        }
      });
    });

    // Card click → modal
    board.querySelectorAll('.card').forEach(card => {
      card.addEventListener('click', (e) => {
        if (e.target.closest('button')) return;
        openTaskModal(card.dataset.id);
      });
    });

    // Add buttons
    board.querySelectorAll('[data-addcol]').forEach(btn => {
      btn.addEventListener('click', () => addTaskToColumn(btn.dataset.addcol));
    });
  }

  function cardHTML(t) {
    const owner = t.owner
      ? `<div class="card__owner" title="${t.owner}">${t.owner.slice(0, 2).toUpperCase()}</div>`
      : `<div class="card__owner card__owner--unassigned" title="未割り当て">?</div>`;
    // 行业徽章 (如果有 _industry 且非 all)
    let industryBadge = '';
    if (t._industry && t._industry !== 'all') {
      const indColors = { finance: '#dc2626', public: '#0ea5e9', ec: '#f59e0b', embedded: '#10b981' };
      const indJa = { finance: '金融', public: '公共', ec: 'EC', embedded: '組込' };
      const c = indColors[t._industry] || '#94a3b8';
      const j = indJa[t._industry] || t._industry;
      industryBadge = `<span class="card__industry" style="--c:${c}" title="業種プリセット: ${j}">${j}</span>`;
    }
    return `
      <article class="card" draggable="true" data-id="${t.id}" style="--pc: var(--priority-${t.priority}, #6b7280)">
        <div class="card__head">
          <span class="card__id">${t.id}</span>
          ${industryBadge}
          <span class="card__prio card__prio--${t.priority}">${t.priority}</span>
        </div>
        <div class="card__title">${escapeHTML(t.title)}</div>
        <div class="card__desc">${escapeHTML(t.desc)}</div>
        ${t.tags && t.tags.length ? `<div class="card__tags">${t.tags.slice(0, 3).map(tag => `<span class="tag">${escapeHTML(tag)}</span>`).join('')}</div>` : ''}
        <div class="card__foot">
          <div class="card__meta">
            <span class="card__meta-item" title="見積もり">⏱ ${t.estimate || 0}h</span>
            ${t.linkedDocs && t.linkedDocs.length ? `<span class="card__meta-item" title="成果物">📄 ${t.linkedDocs.length}</span>` : ''}
            ${t.reviewPoints && t.reviewPoints.length ? `<span class="card__meta-item" title="レビュー">🔍 ${t.reviewPoints.length}</span>` : ''}
          </div>
          ${owner}
        </div>
      </article>
    `;
  }

  // ----- List view -----
  function renderList() {
    const phase = findPhase(state.activePhaseId);
    if (!phase) return;
    const all = tasksForPhase(phase);
    const body = document.getElementById('listBody');
    body.innerHTML = '';
    if (!all.length) {
      body.innerHTML = `<tr><td colspan="7" style="text-align:center;color:var(--text-3);padding:32px">タスクなし</td></tr>`;
      return;
    }
    all.forEach(t => {
      const tr = document.createElement('tr');
      tr.dataset.id = t.id;
      tr.innerHTML = `
        <td class="row-id">${t.id}</td>
        <td class="row-title">${escapeHTML(t.title)}</td>
        <td><span class="status-pill" data-s="${t.status}">${t.status}</span></td>
        <td><span class="card__prio card__prio--${t.priority}">${t.priority}</span></td>
        <td>${t.owner ? escapeHTML(t.owner) : '—'}</td>
        <td>${(t.linkedDocs || []).map(d => `<span class="tag">${d}</span>`).join(' ') || '—'}</td>
        <td>${(t.reviewPoints || []).map(r => `<span class="tag">${r}</span>`).join(' ') || '—'}</td>
      `;
      tr.addEventListener('click', () => openTaskModal(t.id));
      body.appendChild(tr);
    });
  }

  // ----- Timeline view -----
  function renderTimeline() {
    const phase = findPhase(state.activePhaseId);
    if (!phase) return;
    const all = tasksForPhase(phase);
    const el = document.getElementById('timeline');
    if (!all.length) {
      el.innerHTML = `<div style="text-align:center;color:var(--text-3);padding:48px">タスクがありません</div>`;
      return;
    }
    const maxE = Math.max(...all.map(t => t.estimate || 0), 1);
    el.innerHTML = `
      <div class="timeline__gantt">
        ${all.map(t => {
          const w = ((t.estimate || 0) / maxE) * 100;
          return `
            <div class="timeline__gantt-row">
              <div class="timeline__gantt-label">
                <span class="row-id">${t.id}</span>
                <span>${escapeHTML(t.title)}</span>
              </div>
              <div class="timeline__gantt-bar">
                <span style="left:0; width:${w}%; --pc: var(--priority-${t.priority}, #6b7280)"></span>
              </div>
              <div class="timeline__gantt-est">${t.estimate || 0}h</div>
            </div>
          `;
        }).join('')}
      </div>
    `;
  }

  /* ------------------------------------------------------------------
   * Sprint view (P1 — 核心)
   * ----------------------------------------------------------------*/

  // ----- Sprint helpers -----
  function getSprint(id) { return state.sprints.find(s => s.id === id); }
  function getActiveSprint() { return state.sprints.find(s => s.status === 'active') || null; }
  function getSprintTaskIds(sprintId) {
    const s = getSprint(sprintId);
    return s ? (s.taskIds || []) : [];
  }
  function sprintCapacity(sprint) {
    return (sprint.taskIds || []).reduce((sum, tid) => {
      const t = state.tasks[tid];
      return sum + (t ? (t.estimate || 0) : 0);
    }, 0);
  }
  function sprintDoneHours(sprint) {
    return (sprint.taskIds || []).reduce((sum, tid) => {
      const t = state.tasks[tid];
      return sum + (t && t.status === 'done' ? (t.estimate || 0) : 0);
    }, 0);
  }
  function daysRemaining(sprint) {
    if (!sprint || !sprint.endDate) return null;
    const end = new Date(sprint.endDate + 'T23:59:59');
    const now = new Date();
    return Math.ceil((end - now) / (1000 * 60 * 60 * 24));
  }
  function nextSprintId() {
    const max = state.sprints.reduce((m, s) => {
      const n = parseInt(String(s.id).replace(/^SP-/, ''), 10);
      return isNaN(n) ? m : Math.max(m, n);
    }, 0);
    return 'SP-' + String(max + 1).padStart(3, '0');
  }
  function addDaysISO(date, days) {
    const d = new Date(date + 'T00:00:00');
    d.setDate(d.getDate() + days);
    return d.toISOString().slice(0, 10);
  }
  function isTaskInActiveOrPlannedSprint(taskId) {
    return state.sprints.some(s =>
      (s.status === 'active' || s.status === 'planned') &&
      (s.taskIds || []).includes(taskId)
    );
  }

  // ----- Sprint render -----
  function renderSprint() {
    renderSprintHeader();
    renderSprintBoard();
    renderSprintList();
    renderSprintMetrics();
    renderSprintCeremonies();
  }

  function renderSprintHeader() {
    const el = document.getElementById('sprintHeader');
    const active = getActiveSprint();
    if (!active) {
      el.innerHTML = `
        <div class="sprint-empty">
          <div class="sprint-empty__icon">🏃</div>
          <h2 class="sprint-empty__title">アクティブなスプリントはありません</h2>
          <p class="sprint-empty__desc">「+ 新規」からスプリントを作成して開始してください。Sprint は固定時間ボックス (1-4 週) でタスクを束ねる Scrum 方式のビューです。</p>
          <div class="sprint-empty__actions">
            <button class="btn btn--primary" id="sprintEmptyCreateBtn">+ 新規スプリント作成</button>
          </div>
        </div>
      `;
      document.getElementById('sprintEmptyCreateBtn').addEventListener('click', () => openSprintEditModal(null));
      return;
    }

    const capacity = sprintCapacity(active);
    const done = sprintDoneHours(active);
    const remaining = Math.max(capacity - done, 0);
    const pct = capacity > 0 ? Math.min(Math.round((done / capacity) * 100), 100) : 0;
    const dRem = daysRemaining(active);
    const totalDays = active.durationDays || 14;
    const daysPassed = Math.max(totalDays - (dRem == null ? 0 : dRem), 0);
    const dayPct = totalDays > 0 ? Math.min(Math.round((daysPassed / totalDays) * 100), 100) : 0;

    el.innerHTML = `
      <div class="sprint-header__top">
        <div class="sprint-header__main">
          <div class="sprint-header__eyebrow">
            <span class="sprint-status-badge is-active">進行中</span>
            <span class="sprint-header__id">${active.id}</span>
          </div>
          <h1 class="sprint-header__name">${escapeHTML(active.name)}</h1>
          <p class="sprint-header__goal">${escapeHTML(active.goal || '—')}</p>
        </div>
        <div class="sprint-header__actions">
          <button class="btn btn--ghost" id="ceremoniesToggle">${state.ceremoniesOpen ? '📝 仪式を隠す' : '📝 仪式'}</button>
          <button class="btn btn--ghost" id="metricsToggle">${state.metricsOpen ? '📊 メトリクスを隠す' : '📊 メトリクス'}</button>
          <button class="btn btn--ghost" id="sprintPlanBtn">📋 計画編集</button>
          <button class="btn btn--ghost" id="sprintEditBtn">✏️ 編集</button>
          <button class="btn btn--ghost" id="sprintCompleteBtn">✅ 完了</button>
          <button class="btn btn--ghost btn--danger" id="sprintCancelBtn">❌ 中止</button>
        </div>
      </div>
      <div class="sprint-header__meta">
        <div class="sprint-stat">
          <div class="sprint-stat__label">期間</div>
          <div class="sprint-stat__value">${active.startDate} → ${active.endDate}</div>
        </div>
        <div class="sprint-stat">
          <div class="sprint-stat__label">残り日数</div>
          <div class="sprint-stat__value ${dRem != null && dRem < 0 ? 'is-overdue' : ''}">${dRem == null ? '—' : (dRem < 0 ? `${Math.abs(dRem)} 日超過` : `${dRem} 日`)}</div>
        </div>
        <div class="sprint-stat">
          <div class="sprint-stat__label">タスク</div>
          <div class="sprint-stat__value">${active.taskIds.length} 件</div>
        </div>
        <div class="sprint-stat sprint-stat--wide">
          <div class="sprint-stat__label">工数 (完了 / 総計)</div>
          <div class="sprint-stat__value">${done}h / ${capacity}h</div>
          <div class="sprint-bar">
            <div class="sprint-bar__fill" style="width:${pct}%"></div>
          </div>
        </div>
        <div class="sprint-stat">
          <div class="sprint-stat__label">スプリント進捗 (日数)</div>
          <div class="sprint-stat__value">${dayPct}%</div>
          <div class="sprint-bar">
            <div class="sprint-bar__fill sprint-bar__fill--time" style="width:${dayPct}%"></div>
          </div>
        </div>
      </div>
    `;

    document.getElementById('sprintPlanBtn').addEventListener('click', () => openSprintPlanModal(active.id));
    document.getElementById('sprintEditBtn').addEventListener('click', () => openSprintEditModal(active.id));
    document.getElementById('sprintCompleteBtn').addEventListener('click', () => completeSprint(active.id));
    document.getElementById('sprintCancelBtn').addEventListener('click', () => cancelSprint(active.id));
    document.getElementById('metricsToggle').addEventListener('click', toggleMetrics);
    document.getElementById('ceremoniesToggle').addEventListener('click', toggleCeremonies);
  }

  function renderSprintBoard() {
    const board = document.getElementById('sprintBoard');
    board.innerHTML = '';
    const active = getActiveSprint();
    if (!active) {
      board.innerHTML = `<div class="kanban-col" style="--c:#6b7280"><header class="kanban-col__head"><div class="kanban-col__title"><span>—</span></div></header><div class="kanban-col__body"><div class="kanban-col__empty">アクティブなスプリントなし</div></div></div>`;
      return;
    }
    const cols = [
      { id: 'backlog', name: 'バックログ', color: '#6b7280' },
      { id: 'todo',    name: 'To Do',     color: '#3b82f6' },
      { id: 'doing',   name: '進行中',    color: '#eab308' },
      { id: 'review',  name: 'レビュー',  color: '#a855f7' },
      { id: 'done',    name: '完了',      color: '#22c55e' }
    ];
    const sprintTasks = active.taskIds
      .map(id => state.tasks[id])
      .filter(Boolean);

    cols.forEach(col => {
      const colEl = document.createElement('section');
      colEl.className = 'kanban-col';
      colEl.style.setProperty('--c', col.color);
      colEl.dataset.col = col.id;

      const tasks = sprintTasks
        .filter(t => t.status === col.id)
        .filter(t => !state.filter || (t.title + ' ' + t.desc + ' ' + t.tags.join(' ')).toLowerCase().includes(state.filter.toLowerCase()));

      colEl.innerHTML = `
        <header class="kanban-col__head">
          <div class="kanban-col__title">
            <span class="kanban-col__dot"></span>
            <span>${col.name}</span>
          </div>
          <span class="kanban-col__count">${tasks.length}</span>
        </header>
        <div class="kanban-col__body" data-sprint-dropzone="${col.id}">
          ${tasks.length ? tasks.map(cardHTML).join('') : `<div class="kanban-col__empty">タスクなし</div>`}
        </div>
      `;
      board.appendChild(colEl);
    });

    // Drag & drop (sprint-specific dropzones)
    board.querySelectorAll('[data-sprint-dropzone]').forEach(zone => {
      zone.addEventListener('dragover', (e) => {
        e.preventDefault();
        zone.closest('.kanban-col').classList.add('is-drop-target');
      });
      zone.addEventListener('dragleave', () => {
        zone.closest('.kanban-col').classList.remove('is-drop-target');
      });
      zone.addEventListener('drop', (e) => {
        e.preventDefault();
        zone.closest('.kanban-col').classList.remove('is-drop-target');
        const id = e.dataTransfer.getData('text/plain');
        const sp = getActiveSprint();
        if (!sp) return;
        if (id && state.tasks[id] && sp.taskIds.includes(id)) {
          state.tasks[id].status = zone.dataset.sprintDropzone;
          recordSprintSnapshot(sp.id);
          save();
          renderSprint();
          toast(`${id} → ${zone.dataset.sprintDropzone}`);
        }
      });
    });

    // Card click
    board.querySelectorAll('.card').forEach(card => {
      card.addEventListener('click', (e) => {
        if (e.target.closest('button')) return;
        openTaskModal(card.dataset.id);
      });
    });
  }

  function renderSprintList() {
    const el = document.getElementById('sprintList');
    if (!state.sprints.length) {
      el.innerHTML = `<li class="sprint-list__empty">スプリントがありません。「+ 新規」で作成してください。</li>`;
      return;
    }
    const groups = [
      { key: 'active',    title: '進行中', color: '#22c55e' },
      { key: 'planned',   title: '計画中', color: '#3b82f6' },
      { key: 'completed', title: '完了',   color: '#94a3b8' },
      { key: 'cancelled', title: '取消',   color: '#ef4444' }
    ];
    el.innerHTML = groups.map(g => {
      const items = state.sprints.filter(s => s.status === g.key)
        .sort((a, b) => (b.createdAt || '').localeCompare(a.createdAt || ''));
      if (!items.length) return '';
      return `
        <li class="sprint-group">
          <div class="sprint-group__head" style="--c:${g.color}">
            <span class="dot"></span>
            <span>${g.title}</span>
            <span class="sprint-group__count">${items.length}</span>
          </div>
          <ul class="sprint-group__list">
            ${items.map(s => `
              <li class="sprint-item ${state.activeSprintId === s.id ? 'is-active' : ''}" data-sprint-id="${s.id}">
                <div class="sprint-item__id">${s.id}</div>
                <div class="sprint-item__main">
                  <div class="sprint-item__name">${escapeHTML(s.name)}</div>
                  <div class="sprint-item__meta">${s.startDate || '—'} → ${s.endDate || '—'} · ${(s.taskIds || []).length} 件</div>
                </div>
                ${s.status === 'completed' && s.velocity != null ? `<div class="sprint-item__vel">${s.velocity}h</div>` : ''}
              </li>
            `).join('')}
          </ul>
        </li>
      `;
    }).join('');

    el.querySelectorAll('.sprint-item').forEach(item => {
      item.addEventListener('click', () => {
        const s = getSprint(item.dataset.sprintId);
        if (!s) return;
        if (s.status === 'active') {
          // 進行中は自動的に active になる
          renderSprint();
          toast(`${s.id}: 進行中`);
        } else if (s.status === 'planned') {
          startSprint(s.id);
        } else {
          openSprintEditModal(s.id);
        }
      });
    });
  }

  // ----- Sprint CRUD -----
  function openSprintEditModal(sprintId) {
    const isNew = !sprintId;
    const draft = isNew
      ? {
          id: nextSprintId(),
          name: '',
          goal: '',
          startDate: new Date().toISOString().slice(0, 10),
          durationDays: 14,
          status: 'planned',
          taskIds: [],
          createdAt: new Date().toISOString()
        }
      : { ...getSprint(sprintId) };

    document.getElementById('sprintEditTitle').textContent = isNew ? 'スプリント作成' : `スプリント編集: ${draft.name}`;
    const body = document.getElementById('sprintEditBody');
    body.innerHTML = `
      <div class="form-group">
        <label class="form-label">スプリント名 <span class="req">*</span></label>
        <input class="form-input" id="seName" type="text" value="${escapeAttr(draft.name)}" placeholder="例: Sprint 1: 認証 + タスク CRUD">
      </div>
      <div class="form-group">
        <label class="form-label">ゴール (Sprint Goal)</label>
        <textarea class="form-textarea" id="seGoal" placeholder="このスプリントで達成したいこと (例: バックエンド API の認証 + 基本 CRUD 完成)">${escapeHTML(draft.goal || '')}</textarea>
      </div>
      <div class="form-row">
        <div class="form-group">
          <label class="form-label">開始日</label>
          <input class="form-input" id="seStart" type="date" value="${draft.startDate || ''}">
        </div>
        <div class="form-group">
          <label class="form-label">期間</label>
          <select class="form-input" id="seDuration">
            <option value="7"  ${draft.durationDays === 7  ? 'selected' : ''}>1 週間 (7日)</option>
            <option value="10" ${draft.durationDays === 10 ? 'selected' : ''}>10 日</option>
            <option value="14" ${draft.durationDays === 14 ? 'selected' : ''}>2 週間 (14日, 推奨)</option>
            <option value="21" ${draft.durationDays === 21 ? 'selected' : ''}>3 週間 (21日)</option>
            <option value="28" ${draft.durationDays === 28 ? 'selected' : ''}>4 週間 (28日)</option>
          </select>
        </div>
        <div class="form-group">
          <label class="form-label">終了日 (自動計算)</label>
          <input class="form-input" id="seEnd" type="text" value="${draft.endDate || addDaysISO(draft.startDate, draft.durationDays)}" disabled>
        </div>
      </div>
      <div class="form-actions">
        ${!isNew && draft.status !== 'completed' ? '<button class="task-detail__btn" id="seDeleteBtn" style="background:rgba(239,68,68,0.15);border-color:rgba(239,68,68,0.3);color:#fca5a5">🗑️ 削除</button>' : ''}
        <div style="flex:1"></div>
        <button class="task-detail__btn" data-close="sprintEdit">キャンセル</button>
        <button class="task-detail__btn is-primary" id="seSaveBtn">${isNew ? '作成' : '保存'}</button>
      </div>
    `;

    // Auto-calc end date
    const startInp = body.querySelector('#seStart');
    const durSel = body.querySelector('#seDuration');
    const endInp = body.querySelector('#seEnd');
    const updateEnd = () => {
      const s = startInp.value;
      const d = parseInt(durSel.value, 10);
      if (s && d) endInp.value = addDaysISO(s, d);
    };
    startInp.addEventListener('change', updateEnd);
    durSel.addEventListener('change', updateEnd);

    body.querySelector('#seSaveBtn').addEventListener('click', () => {
      const name = body.querySelector('#seName').value.trim();
      if (!name) { toast('スプリント名は必須です', 'error'); return; }
      const startDate = body.querySelector('#seStart').value;
      const durationDays = parseInt(body.querySelector('#seDuration').value, 10);
      const goal = body.querySelector('#seGoal').value.trim();
      const endDate = addDaysISO(startDate, durationDays);

      if (isNew) {
        state.sprints.push({
          id: draft.id, name, goal, startDate, endDate, durationDays,
          status: 'planned', taskIds: [], createdAt: new Date().toISOString()
        });
        toast(`スプリント ${draft.id} を作成`);
      } else {
        const s = getSprint(sprintId);
        Object.assign(s, { name, goal, startDate, endDate, durationDays });
        toast(`${s.id} を更新`);
      }
      save();
      renderSprint();
      closeSprintEditModal();
    });

    const delBtn = body.querySelector('#seDeleteBtn');
    if (delBtn) {
      delBtn.addEventListener('click', () => {
        if (!confirm(`${draft.name} を削除しますか?\n配下のタスクは全件 Backlog に戻します (per Jira 設計)。`)) return;
        // Jira 設計: 削除 Sprint の全タスクは Backlog に戻る
        const returned = returnSprintTasksToBacklog(draft);
        state.sprints = state.sprints.filter(s => s.id !== sprintId);
        save();
        renderSprint();
        closeSprintEditModal();
        toast(`${draft.id} を削除 (Backlog 戻り: ${returned} 件)`);
      });
    }

    openModal('sprintEdit');
  }
  function closeSprintEditModal() { closeModal('sprintEdit'); }

  // ----- Sprint planning (task selection) -----
  function openSprintPlanModal(sprintId) {
    const sprint = getSprint(sprintId);
    if (!sprint) return;
    document.getElementById('sprintPlanTitle').textContent = `スプリント計画: ${sprint.name}`;
    const body = document.getElementById('sprintPlanBody');
    const allTaskIds = Object.keys(state.tasks);
    const inSprint = new Set(sprint.taskIds || []);
    const inOtherSprint = new Set();
    state.sprints.forEach(s => {
      if (s.id === sprintId) return;
      if (s.status === 'active' || s.status === 'planned') {
        (s.taskIds || []).forEach(tid => inOtherSprint.add(tid));
      }
    });
    // Jira 設計: Sprint 計画は Backlog (status === 'backlog') からのみ追加可能
    const backlog = allTaskIds.filter(id => {
      const t = state.tasks[id];
      return t && t.status === 'backlog' && !inSprint.has(id) && !inOtherSprint.has(id);
    });
    // 計画済だが Backlog 以外の status のタスク (他 Sprint 移動後の残留) を警告
    const notBacklogInSprint = (sprint.taskIds || []).filter(id => {
      const t = state.tasks[id];
      return t && t.status !== 'backlog';
    });
    const inThis = sprint.taskIds || [];

    const renderCols = () => {
      const inThisSet = new Set(sprint.taskIds || []);
      const inOtherSet = new Set();
      state.sprints.forEach(s => {
        if (s.id === sprintId) return;
        if (s.status === 'active' || s.status === 'planned') {
          (s.taskIds || []).forEach(tid => inOtherSet.add(tid));
        }
      });

      const cap = sprintCapacity(sprint);
      const list = (ids) => ids
        .map(id => state.tasks[id])
        .filter(Boolean)
        .map(t => {
          const inOth = inOtherSet.has(t.id);
          return `
            <li class="plan-task ${inOth ? 'is-locked' : ''}" data-task-id="${t.id}" draggable="${inOth ? 'false' : 'true'}">
              <span class="plan-task__id">${t.id}</span>
              <span class="plan-task__title">${escapeHTML(t.title)}</span>
              <span class="plan-task__prio card__prio card__prio--${t.priority}">${t.priority}</span>
              <span class="plan-task__est">${t.estimate || 0}h</span>
              <button class="plan-task__btn" data-add="${t.id}" ${inOth ? 'disabled' : ''}>${inOth ? '他 Sprint' : '追加 →'}</button>
            </li>
          `;
        }).join('');

      body.innerHTML = `
        <div class="plan-hint">
          <span class="plan-hint__icon">💡</span>
          <span><strong>Jira 設計準拠</strong>: Sprint には Backlog 状態 (status = backlog) のタスクのみ追加できます。Kanban Board で「バックログ」列に戻してから追加してください。</span>
        </div>
        ${notBacklogInSprint.length ? `
        <div class="plan-warn">
          ⚠️ 計画済 ${notBacklogInSprint.length} 件が Backlog 以外のステータスです (Kanban Board で進行中の可能性)。Sprint 内で作業を続けられます。
        </div>` : ''}
        <div class="plan-grid">
          <div class="plan-col">
            <div class="plan-col__head">
              <h3>📥 バックログ</h3>
              <span class="plan-col__count">${backlog.length} 件</span>
            </div>
            <input class="form-input" id="planFilterBacklog" placeholder="検索…" style="margin-bottom:8px">
            <ul class="plan-list" id="planBacklog">
              ${backlog.length ? list(backlog) : '<li class="plan-list__empty">📭 Backlog にタスクがありません。Kanban Board の「バックログ」列でタスクを backlog に戻すと、ここに表示されます。</li>'}
            </ul>
          </div>
          <div class="plan-col">
            <div class="plan-col__head">
              <h3>🎯 ${sprint.id} 計画済</h3>
              <span class="plan-col__count">${inThisSet.size} 件 / ${cap}h</span>
            </div>
            <ul class="plan-list" id="planInSprint">${list(Array.from(inThisSet))}</ul>
          </div>
        </div>
        <div class="form-actions" style="margin-top:16px">
          <div style="flex:1;font-size:12px;color:var(--text-3)">ドラッグ or ボタンで移動 · 他 Sprint に所属するタスクはロック · <strong>外したタスクは Backlog に戻ります</strong></div>
          <button class="task-detail__btn" data-close="sprintPlan">閉じる</button>
        </div>
      `;

      // Search filter (backlog)
      const filterInp = body.querySelector('#planFilterBacklog');
      filterInp.addEventListener('input', () => {
        const q = filterInp.value.toLowerCase();
        body.querySelectorAll('#planBacklog .plan-task').forEach(li => {
          const txt = li.textContent.toLowerCase();
          li.hidden = q && !txt.includes(q);
        });
      });

      // Add buttons
      body.querySelectorAll('[data-add]').forEach(btn => {
        btn.addEventListener('click', (e) => {
          e.stopPropagation();
          const tid = btn.dataset.add;
          addToSprint(sprintId, tid);
        });
      });
      body.querySelectorAll('[data-remove]').forEach(btn => {
        btn.addEventListener('click', (e) => {
          e.stopPropagation();
          const tid = btn.dataset.remove;
          removeFromSprint(sprintId, tid);
        });
      });

      // Drag & drop
      setupPlanDragDrop(sprintId, body);
    };

    renderCols();
    openModal('sprintPlan');
  }

  function setupPlanDragDrop(sprintId, body) {
    let dragId = null;
    body.querySelectorAll('.plan-task').forEach(li => {
      li.addEventListener('dragstart', (e) => {
        dragId = li.dataset.taskId;
        e.dataTransfer.setData('text/plain', dragId);
        e.dataTransfer.effectAllowed = 'move';
        li.classList.add('is-dragging');
      });
      li.addEventListener('dragend', () => li.classList.remove('is-dragging'));
    });
    body.querySelectorAll('.plan-list').forEach(list => {
      list.addEventListener('dragover', (e) => {
        e.preventDefault();
        list.classList.add('is-drop-target');
      });
      list.addEventListener('dragleave', () => list.classList.remove('is-drop-target'));
      list.addEventListener('drop', (e) => {
        e.preventDefault();
        list.classList.remove('is-drop-target');
        const tid = e.dataTransfer.getData('text/plain');
        if (!tid) return;
        if (list.id === 'planInSprint') {
          addToSprint(sprintId, tid);
        } else {
          removeFromSprint(sprintId, tid);
        }
      });
    });
  }

  function addToSprint(sprintId, taskId) {
    const s = getSprint(sprintId);
    if (!s) return;
    if (s.taskIds.includes(taskId)) return;
    if (isTaskInActiveOrPlannedSprint(taskId)) {
      toast('他 Sprint に既に所属', 'error');
      return;
    }
    // Jira 设计: タスクは Backlog 状態 (status === 'backlog') でのみ Sprint に追加可能
    const t = state.tasks[taskId];
    if (!t) return;
    if (t.status !== 'backlog') {
      toast(`${taskId} は Backlog 状態ではありません。Kanban Board で「バックログ」列に戻してから追加してください。`, 'error');
      return;
    }
    s.taskIds.push(taskId);
    save();
    recordSprintSnapshot(s.id);
    openSprintPlanModal(sprintId);
    renderSprint();
  }
  function removeFromSprint(sprintId, taskId) {
    const s = getSprint(sprintId);
    if (!s) return;
    s.taskIds = s.taskIds.filter(id => id !== taskId);
    // Jira 设计: Sprint から外したタスクは Backlog に戻る
    const t = state.tasks[taskId];
    if (t) t.status = 'backlog';
    save();
    recordSprintSnapshot(s.id);
    openSprintPlanModal(sprintId);
    renderSprint();
  }

  // ヘルパー: Sprint 内の全タスク (完了済以外) を Backlog に戻す (per Jira: 完了 Sprint の未完了タスクは Backlog に戻る)
  function returnSprintTasksToBacklog(sprint, { onlyIncomplete = false } = {}) {
    if (!sprint) return 0;
    let count = 0;
    (sprint.taskIds || []).forEach(tid => {
      const t = state.tasks[tid];
      if (!t) return;
      if (onlyIncomplete && t.status === 'done') return;
      t.status = 'backlog';
      count++;
    });
    return count;
  }

  // ----- Sprint lifecycle -----
  function startSprint(sprintId) {
    if (getActiveSprint()) {
      toast('既に進行中のスプリントがあります。先に完了/中止してください。', 'error');
      return;
    }
    const s = getSprint(sprintId);
    if (!s || s.status !== 'planned') return;
    s.status = 'active';
    state.activeSprintId = s.id;
    s.dailySnapshots = [];  // reset on start
    recordSprintSnapshot(s.id);
    save();
    renderSprint();
    toast(`🏁 ${s.id} 開始`);
  }
  function completeSprint(sprintId) {
    const s = getSprint(sprintId);
    if (!s || s.status !== 'active') return;
    const velocity = sprintDoneHours(s);
    const commitment = sprintCapacity(s);
    const completed = (s.taskIds || []).filter(id => state.tasks[id] && state.tasks[id].status === 'done').length;
    const incomplete = s.taskIds.length - completed;
    if (!confirm(`${s.id} を完了しますか?\n完了タスク: ${completed} / ${s.taskIds.length}\n完了工数: ${velocity}h / 計画 ${commitment}h\n未完了 ${incomplete} 件 は Backlog に戻します (per Jira 設計)`)) return;
    s.status = 'completed';
    s.completedAt = new Date().toISOString();
    s.velocity = velocity;
    state.activeSprintId = null;
    // Jira 設計: 完了 Sprint の未完了タスクは Backlog に戻る
    const returned = returnSprintTasksToBacklog(s, { onlyIncomplete: true });
    save();
    renderSprint();
    toast(`✅ ${s.id} 完了 (Velocity: ${velocity}h, Backlog 戻り: ${returned} 件)`);
  }
  function cancelSprint(sprintId) {
    const s = getSprint(sprintId);
    if (!s) return;
    if (!confirm(`${s.id} を中止しますか?\n配下タスクは全件 Backlog に戻します (per Jira 設計)。他 Sprint で再計画可能。`)) return;
    s.status = 'cancelled';
    s.cancelledAt = new Date().toISOString();
    if (state.activeSprintId === s.id) state.activeSprintId = null;
    // Jira 設計: 中止 Sprint の全タスクは Backlog に戻る
    const returned = returnSprintTasksToBacklog(s);
    s.taskIds = [];
    save();
    renderSprint();
    toast(`❌ ${s.id} 中止 (Backlog 戻り: ${returned} 件)`);
  }

  /* ------------------------------------------------------------------
   * Sprint metrics (P2 — 度量)
   * ----------------------------------------------------------------*/

  // ----- Snapshot helpers -----
  function recordSprintSnapshot(sprintId) {
    const s = getSprint(sprintId);
    if (!s) return;
    if (s.status !== 'active') return;
    s.dailySnapshots = s.dailySnapshots || [];
    const today = new Date().toISOString().slice(0, 10);
    const totalCap = sprintCapacity(s);
    const done = sprintDoneHours(s);
    const remaining = Math.max(totalCap - done, 0);
    const last = s.dailySnapshots[s.dailySnapshots.length - 1];
    if (last && last.date === today) {
      last.remainingHours = remaining;
      last.doneHours = done;
      last.totalCapacity = totalCap;
    } else {
      s.dailySnapshots.push({ date: today, remainingHours: remaining, doneHours: done, totalCapacity: totalCap });
    }
  }

  function sprintCompletionPct(sprint) {
    const total = (sprint.taskIds || []).length;
    if (!total) return 0;
    const done = (sprint.taskIds || []).filter(id => state.tasks[id] && state.tasks[id].status === 'done').length;
    return Math.round((done / total) * 100);
  }

  function teamSprintCapacity(sprint) {
    // 团队规模 × 每周可用工时 × (durationDays / 7)
    const cfg = state.teamConfig || { size: 3, hoursPerWeek: 40 };
    return Math.round(cfg.size * cfg.hoursPerWeek * ((sprint.durationDays || 14) / 7));
  }

  // ----- Render metrics -----
  function renderSprintMetrics() {
    const el = document.getElementById('sprintMetrics');
    if (!el) return;
    el.hidden = !state.metricsOpen;
    if (el.hidden) return;
    // P2 架构 fix: 缓存 (active id + completed count + teamConfig), 数据未变时跳过 re-render
    // 触发重渲染的路径: toggleMetrics / capacity form change / 切换 sprint / 完了 sprint / setView('sprint')
    const activeForKey = getActiveSprint();
    const completedCountForKey = state.sprints.filter(s => s.status === 'completed').length;
    const cfgForKey = state.teamConfig || { size: 3, hoursPerWeek: 40 };
    const renderKey = `${activeForKey?.id || 'NONE'}|${completedCountForKey}|${cfgForKey.size}|${cfgForKey.hoursPerWeek}`;
    if (lastMetricsRenderKey === renderKey) return;
    lastMetricsRenderKey = renderKey;

    const completed = state.sprints
      .filter(s => s.status === 'completed')
      .sort((a, b) => (b.completedAt || '').localeCompare(a.completedAt || ''));
    const active = getActiveSprint();

    el.innerHTML = `
      <div class="sprint-metrics__head">
        <h2 class="sprint-metrics__title">📊 Sprint メトリクス</h2>
        <div class="sprint-metrics__hint">Velocity · Burndown · 履歴 · Capacity</div>
      </div>
      <div class="sprint-metrics__grid">
        <section class="metric-card">
          <header class="metric-card__head">
            <h3>📈 Velocity (最近 5 Sprint)</h3>
            <span class="metric-card__sub">完了工数 (h)</span>
          </header>
          <div class="metric-card__body" id="velocityChart"></div>
        </section>
        <section class="metric-card">
          <header class="metric-card__head">
            <h3>📉 Burndown (現在の Sprint)</h3>
            <span class="metric-card__sub">日次残工数 (h)</span>
          </header>
          <div class="metric-card__body" id="burndownChart">
            ${active ? renderBurndownChart(active) : '<div class="metric-empty">アクティブな Sprint がありません</div>'}
          </div>
        </section>
        <section class="metric-card metric-card--wide">
          <header class="metric-card__head">
            <h3>📋 Sprint 履歴</h3>
            <span class="metric-card__sub">完了 Sprint 全件</span>
          </header>
          <div class="metric-card__body" id="sprintHistory">
            ${renderSprintHistory(completed)}
          </div>
        </section>
        <section class="metric-card">
          <header class="metric-card__head">
            <h3>👥 チーム Capacity</h3>
            <span class="metric-card__sub">Sprint ごとの利用可能工数</span>
          </header>
          <div class="metric-card__body" id="capacityConfig">
            ${renderCapacityConfig()}
          </div>
        </section>
      </div>
    `;

    // Render velocity chart (after DOM is in place)
    const vc = document.getElementById('velocityChart');
    if (vc) vc.innerHTML = renderVelocityChart(completed.slice(0, 5));

    // Bind capacity form
    const sizeInp = el.querySelector('#capSize');
    const hoursInp = el.querySelector('#capHours');
    if (sizeInp) sizeInp.addEventListener('change', () => {
      const n = parseInt(sizeInp.value, 10);
      if (n > 0) { state.teamConfig.size = n; save(); renderSprintMetrics(); }
    });
    if (hoursInp) hoursInp.addEventListener('change', () => {
      const n = parseInt(hoursInp.value, 10);
      if (n > 0) { state.teamConfig.hoursPerWeek = n; save(); renderSprintMetrics(); }
    });
  }

  function renderVelocityChart(sprints) {
    if (!sprints.length) {
      return '<div class="metric-empty">完了した Sprint がまだありません</div>';
    }
    const W = 360, H = 180, padX = 36, padY = 24;
    const maxV = Math.max(...sprints.map(s => s.velocity || 0), 1);
    const barW = (W - padX * 2) / sprints.length;
    const bars = sprints.map((s, i) => {
      const x = padX + i * barW + barW * 0.2;
      const w = barW * 0.6;
      const h = ((s.velocity || 0) / maxV) * (H - padY * 2);
      const y = H - padY - h;
      const label = (s.id || '').replace('SP-', 'S');
      return `
        <g>
          <rect x="${x}" y="${y}" width="${w}" height="${h}" rx="3" fill="url(#velGrad)" class="vel-bar">
            <title>${s.id}: ${s.velocity || 0}h / ${sprintCapacity(s)}h</title>
          </rect>
          <text x="${x + w/2}" y="${y - 4}" text-anchor="middle" class="vel-val">${s.velocity || 0}h</text>
          <text x="${x + w/2}" y="${H - 6}" text-anchor="middle" class="vel-label">${label}</text>
        </g>
      `;
    }).join('');
    return `
      <svg class="chart-svg" viewBox="0 0 ${W} ${H}" xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="xMidYMid meet">
        <defs>
          <linearGradient id="velGrad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#22c55e"/>
            <stop offset="100%" stop-color="#16a34a"/>
          </linearGradient>
        </defs>
        <line x1="${padX}" y1="${H - padY}" x2="${W - padX/2}" y2="${H - padY}" stroke="rgba(255,255,255,0.1)"/>
        <line x1="${padX}" y1="${padY}" x2="${padX}" y2="${H - padY}" stroke="rgba(255,255,255,0.05)"/>
        <text x="${padX - 4}" y="${padY + 4}" text-anchor="end" class="chart-axis">${maxV}h</text>
        <text x="${padX - 4}" y="${H - padY + 4}" text-anchor="end" class="chart-axis">0</text>
        ${bars}
      </svg>
    `;
  }

  function renderBurndownChart(sprint) {
    const W = 360, H = 180, padX = 36, padY = 24;
    const total = sprintCapacity(sprint);
    const days = sprint.durationDays || 14;
    const start = new Date(sprint.startDate + 'T00:00:00');
    const end = new Date(sprint.endDate + 'T23:59:59');
    const today = new Date();
    const elapsed = Math.max(0, Math.min(days, Math.ceil((today - start) / (1000 * 60 * 60 * 24))));

    // Ideal line: linear from (0, total) to (days, 0)
    const idealPts = [
      { x: padX, y: padY },
      { x: padX + (days / Math.max(days, 1)) * (W - padX - 8), y: H - padY }
    ];
    // Actual line: from snapshots (or build from 0..elapsed)
    const snaps = (sprint.dailySnapshots || []).slice().sort((a, b) => a.date.localeCompare(b.date));
    if (!snaps.length && sprint.status === 'active') {
      // Seed an initial snapshot
      snaps.push({ date: sprint.startDate, remainingHours: total, doneHours: 0, totalCapacity: total });
      sprint.dailySnapshots = snaps;
    }
    const actualPts = snaps.map((sn, i) => {
      const dayOffset = Math.ceil((new Date(sn.date + 'T00:00:00') - start) / (1000 * 60 * 60 * 24));
      const x = padX + (dayOffset / Math.max(days, 1)) * (W - padX - 8);
      const y = padY + (sn.remainingHours / Math.max(total, 1)) * (H - padY * 2);
      return { x, y, label: sn.date, val: sn.remainingHours };
    });

    const xAxisDays = Array.from({ length: days + 1 }, (_, i) => i).filter(d => d === 0 || d === days || d % Math.ceil(days / 7) === 0);

    return `
      <svg class="chart-svg" viewBox="0 0 ${W} ${H}" xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="xMidYMid meet">
        <line x1="${padX}" y1="${H - padY}" x2="${W - 8}" y2="${H - padY}" stroke="rgba(255,255,255,0.1)"/>
        <line x1="${padX}" y1="${padY}" x2="${padX}" y2="${H - padY}" stroke="rgba(255,255,255,0.05)"/>
        <text x="${padX - 4}" y="${padY + 4}" text-anchor="end" class="chart-axis">${total}h</text>
        <text x="${padX - 4}" y="${H - padY + 4}" text-anchor="end" class="chart-axis">0</text>
        ${xAxisDays.map(d => {
          const x = padX + (d / Math.max(days, 1)) * (W - padX - 8);
          return `<text x="${x}" y="${H - padY + 12}" text-anchor="middle" class="chart-axis-sm">D${d}</text>`;
        }).join('')}
        <line x1="${idealPts[0].x}" y1="${idealPts[0].y}" x2="${idealPts[1].x}" y2="${idealPts[1].y}" stroke="#94a3b8" stroke-width="1.5" stroke-dasharray="4 3" class="ideal-line"/>
        <text x="${idealPts[1].x - 4}" y="${idealPts[1].y - 4}" text-anchor="end" class="chart-legend">理想</text>
        ${actualPts.length ? `
          <polyline points="${actualPts.map(p => `${p.x},${p.y}`).join(' ')}" fill="none" stroke="#22d3ee" stroke-width="2" class="actual-line"/>
          ${actualPts.map(p => `<circle cx="${p.x}" cy="${p.y}" r="3" fill="#22d3ee"><title>${p.label}: 残 ${p.val}h</title></circle>`).join('')}
        ` : ''}
        ${elapsed > 0 ? `<line x1="${padX + (elapsed / Math.max(days, 1)) * (W - padX - 8)}" y1="${padY}" x2="${padX + (elapsed / Math.max(days, 1)) * (W - padX - 8)}" y2="${H - padY}" stroke="rgba(251, 191, 36, 0.4)" stroke-dasharray="2 2"/>` : ''}
      </svg>
    `;
  }

  function renderSprintHistory(sprints) {
    if (!sprints.length) {
      return '<div class="metric-empty">完了した Sprint がまだありません</div>';
    }
    return `
      <table class="history-table">
        <thead>
          <tr>
            <th>ID</th>
            <th>名称</th>
            <th>期間</th>
            <th>完了率</th>
            <th>Velocity</th>
            <th>達成</th>
          </tr>
        </thead>
        <tbody>
          ${sprints.map(s => {
            const cap = sprintCapacity(s);
            const vel = s.velocity || 0;
            const pct = cap > 0 ? Math.min(Math.round((vel / cap) * 100), 100) : 0;
            const achieved = pct >= 80 ? '✅' : (pct >= 50 ? '⚠️' : '❌');
            return `
              <tr>
                <td class="row-id">${s.id}</td>
                <td>${escapeHTML(s.name)}</td>
                <td>${s.startDate} → ${s.endDate}</td>
                <td>
                  <div class="pct-bar"><div class="pct-bar__fill" style="width:${pct}%"></div></div>
                  <span class="pct-num">${pct}%</span>
                </td>
                <td><strong>${vel}h</strong> / ${cap}h</td>
                <td>${achieved}</td>
              </tr>
            `;
          }).join('')}
        </tbody>
      </table>
    `;
  }

  function renderCapacityConfig() {
    const cfg = state.teamConfig || { size: 3, hoursPerWeek: 40 };
    const active = getActiveSprint();
    const curCap = active ? teamSprintCapacity(active) : (cfg.size * cfg.hoursPerWeek * 2);
    return `
      <div class="capacity-form">
        <div class="capacity-form__row">
          <label>チーム人数</label>
          <input class="form-input" id="capSize" type="number" min="1" max="50" value="${cfg.size}">
        </div>
        <div class="capacity-form__row">
          <label>週あたり工数 (h/人)</label>
          <input class="form-input" id="capHours" type="number" min="1" max="80" value="${cfg.hoursPerWeek}">
        </div>
        <div class="capacity-form__result">
          <div class="capacity-form__formula">${cfg.size} 人 × ${cfg.hoursPerWeek}h × ${active ? (active.durationDays / 7).toFixed(1) : '2.0'} 週</div>
          <div class="capacity-form__value">= <strong>${curCap}h</strong></div>
        </div>
        <div class="capacity-form__hint">現在の Sprint (${active ? active.id : 'なし'}) の Capacity = <strong>${curCap}h</strong>。Sprint 計画時の参照値です。</div>
      </div>
    `;
  }

  function toggleMetrics() {
    state.metricsOpen = !state.metricsOpen;
    store.save(METRICS_OPEN_KEY, state.metricsOpen);
    lastMetricsRenderKey = null;  // 强制 re-render (panel 显隐切换)
    renderSprintMetrics();
    // Update button text
    const btn = document.getElementById('metricsToggle');
    if (btn) btn.textContent = state.metricsOpen ? '📊 メトリクスを隠す' : '📊 メトリクス';
  }

  /* ------------------------------------------------------------------
   * Sprint ceremonies (P3 — 仪式)
   * ----------------------------------------------------------------*/

  // ----- Ceremony helpers -----
  function getOrInitCeremonies(sprint) {
    if (!sprint) return null;
    if (!sprint.ceremonies) {
      sprint.ceremonies = {
        standupNotes: [],   // [{ date, yesterday, today, blockers }]
        reviewNotes: '',    // markdown
        demoTaskIds: [],    // completed tasks for demo
        retrospective: { wentWell: '', toImprove: '', actions: '' }
      };
    }
    return sprint.ceremonies;
  }
  function todayISO() { return new Date().toISOString().slice(0, 10); }
  function formatDateJa(iso) {
    if (!iso) return '';
    const d = new Date(iso + 'T00:00:00');
    return `${d.getFullYear()}/${String(d.getMonth() + 1).padStart(2, '0')}/${String(d.getDate()).padStart(2, '0')}`;
  }
  function getTodayStandup(ceremonies) {
    if (!ceremonies) return null;
    const t = todayISO();
    return ceremonies.standupNotes.find(s => s.date === t) || null;
  }
  function saveStandup(ceremonies, entry) {
    if (!ceremonies) return;
    const idx = ceremonies.standupNotes.findIndex(s => s.date === entry.date);
    if (idx >= 0) ceremonies.standupNotes[idx] = entry;
    else ceremonies.standupNotes.push(entry);
  }

  // ----- Render ceremonies -----
  let lastCeremoniesRenderKey = null;  // P3 self-review fix: 防止在用户输入时 textarea 被 innerHTML 替换导致数据丢失
  let lastMetricsRenderKey = null;     // P2 self-review fix: 同上, 防止 capacity form 输入丢失

  function renderSprintCeremonies() {
    const el = document.getElementById('sprintCeremonies');
    if (!el) return;
    el.hidden = !state.ceremoniesOpen;
    if (el.hidden) return;
    // P3 架构 fix: 缓存 active sprint id, 数据未变时跳过 re-render
    // 触发重渲染的路径: toggleCeremonies / saveStandup / saveRetro / saveGoal / 切换 sprint / setView('sprint')
    const activeForKey = getActiveSprint();
    const renderKey = activeForKey?.id || 'NONE';
    if (lastCeremoniesRenderKey === renderKey) return;
    lastCeremoniesRenderKey = renderKey;

    const active = getActiveSprint();
    if (!active) {
      el.innerHTML = `<div class="ceremony-empty">🏃 アクティブな Sprint がありません。Sprint を開始してから Daily Standup / Review / Retrospective を記録してください。</div>`;
      return;
    }
    const c = getOrInitCeremonies(active);
    const today = todayISO();
    const todayEntry = getTodayStandup(c);
    const sortedStandups = (c.standupNotes || []).slice().sort((a, b) => b.date.localeCompare(a.date));
    const completedTasks = (active.taskIds || [])
      .map(id => state.tasks[id])
      .filter(t => t && t.status === 'done');
    const goalBlock = renderCeremonyGoalBlock(active, c);

    el.innerHTML = `
      <div class="sprint-ceremonies__head">
        <h2 class="sprint-ceremonies__title">📝 Sprint 仪式</h2>
        <div class="sprint-ceremonies__hint">Daily Standup · Review · Retrospective · Goal</div>
      </div>
      <div class="sprint-ceremonies__grid">
        ${goalBlock}
        <section class="ceremony-card">
          <header class="ceremony-card__head">
            <h3>☀️ Daily Standup (今日 ${formatDateJa(today)})</h3>
            <span class="ceremony-card__sub">${sortedStandups.length} 件の履歴</span>
          </header>
          <div class="ceremony-card__body">
            <div class="standup-form">
              <div class="standup-form__row">
                <label>昨日やったこと</label>
                <textarea class="form-textarea" id="standupYesterday" rows="2" placeholder="例: 認証 API 完成 / PR レビュー 2 件">${escapeHTML(todayEntry?.yesterday || '')}</textarea>
              </div>
              <div class="standup-form__row">
                <label>今日やること</label>
                <textarea class="form-textarea" id="standupToday" rows="2" placeholder="例: タスク CRUD API 実装 / 単体テスト">${escapeHTML(todayEntry?.today || '')}</textarea>
              </div>
              <div class="standup-form__row">
                <label>障害・相談事項</label>
                <textarea class="form-textarea" id="standupBlockers" rows="2" placeholder="例: DB スキーマレビュー待ち">${escapeHTML(todayEntry?.blockers || '')}</textarea>
              </div>
              <div class="standup-form__actions">
                <span class="standup-form__hint" id="standupHint">${todayEntry ? '✅ 今日分は保存済' : '未保存'}</span>
                <button class="task-detail__btn is-primary" id="standupSaveBtn">保存</button>
              </div>
            </div>
            ${sortedStandups.length > 1 || (sortedStandups.length === 1 && sortedStandups[0].date !== today) ? `
              <div class="standup-history">
                <h4>📜 過去の Standup</h4>
                ${sortedStandups.filter(s => s.date !== today).slice(0, 7).map(s => `
                  <details class="standup-history__item">
                    <summary>${formatDateJa(s.date)}</summary>
                    <div class="standup-history__body">
                      <div><strong>昨日:</strong> ${escapeHTML(s.yesterday || '—')}</div>
                      <div><strong>今日:</strong> ${escapeHTML(s.today || '—')}</div>
                      <div><strong>障害:</strong> ${escapeHTML(s.blockers || '—')}</div>
                    </div>
                  </details>
                `).join('') || '<div class="ceremony-empty-sm">履歴なし</div>'}
              </div>
            ` : ''}
          </div>
        </section>
        <section class="ceremony-card ceremony-card--wide">
          <header class="ceremony-card__head">
            <h3>🎉 Sprint Review (完了済 ${completedTasks.length} 件 / 計画 ${(active.taskIds || []).length} 件)</h3>
            <span class="ceremony-card__sub">Sprint 完了時に Demo 候補を選択</span>
          </header>
          <div class="ceremony-card__body">
            <div class="review-grid">
              <div class="review-col">
                <h4>✅ 完了したタスク (Demo 候補)</h4>
                <ul class="review-task-list" id="reviewTaskList">
                  ${completedTasks.length ? completedTasks.map(t => {
                    const checked = (c.demoTaskIds || []).includes(t.id);
                    return `
                      <li class="review-task">
                        <label>
                          <input type="checkbox" data-demo="${t.id}" ${checked ? 'checked' : ''}>
                          <span class="review-task__id">${t.id}</span>
                          <span class="review-task__title">${escapeHTML(t.title)}</span>
                          <span class="review-task__prio card__prio card__prio--${t.priority}">${t.priority}</span>
                        </label>
                      </li>
                    `;
                  }).join('') : '<li class="ceremony-empty-sm">完了したタスクがありません</li>'}
                </ul>
              </div>
              <div class="review-col">
                <h4>📝 Review Notes (Markdown)</h4>
                <textarea class="form-textarea" id="reviewNotes" rows="8" placeholder="Demo の流れ · 参加者 · フィードバック ...">${escapeHTML(c.reviewNotes || '')}</textarea>
                <div class="review-col__hint">選択中: <strong id="reviewDemoCount">${(c.demoTaskIds || []).length}</strong> 件 / 全 ${completedTasks.length} 件</div>
              </div>
            </div>
          </div>
        </section>
        <section class="ceremony-card ceremony-card--wide">
          <header class="ceremony-card__head">
            <h3>🔄 Sprint Retrospective (3 列 Markdown)</h3>
            <span class="ceremony-card__sub">Sprint 振り返り · KPT フレームワーク</span>
          </header>
          <div class="ceremony-card__body">
            <div class="retrospective-grid">
              <div class="retrospective-col retrospective-col--good">
                <label>✅ 良かったこと (Keep)</label>
                <textarea class="form-textarea retrospective-textarea" id="retroWentWell" rows="8" placeholder="例:
- Daily Standup 15 分定時で回せた
- ペアプロで属人化解消
- 自動テスト追加で安心してリファクタ">${escapeHTML(c.retrospective.wentWell || '')}</textarea>
              </div>
              <div class="retrospective-col retrospective-col--improve">
                <label>⚠️ 改善すること (Problem)</label>
                <textarea class="form-textarea retrospective-textarea" id="retroToImprove" rows="8" placeholder="例:
- PR レビュー待ちが長い (平均 2 日)
- テスト書く時間がない
- 設計レビューが後手">${escapeHTML(c.retrospective.toImprove || '')}</textarea>
              </div>
              <div class="retrospective-col retrospective-col--action">
                <label>🎯 アクション (Try)</label>
                <textarea class="form-textarea retrospective-textarea" id="retroActions" rows="8" placeholder="例:
- レビュー SLA 24h ルール化
- テスト書く時間を朝 30 分確保
- 設計レビューを実装前に">${escapeHTML(c.retrospective.actions || '')}</textarea>
              </div>
            </div>
            <div class="retrospective-actions">
              <button class="task-detail__btn is-primary" id="retroSaveBtn">Retrospective を保存</button>
              <button class="task-detail__btn" id="retroExportBtn">📋 Markdown エクスポート</button>
            </div>
          </div>
        </section>
      </div>
    `;

    bindCeremonyEvents(active, c);
  }

  function renderCeremonyGoalBlock(sprint, c) {
    return `
      <section class="ceremony-card ceremony-card--goal">
        <header class="ceremony-card__head">
          <h3>🎯 Sprint Goal</h3>
          <span class="ceremony-card__sub">${sprint.startDate} → ${sprint.endDate}</span>
        </header>
        <div class="ceremony-card__body">
          <div class="goal-block">
            <div class="goal-block__label">ゴール (現状)</div>
            <div class="goal-block__text" id="goalDisplay">${escapeHTML(sprint.goal || '—')}</div>
            <textarea class="form-textarea" id="goalEdit" rows="3" placeholder="ゴールを入力 (例: 認証 + タスク CRUD API 完成)" hidden>${escapeHTML(sprint.goal || '')}</textarea>
            <div class="goal-block__actions">
              <button class="task-detail__btn" id="goalEditBtn" style="font-size:11px">✏️ 編集</button>
              <button class="task-detail__btn is-primary" id="goalSaveBtn" style="font-size:11px" hidden>💾 保存</button>
              <button class="task-detail__btn" id="goalCancelBtn" style="font-size:11px" hidden>キャンセル</button>
              <button class="task-detail__btn" id="goalTemplateBtn" style="font-size:11px">📋 起動テンプレ</button>
            </div>
            <div class="goal-block__template" id="goalTemplate" hidden>
              <pre>📌 Sprint Planning Meeting テンプレート
━━━━━━━━━━━━━━━━━━━━━━━━━━
1. 前 Sprint 振り返り (5 min)
   - 達成率: __% / Velocity: __h
2. Product Owner から今 Sprint の目標提示 (5 min)
3. チームでタスク見積もり + コミットメント (15 min)
4. リスク・依存関係の共有 (5 min)
5. 開始宣言 🏁</pre>
              <button class="task-detail__btn" id="goalTemplateClose" style="font-size:11px">閉じる</button>
            </div>
          </div>
        </div>
      </section>
    `;
  }

  function bindCeremonyEvents(sprint, c) {
    // P3 self-review fix: 保存后强制 re-render ceremonies panel (更新履歴列表 + hint 状态)
    const refreshCeremonies = () => {
      lastCeremoniesRenderKey = null;
      renderSprintCeremonies();
    };

    // Standup save
    const standupSave = document.getElementById('standupSaveBtn');
    if (standupSave) {
      standupSave.addEventListener('click', () => {
        const yesterday = document.getElementById('standupYesterday').value.trim();
        const today = document.getElementById('standupToday').value.trim();
        const blockers = document.getElementById('standupBlockers').value.trim();
        if (!yesterday && !today && !blockers) {
          toast('Standup に最低 1 項目は入力してください', 'error');
          return;
        }
        saveStandup(c, { date: todayISO(), yesterday, today, blockers });
        save();
        toast('✅ 今日分の Standup を保存');
        // 在 save 后只更新 hint (避免 re-render 替换 textarea 丢失用户刚保存的输入)
        const hint = document.getElementById('standupHint');
        if (hint) { hint.textContent = '✅ 今日分は保存済'; hint.style.color = '#4ade80'; }
      });
    }

    // Review demo checkboxes
    const reviewTaskList = document.getElementById('reviewTaskList');
    const reviewDemoCount = document.getElementById('reviewDemoCount');
    if (reviewTaskList) {
      reviewTaskList.querySelectorAll('[data-demo]').forEach(cb => {
        cb.addEventListener('change', () => {
          const tid = cb.dataset.demo;
          if (cb.checked) {
            if (!c.demoTaskIds.includes(tid)) c.demoTaskIds.push(tid);
          } else {
            c.demoTaskIds = c.demoTaskIds.filter(id => id !== tid);
          }
          save();
          if (reviewDemoCount) reviewDemoCount.textContent = c.demoTaskIds.length;
        });
      });
    }
    // Review notes (debounced save on blur)
    const reviewNotes = document.getElementById('reviewNotes');
    if (reviewNotes) {
      reviewNotes.addEventListener('blur', () => {
        c.reviewNotes = reviewNotes.value;
        save();
        toast('📝 Review Notes を保存');
      });
    }

    // Retrospective save
    const retroSave = document.getElementById('retroSaveBtn');
    if (retroSave) {
      retroSave.addEventListener('click', () => {
        c.retrospective.wentWell = document.getElementById('retroWentWell').value;
        c.retrospective.toImprove = document.getElementById('retroToImprove').value;
        c.retrospective.actions = document.getElementById('retroActions').value;
        save();
        toast('🔄 Retrospective を保存');
      });
    }
    // Retrospective export
    const retroExport = document.getElementById('retroExportBtn');
    if (retroExport) {
      retroExport.addEventListener('click', () => {
        const md = `# ${sprint.id} ${sprint.name} — Retrospective
期間: ${sprint.startDate} → ${sprint.endDate}
完了日: ${formatDateJa(todayISO())}

## ✅ 良かったこと (Keep)
${document.getElementById('retroWentWell').value || '_(記録なし)_'}

## ⚠️ 改善すること (Problem)
${document.getElementById('retroToImprove').value || '_(記録なし)_'}

## 🎯 アクション (Try)
${document.getElementById('retroActions').value || '_(記録なし)_'}

---
Velocity: ${sprint.velocity || '—'}h / Capacity: ${sprintCapacity(sprint)}h
`;
        const blob = new Blob([md], { type: 'text/markdown' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${sprint.id}-retrospective.md`;
        a.click();
        URL.revokeObjectURL(url);
        toast('📋 Retrospective を Markdown エクスポート');
      });
    }

    // Goal editing
    const goalEditBtn = document.getElementById('goalEditBtn');
    const goalSaveBtn = document.getElementById('goalSaveBtn');
    const goalCancelBtn = document.getElementById('goalCancelBtn');
    const goalDisplay = document.getElementById('goalDisplay');
    const goalEdit = document.getElementById('goalEdit');
    if (goalEditBtn) goalEditBtn.addEventListener('click', () => {
      goalDisplay.hidden = true;
      goalEdit.hidden = false;
      goalEditBtn.hidden = true;
      goalSaveBtn.hidden = false;
      goalCancelBtn.hidden = false;
    });
    if (goalCancelBtn) goalCancelBtn.addEventListener('click', () => {
      goalDisplay.hidden = false;
      goalEdit.hidden = true;
      goalEditBtn.hidden = false;
      goalSaveBtn.hidden = true;
      goalCancelBtn.hidden = true;
    });
    if (goalSaveBtn) goalSaveBtn.addEventListener('click', () => {
      sprint.goal = goalEdit.value.trim();
      save();
      goalDisplay.textContent = sprint.goal || '—';
      toast('🎯 Goal を保存');
      goalCancelBtn.click();
    });
    const goalTemplateBtn = document.getElementById('goalTemplateBtn');
    const goalTemplate = document.getElementById('goalTemplate');
    if (goalTemplateBtn) goalTemplateBtn.addEventListener('click', () => {
      goalTemplate.hidden = !goalTemplate.hidden;
    });
    const goalTemplateClose = document.getElementById('goalTemplateClose');
    if (goalTemplateClose) goalTemplateClose.addEventListener('click', () => {
      goalTemplate.hidden = true;
    });
  }

  function toggleCeremonies() {
    state.ceremoniesOpen = !state.ceremoniesOpen;
    store.save(CEREMONIES_OPEN_KEY, state.ceremoniesOpen);
    lastCeremoniesRenderKey = null;  // 强制 re-render (panel 显隐切换)
    renderSprintCeremonies();
    const btn = document.getElementById('ceremoniesToggle');
    if (btn) btn.textContent = state.ceremoniesOpen ? '📝 仪式を隠す' : '📝 仪式';
  }

  /* ------------------------------------------------------------------
   * View switching
   * ----------------------------------------------------------------*/
  function setView(v) {
    state.view = v;
    document.querySelectorAll('[data-view]').forEach(el => {
      el.hidden = el.dataset.view !== v;
    });
    document.querySelectorAll('.seg__btn').forEach(btn => {
      const isActive = btn.dataset.view === v;
      btn.classList.toggle('is-active', isActive);
      btn.setAttribute('aria-selected', isActive);
    });
    // Sprint view uses full width (hide phasebar)
    const phasebar = document.querySelector('.phasebar');
    if (phasebar) phasebar.hidden = (v === 'sprint');
    if (v === 'kanban') renderKanban();
    if (v === 'list') renderList();
    if (v === 'timeline') renderTimeline();
    if (v === 'sprint') {
      // P3 self-review fix: 切换到 Sprint 视图时强制重渲染 ceremonies/metrics
      // (确保切换回来时看到最新数据; 用户在原视图输入已 save 不会丢失)
      lastCeremoniesRenderKey = null;
      lastMetricsRenderKey = null;
      renderSprint();
    }
  }

  /* ------------------------------------------------------------------
   * Switch phase
   * ----------------------------------------------------------------*/
  function switchPhase(id) {
    state.activePhaseId = id;
    renderVStrip();
    renderPhasebar();
    renderStageHeader();
    setView(state.view);
  }

  /* ------------------------------------------------------------------
   * Task detail modal
   * ----------------------------------------------------------------*/
  function openTaskModal(id) {
    const t = state.tasks[id];
    if (!t) return;
    document.getElementById('taskModalId').textContent = t.id;
    const body = document.getElementById('taskModalBody');
    body.innerHTML = `
      <h2 class="task-detail__title">${escapeHTML(t.title)}</h2>
      <p class="task-detail__desc">${escapeHTML(t.desc)}</p>

      <div class="task-detail__row">
        <div class="task-detail__label">優先度</div>
        <div class="task-detail__value">
          <span class="card__prio card__prio--${t.priority}">${t.priority}</span>
        </div>
      </div>
      <div class="task-detail__row">
        <div class="task-detail__label">ステータス</div>
        <div class="task-detail__value">
          <select class="form-select" id="taskStatusSel" style="width:auto">
            <option value="backlog" ${t.status === 'backlog' ? 'selected' : ''}>バックログ</option>
            <option value="todo"    ${t.status === 'todo'    ? 'selected' : ''}>To Do</option>
            <option value="doing"   ${t.status === 'doing'   ? 'selected' : ''}>進行中</option>
            <option value="review"  ${t.status === 'review'  ? 'selected' : ''}>レビュー</option>
            <option value="done"    ${t.status === 'done'    ? 'selected' : ''}>完了</option>
          </select>
        </div>
      </div>
      <div class="task-detail__row">
        <div class="task-detail__label">担当者</div>
        <div class="task-detail__value">
          <input class="form-input" id="taskOwnerInp" type="text" value="${escapeAttr(t.owner || '')}" placeholder="例: 山田太郎" style="max-width:200px">
        </div>
      </div>
      <div class="task-detail__row">
        <div class="task-detail__label">見積もり</div>
        <div class="task-detail__value">
          <span class="task-detail__pill">⏱ ${t.estimate || 0} 時間</span>
        </div>
      </div>
      <div class="task-detail__row">
        <div class="task-detail__label">タグ</div>
        <div class="task-detail__value">
          ${(t.tags || []).map(tg => `<span class="tag">${escapeHTML(tg)}</span>`).join(' ') || '<span style="color:var(--text-3)">—</span>'}
        </div>
      </div>
      <div class="task-detail__row">
        <div class="task-detail__label">関連成果物</div>
        <div class="task-detail__value">
          ${(t.linkedDocs || []).map(d => `<span class="task-detail__pill">📄 ${d}</span>`).join(' ') || '<span style="color:var(--text-3)">—</span>'}
        </div>
      </div>
      <div class="task-detail__row">
        <div class="task-detail__label">レビュー</div>
        <div class="task-detail__value">
          ${(t.reviewPoints || []).map(r => `<span class="task-detail__pill">🔍 ${r}</span>`).join(' ') || '<span style="color:var(--text-3)">—</span>'}
        </div>
      </div>

      <div class="task-detail__actions">
        <button class="task-detail__btn" id="taskDeleteBtn">🗑️ 削除</button>
        <div style="flex:1"></div>
        <button class="task-detail__btn" data-close="task">閉じる</button>
        <button class="task-detail__btn is-primary" id="taskSaveBtn">保存</button>
      </div>
    `;
    document.getElementById('taskStatusSel').addEventListener('change', (e) => {
      state.tasks[t.id].status = e.target.value;
      save();
      renderAll();
      toast(`${t.id} のステータスを更新`);
    });
    document.getElementById('taskSaveBtn').addEventListener('click', () => {
      const owner = document.getElementById('taskOwnerInp').value.trim();
      state.tasks[t.id].owner = owner || null;
      save();
      renderAll();
      closeTaskModal();
      toast(`${t.id} を保存`);
    });
    document.getElementById('taskDeleteBtn').addEventListener('click', () => {
      if (!confirm(`${t.id} を削除しますか?`)) return;
      delete state.tasks[t.id];
      save();
      renderAll();
      closeTaskModal();
      toast(`${t.id} を削除`);
    });

    openModal('task');
  }

  function closeTaskModal() { closeModal('task'); }

  /* ------------------------------------------------------------------
   * Phase edit modal
   * ----------------------------------------------------------------*/
  function openPhaseEdit(phaseId) {
    const p = findPhase(phaseId);
    if (!p) return;
    const isNew = phaseId === '__new__';
    const draft = isNew
      ? { id: '', num: '', kana: '', name: '', desc: '', color: '#a78bfa', icon: '✨', gradient: '', tasks: [] }
      : { ...p };

    document.getElementById('phaseModalTitle').textContent = isNew ? 'フェーズ追加' : `フェーズ編集: ${p.name}`;
    const body = document.getElementById('phaseModalBody');
    const colors = ['#a78bfa','#818cf8','#22d3ee','#34d399','#fbbf24','#f59e0b','#f97316','#ef4444','#f43f5e','#ec4899','#a855f7','#94a3b8','#64748b','#06b6d4','#84cc16','#fb923c'];
    const icons  = ['🌌','📋','🧩','🔬','🛠️','🧪','🚀','🛡️','🏁','⭐','💎','🎯','🔧','🎨','🧭','⚙️'];
    body.innerHTML = `
      <div class="form-group">
        <label class="form-label">フェーズ名</label>
        <input class="form-input" id="peName" type="text" value="${escapeAttr(draft.name)}" placeholder="例: 性能最適化">
      </div>
      <div class="form-group">
        <label class="form-label">ふりがな</label>
        <input class="form-input" id="peKana" type="text" value="${escapeAttr(draft.kana || '')}" placeholder="例: せいのうさいてきか">
      </div>
      <div class="form-group">
        <label class="form-label">説明</label>
        <textarea class="form-textarea" id="peDesc" placeholder="このフェーズの目的と典型的な成果">${escapeHTML(draft.desc || '')}</textarea>
      </div>
      <div class="form-group">
        <label class="form-label">アイコン</label>
        <div style="display:grid;grid-template-columns:repeat(8,1fr);gap:6px" id="peIcons">
          ${icons.map(ic => `
            <button class="color-swatch" data-icon="${ic}" style="background:var(--bg-3);font-size:18px;display:grid;place-items:center;aspect-ratio:1;border-radius:8px;cursor:pointer;border:2px solid transparent;color:var(--text-0)">${ic}</button>
          `).join('')}
        </div>
      </div>
      <div class="form-group">
        <label class="form-label">テーマカラー</label>
        <div class="color-grid" id="peColors">
          ${colors.map(c => `
            <button class="color-swatch" data-color="${c}" style="background:${c};color:${c}"></button>
          `).join('')}
        </div>
      </div>

      ${!isNew ? `
      <div class="form-actions">
        <button class="task-detail__btn" id="peDeleteBtn" style="background:rgba(239,68,68,0.15);border-color:rgba(239,68,68,0.3);color:#fca5a5">🗑️ このフェーズを削除</button>
        <div style="flex:1"></div>
        <button class="task-detail__btn" data-close="phase">キャンセル</button>
        <button class="task-detail__btn is-primary" id="peSaveBtn">保存</button>
      </div>
      ` : `
      <div class="form-actions">
        <div style="flex:1"></div>
        <button class="task-detail__btn" data-close="phase">キャンセル</button>
        <button class="task-detail__btn is-primary" id="peSaveBtn">追加</button>
      </div>
      `}
    `;

    const updateSwatchSelection = () => {
      body.querySelectorAll('#peColors .color-swatch').forEach(s => {
        s.classList.toggle('is-selected', s.dataset.color === draft.color);
      });
      body.querySelectorAll('#peIcons .color-swatch').forEach(s => {
        s.classList.toggle('is-selected', s.dataset.icon === draft.icon);
      });
    };
    updateSwatchSelection();

    body.querySelectorAll('#peColors .color-swatch').forEach(s => {
      s.addEventListener('click', () => { draft.color = s.dataset.color; updateSwatchSelection(); });
    });
    body.querySelectorAll('#peIcons .color-swatch').forEach(s => {
      s.addEventListener('click', () => { draft.icon = s.dataset.icon; updateSwatchSelection(); });
    });
    body.querySelector('#peSaveBtn').addEventListener('click', () => {
      const name = body.querySelector('#peName').value.trim();
      if (!name) { toast('フェーズ名は必須です', 'error'); return; }
      draft.name = name;
      draft.kana = body.querySelector('#peKana').value.trim();
      draft.desc = body.querySelector('#peDesc').value.trim();
      draft.gradient = `linear-gradient(135deg, ${draft.color} 0%, ${mix(draft.color, '#818cf8', 0.4)} 100%)`;

      if (isNew) {
        const newId = 'CUSTOM-' + (state.phases.length + 1);
        const newNum = String(state.phases.length + 1).padStart(2, '0');
        state.phases.push({
          id: newId, num: newNum, kana: draft.kana, name: draft.name,
          color: draft.color, gradient: draft.gradient, icon: draft.icon,
          desc: draft.desc,
          cols: [
            { id: 'backlog',  name: 'バックログ', color: '#6b7280' },
            { id: 'todo',     name: 'To Do',     color: '#3b82f6' },
            { id: 'doing',    name: '進行中',    color: '#eab308' },
            { id: 'review',   name: 'レビュー',  color: '#a855f7' },
            { id: 'done',     name: '完了',      color: '#22c55e' }
          ],
          tasks: []
        });
        state.activePhaseId = newId;
        toast('フェーズを追加しました');
      } else {
        const p2 = findPhaseStrict(phaseId);
        Object.assign(p2, {
          name: draft.name, kana: draft.kana, desc: draft.desc,
          color: draft.color, gradient: draft.gradient, icon: draft.icon
        });
        toast(`${p2.name} を更新`);
      }
      save();
      renderAll();
      closePhaseModal();
    });
    if (!isNew) {
      body.querySelector('#peDeleteBtn').addEventListener('click', () => {
        if (!confirm(`${draft.name} を削除しますか?\n配下のタスクは保持されます。`)) return;
        const idx = state.phases.findIndex(p => p.id === phaseId);
        if (idx >= 0) {
          // 配下タスクをバックログ化
          const target = state.phases[idx];
          const targetIds = new Set((target.tasks || []).map(t => t.id));
          if (target.subphases) target.subphases.forEach(sp => sp.tasks.forEach(t => targetIds.add(t.id)));
          state.phases.splice(idx, 1);
          if (state.activePhaseId === phaseId) state.activePhaseId = state.phases[0]?.id;
          toast(`${draft.name} を削除`);
          save();
          renderAll();
          closePhaseModal();
        }
      });
    }

    openModal('phase');
  }

  function closePhaseModal() { closeModal('phase'); }

  /* ------------------------------------------------------------------
   * Aux drawer
   * ----------------------------------------------------------------*/
  function openAuxDrawer(key) {
    const aux = AUX[key];
    if (!aux) return;
    document.getElementById('auxEyebrow').textContent = aux.eyebrow;
    document.getElementById('auxTitle').textContent  = aux.title;
    const body = document.getElementById('auxBody');
    body.innerHTML = `
      <p style="font-size:13px;color:var(--text-2);margin-bottom:18px;font-family:'Noto Sans JP',sans-serif">${escapeHTML(aux.subtitle)}</p>
      ${aux.columns.map(col => `
        <section class="aux-section">
          <h3 class="aux-section__title"><span class="dot" style="--c:${col.color};background:${col.color}"></span>${col.name}</h3>
          <div class="aux-list">
            ${col.items.map(it => {
              const meta = [];
              if (it.code)   meta.push(`<span class="aux-row__code">${it.code}</span>`);
              if (it.abbr)   meta.push(`<span class="aux-row__abbr">${it.abbr}</span>`);
              if (it.output) meta.push(`<span class="aux-row__phase">${it.output}</span>`);
              if (it.phase)  meta.push(`<span class="aux-row__phase">P${it.phase}</span>`);
              if (typeof it.gate === 'boolean') meta.push(`<span class="aux-row__gate ${it.gate ? 'is-gate' : ''}">${it.gate ? '🚦 GATE' : '内部'}</span>`);
              if (it.when)   meta.push(`<span class="aux-row__phase">${it.when}</span>`);
              return `
                <div class="aux-row">
                  <div class="aux-row__head">${meta.join('')}</div>
                  <div class="aux-row__name">${escapeHTML(it.name)}</div>
                  ${it.en   ? `<div class="aux-row__en">${escapeHTML(it.en)}</div>` : ''}
                  ${it.desc ? `<div class="aux-row__desc">${escapeHTML(it.desc)}</div>` : ''}
                </div>
              `;
            }).join('')}
          </div>
        </section>
      `).join('')}
    `;
    openModal('aux');
  }

  /* ------------------------------------------------------------------
   * Context menu
   * ----------------------------------------------------------------*/
  function openCtxMenu(x, y, phaseId) {
    const menu = document.getElementById('ctxmenu');
    menu.hidden = false;
    menu.dataset.phase = phaseId;
    // Position with viewport awareness
    const w = 200, h = 280;
    const px = Math.min(x, window.innerWidth - w - 8);
    const py = Math.min(y, window.innerHeight - h - 8);
    menu.style.left = px + 'px';
    menu.style.top  = py + 'px';
  }
  function closeCtxMenu() {
    document.getElementById('ctxmenu').hidden = true;
  }

  document.addEventListener('click', (e) => {
    if (!e.target.closest('.ctxmenu')) closeCtxMenu();
  });
  document.getElementById('ctxmenu').addEventListener('click', (e) => {
    const li = e.target.closest('li');
    if (!li) return;
    const act = li.dataset.act;
    const phaseId = document.getElementById('ctxmenu').dataset.phase;
    closeCtxMenu();
    handleCtxAct(act, phaseId);
  });

  function handleCtxAct(act, phaseId) {
    const idx = state.phases.findIndex(p => p.id === phaseId);
    if (idx < 0) return;
    switch (act) {
      case 'rename':
      case 'recolor':
        openPhaseEdit(phaseId);
        break;
      case 'duplicate': {
        const src = state.phases[idx];
        const copy = deepClone(src);
        copy.id = 'CUSTOM-' + Date.now();
        copy.num = String(state.phases.length + 1).padStart(2, '0');
        copy.name = copy.name + ' (コピー)';
        copy.tasks = (copy.tasks || []).map(t => ({ ...t, id: t.id + '-D' + idx }));
        state.phases.push(copy);
        save(); renderAll();
        toast(`${src.name} を複製`);
        break;
      }
      case 'move-up':
        if (idx > 0) {
          [state.phases[idx-1], state.phases[idx]] = [state.phases[idx], state.phases[idx-1]];
          state.phases.forEach((p, i) => p.num = String(i+1).padStart(2, '0'));
          save(); renderAll();
          toast('フェーズを上に移動');
        }
        break;
      case 'move-down':
        if (idx < state.phases.length - 1) {
          [state.phases[idx+1], state.phases[idx]] = [state.phases[idx], state.phases[idx+1]];
          state.phases.forEach((p, i) => p.num = String(i+1).padStart(2, '0'));
          save(); renderAll();
          toast('フェーズを下に移動');
        }
        break;
      case 'delete': {
        const p = state.phases[idx];
        if (idx === 0) { toast('最初のフェーズは削除できません', 'error'); return; }
        if (!confirm(`${p.name} を削除しますか?`)) return;
        state.phases.splice(idx, 1);
        if (state.activePhaseId === phaseId) state.activePhaseId = state.phases[0]?.id;
        save(); renderAll();
        toast(`${p.name} を削除`);
        break;
      }
    }
  }

  /* ------------------------------------------------------------------
   * Modal open/close
   * ----------------------------------------------------------------*/
  function openModal(kind) {
    const id = modalIdFor(kind);
    document.getElementById(id).classList.add('is-open');
    document.body.style.overflow = 'hidden';
  }
  function closeModal(kind) {
    const id = modalIdFor(kind);
    document.getElementById(id).classList.remove('is-open');
    document.body.style.overflow = '';
  }
  function modalIdFor(kind) {
    if (kind === 'aux') return 'auxDrawer';
    if (kind === 'task') return 'taskModal';
    if (kind === 'phase') return 'phaseModal';
    if (kind === 'sprintEdit') return 'sprintEditModal';
    if (kind === 'sprintPlan') return 'sprintPlanModal';
    return 'phaseModal';
  }
  document.addEventListener('click', (e) => {
    const closer = e.target.closest('[data-close]');
    if (closer) {
      const kind = closer.dataset.close;
      closeModal(kind);
    }
  });
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      ['aux', 'task', 'phase', 'sprintEdit', 'sprintPlan'].forEach(closeModal);
      closeCtxMenu();
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      document.getElementById('searchInput').focus();
    }
  });

  /* ------------------------------------------------------------------
   * Add task (lightweight, in-memory only)
   * ----------------------------------------------------------------*/
  function addTaskToColumn(col) {
    const phase = findPhase(state.activePhaseId);
    if (!phase) return;
    const phaseStrict = findPhaseStrict(phase._parentId || phase.id);
    const target = phaseStrict || phase;
    const nextIdx = (target.tasks || []).length + 1;
    const newId = `${target.id}-NEW${String(nextIdx).padStart(3, '0')}`;
    const t = {
      id: newId,
      title: '新しいタスク',
      desc: 'クリックして詳細を編集してください。',
      priority: 'P2',
      tags: ['新規'],
      linkedDocs: [],
      reviewPoints: [],
      estimate: 4,
      status: col,
      owner: null
    };
    state.tasks[newId] = t;
    if (target.id === phase.id) {
      target.tasks.push(t);
    } else if (target.subphases) {
      const sp = target.subphases.find(s => s.id === phase.id);
      if (sp) sp.tasks.push(t);
    }
    save(); renderAll();
    openTaskModal(newId);
  }

  /* ------------------------------------------------------------------
   * Toast
   * ----------------------------------------------------------------*/
  let toastTimer = null;
  function toast(msg) {
    const el = document.getElementById('toast');
    el.textContent = msg;
    el.hidden = false;
    requestAnimationFrame(() => el.classList.add('is-show'));
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      el.classList.remove('is-show');
      setTimeout(() => { el.hidden = true; }, 250);
    }, 2200);
  }

  /* ------------------------------------------------------------------
   * Save (persist)
   * ----------------------------------------------------------------*/
  function save() {
    store.save(PHASE_STORAGE_KEY, state.phases);
    store.save(TASK_STORAGE_KEY, state.tasks);
    store.save(SPRINT_STORAGE_KEY, state.sprints);
    store.save(TEAM_CONFIG_KEY, state.teamConfig);
  }

  /* ------------------------------------------------------------------
   * Util
   * ----------------------------------------------------------------*/
  function escapeHTML(s) {
    if (s == null) return '';
    return String(s)
      .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
  }
  function escapeAttr(s) {
    if (s == null) return '';
    return String(s).replace(/"/g, '&quot;').replace(/'/g, '&#39;');
  }
  function mix(hex1, hex2, t) {
    const c1 = hexToRgb(hex1), c2 = hexToRgb(hex2);
    const r = Math.round(c1.r * (1 - t) + c2.r * t);
    const g = Math.round(c1.g * (1 - t) + c2.g * t);
    const b = Math.round(c1.b * (1 - t) + c2.b * t);
    return `rgb(${r}, ${g}, ${b})`;
  }
  function hexToRgb(hex) {
    const h = hex.replace('#', '');
    return { r: parseInt(h.slice(0,2),16), g: parseInt(h.slice(2,4),16), b: parseInt(h.slice(4,6),16) };
  }

  /* ------------------------------------------------------------------
   * Bind global events
   * ----------------------------------------------------------------*/
  function bindEvents() {
    document.getElementById('themeToggle').addEventListener('click', () => {
      state.theme = state.theme === 'dark' ? 'light' : 'dark';
      applyTheme(state.theme);
    });

    document.querySelectorAll('.seg__btn').forEach(btn => {
      btn.addEventListener('click', () => setView(btn.dataset.view));
    });

    document.getElementById('searchInput').addEventListener('input', (e) => {
      state.filter = e.target.value;
      if (state.view === 'kanban') renderKanban();
      else if (state.view === 'list') renderList();
      else if (state.view === 'timeline') renderTimeline();
      else if (state.view === 'sprint') renderSprintBoard();
    });

    // Sprint "新規" ボタン
    const sprintCreateBtn = document.getElementById('sprintCreateBtn');
    if (sprintCreateBtn) {
      sprintCreateBtn.addEventListener('click', () => openSprintEditModal(null));
    }

    document.getElementById('addTaskBtn').addEventListener('click', () => {
      addTaskToColumn('todo');
    });

    // Industry switcher
    document.querySelectorAll('.industry-switch__btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const ind = btn.dataset.industry;
        state.industry = ind;
        store.save('vmodel-industry-v1', ind);
        document.querySelectorAll('.industry-switch__btn').forEach(b => b.classList.toggle('is-active', b === btn));
        renderAll();
        toast(`業種切替: ${btn.querySelector('.industry-switch__label').textContent}`);
      });
      // restore active state from saved preference
      if (btn.dataset.industry === state.industry) btn.classList.add('is-active');
      else btn.classList.remove('is-active');
    });

    document.getElementById('addPhaseBtn').addEventListener('click', () => {
      openPhaseEdit('__new__');
    });

    document.getElementById('resetPhases').addEventListener('click', () => {
      if (!confirm('V字モデルの既定構成に戻します。カスタマイズは失われます。続行しますか?')) return;
      state.phases = deepClone(DEFAULT_PHASES);
      state.tasks = buildInitialTasks();
      state.activePhaseId = 'P1';
      save(); renderAll();
      toast('既定に戻しました');
    });

    document.getElementById('exportBtn').addEventListener('click', exportJSON);

    document.querySelectorAll('#auxList li').forEach(li => {
      li.addEventListener('click', () => openAuxDrawer(li.dataset.aux));
    });

    // Drag start (delegation)
    document.addEventListener('dragstart', (e) => {
      const card = e.target.closest('.card');
      if (card) {
        e.dataTransfer.setData('text/plain', card.dataset.id);
        card.classList.add('is-dragging');
      }
    });
    document.addEventListener('dragend', (e) => {
      const card = e.target.closest('.card');
      if (card) card.classList.remove('is-dragging');
    });
  }

  function exportJSON() {
    const blob = new Blob([JSON.stringify({ phases: state.phases, tasks: state.tasks, sprints: state.sprints }, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `vmodel-kanban-${new Date().toISOString().slice(0,10)}.json`;
    a.click();
    URL.revokeObjectURL(url);
    toast('JSON をエクスポートしました');
  }

  /* ------------------------------------------------------------------
   * Render-all
   * ----------------------------------------------------------------*/
  function renderAll() {
    renderVStrip();
    renderPhasebar();
    renderStageHeader();
    setView(state.view);
  }

  /* ------------------------------------------------------------------
   * Init
   * ----------------------------------------------------------------*/
  function init() {
    // Sync activeSprintId from persisted sprints
    const activeSp = getActiveSprint();
    state.activeSprintId = activeSp ? activeSp.id : null;
    bindEvents();
    renderAll();
  }

  document.addEventListener('DOMContentLoaded', init);
})();
