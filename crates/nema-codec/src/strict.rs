/*
@cypher schema=1 source_sha256=cb1def3c388c3c821f335b926fb46db965bd9d288409333fdce539cef9e15c2a
MERGE (self:File {path:"crates/nema-codec/src/strict.rs"})
MERGE (plugin_file:File {path:"crates/nema-codec/src/interpreter_plugin.rs"})
MERGE (parse:Symbol {id:"crates/nema-codec/src/strict.rs::parse_strict_v1",kind:"function"})
MERGE (parse_v2:Symbol {id:"crates/nema-codec/src/strict.rs::parse_strict_v2",kind:"function"})
MERGE (parse_impl:Symbol {id:"crates/nema-codec/src/strict.rs::parse_strict",kind:"function"})
MERGE (format:Symbol {id:"crates/nema-codec/src/strict.rs::format_strict_v1",kind:"function"})
MERGE (format_v2:Symbol {id:"crates/nema-codec/src/strict.rs::format_strict_v2",kind:"function"})
MERGE (fields:Symbol {id:"crates/nema-codec/src/strict.rs::fields",kind:"function"})
MERGE (graph:Symbol {id:"crates/nema-codec/src/strict.rs::check_graph",kind:"function"})
MERGE (emit:Symbol {id:"crates/nema-codec/src/strict.rs::emit_field",kind:"function"})
MERGE (document:Type {id:"crates/nema-codec/src/strict.rs::Document"})
MERGE (cell:Type {id:"crates/nema-codec/src/strict.rs::Cell"})
MERGE (profile:Type {id:"crates/nema-codec/src/interpreter_plugin.rs::InterpreterProfile"})
MERGE (registry:Type {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterRegistry"})
MERGE (adapter:Symbol {id:"crates/nema-codec/src/interpreter_plugin.rs::NemaInterpreterAdapter",kind:"trait"})
MERGE (self)-[:DEFINES]->(parse)
MERGE (self)-[:DEFINES]->(parse_v2)
MERGE (self)-[:DEFINES]->(parse_impl)
MERGE (self)-[:DEFINES]->(format)
MERGE (self)-[:DEFINES]->(format_v2)
MERGE (self)-[:DEFINES]->(fields)
MERGE (self)-[:DEFINES]->(graph)
MERGE (self)-[:DEFINES]->(emit)
MERGE (self)-[:DEFINES]->(document)
MERGE (self)-[:DEFINES]->(cell)
MERGE (self)-[:IMPORTS]->(plugin_file)
MERGE (plugin_file)-[:DEFINES]->(profile)
MERGE (plugin_file)-[:DEFINES]->(registry)
MERGE (plugin_file)-[:DEFINES]->(adapter)
MERGE (parse)-[:CALLS]->(parse_impl)
MERGE (parse_v2)-[:CALLS]->(parse_impl)
MERGE (parse_impl)-[:CALLS]->(fields)
MERGE (parse_impl)-[:CALLS]->(graph)
MERGE (parse)-[:USES_TYPE]->(document)
MERGE (format)-[:CALLS]->(emit)
MERGE (format)-[:USES_TYPE]->(document)
MERGE (format)-[:CALLS]->(parse)
MERGE (parse_v2)-[:CALLS]->(fields)
MERGE (parse_v2)-[:CALLS]->(graph)
MERGE (parse_v2)-[:USES_TYPE]->(document)
MERGE (format_v2)-[:CALLS]->(format)
MERGE (format_v2)-[:CALLS]->(parse_v2)
MERGE (format_v2)-[:USES_TYPE]->(document)
@endcypher
*/
//! Native Rust NEMA parser and formatter, independently versioned from the legacy codec.
//! v1 snapshot: rule-matrix SHA-256 0e1ae91167227f595f7586fa2174b19cb0eab4458c72e00c5790de770130f7b3;
//! parser SHA-256 88ed46594a2f8af6e2f735f3aa3b68fab42e88cae0a809defc66f8c05cb80891.
//! v2 local snapshot: rule-matrix SHA-256 ec274eda324c8fc88edc88a4bbc5afe3b508d4b4ca953ead173809aca6ae058a;
//! parser SHA-256 d2f2d2fead79c4f3127a695e8ef5906880c42f5811e7c14c4be5aed5cd851fe8.
//! Snapshot inputs are local files; live page revision and source repository status are unverified.
//! These are source-pinned grammar snapshots that can evolve independently.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub(crate) const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_COLUMNS: usize = 4096;
pub(crate) const MAX_DOMAINS: usize = 4096;
pub(crate) const MAX_RECORDS: usize = 100_000;
pub(crate) const MAX_TOTAL_CELLS: usize = 262_144;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Cell {
    Value(String),
    Null,
    Empty,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Table,
    FiveWOneH,
    Graph,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain {
    pub mode: Mode,
    pub id: u64,
    pub revision: u64,
    pub columns: Option<Vec<String>>,
    pub explicit_header: bool,
    pub declared_width: Option<usize>,
    pub records: Vec<Vec<Cell>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub domains: Vec<Domain>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub line: usize,
    pub message: &'static str,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}
impl std::error::Error for Error {}
fn err(line: usize, message: &'static str) -> Error {
    Error { line, message }
}

fn positive(s: &str) -> Option<u64> {
    if s.is_empty() || s.starts_with('0') || !s.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn header(line: &str) -> Option<(Mode, u64, u64)> {
    let mut s = line.strip_prefix(" \\")?;
    let mode = if let Some(rest) = s.strip_prefix('?') {
        s = rest;
        Mode::FiveWOneH
    } else if let Some(rest) = s.strip_prefix('>') {
        s = rest;
        Mode::Graph
    } else {
        Mode::Table
    };
    let (id, revision) = s.split_once(' ')?;
    Some((mode, positive(id)?, positive(revision)?))
}

#[derive(Clone, Copy)]
enum Atom {
    Text(char),
    Empty,
    Missing,
    RawTilde,
}

fn fields(raw: &str, line: usize) -> Result<Vec<Cell>, Error> {
    if raw.is_empty() {
        return Err(err(line, "empty field expression"));
    }
    let mut result = Vec::new();
    let mut atoms = Vec::new();
    let mut it = raw.chars();
    let finish = |atoms: &mut Vec<Atom>, result: &mut Vec<Cell>| -> Result<(), Error> {
        if atoms.is_empty() {
            return Err(err(line, "empty field or consecutive spaces"));
        }
        let cell = match atoms.as_slice() {
            [Atom::RawTilde] => Cell::Null,
            [Atom::Empty] => Cell::Empty,
            [Atom::Missing] => Cell::Missing,
            _ => {
                let mut s = String::new();
                for a in atoms.iter() {
                    match a {
                        Atom::Text(c) => s.push(*c),
                        Atom::RawTilde => s.push('~'),
                        Atom::Empty | Atom::Missing => {
                            return Err(err(line, "special value mixed with text"));
                        }
                    }
                }
                Cell::Value(s)
            }
        };
        if result.len() >= MAX_COLUMNS {
            return Err(err(line, "too many fields"));
        }
        result.push(cell);
        atoms.clear();
        Ok(())
    };
    while let Some(ch) = it.next() {
        match ch {
            ' ' => finish(&mut atoms, &mut result)?,
            '~' => atoms.push(Atom::RawTilde),
            '\\' => {
                let escaped = it.next().ok_or_else(|| err(line, "trailing escape"))?;
                atoms.push(match escaped {
                    'e' => Atom::Empty,
                    '_' => Atom::Missing,
                    'n' => Atom::Text('\n'),
                    'r' => Atom::Text('\r'),
                    't' => Atom::Text('\t'),
                    ' ' | '\\' | '?' | '>' | '<' | '=' | '~' | '^' | '#' | '%' => {
                        Atom::Text(escaped)
                    }
                    _ => return Err(err(line, "unknown escape")),
                });
            }
            c => atoms.push(Atom::Text(c)),
        }
    }
    finish(&mut atoms, &mut result)?;
    if result.len() > MAX_COLUMNS {
        return Err(err(line, "too many fields"));
    }
    Ok(result)
}

fn graph_ident(s: &str) -> bool {
    let mut bytes = s.bytes();
    matches!(bytes.next(), Some(c) if c.is_ascii_alphabetic())
        && bytes.all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b':' | b'-'))
}
fn graph_token(s: &str) -> Option<(&str, Option<(char, &str, &str)>)> {
    let pos = s.find(['>', '<', '=']);
    if let Some(first) = pos {
        let op = s.as_bytes()[first] as char;
        let remain = &s[first + 1..];
        let second = remain.find(['>', '<', '='])?;
        if remain.as_bytes()[second] as char != op {
            return None;
        }
        let a = &s[..first];
        let edge = &remain[..second];
        let b = &remain[second + 1..];
        if !graph_ident(a) || !graph_ident(edge) || !graph_ident(b) {
            return None;
        }
        Some((edge, Some((op, a, b))))
    } else if graph_ident(s) {
        Some((s, None))
    } else {
        None
    }
}
fn check_graph(domain: &Domain) -> Result<(), Error> {
    let mut nodes = HashSet::new();
    let mut edges = HashSet::new();
    let mut endpoints = Vec::new();
    for row in &domain.records {
        let Cell::Value(token) = &row[0] else {
            return Err(err(0, "invalid graph id"));
        };
        let (id, edge) = graph_token(token).ok_or_else(|| err(0, "invalid graph id"))?;
        let pool = if edge.is_some() {
            &mut edges
        } else {
            &mut nodes
        };
        if !pool.insert(id) {
            return Err(err(0, "duplicate graph id"));
        }
        if let Some((_, a, b)) = edge {
            endpoints.push((a, b));
        }
    }
    if !nodes.is_disjoint(&edges) {
        return Err(err(0, "node and edge IDs overlap"));
    }
    if endpoints
        .iter()
        .any(|(a, b)| !nodes.contains(a) || !nodes.contains(b))
    {
        return Err(err(0, "dangling graph endpoint"));
    }
    Ok(())
}

/// Parse a pinned grammar snapshot into a lossless four-state semantic model.
pub fn parse_strict_v1(source: &str) -> Result<Document, Error> {
    parse_strict(source, false)
}

/// Parse the indexed-continuation grammar snapshot as an independent v2 profile.
pub fn parse_strict_v2(source: &str) -> Result<Document, Error> {
    parse_strict(source, true)
}

fn parse_strict(source: &str, indexed_continuations: bool) -> Result<Document, Error> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(err(0, "source too large"));
    }
    if source.contains(['\t', '\r', '\u{3000}']) {
        return Err(err(0, "forbidden raw whitespace"));
    }
    let mut domains: Vec<Domain> = Vec::new();
    // This parse-only provenance distinguishes explicit Missing from implicit padding.
    let mut explicit_missing: Vec<Vec<Vec<bool>>> = Vec::new();
    let mut ids = HashSet::new();
    let mut record_count = 0usize;
    let mut total_cells = 0usize;
    for (index, line) in source.split('\n').enumerate() {
        let n = index + 1;
        if line.is_empty() {
            continue;
        }
        if let Some((mode, id, revision)) = header(line) {
            if domains.len() >= MAX_DOMAINS {
                return Err(err(n, "too many domains"));
            }
            if !ids.insert(id) {
                return Err(err(n, "duplicate domain id"));
            }
            domains.push(Domain {
                mode,
                id,
                revision,
                columns: None,
                explicit_header: false,
                declared_width: None,
                records: Vec::new(),
            });
            explicit_missing.push(Vec::new());
            continue;
        }
        let trimmed = line.trim_start_matches(' ');
        if line.starts_with(' ')
            && trimmed.starts_with('\\')
            && trimmed
                .chars()
                .nth(1)
                .is_some_and(|c| c.is_ascii_digit() || c == '?' || c == '>')
        {
            return Err(err(n, "malformed domain header"));
        }
        let domain = domains
            .last_mut()
            .ok_or_else(|| err(n, "body before domain"))?;
        if domain.mode == Mode::Table && line.starts_with("\\#") {
            if !domain.records.is_empty() || domain.explicit_header {
                return Err(err(n, "header after records or duplicate"));
            }
            let rest = line
                .strip_prefix("\\# ")
                .ok_or_else(|| err(n, "invalid header"))?;
            let mut names = Vec::new();
            for field in fields(rest, n)? {
                match field {
                    Cell::Value(s) if !s.is_empty() => names.push(s),
                    _ => return Err(err(n, "invalid header name")),
                }
            }
            if names.iter().collect::<HashSet<_>>().len() != names.len() {
                return Err(err(n, "duplicate header"));
            }
            if domain.declared_width.is_some_and(|w| w != names.len()) {
                return Err(err(n, "header width mismatch"));
            }
            total_cells = total_cells
                .checked_add(names.len())
                .filter(|x| *x <= MAX_TOTAL_CELLS)
                .ok_or_else(|| err(n, "cell budget exceeded"))?;
            domain.columns = Some(names);
            domain.explicit_header = true;
            continue;
        }
        if domain.mode == Mode::Table && line.starts_with("\\%") {
            if !domain.records.is_empty() || domain.declared_width.is_some() {
                return Err(err(n, "duplicate or late width"));
            }
            let width = positive(&line[2..])
                .and_then(|x| usize::try_from(x).ok())
                .filter(|x| *x <= MAX_COLUMNS)
                .ok_or_else(|| err(n, "invalid width"))?;
            total_cells = total_cells
                .checked_add(width)
                .filter(|x| *x <= MAX_TOTAL_CELLS)
                .ok_or_else(|| err(n, "cell budget exceeded"))?;
            domain.declared_width = Some(width);
            domain.columns = Some((1..=width).map(|i| format!("column{i}")).collect());
            continue;
        }
        let leading = line.len() - trimmed.len();
        if leading > 0 {
            let (col, addition) = if indexed_continuations {
                if leading != 1 {
                    return Err(err(n, "consecutive leading spaces prohibited"));
                }
                let values = fields(trimmed, n)?;
                match values.as_slice() {
                    [Cell::Value(index), value] => {
                        let col = if index == "0" {
                            0
                        } else {
                            positive(index)
                                .and_then(|x| usize::try_from(x).ok())
                                .ok_or_else(|| err(n, "invalid continuation column index"))?
                        };
                        let addition = match value {
                            Cell::Value(text) => text.clone(),
                            Cell::Empty => String::new(),
                            _ => return Err(err(n, "invalid continuation value")),
                        };
                        (col, addition)
                    }
                    [value] => {
                        let col = if domain.mode == Mode::Graph { 2 } else { 1 };
                        let addition = match value {
                            Cell::Value(text) => text.clone(),
                            Cell::Empty => String::new(),
                            _ => return Err(err(n, "invalid continuation value")),
                        };
                        (col, addition)
                    }
                    _ => return Err(err(n, "continuation must contain one field")),
                }
            } else {
                let col = leading + usize::from(domain.mode == Mode::Graph);
                let mut values = fields(trimmed, n)?;
                if values.len() != 1 {
                    return Err(err(n, "continuation must have one field"));
                }
                let addition = match values.remove(0) {
                    Cell::Value(text) => text,
                    Cell::Empty => String::new(),
                    _ => return Err(err(n, "invalid continuation value")),
                };
                (col, addition)
            };
            if indexed_continuations && domain.mode == Mode::Graph && col == 0 {
                return Err(err(n, "cannot continue graph ID X0"));
            }
            if domain.mode == Mode::FiveWOneH && col >= 6 {
                return Err(err(n, "continuation outside six fields"));
            }
            if domain.mode == Mode::Table
                && domain.columns.as_ref().map_or(true, |c| col >= c.len())
            {
                return Err(err(n, "continuation outside table width"));
            }
            let row = domain
                .records
                .last_mut()
                .ok_or_else(|| err(n, "continuation without record"))?;
            let explicit_row = explicit_missing
                .last_mut()
                .and_then(|rows| rows.last_mut())
                .ok_or_else(|| err(n, "continuation without record"))?;
            let was_new = col >= row.len();
            if was_new {
                total_cells = total_cells
                    .checked_add(col + 1 - row.len())
                    .filter(|x| *x <= MAX_TOTAL_CELLS)
                    .ok_or_else(|| err(n, "cell budget exceeded"))?;
            }
            while row.len() <= col {
                row.push(Cell::Missing);
                explicit_row.push(false);
            }
            match &mut row[col] {
                Cell::Value(s) => s.push_str(&addition),
                Cell::Empty => {
                    if !addition.is_empty() {
                        row[col] = Cell::Value(addition);
                    }
                }
                Cell::Missing if indexed_continuations && !explicit_row[col] => {
                    row[col] = if addition.is_empty() {
                        Cell::Empty
                    } else {
                        Cell::Value(addition)
                    }
                }
                Cell::Missing
                    if !indexed_continuations && domain.mode == Mode::Graph && was_new =>
                {
                    row[col] = if addition.is_empty() {
                        Cell::Empty
                    } else {
                        Cell::Value(addition)
                    }
                }
                _ => return Err(err(n, "cannot append to null or missing")),
            }
            continue;
        }
        let mut values = fields(line, n)?;
        let mut explicit_row = values
            .iter()
            .map(|cell| matches!(cell, Cell::Missing))
            .collect::<Vec<_>>();
        match domain.mode {
            Mode::Table => {
                if domain.columns.is_none() {
                    total_cells = total_cells
                        .checked_add(values.len())
                        .filter(|x| *x <= MAX_TOTAL_CELLS)
                        .ok_or_else(|| err(n, "cell budget exceeded"))?;
                    domain.columns =
                        Some((1..=values.len()).map(|i| format!("column{i}")).collect());
                }
                let width = domain.columns.as_ref().unwrap().len();
                if values.len() > width {
                    return Err(err(n, "record exceeds width"));
                }
                values.resize(width, Cell::Missing);
                explicit_row.resize(width, false);
            }
            Mode::FiveWOneH => {
                if values.len() > 6 {
                    return Err(err(n, "5w1h exceeds six fields"));
                }
                values.resize(6, Cell::Missing);
                explicit_row.resize(6, false);
            }
            Mode::Graph => {
                let Cell::Value(token) = &values[0] else {
                    return Err(err(n, "invalid graph id"));
                };
                if graph_token(token).is_none() {
                    return Err(err(n, "invalid graph expression"));
                }
                if values.len() < 2 {
                    return Err(err(n, "graph name missing"));
                }
            }
        }
        record_count += 1;
        if record_count > MAX_RECORDS {
            return Err(err(n, "too many records"));
        }
        total_cells = total_cells
            .checked_add(values.len())
            .filter(|x| *x <= MAX_TOTAL_CELLS)
            .ok_or_else(|| err(n, "cell budget exceeded"))?;
        domain.records.push(values);
        explicit_missing
            .last_mut()
            .ok_or_else(|| err(n, "missing domain parse state"))?
            .push(explicit_row);
    }
    for d in &domains {
        if d.mode == Mode::Graph {
            check_graph(d)?;
        }
    }
    Ok(Document { domains })
}

fn emit_field(cell: &Cell) -> Result<String, Error> {
    match cell {
        Cell::Null => Ok("~".into()),
        Cell::Empty => Ok("\\e".into()),
        Cell::Missing => Ok("\\_".into()),
        Cell::Value(s) if s.is_empty() => Err(err(0, "use Empty for empty value")),
        Cell::Value(s) => {
            let mut out = String::new();
            for c in s.chars() {
                match c {
                    ' ' => out.push_str("\\ "),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    '~' => out.push_str("\\~"),
                    '\u{3000}' => return Err(err(0, "fullwidth space has no native escape")),
                    c => out.push(c),
                }
            }
            Ok(out)
        }
    }
}

/// Serialize a document canonically; reject inconsistent ASTs rather than emit lossy text.
pub fn format_strict_v1(doc: &Document) -> Result<String, Error> {
    if doc.domains.len() > MAX_DOMAINS {
        return Err(err(0, "too many domains"));
    }
    let mut records = 0usize;
    let mut cells = 0usize;
    let mut input_bytes = 0usize;
    for d in &doc.domains {
        records = records
            .checked_add(d.records.len())
            .filter(|x| *x <= MAX_RECORDS)
            .ok_or_else(|| err(0, "too many records"))?;
        if let Some(columns) = &d.columns {
            if columns.len() > MAX_COLUMNS {
                return Err(err(0, "too many columns"));
            }
            cells = cells
                .checked_add(columns.len())
                .filter(|x| *x <= MAX_TOTAL_CELLS)
                .ok_or_else(|| err(0, "cell budget exceeded"))?;
        }
        for name in d.columns.iter().flatten() {
            input_bytes = input_bytes
                .checked_add(name.len())
                .filter(|x| *x <= MAX_SOURCE_BYTES)
                .ok_or_else(|| err(0, "document too large"))?;
        }
        for row in &d.records {
            if row.len() > MAX_COLUMNS {
                return Err(err(0, "too many fields"));
            }
            cells = cells
                .checked_add(row.len())
                .filter(|x| *x <= MAX_TOTAL_CELLS)
                .ok_or_else(|| err(0, "cell budget exceeded"))?;
            for cell in row {
                if let Cell::Value(s) = cell {
                    input_bytes = input_bytes
                        .checked_add(s.len())
                        .filter(|x| *x <= MAX_SOURCE_BYTES)
                        .ok_or_else(|| err(0, "document too large"))?;
                }
            }
        }
    }
    let mut lines = Vec::new();
    for d in &doc.domains {
        let tag = match d.mode {
            Mode::Table => "",
            Mode::FiveWOneH => "?",
            Mode::Graph => ">",
        };
        lines.push(format!(" \\{tag}{} {}", d.id, d.revision));
        if let Some(width) = d.declared_width {
            lines.push(format!("\\%{width}"));
        }
        if d.explicit_header {
            let cols = d
                .columns
                .as_ref()
                .ok_or_else(|| err(0, "missing explicit columns"))?;
            let encoded: Result<Vec<_>, _> = cols
                .iter()
                .map(|s| emit_field(&Cell::Value(s.clone())))
                .collect();
            lines.push(format!("\\# {}", encoded?.join(" ")));
        }
        for row in &d.records {
            let encoded: Result<Vec<_>, _> = row.iter().map(emit_field).collect();
            lines.push(encoded?.join(" "));
        }
    }
    let text = if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    };
    if text.len() > MAX_SOURCE_BYTES {
        return Err(err(0, "document too large"));
    }
    if parse_strict_v1(&text)? != *doc {
        return Err(err(0, "AST cannot be serialized without loss"));
    }
    Ok(text)
}

/// Format a v2 document canonically and prove the output reparses under v2.
pub fn format_strict_v2(doc: &Document) -> Result<String, Error> {
    let text = format_strict_v1(doc)?;
    if parse_strict_v2(&text)? != *doc {
        return Err(err(0, "v2 AST cannot be serialized without loss"));
    }
    Ok(text)
}
