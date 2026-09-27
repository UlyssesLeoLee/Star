//! canvas-game lib: 画布游戏引擎元数据 + axum router builder (per ULYS-160 §4 阶段 1 骨架)
//!
//! 阶段 1: 健康检查 + 服务版本 + 占位 endpoint + HTML 主页 (可视化).
//! 阶段 2+: 角色 / 弹幕 / 战斗 / 3D sprite 游戏逻辑 (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.4).

#![warn(missing_docs)]

use axum::{http::header, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

/// 服务元数据.
#[derive(Debug, Clone, Copy)]
pub struct ServiceMetadata {
    /// 服务名.
    pub name: &'static str,
    /// 服务语义版本.
    pub version: &'static str,
    /// HTTP 监听端口 (per deploy/canvas-game-k3s.yaml containerPort: 8084).
    pub http_port: u16,
    /// 阶段 1 骨架 marker.
    pub phase: Phase,
}

/// 实现阶段 marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    /// 阶段 1: 骨架.
    Skeleton,
    /// 阶段 2+: 业务 logic 已落地.
    Implemented,
}

impl ServiceMetadata {
    /// canvas-game 服务元数据.
    pub const CANVAS_GAME: Self = Self {
        name: "canvas-game",
        version: env!("CARGO_PKG_VERSION"),
        http_port: 8084,
        phase: Phase::Skeleton,
    };
}

/// canvas-game 错误类型.
#[derive(Debug, thiserror::Error)]
pub enum CanvasGameError {
    /// 阶段 1 基础 占位.
    #[error("canvas-game skeleton: not implemented (阶段 2 落地)")]
    NotImplemented,
}

/// 构建 axum Router.
pub fn router() -> Router {
    Router::new()
        .route("/", get(index_html))
        .route("/healthz", get(healthz))
        .route("/ready", get(ready))
        .route("/version", get(version))
        .route("/api/v1/gameplay", get(gameplay_placeholder))
        .route("/api/v1/services", get(services_status))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn ready() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_GAME.name,
        "ready": true,
        "phase": "skeleton",
    }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_GAME.name,
        "version": ServiceMetadata::CANVAS_GAME.version,
        "phase": ServiceMetadata::CANVAS_GAME.phase,
    }))
}

async fn gameplay_placeholder() -> Json<serde_json::Value> {
    Json(json!({
        "endpoint": "/api/v1/gameplay",
        "phase": "skeleton",
        "message": "阶段 2 落地: 角色 / 弹幕 / 战斗 / 3D sprite 游戏逻辑 (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.4)",
    }))
}

/// canvas-game 主页 — HTML 渲染,可交互 (per ULYS-160 §4.4 stage-2 UI preview).
async fn index_html() -> impl IntoResponse {
    let body = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>canvas-game · k3s skeleton</title>
  <style>
    body { font-family: -apple-system, system-ui, sans-serif; background: #0a0e14; color: #e6edf3; margin: 0; padding: 24px; }
    h1 { color: #58a6ff; margin: 0 0 8px 0; }
    h2 { color: #8b949e; border-bottom: 1px solid #21262d; padding-bottom: 8px; margin-top: 32px; }
    .meta { color: #8b949e; font-size: 14px; margin-bottom: 24px; }
    .card { background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 16px; margin: 12px 0; }
    .card .name { color: #58a6ff; font-weight: bold; font-family: ui-monospace, monospace; }
    .card .desc { color: #c9d1d9; font-size: 14px; margin-top: 8px; }
    .card .links a { color: #58a6ff; text-decoration: none; margin-right: 12px; font-size: 13px; }
    .card .links a:hover { text-decoration: underline; }
    button { background: #238636; color: white; border: none; padding: 8px 16px; border-radius: 6px; cursor: pointer; font-size: 14px; }
    button:hover { background: #2ea043; }
    button:disabled { background: #21262d; color: #8b949e; cursor: not-allowed; }
    #console { background: #0d1117; border: 1px solid #30363d; border-radius: 6px; padding: 12px; font-family: ui-monospace, monospace; font-size: 13px; min-height: 120px; white-space: pre-wrap; margin-top: 16px; }
    .ok { color: #3fb950; }
    .err { color: #f85149; }
    .pill { display: inline-block; background: #1f6feb; color: white; padding: 2px 8px; border-radius: 10px; font-size: 11px; margin-left: 8px; }
  </style>
</head>
<body>
  <h1>🎮 canvas-game</h1>
  <div class="meta">
    phase: <span class="pill">Skeleton</span>
    version: <span id="ver">0.1.0</span>
    HTTP: <code>:8084</code>
    branch: <code>dev</code>
  </div>

  <h2>📡 4 Canvas 服务 (k3s cluster)</h2>

  <div class="card">
    <div class="name">canvas-game <span class="pill">本服务</span></div>
    <div class="desc">游戏引擎主服务 — 阶段 1 骨架,占位 endpoint /api/v1/gameplay</div>
    <div class="links">
      <a href="/healthz">/healthz</a>
      <a href="/version">/version</a>
      <a href="/api/v1/gameplay">/api/v1/gameplay</a>
      <a href="/api/v1/services">/api/v1/services</a>
    </div>
  </div>

  <div class="card">
    <div class="name">canvas-engine</div>
    <div class="desc">画布渲染引擎 — HTML5 Canvas / WebGL / 2D drawing primitives</div>
    <div class="links">
      <a href="http://localhost:13080/healthz" target="_blank">:13080/healthz</a>
      <a href="http://localhost:13080/version" target="_blank">:13080/version</a>
    </div>
  </div>

  <div class="card">
    <div class="name">canvas-realtime</div>
    <div class="desc">实时协作 / WebSocket gateway — 多人同步光标 / 共享画布</div>
    <div class="links">
      <a href="http://localhost:15080/healthz" target="_blank">:15080/healthz</a>
      <a href="http://localhost:15080/version" target="_blank">:15080/version</a>
    </div>
  </div>

  <div class="card">
    <div class="name">domain-canvas</div>
    <div class="desc">领域画布 — 项目 / 业务 / 状态聚合</div>
    <div class="links">
      <a href="http://localhost:14080/healthz" target="_blank">:14080/healthz</a>
      <a href="http://localhost:14080/version" target="_blank">:14080/version</a>
    </div>
  </div>

  <h2>🧪 实时 API 测试</h2>
  <button onclick="testApi('/healthz')">GET /healthz</button>
  <button onclick="testApi('/version')">GET /version</button>
  <button onclick="testApi('/api/v1/gameplay')">GET /api/v1/gameplay</button>
  <button onclick="testApi('/api/v1/services')">GET /api/v1/services</button>
  <button onclick="testAll()">🔁 测试全部 4 服务</button>

  <div id="console">[ 点击按钮测试 API ]</div>

  <h2>📋 Star 项目主页链接</h2>
  <div class="card">
    <div class="desc">
      本 UI 是 canvas-game 阶段 2 落地前的可视化 shell.<br>
      完整 HTML 主页 + 3D sprite + 战斗 / 弹幕 UI 见 issue
      <a href="https://github.com/UlyssesLeoLee/Star/issues/160" target="_blank" style="color:#58a6ff">#160 stage 2</a>.
    </div>
  </div>

<script>
async function testApi(path) {
  const log = document.getElementById('console');
  log.textContent = `> GET ${path}\n`;
  try {
    const r = await fetch(path);
    const text = await r.text();
    log.textContent += `< ${r.status} ${r.statusText}\n`;
    try {
      const json = JSON.parse(text);
      log.textContent += JSON.stringify(json, null, 2);
    } catch {
      log.textContent += text;
    }
    log.className = r.ok ? 'ok' : 'err';
  } catch (e) {
    log.textContent += `× ${e}`;
    log.className = 'err';
  }
}

async function testAll() {
  const log = document.getElementById('console');
  log.textContent = '> 测试全部 4 服务\n';
  const endpoints = [
    { url: '/healthz', name: 'canvas-game (本服务)' },
    { url: 'http://localhost:13080/healthz', name: 'canvas-engine' },
    { url: 'http://localhost:15080/healthz', name: 'canvas-realtime' },
    { url: 'http://localhost:14080/healthz', name: 'domain-canvas' },
  ];
  for (const ep of endpoints) {
    try {
      const r = await fetch(ep.url);
      log.textContent += `  ${ep.name.padEnd(30)} ${r.status} ${r.statusText}\n`;
    } catch (e) {
      log.textContent += `  ${ep.name.padEnd(30)} × ${e}\n`;
    }
  }
}

// Load version on page load
fetch('/version').then(r => r.json()).then(j => {
  document.getElementById('ver').textContent = j.version;
  document.getElementById('ver').style.color = '#3fb950';
}).catch(() => {});
</script>
</body>
</html>"#;

    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body,
    )
}

/// 服务发现 — 列出 4 个 canvas service + 其 health endpoint (per ULYS-160 §4).
async fn services_status() -> Json<serde_json::Value> {
    Json(json!({
        "cluster": "k3s",
        "namespace": "star-system",
        "services": [
            {
                "name": "canvas-engine",
                "port": 8080,
                "cluster_ip": "10.43.76.231",
                "endpoint": "/healthz",
                "description": "画布渲染引擎 — Canvas / WebGL 2D drawing"
            },
            {
                "name": "canvas-game",
                "port": 8084,
                "cluster_ip": "10.43.76.79",
                "endpoint": "/healthz",
                "description": "游戏引擎主服务 — 阶段 1 骨架"
            },
            {
                "name": "canvas-realtime",
                "port": 8082,
                "cluster_ip": "10.43.0.79",
                "endpoint": "/healthz",
                "description": "实时协作 / WebSocket gateway"
            },
            {
                "name": "domain-canvas",
                "port": 8081,
                "cluster_ip": "10.43.28.84",
                "endpoint": "/healthz",
                "description": "领域画布 — 项目 / 业务聚合"
            }
        ],
        "phase": "skeleton",
        "message": "阶段 2 落地: canvas-game 完整 HTML UI (per SRS-STAR-CANVAS-GAME-001 §4.4)",
    }))
}