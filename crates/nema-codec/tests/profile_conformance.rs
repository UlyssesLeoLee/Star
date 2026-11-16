/*
@cypher schema=1 source_sha256=a6f993cdec52b5b4f3a408af5ad55c5a26894fca6cfc4fc87a78a251cd8dd131
MERGE (self:File {path:"crates/nema-codec/tests/profile_conformance.rs"})
MERGE (suite:Symbol {id:"crates/nema-codec/tests/profile_conformance.rs::website_matrix",kind:"test"})
MERGE (current_suite:Symbol {id:"crates/nema-codec/tests/profile_conformance.rs::website_matrix_current_snapshot",kind:"test"})
MERGE (semantic:Symbol {id:"crates/nema-codec/tests/profile_conformance.rs::prior_snapshot_semantic_values_match_reference",kind:"test"})
MERGE (current_semantic:Symbol {id:"crates/nema-codec/tests/profile_conformance.rs::current_snapshot_semantic_values_match_reference",kind:"test"})
MERGE (states:Symbol {id:"crates/nema-codec/tests/profile_conformance.rs::four_distinct_values_round_trip",kind:"test"})
MERGE (idempotent:Symbol {id:"crates/nema-codec/tests/profile_conformance.rs::canonical_output_rejects_invalid_input_and_is_idempotent",kind:"test"})
MERGE (limits:Symbol {id:"crates/nema-codec/tests/profile_conformance.rs::oversized_columns_are_rejected_before_expansion",kind:"test"})
MERGE (oracle:Symbol {id:"crates/nema-codec/tests/profile_conformance.rs::local_reference_oracle_has_not_drifted",kind:"test"})
MERGE (current_fixture:File {path:"crates/nema-codec/tests/fixtures/matrix-current.json"})
MERGE (previous_fixture:File {path:"crates/nema-codec/tests/fixtures/matrix.json"})
MERGE (parse:Symbol {id:"crates/nema-codec/src/strict.rs::parse_strict_v1",kind:"function"})
MERGE (format:Symbol {id:"crates/nema-codec/src/strict.rs::format_strict_v1",kind:"function"})
MERGE (parse_v2:Symbol {id:"crates/nema-codec/src/strict.rs::parse_strict_v2",kind:"function"})
MERGE (format_v2:Symbol {id:"crates/nema-codec/src/strict.rs::format_strict_v2",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:DEFINES]->(current_suite)
MERGE (self)-[:DEFINES]->(semantic)
MERGE (self)-[:DEFINES]->(current_semantic)
MERGE (self)-[:DEFINES]->(states)
MERGE (self)-[:DEFINES]->(idempotent)
MERGE (self)-[:DEFINES]->(limits)
MERGE (self)-[:DEFINES]->(oracle)
MERGE (suite)-[:TESTS]->(parse)
MERGE (suite)-[:TESTS]->(format)
MERGE (suite)-[:READS]->(previous_fixture)
MERGE (current_suite)-[:TESTS]->(parse_v2)
MERGE (current_suite)-[:TESTS]->(format_v2)
MERGE (current_suite)-[:READS]->(current_fixture)
MERGE (semantic)-[:TESTS]->(parse)
MERGE (semantic)-[:READS]->(previous_fixture)
MERGE (current_semantic)-[:TESTS]->(parse_v2)
MERGE (current_semantic)-[:READS]->(current_fixture)
MERGE (states)-[:TESTS]->(parse)
MERGE (states)-[:TESTS]->(format)
MERGE (idempotent)-[:TESTS]->(parse)
MERGE (idempotent)-[:TESTS]->(format)
MERGE (limits)-[:TESTS]->(parse)
@endcypher
*/
//! Version-pinned Rust parser and formatter conformance against captured NEMA snapshots.
use nema_codec::strict::{
    format_strict_v1, format_strict_v2, parse_strict_v1, parse_strict_v2, Cell, Mode,
};

#[test]
fn website_matrix() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/matrix.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    let mut seen = 0;
    let mut failures = Vec::new();
    for case in cases {
        let status = case["status"].as_str().unwrap();
        if status != "valid" && status != "error" {
            continue;
        }
        seen += 1;
        let id = case["id"].as_str().unwrap();
        let parsed = parse_strict_v1(case["source"].as_str().unwrap());
        if parsed.is_ok() != (status == "valid") {
            failures.push(format!("{id} expected {status}, got {parsed:?}"));
        }
        if let Ok(doc) = parsed {
            match format_strict_v1(&doc) {
                Ok(s) => assert_eq!(parse_strict_v1(&s).unwrap(), doc, "{id}"),
                Err(e) => failures.push(format!("{id} formatter rejected parsed AST: {e}")),
            }
        }
    }
    assert_eq!(seen, 126);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn website_matrix_current_snapshot() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/matrix-current.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    let mut seen = 0;
    let mut failures = Vec::new();
    for case in cases {
        let status = case["status"].as_str().unwrap();
        if status != "valid" && status != "error" {
            continue;
        }
        seen += 1;
        let id = case["id"].as_str().unwrap();
        let parsed = parse_strict_v2(case["source"].as_str().unwrap());
        if parsed.is_ok() != (status == "valid") {
            failures.push(format!("{id} expected {status}, got {parsed:?}"));
        }
        if let Ok(doc) = parsed {
            match format_strict_v2(&doc) {
                Ok(s) => assert_eq!(parse_strict_v2(&s).unwrap(), doc, "{id}"),
                Err(e) => failures.push(format!("{id} formatter rejected parsed AST: {e}")),
            }
        }
    }
    assert_eq!(seen, 126);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn four_distinct_values_round_trip() {
    let source = " \\1 1\nA ~ \\e \\_\n";
    let doc = parse_strict_v1(source).unwrap();
    assert_eq!(
        doc.domains[0].records[0],
        vec![
            Cell::Value("A".into()),
            Cell::Null,
            Cell::Empty,
            Cell::Missing
        ]
    );
    assert_eq!(
        parse_strict_v1(&format_strict_v1(&doc).unwrap()).unwrap(),
        doc
    );
}

#[test]
fn canonical_output_rejects_invalid_input_and_is_idempotent() {
    assert!(parse_strict_v1(" \\1 1\nA ~\n More\n").is_err());
    let source = " \\1 1\n\\# Full\\ Name Status\nAlice Ready\n Again\n";
    let first = format_strict_v1(&parse_strict_v1(source).unwrap()).unwrap();
    let second = format_strict_v1(&parse_strict_v1(&first).unwrap()).unwrap();
    assert_eq!(first, second);
}

#[test]
fn oversized_columns_are_rejected_before_expansion() {
    assert!(parse_strict_v1(" \\1 1\n\\%4097\n").is_err());
    let too_many_fields = format!(" \\1 1\n{}\n", vec!["a"; 4097].join(" "));
    assert!(parse_strict_v1(&too_many_fields).is_err());
}

#[test]
fn prior_snapshot_semantic_values_match_reference() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/matrix.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        if case["status"] != "valid" {
            continue;
        }
        let id = case["id"].as_str().unwrap();
        let actual = parse_strict_v1(case["source"].as_str().unwrap()).unwrap();
        let expected = case["expected"].as_array().unwrap();
        assert_eq!(actual.domains.len(), expected.len(), "{id}");
        for (domain, e) in actual.domains.iter().zip(expected) {
            let mode = match domain.mode {
                Mode::Table => "table",
                Mode::FiveWOneH => "5w1h",
                Mode::Graph => "graph",
            };
            assert_eq!(mode, e["mode"].as_str().unwrap(), "{id}");
            assert_eq!(domain.id, e["id"].as_u64().unwrap(), "{id}");
            assert_eq!(domain.revision, e["revision"].as_u64().unwrap(), "{id}");
            if domain.mode == Mode::Table {
                assert_eq!(
                    serde_json::to_value(&domain.columns).unwrap(),
                    e["columns"],
                    "{id}"
                );
            }
            let rows = e["records"].as_array().unwrap();
            assert_eq!(domain.records.len(), rows.len(), "{id}");
            for (row, expected_row) in domain.records.iter().zip(rows) {
                let values: Vec<_> = row
                    .iter()
                    .map(|c| match c {
                        Cell::Value(s) => serde_json::json!(s),
                        Cell::Empty => serde_json::json!(""),
                        Cell::Null => serde_json::Value::Null,
                        Cell::Missing => serde_json::json!({"type":"missing"}),
                    })
                    .collect();
                assert_eq!(serde_json::json!(values), expected_row["values"], "{id}");
            }
        }
    }
}

#[test]
fn current_snapshot_semantic_values_match_reference() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/matrix-current.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        if case["status"] != "valid" {
            continue;
        }
        let id = case["id"].as_str().unwrap();
        let actual = parse_strict_v2(case["source"].as_str().unwrap()).unwrap();
        let expected = case["expected"].as_array().unwrap();
        assert_eq!(actual.domains.len(), expected.len(), "{id}");
        for (domain, e) in actual.domains.iter().zip(expected) {
            let mode = match domain.mode {
                Mode::Table => "table",
                Mode::FiveWOneH => "5w1h",
                Mode::Graph => "graph",
            };
            assert_eq!(mode, e["mode"].as_str().unwrap(), "{id}");
            assert_eq!(domain.id, e["id"].as_u64().unwrap(), "{id}");
            assert_eq!(domain.revision, e["revision"].as_u64().unwrap(), "{id}");
            if domain.mode == Mode::Table {
                assert_eq!(
                    serde_json::to_value(&domain.columns).unwrap(),
                    e["columns"],
                    "{id}"
                );
            }
            let rows = e["records"].as_array().unwrap();
            assert_eq!(domain.records.len(), rows.len(), "{id}");
            for (row, expected_row) in domain.records.iter().zip(rows) {
                let values: Vec<_> = row
                    .iter()
                    .map(|cell| match cell {
                        Cell::Value(value) => serde_json::json!(value),
                        Cell::Empty => serde_json::json!(""),
                        Cell::Null => serde_json::Value::Null,
                        Cell::Missing => serde_json::json!({"type":"missing"}),
                    })
                    .collect();
                assert_eq!(serde_json::json!(values), expected_row["values"], "{id}");
            }
        }
    }
}

#[test]
#[ignore = "requires exact local E:/.NEMA files and Python"]
fn local_reference_oracle_has_not_drifted() {
    let script = r#"
import runpy,json,pathlib,hashlib
m=pathlib.Path('E:/.NEMA/website/rule-matrix-data.js')
p=pathlib.Path('E:/.NEMA/interpreter/nema_strict.py')
assert hashlib.sha256(m.read_bytes()).hexdigest()=='ec274eda324c8fc88edc88a4bbc5afe3b508d4b4ca953ead173809aca6ae058a'
assert hashlib.sha256(p.read_bytes()).hexdigest()=='d2f2d2fead79c4f3127a695e8ef5906880c42f5811e7c14c4be5aed5cd851fe8'
parse=runpy.run_path(str(p))['parse']
rules=json.loads(m.read_text(encoding='utf-8').removeprefix('window.NEMA_RULES=').strip().removesuffix(';'))
print(json.dumps({r['id']:parse(r['variant']) for r in rules if r['status']=='valid'}))
"#;
    let output = std::process::Command::new("python")
        .args(["-B", "-c", script])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/matrix-current.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        if case["status"] == "valid" {
            assert_eq!(case["expected"], expected[case["id"].as_str().unwrap()]);
        }
    }
}
