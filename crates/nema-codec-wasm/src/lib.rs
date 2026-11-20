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
MERGE (migration_preview:Symbol {id:"crates/nema-codec-wasm/src/lib.rs::preview_nema_migration",kind:"function"})
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
use nema_codec::plugin_runtime::{MigrationPreview, NemaFileBinding};
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

/// Renders a migration preview JSON for one file binding.
///
/// `binding_json` is a [`NemaFileBinding`] (profile + source digest) as JSON;
/// the preview is computed with the source text under the bound profile and
/// the canonical target profile output. Domain `id`/`revision` u64 values are
/// serialized as decimal strings, matching the other profile bridge exports.
#[wasm_bindgen(js_name = previewNemaMigration)]
pub fn preview_nema_migration(
    binding_json: &str,
    source: &str,
    target_profile_json: &str,
) -> Result<String, JsError> {
    let binding: NemaFileBinding = serde_json::from_str(binding_json)
        .map_err(|e| JsError::new(&format!("invalid NEMA file binding: {e}")))?;
    let target = decode_profile(target_profile_json)?;
    let registry = NemaInterpreterRegistry::bundled().map_err(|e| JsError::new(&e.to_string()))?;
    let preview = registry
        .preview_migration(&binding, source, &target)
        .map_err(|e| JsError::new(&e.to_string()))?;
    migration_preview_wasm_json(&preview)
}

fn migration_preview_wasm_json(preview: &MigrationPreview) -> Result<String, JsError> {
    let mut value = serde_json::to_value(preview).map_err(|e| JsError::new(&e.to_string()))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| JsError::new("migration preview JSON is not an object"))?;
    if let Some(ast) = object.get_mut("new_ast") {
        stringify_u64_ids(ast)?;
    }
    serde_json::to_string(&value).map_err(|e| JsError::new(&e.to_string()))
}

fn stringify_u64_ids(value: &mut Value) -> Result<(), JsError> {
    let domains = value
        .get_mut("domains")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| JsError::new("migrated NEMA document has no domains array"))?;
    for domain in domains {
        let object = domain
            .as_object_mut()
            .ok_or_else(|| JsError::new("NEMA domain is not an object"))?;
        for field in ["id", "revision"] {
            let number = object
                .get(field)
                .and_then(Value::as_u64)
                .ok_or_else(|| JsError::new(&format!("NEMA domain {field} is not a u64")))?;
            object.insert(field.into(), Value::String(number.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nema_codec::interpreter_plugin::{
        bundled_profile_for_revision, NEMA_STRICT_V1_REVISION, NEMA_STRICT_V2_REVISION,
    };
    use nema_codec::plugin_runtime::NemaFileBinding;

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

    #[test]
    fn wasm_bridge_matches_rust_registry_for_four_value_states() {
        // The web surface must produce exactly what the Rust registry produces.
        let v1 = bundled_profile_for_revision(NEMA_STRICT_V1_REVISION).unwrap();
        let v2 = bundled_profile_for_revision(NEMA_STRICT_V2_REVISION).unwrap();
        let source = " \\1 1\nA ~ \\e \\_\n";
        let rust_registry =
            nema_codec::interpreter_plugin::NemaInterpreterRegistry::bundled().unwrap();
        for profile in [&v1, &v2] {
            let profile_json = serde_json::to_string(profile).unwrap();
            let bridge_json = parse_nema_with_profile(&profile_json, source).unwrap();
            let bridge_value: Value = serde_json::from_str(&bridge_json).unwrap();

            let rust_doc = rust_registry.parse(profile, source).unwrap();
            let mut rust_value = serde_json::to_value(&rust_doc).unwrap();
            let domains = rust_value["domains"].as_array_mut().unwrap();
            for (raw, domain) in domains.iter_mut().zip(&rust_doc.domains) {
                raw["id"] = Value::String(domain.id.to_string());
                raw["revision"] = Value::String(domain.revision.to_string());
            }
            assert_eq!(bridge_value, rust_value);

            // Format on the bridge and prove the four states survive.
            let formatted = format_nema_with_profile(&profile_json, &bridge_json).unwrap();
            let reparsed = parse_nema_with_profile(&profile_json, &formatted).unwrap();
            assert_eq!(reparsed, bridge_json);
        }
    }

    #[test]
    fn wasm_migration_preview_matches_rust_preview() {
        let v1 = bundled_profile_for_revision(NEMA_STRICT_V1_REVISION).unwrap();
        let v2 = bundled_profile_for_revision(NEMA_STRICT_V2_REVISION).unwrap();
        let source = " \\>1 1\nA LoginAPI\n V2\n";
        let binding = NemaFileBinding::new(v1.clone(), source).unwrap();
        let binding_json = serde_json::to_string(&binding).unwrap();
        let target_json = serde_json::to_string(&v2).unwrap();

        let bridge_json = preview_nema_migration(&binding_json, source, &target_json).unwrap();
        let bridge_value: Value = serde_json::from_str(&bridge_json).unwrap();
        assert_eq!(bridge_value["original_preserved"], Value::Bool(true));
        assert_eq!(
            bridge_value["source_reparses_under_target"],
            Value::Bool(true)
        );

        // Compare against the pure Rust computation with u64 ids as strings.
        let registry = nema_codec::interpreter_plugin::NemaInterpreterRegistry::bundled().unwrap();
        let rust_preview = registry.preview_migration(&binding, source, &v2).unwrap();
        assert_eq!(
            bridge_value["diff_summary"],
            Value::String(rust_preview.diff_summary.clone())
        );
        let rust_ast = serde_json::to_value(&rust_preview.new_ast.unwrap()).unwrap();
        let mut rust_ast = rust_ast;
        for domain in rust_ast["domains"].as_array_mut().unwrap() {
            let id = domain["id"].as_u64().unwrap().to_string();
            let revision = domain["revision"].as_u64().unwrap().to_string();
            domain["id"] = Value::String(id);
            domain["revision"] = Value::String(revision);
        }
        assert_eq!(bridge_value["new_ast"], rust_ast);
        assert_eq!(
            bridge_value["formatted_text"],
            serde_json::to_value(&rust_preview.formatted_text).unwrap()
        );
    }
}
