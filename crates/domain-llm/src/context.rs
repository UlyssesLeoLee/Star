// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/domain-llm/src/context.rs` — W3 (ULYS-98-W3) RAG 上下文检索 (per
//! `docs/briefs/ulys-98-star-cursor-min-v1.md` §"Sub-task 3.6 上下文 RAG (基础)").
//!
//! **v0.0.1 stub**: 基础 chunking + mock embedding (deterministic hash-based
//! vector) + cosine similarity ranking. 0 真实 embedding 模型调用 (W4
//! 可切真 embedding provider per W4 决策). 上下文检索走 BM25-like 简化为
//! cosine-similarity Top-K, 真实部署时切 provider (OpenAI text-embedding-3 /
//! Voyage / Cohere).
//!
//! **核心类型 (5 类)**:
//! - [`Chunk`]         — 1 段被切分的原文 (path + offset + text)
//! - [`Embedding`]     — 1 个 chunk 的向量表示 (Vec<f32> mock)
//! - [`ContextHit`]    — 1 个检索结果 (chunk + score)
//! - [`ContextQuery`]  — 检索入参 (query text + TopK + threshold)
//! - [`ContextBuilder`]trait — 构建 + 检索抽象, 跨 embedding provider
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #11 + 守门 #14 v2):
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"` workspace lint).
//! - DTOs derive `Serialize` / `Deserialize` for HTTP / JSON-RPC transport
//!   (per 守门 #11 缺标比错标).
//! - 5 域 Lead 真人到位前 Mavis 临时代签.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// =====================================================================
// Chunk — 1 段被切分的原文 (per W3.6 RAG 基础)
// =====================================================================

/// **Chunk** — 一段被切分的原文片段（构成 RAG 检索的最小单元）。
///
/// v0.0.1 简化模型：1 chunk = 1 段 plain text（无重叠窗口、无 sentence
/// splitter）。`source_path` 用于追溯出处（file path 或 URL），`offset`
/// 标注在源文档中的起始字节位置。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Chunk {
    /// 来源路径（文件路径 / URL / doc id）。
    pub source_path: String,
    /// 在源文档中的起始字节偏移（用于前端跳转到原文位置）。
    pub offset: u32,
    /// UTF-8 文本片段。
    pub text: String,
}

impl Chunk {
    /// 构造一个新 chunk。
    pub fn new(source_path: impl Into<String>, offset: u32, text: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            offset,
            text: text.into(),
        }
    }

    /// 按 word 边界近似切分原文为多个 chunks（v0.0.1 简化：固定窗口大小，
    /// 无 sentence boundary detection）。
    ///
    /// `words_per_chunk` 控制每段目标词数。返回的 chunk 列表至少 1 项，
    /// 即使原文为空（返回 1 个空文本 chunk 以保证下游 embedder 输入非空）。
    pub fn split_words(text: &str, words_per_chunk: usize) -> Vec<Self> {
        if words_per_chunk == 0 {
            // 防御性 fallback：避免除零，1 word/chunk。
            return Self::split_words(text, 1);
        }
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return vec![Chunk::new("<empty>", 0, "")];
        }
        let mut chunks = Vec::new();
        let mut byte_offset: u32 = 0;
        for window in words.chunks(words_per_chunk) {
            let joined = window.join(" ");
            // 近似计算 byte offset（按 char 长度对齐；UTF-8 multi-byte
            // 字符按 1 char 计，足够 v0.0.1 前端跳转）。
            let char_offset = byte_offset as usize;
            chunks.push(Chunk::new("<inline>", char_offset as u32, joined));
            let advance: usize = window.iter().map(|w| w.len() + 1).sum();
            byte_offset = (byte_offset as usize + advance) as u32;
        }
        chunks
    }
}

// =====================================================================
// Embedding — 1 个 chunk 的向量表示
// =====================================================================

/// **Embedding** — 一个 chunk 的向量表示（f32 数组）。
///
/// v0.0.1 mock embedding 维度固定 `MOCK_EMBED_DIM = 64`，由
/// [`MockEmbedder`] 用确定性 hash 函数生成（同一文本 → 同一向量，便于
/// 测试与回放）。真实部署时切 [`ContextBuilder`] 实现。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Embedding {
    /// f32 向量，长度 == [`MOCK_EMBED_DIM`]。
    pub vector: Vec<f32>,
}

/// Mock embedding 向量维度（v0.0.1 固定，W4 切真 provider 时按 provider 调整）。
pub const MOCK_EMBED_DIM: usize = 64;

impl Embedding {
    /// 计算两个 embedding 的 cosine 相似度。
    ///
    /// 维度不匹配返回 0.0（防御性 fallback；正常路径维度必一致）。
    pub fn cosine_similarity(&self, other: &Self) -> f32 {
        if self.vector.len() != other.vector.len() || self.vector.is_empty() {
            return 0.0;
        }
        let dot: f32 = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| a * b)
            .sum();
        let norm_a: f32 = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }
}

// =====================================================================
// ContextHit / ContextQuery — 检索请求与结果
// =====================================================================

/// **ContextHit** — 一次检索命中的 1 个结果（chunk + 相似度分数）。
///
/// `score` 是 [-1.0, 1.0] 范围的 cosine 相似度（`Embedding::cosine_similarity`
/// 输出）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextHit {
    /// 命中的原文 chunk。
    pub chunk: Chunk,
    /// 相似度分数。
    pub score: f32,
}

/// **ContextQuery** — 检索请求。
///
/// `top_k` 是返回的最大命中数（默认 5），`min_score` 是过滤阈值
/// （默认 0.0 = 不过滤）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextQuery {
    /// 查询文本（UTF-8）。
    pub query: String,
    /// TopK（返回的最大命中数；默认 5）。
    pub top_k: usize,
    /// 最低相似度阈值（低于此分数的命中被过滤；默认 0.0）。
    pub min_score: f32,
}

impl ContextQuery {
    /// 默认 top_k = 5。
    pub const DEFAULT_TOP_K: usize = 5;
    /// 默认 min_score = 0.0。
    pub const DEFAULT_MIN_SCORE: f32 = 0.0;

    /// 构造默认参数的 query（top_k = 5, min_score = 0.0）。
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            top_k: Self::DEFAULT_TOP_K,
            min_score: Self::DEFAULT_MIN_SCORE,
        }
    }

    /// 校验 query：非空字符串。
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.query.trim().is_empty() {
            return Err("context query must be non-empty");
        }
        if self.top_k == 0 {
            return Err("context query top_k must be > 0");
        }
        Ok(())
    }
}

// =====================================================================
// ContextBuilder trait — RAG 检索抽象（跨 embedding provider）
// =====================================================================

/// **ContextBuilder** — RAG 检索抽象 trait。
///
/// v0.0.1 stub：仅 [`InMemoryContextBuilder`] 一个实现，用 mock embedding。
/// W4 起可切真 embedding provider（OpenAI text-embedding-3 / Voyage / Cohere），
/// trait 表面不变。
pub trait ContextBuilder: Send + Sync {
    /// 把原文 chunk 索引化（构建/更新内部向量索引）。
    fn index(&mut self, chunks: Vec<Chunk>) -> Result<(), ContextBuilderError>;

    /// 按 query 检索 TopK 命中。
    fn retrieve(&self, query: &ContextQuery) -> Result<Vec<ContextHit>, ContextBuilderError>;

    /// 当前已索引的 chunk 数量（用于前端展示 + 测试断言）。
    fn indexed_count(&self) -> usize;

    /// 清空索引（用于测试隔离 + 重新构建场景）。
    fn clear(&mut self);
}

/// **ContextBuilderError** — RAG 检索错误的统一类型。
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize, PartialEq)]
pub enum ContextBuilderError {
    /// 查询参数无效（空 query / top_k = 0）。
    #[error("invalid query: {0}")]
    InvalidQuery(String),
    /// chunk 切分或 embedding 计算失败。
    #[error("embedding failed: {0}")]
    EmbeddingFailed(String),
}

// =====================================================================
// InMemoryContextBuilder + MockEmbedder — v0.0.1 stub 实现
// =====================================================================

/// **MockEmbedder** — 确定性 hash-based embedding（v0.0.1）。
///
/// 用 token 级别 FNV-1a hash 把每个 word 映射到向量维度 bin 并累加 +
/// 归一化。同一文本 → 同一向量，便于测试回放。**非语义**：仅作
/// similarity ranking 的占位实现。
#[derive(Debug, Default, Clone)]
pub struct MockEmbedder;

impl MockEmbedder {
    /// 对输入文本生成 mock embedding（v0.0.1）。
    pub fn embed(&self, text: &str) -> Embedding {
        let mut vector = vec![0.0_f32; MOCK_EMBED_DIM];
        for token in text.split_whitespace() {
            let h = fnv1a_64(token.as_bytes());
            // 用 hash mod DIM 选 bin，加 1.0 权重；mod 与 DIM 互质不需要。
            let bin = (h as usize) % MOCK_EMBED_DIM;
            vector[bin] += 1.0;
        }
        // 归一化到 unit length（cosine similarity 友好）。
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in vector.iter_mut() {
                *v /= norm;
            }
        }
        Embedding { vector }
    }
}

/// 64-bit FNV-1a hash（确定性，0 依赖）。
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// **InMemoryContextBuilder** — v0.0.1 in-memory RAG 索引 + 检索。
///
/// 内部用 `Vec<(Chunk, Embedding)>` 存所有索引项，检索时遍历全部
/// 计算 cosine similarity 后按分数排序 + TopK 截断。
#[derive(Debug, Default, Clone)]
pub struct InMemoryContextBuilder {
    chunks: Vec<Chunk>,
    embeddings: Vec<Embedding>,
}

impl InMemoryContextBuilder {
    /// 构造空索引。
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造带初始 chunks 的索引（便捷构造器）。
    pub fn with_chunks(chunks: Vec<Chunk>) -> Self {
        let mut b = Self::new();
        // index() 返 Result 但我们忽略错误（mock embedder 永不失败）。
        let _ = b.index(chunks);
        b
    }

    /// 默认 mock embedder（暴露给 trait 之外的调用方）。
    pub fn mock_embedder() -> MockEmbedder {
        MockEmbedder
    }
}

impl ContextBuilder for InMemoryContextBuilder {
    fn index(&mut self, chunks: Vec<Chunk>) -> Result<(), ContextBuilderError> {
        let embedder = MockEmbedder;
        for c in chunks {
            let emb = embedder.embed(&c.text);
            self.chunks.push(c);
            self.embeddings.push(emb);
        }
        Ok(())
    }

    fn retrieve(&self, query: &ContextQuery) -> Result<Vec<ContextHit>, ContextBuilderError> {
        query
            .validate()
            .map_err(|e| ContextBuilderError::InvalidQuery(e.to_string()))?;
        let embedder = MockEmbedder;
        let q_emb = embedder.embed(&query.query);

        let mut hits: Vec<ContextHit> = self
            .chunks
            .iter()
            .zip(self.embeddings.iter())
            .map(|(c, e)| ContextHit {
                chunk: c.clone(),
                score: q_emb.cosine_similarity(e),
            })
            .filter(|h| h.score >= query.min_score)
            .collect();

        // 按 score 降序排序。
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(query.top_k);
        Ok(hits)
    }

    fn indexed_count(&self) -> usize {
        self.chunks.len()
    }

    fn clear(&mut self) {
        self.chunks.clear();
        self.embeddings.clear();
    }
}

// =====================================================================
// InvertedIndexContextBuilder — v0.0.1 词频倒排索引（per-token 命中）
// =====================================================================

/// **InvertedIndexContextBuilder** — v0.0.1 简易倒排索引（per-token 命中数
/// 累加作为 score）。用于前端 keyword-style 检索场景，与 cosine
/// similarity embedding 互补。
///
/// 评分规则：`score = 命中 query tokens 的 chunk token 数 / chunk 总 token 数`
/// —— 高于 `min_score`（默认 0.0）即保留，按分数降序 + TopK 截断。
#[derive(Debug, Default, Clone)]
pub struct InvertedIndexContextBuilder {
    /// chunk 文本 → token set (lowercased)
    chunk_tokens: Vec<Vec<String>>,
    chunks: Vec<Chunk>,
}

impl InvertedIndexContextBuilder {
    /// 构造空索引。
    pub fn new() -> Self {
        Self::default()
    }

    /// token 切分（lowercase + 简单 ASCII alpha-num 过滤；非 ASCII 按 char 保留）。
    fn tokenize(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|w| {
                w.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect()
    }

    /// query 与 chunk 的命中 token 数（lowercase 比较）。
    fn hit_count(query_tokens: &[String], chunk_tokens: &[String]) -> usize {
        let chunk_set: std::collections::HashSet<&String> = chunk_tokens.iter().collect();
        query_tokens
            .iter()
            .filter(|t| chunk_set.contains(t))
            .count()
    }
}

impl ContextBuilder for InvertedIndexContextBuilder {
    fn index(&mut self, chunks: Vec<Chunk>) -> Result<(), ContextBuilderError> {
        for c in chunks {
            let toks = Self::tokenize(&c.text);
            self.chunks.push(c);
            self.chunk_tokens.push(toks);
        }
        Ok(())
    }

    fn retrieve(&self, query: &ContextQuery) -> Result<Vec<ContextHit>, ContextBuilderError> {
        query
            .validate()
            .map_err(|e| ContextBuilderError::InvalidQuery(e.to_string()))?;
        let q_tokens = Self::tokenize(&query.query);
        if q_tokens.is_empty() {
            return Ok(Vec::new());
        }

        let mut hits: Vec<ContextHit> = self
            .chunks
            .iter()
            .zip(self.chunk_tokens.iter())
            .filter_map(|(c, ct)| {
                let hits_ct = Self::hit_count(&q_tokens, ct);
                if ct.is_empty() || hits_ct == 0 {
                    return None;
                }
                Some(ContextHit {
                    chunk: c.clone(),
                    score: hits_ct as f32 / ct.len() as f32,
                })
            })
            .filter(|h| h.score >= query.min_score)
            .collect();

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(query.top_k);
        Ok(hits)
    }

    fn indexed_count(&self) -> usize {
        self.chunks.len()
    }

    fn clear(&mut self) {
        self.chunks.clear();
        self.chunk_tokens.clear();
    }
}

// =====================================================================
// Helper — multi-builder dispatch（per W3.6 RAG hybrid retrieval）
// =====================================================================

/// **HybridContextBuilder** — 组合多个 [`ContextBuilder`]，按等权融合分数。
///
/// 用于 W3.6 hybrid retrieval（mock embedding + inverted index 同时跑，
/// 取并集 + score 融合）。`indexed_count` 返回所有子 builder 之和。
pub struct HybridContextBuilder {
    builders: Vec<Box<dyn ContextBuilder>>,
}

impl HybridContextBuilder {
    /// 构造带子 builders 的 hybrid。
    pub fn new(builders: Vec<Box<dyn ContextBuilder>>) -> Self {
        Self { builders }
    }
}

impl ContextBuilder for HybridContextBuilder {
    fn index(&mut self, chunks: Vec<Chunk>) -> Result<(), ContextBuilderError> {
        for b in self.builders.iter_mut() {
            b.index(chunks.clone())?;
        }
        Ok(())
    }

    fn retrieve(&self, query: &ContextQuery) -> Result<Vec<ContextHit>, ContextBuilderError> {
        // 聚合每 chunk 在所有子 builder 中的最大分数（max-fusion）。
        let mut fused: HashMap<(String, u32), ContextHit> = HashMap::new();
        for b in &self.builders {
            for hit in b.retrieve(query)? {
                let key = (hit.chunk.source_path.clone(), hit.chunk.offset);
                fused
                    .entry(key)
                    .and_modify(|existing| {
                        if hit.score > existing.score {
                            existing.score = hit.score;
                        }
                    })
                    .or_insert(hit);
            }
        }
        let mut hits: Vec<ContextHit> = fused.into_values().collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(query.top_k);
        Ok(hits)
    }

    fn indexed_count(&self) -> usize {
        self.builders.iter().map(|b| b.indexed_count()).sum()
    }

    fn clear(&mut self) {
        for b in self.builders.iter_mut() {
            b.clear();
        }
    }
}

// =====================================================================
// Unit Tests (per 守门 #1 v25 — >= 1 unit test per module)
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// **Test 1 (unit)**: Chunk::split_words 切分 + 空字符串 fallback。
    #[test]
    fn chunk_split_words_handles_empty_and_words() {
        let empty = Chunk::split_words("", 10);
        assert_eq!(empty.len(), 1);
        assert_eq!(empty[0].text, "");

        let words = Chunk::split_words("alpha beta gamma delta epsilon zeta", 2);
        assert_eq!(words.len(), 3);
        assert_eq!(words[0].text, "alpha beta");
        assert_eq!(words[1].text, "gamma delta");
        assert_eq!(words[2].text, "epsilon zeta");
    }

    /// **Test 2 (unit)**: Embedding::cosine_similarity 同向量 → 1.0，正交 → 0。
    #[test]
    fn embedding_cosine_similarity_self_is_one() {
        let emb = Embedding {
            vector: vec![1.0; MOCK_EMBED_DIM],
        };
        let s = emb.cosine_similarity(&emb);
        assert!((s - 1.0).abs() < 1e-6, "self-sim should be 1.0, got {s}");
    }

    /// **Test 3 (unit)**: MockEmbedder 同文本 → 同 embedding。
    #[test]
    fn mock_embedder_is_deterministic() {
        let e1 = MockEmbedder.embed("hello world hello");
        let e2 = MockEmbedder.embed("hello world hello");
        assert_eq!(e1.vector, e2.vector);
    }

    /// **Test 4 (unit)**: InMemoryContextBuilder index + retrieve top-K。
    #[test]
    fn in_memory_context_builder_retrieve_top_k() {
        let mut b = InMemoryContextBuilder::new();
        b.index(vec![
            Chunk::new("a.rs", 0, "fn foo() -> i32 { 42 }"),
            Chunk::new("b.rs", 0, "let bar = vec![1, 2, 3];"),
            Chunk::new("c.rs", 0, "fn baz() { println!(\"hello\"); }"),
        ])
        .unwrap();

        let mut q = ContextQuery::new("hello world");
        q.top_k = 2;
        let hits = b.retrieve(&q).unwrap();
        assert_eq!(hits.len(), 2, "should return at most top_k hits");
        // Scores are descending.
        assert!(hits[0].score >= hits[1].score);
        assert_eq!(b.indexed_count(), 3);
    }

    /// **Test 5 (unit)**: ContextQuery::validate 拒绝空 query 与 top_k=0。
    #[test]
    fn context_query_validate_rejects_invalid() {
        let mut q = ContextQuery::new("   ");
        assert!(q.validate().is_err());
        q = ContextQuery::new("hello");
        q.top_k = 0;
        assert!(q.validate().is_err());
        q = ContextQuery::new("hello");
        assert!(q.validate().is_ok());
    }

    /// **Test 6 (unit)**: InvertedIndexContextBuilder 关键词命中 + 分数排序。
    #[test]
    fn inverted_index_retrieve_ranks_by_overlap() {
        let mut b = InvertedIndexContextBuilder::new();
        b.index(vec![
            Chunk::new("a.md", 0, "rust async tokio runtime scheduler"),
            Chunk::new("b.md", 0, "python asyncio event loop"),
            Chunk::new("c.md", 0, "rust borrow checker lifetime"),
        ])
        .unwrap();

        let q = ContextQuery::new("rust async");
        let hits = b.retrieve(&q).unwrap();
        assert!(!hits.is_empty(), "should hit at least one chunk");
        // 'rust async tokio runtime scheduler' shares 2/4 tokens (0.5) which
        // should be the top hit.
        assert_eq!(hits[0].chunk.source_path, "a.md");
        assert!(hits[0].score > 0.0);
        // 'rust borrow checker lifetime' shares 1/4 tokens (0.25).
        // 'python asyncio event loop' shares 0 → filtered out.
        assert_eq!(hits.len(), 2);
    }

    /// **Test 7 (unit)**: HybridContextBuilder 融合 2 builder 命中。
    #[test]
    fn hybrid_context_builder_fuses_two_builders() {
        let mut hybrid = HybridContextBuilder::new(vec![
            Box::new(InMemoryContextBuilder::new()),
            Box::new(InvertedIndexContextBuilder::new()),
        ]);
        hybrid
            .index(vec![
                Chunk::new("a.md", 0, "rust async tokio"),
                Chunk::new("b.md", 0, "python asyncio"),
            ])
            .unwrap();

        let q = ContextQuery::new("rust");
        let hits = hybrid.retrieve(&q).unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hybrid.indexed_count(), 4); // 2 chunks × 2 builders
    }
}
