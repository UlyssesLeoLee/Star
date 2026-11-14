// =====================================================================
// nema-codec-wasm — wasm-bindgen bindings for frontend NEMA v1.5 codec
// 来源: IDE1.0 (E:\IDE1.0) crates/nema-codec @ a200e5d
//   许可: 权利人授权按 Star MIT 复用 (per REUSE-MAP-COLLAB-SANDBOX-001 §4 v0.2)
// 导出 (JSON 字符串接口, 前端解析为 TS 类型):
//   parseNema(source)      -> NemaDocument JSON
//   renderView(source)     -> adaptive_render_view JSON (Table/5W1H/Graph 行视图)
// =====================================================================
#![allow(missing_docs, unreachable_pub)]

use wasm_bindgen::prelude::*;
use nema_codec::{NemaCodec, StandardNemaCodec};

/// 解析 .nema 源码 → NemaDocument JSON 字符串。
/// 前端: JSON.parse(parseNema(src)) 后按 nema.ts 的 NemaDocument 类型消费。
#[wasm_bindgen(js_name = parseNema)]
pub fn parse_nema_wasm(source: &str) -> Result<String, JsError> {
    let codec = StandardNemaCodec;
    let doc = codec.parse(source).map_err(|e| JsError::new(&e.to_string()))?;
    serde_json::to_string(&doc).map_err(|e| JsError::new(&e.to_string()))
}

/// 自适应渲染视图 (Table / 5W1H / Graph 行视图) → JSON 字符串。
/// 对应 IDE1.0 interpreter::adaptive_render_view。
#[wasm_bindgen(js_name = renderView)]
pub fn render_view_wasm(source: &str) -> Result<String, JsError> {
    let (view, rev, hash, explicit) = nema_codec::interpreter::adaptive_render_view(source);
    let out = serde_json::json!({
        "view": view,
        "revision": rev,
        "hash": hash,
        "explicit_header": explicit,
    });
    serde_json::to_string(&out).map_err(|e| JsError::new(&e.to_string()))
}
