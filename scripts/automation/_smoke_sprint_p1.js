// Sprint view P1 smoke test (one-off, not committed to automation library)
const fs = require('fs');
const path = require('path');

const html = fs.readFileSync(path.join('D:/Star/deliverables/kanban-vmodel-jp/index.html'), 'utf-8');
const checks = [
  ['Sprint tab', /data-view="sprint"/],
  ['Sprint view section', /id="sprintView"/],
  ['Sprint header', /id="sprintHeader"/],
  ['Sprint board', /id="sprintBoard"/],
  ['Sprint list', /id="sprintList"/],
  ['Sprint create btn', /id="sprintCreateBtn"/],
  ['Sprint edit modal', /id="sprintEditModal"/],
  ['Sprint plan modal', /id="sprintPlanModal"/],
];
let pass = 0;
for (const [name, re] of checks) {
  const ok = re.test(html);
  console.log((ok ? 'OK ' : 'FAIL') + ' : ' + name);
  if (ok) pass++;
}
console.log('\n' + pass + '/' + checks.length + ' HTML structure checks passed');

// Try parsing app.js (just syntax check, don't execute)
try {
  const acorn = require('acorn');
  const code = fs.readFileSync(path.join('D:/Star/deliverables/kanban-vmodel-jp/app.js'), 'utf-8');
  acorn.parse(code, { ecmaVersion: 2022, sourceType: 'script' });
  console.log('OK: app.js parses as valid JS (acorn)');
} catch (e) {
  // fall back: try to require it
  try {
    new Function(fs.readFileSync(path.join('D:/Star/deliverables/kanban-vmodel-jp/app.js'), 'utf-8'));
    console.log('OK: app.js parses via Function constructor');
  } catch (e2) {
    console.log('FAIL: app.js parse error: ' + e2.message);
    process.exit(1);
  }
}

// Parse data.js
try {
  new Function(fs.readFileSync(path.join('D:/Star/deliverables/kanban-vmodel-jp/data.js'), 'utf-8'));
  console.log('OK: data.js parses via Function constructor');
} catch (e) {
  console.log('FAIL: data.js parse error: ' + e.message);
  process.exit(1);
}
