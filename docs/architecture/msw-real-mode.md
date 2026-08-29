# MSW Real-Mode 开关使用指南

> **Status**: 🟢 Active
> **Created**: 2026-08-29
> **Per**: P3-A.7 (commit 6976772) + P3-A.8 文档同步

本文件说明如何在 Star 前端开发中切换 MSW mock 与真实 API。

---

## 0. 一句话定位

> **三档优先级开关: `localStorage > env > 默认 false`; 开启时所有 MSW handler 头部短路到 `realFetch` 真发请求。**

---

## 1. 三档开关 (优先级降序)

### 1.1 localStorage 最高优先级 (运行时)

```js
// 浏览器 DevTools Console
localStorage.setItem('use_real_api', 'true');
localStorage.setItem('real_api_base', 'http://localhost:8080');  // 可选, 默认 localhost:3000
localStorage.setItem('real_api_key', 'sk-test-xxx');             // 可选, Bearer auth
```

| Key | 取值 | 行为 |
|---|---|---|
| `use_real_api` | `"true"` | 启用 real-mode |
| `use_real_api` | `"false"` | 强制 mock, **覆盖 env** |
| `real_api_base` | URL 字符串 | real-mode 时的 base URL |
| `real_api_key` | 任意字符串 | Bearer auth (自动注入) |

### 1.2 env 次优先级 (build-time, Next.js)

```bash
# .env.local
NEXT_PUBLIC_USE_REAL_API=true
NEXT_PUBLIC_API_BASE_URL=https://api.star.example.com
NEXT_PUBLIC_API_KEY=sk-prod-xxx
```

| Env var | 行为 |
|---|---|
| `NEXT_PUBLIC_USE_REAL_API=true` | 启用 real-mode (build 时固化) |
| `NEXT_PUBLIC_USE_REAL_API=false` 或未设 | 走 mock (除非 localStorage 覆盖) |

### 1.3 默认 false

- 无 localStorage 设置 + 无 env 设置 → 走 mock
- 安全默认, dev 体验不受影响

---

## 2. realFetch 自动注入

`realFetch(path, init)` 自动做:

1. **URL 拼接**: `path` 以 `http` 开头则原样用, 否则拼 `${base_url}${path}`
2. **Bearer auth**: 从 localStorage `real_api_key` 读, 注入 `Authorization: Bearer ...`
3. **Content-Type**: 未设且有 body 时, 注入 `application/json`

```ts
import { isRealMode, realFetch } from "@/mocks/real-mode";

if (isRealMode()) {
    const r = await realFetch("/api/cli-profiles", {
        method: "POST",
        body: JSON.stringify({ name: "my-cli" }),
    });
    return r.json();
}
```

---

## 3. cli handler 覆盖范围 (P3-A.7 实证)

| Endpoint | Method | real-mode 短路 |
|---|---|---|
| `/api/cli-profiles` | GET | ✅ |
| `/api/cli-profiles` | POST | ✅ |
| `/api/cli-profiles/:id` | PATCH | ✅ |
| `/api/cli-profiles/:id` | DELETE | ✅ |
| `/api/api-keys` | GET | ✅ |
| `/api/api-keys` | POST | ✅ |
| `/api/api-keys/:id` | DELETE | ✅ |
| `/api/task-windows` | GET | ✅ |
| `/api/task-windows` | POST | ✅ |
| `/api/task-windows/:wid/upload` | POST | ✅ |

**未覆盖** (P3-A.7 §3 缺口 #1): agents / analytics / inbox 3 个 handler 暂未 real-mode 化。

---

## 4. 使用场景

### 4.1 开发期调试真后端

```bash
# 1. 启后端 (Star Rust server)
cd D:/Star && cargo run -p star-server

# 2. 前端 dev server
cd frontend && npm run dev

# 3. 浏览器开 DevTools Console
localStorage.setItem('use_real_api', 'true');
localStorage.setItem('real_api_base', 'http://localhost:8080');

# 4. 刷新页面, 所有 cli/* 请求走真后端
```

### 4.2 CI 跑 mock (节省后端依赖)

- 不设 localStorage + 不设 env → 默认 mock
- CI `frontend-ci` job (P3-A.6) 跑 `npx vitest run` 全 mock 通过

### 4.3 Staging 环境接 staging 后端

```bash
# .env.staging
NEXT_PUBLIC_USE_REAL_API=true
NEXT_PUBLIC_API_BASE_URL=https://api.staging.star.example.com
NEXT_PUBLIC_API_KEY=sk-staging-xxx
```

```bash
# build
NEXT_PUBLIC_USE_REAL_API=true npm run build
```

---

## 5. 调试 / 验证

### 5.1 单元测试 (3 个)

```bash
cd frontend
npx vitest run src/mocks/__tests__/real-mode.test.ts
```

- `default is false when no env and no localStorage`
- `localStorage true overrides everything`
- `localStorage false forces disabled even if env true`

### 5.2 手动验证

```js
// Console
isRealMode()  // false (默认)
localStorage.setItem('use_real_api', 'true');
location.reload();
isRealMode()  // true
// 之后 Network 面板看 /api/cli-profiles 请求:
//   - mock 模式: 状态 200 + 静态 mock JSON
//   - real 模式: 状态 200 + 真后端响应 (可能有 Authorization 头)
```

### 5.3 关闭 real-mode

```js
localStorage.removeItem('use_real_api');
localStorage.removeItem('real_api_base');
localStorage.removeItem('real_api_key');
location.reload();
```

---

## 6. 已知限制 (per P3-A.7 §3 缺口)

| # | 限制 | 影响 | 后续 |
|---|---|---|---|
| 1 | agents/analytics/inbox 3 handler 未 real-mode 化 | 这 3 块仍只能 mock | 单独 wt 推 (低优, ~1M) |
| 2 | `realFetch` 不转 4xx/5xx → MSW HttpResponse 格式 | 上游 500 → 原样抛 | P3-D error wrapper |
| 3 | 无 retry / timeout | 网络差时无降级 | P3-D |
| 4 | localStorage api_key 明文 | 安全风险 (开发期可接受) | P3-E 加密 |
| 5 | 无 UI 状态提示 (用户不知是否开了真 API) | 易混淆 | P3-D UserMenu 状态条 |
| 6 | 无 e2e 验证 (Playwright) | 切换行为未真验证 | P3-D 前端 e2e |

---

## 7. 相关文档

- `docs/architecture/domain-local-runtime.md` — 整体架构
- `frontend/src/mocks/real-mode.ts` — 实现
- `frontend/src/mocks/__tests__/real-mode.test.ts` — 3 test
- `PHASE-P3-A7-IMPL-REPORT.md` — 实施报告

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 三档开关 + realFetch 自动注入 + 10 endpoint 覆盖 + 6 已知限制 | 2026-08-29 11:52 JST P3-A.8 文档同步 |
