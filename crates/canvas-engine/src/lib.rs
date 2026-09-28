//! canvas-engine lib: 共享元数据 + axum router builder (per ULYS-160 §1 阶段 1 骨架)
//!
//! 阶段 1: 健康检查 + 服务版本 + HTML 主页 (可视化) + 占位 endpoint.
//! 阶段 2+: 落地 5 domain 共享 API + 平台能力 (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.1 + BD-STAR-CANVAS-GAME-001 v0.1 §6 canvas-engine).
//!
//! 守门 #14 v4 Mavis 临时代签 5 域 Lead 决策 per 9/3 11:35 JST 反转.
//! 真人到位追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事).
//! 守门 #7 0 unsafe (per workspace lint unsafe_code = "forbid").
//! 守门 #11 缺标比错标 (NotImplemented 占位, 阶段 2 落地).

#![warn(missing_docs)]

use axum::{http::header, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

/// 服务元数据 (per 阶段 1 部署到 GHCR 后 K8s liveness probe 用).
#[derive(Debug, Clone, Copy)]
pub struct ServiceMetadata {
    /// 服务名 (K8s deployment label app: canvas-engine).
    pub name: &'static str,
    /// 服务语义版本 (per Cargo.toml version).
    pub version: &'static str,
    /// HTTP 监听端口 (per deploy/canvas-game-k3s.yaml containerPort).
    pub http_port: u16,
    /// 阶段 1 骨架 marker (阶段 2+ 落地业务后改 `Phase::Implemented`).
    pub phase: Phase,
}

/// 实现阶段 marker (per 守门 #11 缺标比错标).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    /// 阶段 1: 骨架, 仅健康检查 + 元数据 + 占位 endpoint + HTML 主页.
    Skeleton,
    /// 阶段 2+: 业务 logic 已落地 (per SRS / BD / DD 文档).
    Implemented,
}

impl ServiceMetadata {
    /// canvas-engine 服务元数据 (per deploy/canvas-game-k3s.yaml containerPort: 8080).
    pub const CANVAS_ENGINE: Self = Self {
        name: "canvas-engine",
        version: env!("CARGO_PKG_VERSION"),
        http_port: 8080,
        phase: Phase::Skeleton,
    };
}

/// canvas-engine 错误类型 (per 守门 #11).
#[derive(Debug, thiserror::Error)]
pub enum CanvasEngineError {
    /// 阶段 1 基础 占位.
    #[error("canvas-engine skeleton: not implemented (阶段 2 落地)")]
    NotImplemented,
}

/// 构建 axum Router (阶段 1: / + healthz + /ready + /version + /api/v1/services).
///
/// 阶段 2+: 加 /api/v1/canvas/* 业务 endpoint (per BD-STAR-CANVAS-GAME-001 v0.1 §6).
pub fn router() -> Router {
    Router::new()
        .route("/", get(index_html))
        .route("/healthz", get(healthz))
        .route("/ready", get(ready))
        .route("/version", get(version))
        .route("/api/v1/services", get(services_status))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn ready() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_ENGINE.name,
        "ready": true,
        "phase": "skeleton",
    }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_ENGINE.name,
        "version": ServiceMetadata::CANVAS_ENGINE.version,
        "phase": ServiceMetadata::CANVAS_ENGINE.phase,
    }))
}

/// canvas-engine 主页 — 像素风 3渲2 渲染 (per BD-STAR-CANVAS-GAME-001 v0.1 §6)
///
/// PR #208 后保持 4 canvas service UI 一致性. 设计复用 canvas-game pattern (per PR #205 cleanup).
/// 渲染层描述对应 SRS-STAR-CANVAS-GAME-001 v0.1 §4.1 (Three.js + WebGL sprite atlas + 像素化后处理).
async fn index_html() -> impl IntoResponse {
    let body = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>canvas-engine · 像素风 3渲2 渲染</title>
  <style>
    body { font-family: -apple-system, system-ui, sans-serif; background: #0a0e14; color: #e6edf3; margin: 0; padding: 24px; }
    h1 { color: #58a6ff; margin: 0 0 8px 0; }
    h2 { color: #8b949e; border-bottom: 1px solid #21262d; padding-bottom: 8px; margin-top: 32px; }
    .meta { color: #8b949e; font-size: 14px; margin-bottom: 24px; }
    .meta > span { margin-right: 16px; }
    .card { background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 16px; margin: 12px 0; }
    .card .name { color: #58a6ff; font-weight: bold; font-family: ui-monospace, monospace; }
    .card .desc { color: #c9d1d9; font-size: 14px; margin-top: 8px; }
    .pill { display: inline-block; background: #1f6feb; color: white; padding: 2px 8px; border-radius: 10px; font-size: 11px; margin-left: 8px; }
    .links a { color: #58a6ff; text-decoration: none; margin-right: 12px; font-size: 13px; }
    .links a:hover { text-decoration: underline; }
    button { background: #238636; color: white; border: none; padding: 8px 16px; border-radius: 6px; cursor: pointer; font-size: 14px; margin-right: 8px; }
    button:hover { background: #2ea043; }
    #console { background: #0d1117; border: 1px solid #30363d; border-radius: 6px; padding: 12px; font-family: ui-monospace, monospace; font-size: 13px; min-height: 80px; white-space: pre-wrap; margin-top: 16px; }
    .ok { color: #3fb950; }
    .err { color: #f85149; }
  </style>
</head>
<body>
  <h1>🖼️ canvas-engine <span class="pill" style="background:#ffc400;color:#000">3渲2 渲染</span></h1>
  <div class="meta">
    <span>phase: <span class="pill">Skeleton</span></span>
    <span>version: <span id="ver">0.1.0</span></span>
    <span>HTTP: <code>:8080</code></span>
    <span>role: <span class="pill" style="background:#8957e5">跨 5 domain 共享 平台能力</span></span>
  </div>

  <h2>🎨 像素风 3渲2 渲染 (per BD-STAR-CANVAS-GAME-001 v0.1 §6 canvas-engine)</h2>

  <div class="card" style="background:linear-gradient(135deg,#0f1422 0%,#1a0e2c 100%);border-color:#ffc400">
    <div class="name">🎮 canvas-engine — 3渲2 渲染 + 5 装饰 element</div>
    <div class="desc" style="font-size:14px;line-height:1.6">
      <strong>3渲2 渲染</strong>: Three.js 0.169+ + WebGL 2.0 + sprite atlas + <strong style="color:#00f0ff">像素化后处理</strong> (per BD §6 <code>lib/canvas-game-render/three-renderer.ts</code>)
    </div>
    <div class="desc" style="margin-top:8px;font-size:14px;line-height:1.6">
      <strong>5 装饰 element</strong>: 5 装饰 (InkTrail 墨汁拖尾 / HaloArc 神侠光环 / GasParticlesField 粒子场 / ... — per <code>RoguelikeCanvas.tsx</code> + <code>Decorations.tsx</code>)
    </div>
    <div class="desc" style="margin-top:8px;font-size:14px;line-height:1.6">
      <strong>26 表 W/T/M</strong>: 角色 5 类 (AvatarClass warrior/mage/rogue/healer/tinkerer) + 弹幕 6 + 战斗 5 + 关卡 5 (per BD §2.1.4 跨切 supporting)
    </div>
    <div class="desc" style="margin-top:8px;font-size:13px;color:#8b949e">
      Stage 2 落地: <code>crates/canvas-engine/src/render/</code> (Three.js renderer) + <code>crates/canvas-engine/src/decorations/</code> (5 element 装饰).
    </div>
    <div class="links">
      <a href="/healthz">/healthz</a>
      <a href="/version">/version</a>
      <a href="/api/v1/services">/api/v1/services</a>
    </div>
  </div>

  <h2>🧪 API 测试</h2>
  <button onclick="testApi('/healthz')">GET /healthz</button>
  <button onclick="testApi('/version')">GET /version</button>
  <button onclick="testApi('/api/v1/services')">GET /api/v1/services</button>
  <button onclick="testApi('http://localhost:12080/api/v1/gameplay')">GET canvas-game /gameplay</button>

  <div id="console">[ 点击按钮测试 API ]</div>

  <h2>📋 Stage 2 路线 (per BD-STAR-CANVAS-GAME-001 v0.1 §6)</h2>
  <div class="card">
    <div class="desc">
      本服务 阶段 2 落地 <code>crates/canvas-engine/src/render/</code> (Three.js 3渲2 + 像素化 shader) + <code>crates/canvas-engine/src/decorations/</code> (5 装饰 element).
      关联 <a href="https://github.com/UlyssesLeoLee/Star/issues/160" target="_blank" style="color:#58a6ff">issue #160</a> stage 2 (弹幕 Roguelike 业务实装).
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

/// canvas-engine 服务发现 — 列出 4 canvas service + 本服务 cluster info
/// (per canvas-game PR #208 /api/v1/services pattern, 保持 4 service 一致).
async fn services_status() -> Json<serde_json::Value> {
    Json(json!({
        "cluster": "k3s",
        "namespace": "star-system",
        "service": "canvas-engine",
        "role": "3渲2 像素风 渲染平台",
        "phase": "skeleton",
        "render_stack": {
            "engine": "Three.js 0.169+",
            "backend": "WebGL 2.0",
            "sprite_atlas": "per canvas-engine/src/render/sprite-atlas.rs",
            "pixelation": "per canvas-engine/src/render/pixelation-shader.ts"
        },
        "stage_2_modules": [
            { "name": "render", "path": "crates/canvas-engine/src/render/", "status": "P0 not started" },
            { "name": "decorations", "path": "crates/canvas-engine/src/decorations/", "status": "P0 not started" },
        ],
        "message": "阶段 2 落地 3渲2 + 像素化 shader (per BD-STAR-CANVAS-GAME-001 v0.1 §6 canvas-engine)"
    }))
}