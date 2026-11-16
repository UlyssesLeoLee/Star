/*
@cypher schema=1 source_sha256=45a741a7bf1e50f4e883f1da18c5eb9fdd9fa9b479c6ec2ac745e32440d14612
MERGE (self:File {path:"crates/nema-codec-wasm/src/lib.rs"})
MERGE (codec:File {path:"crates/nema-codec/src/interpreter_plugin.rs"})
MERGE (strict:File {path:"crates/nema-codec/src/strict.rs"})
MERGE (parse:Symbol {id:"crates/nema-codec-wasm/src/lib.rs::parse_nema_wasm",kind:"function"})
MERGE (render:Symbol {id:"crates/nema-codec-wasm/src/lib.rs::render_view_wasm",kind:"function"})
MERGE (profiles:Symbol {id:"crates/nema-codec-wasm/src/lib.rs::list_nema_interpreter_profiles",kind:"function"})
MERGE (parse_profile:Symbol {id:"crates/nema-codec-wasm/src/lib.rs::parse_nema_with_profile",kind:"function"})
MERGE (format_profile:Symbol {id:"crates/nema-codec-wasm/src/lib.rs::format_nema_with_profile",kind:"function"})
MERGE (self)-[:DEFINES]->(parse)
MERGE (self)-[:DEFINES]->(render)
MERGE (self)-[:DEFINES]->(profiles)
MERGE (self)-[:DEFINES]->(parse_profile)
MERGE (self)-[:DEFINES]->(format_profile)
MERGE (self)-[:IMPORTS]->(codec)
MERGE (self)-[:IMPORTS]->(strict)
MERGE (profiles)-[:CALLS]->(codec)
MERGE (parse_profile)-[:CALLS]->(codec)
MERGE (format_profile)-[:CALLS]->(codec)
@endcypher
*/
// =====================================================================
// nema-codec-wasm — wasm-bindgen bindings for frontend NEMA v1.5 codec
// 来源: IDE1.0 (E:\IDE1.0) crates/nema-codec @ a200e5d
//   许可: 权利人授权按 Star MIT 复用 (per REUSE-MAP-COLLAB-SANDBOX-001 §4 v0.2)
// 导出 (JSON 字符串接口, 前端解析为 TS 类型):
//   parseNema(source)      -> Rust NemaDocument JSON (preserves spans/chunks)
//   renderView(source)     -> adaptive_render_view JSON (Table/5W1H/Graph 行视图)
// =====================================================================
#![allow(missing_docs, unreachable_pub)]

use nema_codec::interpreter_plugin::{InterpreterProfile, NemaInterpreterRegistry};
use nema_codec::strict::Document;
use nema_codec::{NemaCodec, StandardNemaCodec};
use serde_json::{Number, Value};
use wasm_bindgen::prelude::*;

/// 解析 `.nema` 源码并返回 Rust 权威 AST JSON（保留 spans、字典和语义块）。
#[wasm_bindgen(js_name = parseNema)]
pub fn parse_nema_wasm(source: &str) -> Result<String, JsError> {
    let codec = StandardNemaCodec;
    let doc = codec
        .parse(source)
        .map_err(|e| JsError::new(&e.to_string()))?;
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

fn decode_profile(profile_json: &str) -> Result<InterpreterProfile, JsError> {
    serde_json::from_str(profile_json)
        .map_err(|e| JsError::new(&format!("invalid NEMA interpreter profile: {e}")))
}

fn profile_document_json(document: &Document) -> Result<String, JsError> {
    let mut value = serde_json::to_value(document).map_err(|e| JsError::new(&e.to_string()))?;
    let domains = value
        .get_mut("domains")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| JsError::new("NEMA document has no domains array"))?;
    if domains.len() != document.domains.len() {
        return Err(JsError::new(
            "NEMA domain JSON count changed during serialization",
        ));
    }
    for (raw, domain) in domains.iter_mut().zip(&document.domains) {
        let object = raw
            .as_object_mut()
            .ok_or_else(|| JsError::new("NEMA domain JSON is not an object"))?;
        object.insert("id".into(), Value::String(domain.id.to_string()));
        object.insert(
            "revision".into(),
            Value::String(domain.revision.to_string()),
        );
    }
    serde_json::to_string(&value).map_err(|e| JsError::new(&e.to_string()))
}

fn parse_profile_document_json(document_json: &str) -> Result<Document, JsError> {
    let mut value: Value = serde_json::from_str(document_json)
        .map_err(|e| JsError::new(&format!("invalid NEMA document: {e}")))?;
    let domains = value
        .get_mut("domains")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| JsError::new("NEMA document must contain a domains array"))?;
    for domain in domains {
        let object = domain
            .as_object_mut()
            .ok_or_else(|| JsError::new("NEMA domain must be an object"))?;
        for field in ["id", "revision"] {
            let decimal = object.get(field).and_then(Value::as_str).ok_or_else(|| {
                JsError::new(&format!("NEMA domain {field} must be a decimal string"))
            })?;
            let parsed = decimal
                .parse::<u64>()
                .map_err(|_| JsError::new(&format!("NEMA domain {field} is outside u64")))?;
            object.insert(field.into(), Value::Number(Number::from(parsed)));
        }
    }
    serde_json::from_value(value).map_err(|e| JsError::new(&format!("invalid NEMA document: {e}")))
}

/// Lists bundled Rust interpreter profiles. Callers must select one explicitly.
#[wasm_bindgen(js_name = listNemaInterpreterProfiles)]
pub fn list_nema_interpreter_profiles() -> Result<String, JsError> {
    let registry = NemaInterpreterRegistry::bundled().map_err(|e| JsError::new(&e.to_string()))?;
    serde_json::to_string(&registry.profiles()).map_err(|e| JsError::new(&e.to_string()))
}

/// Parses `.nema` source using the caller's exact, pinned interpreter profile.
#[wasm_bindgen(js_name = parseNemaWithProfile)]
pub fn parse_nema_with_profile(profile_json: &str, source: &str) -> Result<String, JsError> {
    let profile = decode_profile(profile_json)?;
    let registry = NemaInterpreterRegistry::bundled().map_err(|e| JsError::new(&e.to_string()))?;
    let document = registry
        .parse(&profile, source)
        .map_err(|e| JsError::new(&e.to_string()))?;
    profile_document_json(&document)
}

/// Formats a strict NEMA document and verifies it reparses under the same profile.
#[wasm_bindgen(js_name = formatNemaWithProfile)]
pub fn format_nema_with_profile(
    profile_json: &str,
    document_json: &str,
) -> Result<String, JsError> {
    let profile = decode_profile(profile_json)?;
    let document = parse_profile_document_json(document_json)?;
    let registry = NemaInterpreterRegistry::bundled().map_err(|e| JsError::new(&e.to_string()))?;
    registry
        .format(&profile, &document)
        .map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nema_codec::interpreter_plugin::NEMA_STRICT_V2_REVISION;

    #[test]
    fn profile_bridge_preserves_u64_ids_as_decimal_strings() {
        let profile =
            nema_codec::interpreter_plugin::bundled_profile_for_revision(NEMA_STRICT_V2_REVISION)
                .unwrap();
        let profile_json = serde_json::to_string(&profile).unwrap();
        let source = " \\9007199254740993 18446744073709551615\nValue\n";

        let parsed_json = parse_nema_with_profile(&profile_json, source).unwrap();
        let parsed_value: Value = serde_json::from_str(&parsed_json).unwrap();
        assert_eq!(
            parsed_value["domains"][0]["id"].as_str(),
            Some("9007199254740993")
        );
        assert_eq!(
            parsed_value["domains"][0]["revision"].as_str(),
            Some("18446744073709551615")
        );
        let formatted = format_nema_with_profile(&profile_json, &parsed_json).unwrap();
        let reparsed_json = parse_nema_with_profile(&profile_json, &formatted).unwrap();
        assert_eq!(reparsed_json, parsed_json);
    }
}
