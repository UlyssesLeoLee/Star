// TokenMeter (per G.9): 计量 token 用量 per agent / per call
use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::CallRecord;

/// Token 用量 (per call)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 累计 input tokens
    pub input_tokens: u64,
    /// 累计 output tokens
    pub output_tokens: u64,
    /// 调用次数
    pub call_count: u64,
    /// 累计成本 (USD)
    pub cost_usd: f64,
}

impl TokenUsage {
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    pub fn record(&mut self, r: &CallRecord) {
        self.input_tokens += r.input_tokens;
        self.output_tokens += r.output_tokens;
        self.call_count += 1;
        if let Some(c) = r.cost_usd {
            self.cost_usd += c;
        }
    }
}

/// Per agent 用量
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsageByAgent {
    pub usage: TokenUsage,
    /// 各模型分桶
    pub by_model: HashMap<String, TokenUsage>,
}

/// Token 计量器 (per G.9)
pub struct TokenMeter {
    /// 全部用量
    pub global: Arc<RwLock<TokenUsage>>,
    /// per agent
    pub by_agent: Arc<RwLock<HashMap<Uuid, TokenUsageByAgent>>>,
    /// per model
    pub by_model: Arc<RwLock<HashMap<String, TokenUsage>>>,
    /// 调用记录 (T 派生: append-only)
    pub records: Arc<RwLock<Vec<CallRecord>>>,
}

impl TokenMeter {
    pub fn new() -> Self {
        Self {
            global: Arc::new(RwLock::new(TokenUsage::default())),
            by_agent: Arc::new(RwLock::new(HashMap::new())),
            by_model: Arc::new(RwLock::new(HashMap::new())),
            records: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 记录一次调用
    pub async fn record(&self, r: CallRecord) {
        // global
        {
            let mut g = self.global.write().await;
            g.record(&r);
        }
        // by_agent
        {
            let mut ba = self.by_agent.write().await;
            let entry = ba.entry(r.agent_id).or_default();
            entry.usage.record(&r);
            let model_entry = entry.by_model.entry(r.model.clone()).or_default();
            model_entry.record(&r);
        }
        // by_model
        {
            let mut bm = self.by_model.write().await;
            let entry = bm.entry(r.model.clone()).or_default();
            entry.record(&r);
        }
        // records (append-only T 派生)
        {
            let mut recs = self.records.write().await;
            recs.push(r);
        }
    }

    /// 查询全局用量
    pub async fn global_usage(&self) -> TokenUsage {
        self.global.read().await.clone()
    }

    /// 查询 per agent 用量
    pub async fn agent_usage(&self, agent_id: Uuid) -> Option<TokenUsageByAgent> {
        self.by_agent.read().await.get(&agent_id).cloned()
    }

    /// 查询 per model 用量
    pub async fn model_usage(&self, model: &str) -> Option<TokenUsage> {
        self.by_model.read().await.get(model).cloned()
    }

    /// 调用记录数
    pub async fn record_count(&self) -> u64 {
        self.records.read().await.len() as u64
    }
}

impl Default for TokenMeter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_call(agent: Uuid, model: &str, input: u64, output: u64) -> CallRecord {
        CallRecord {
            call_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            agent_id: agent,
            model: model.into(),
            input_tokens: input,
            output_tokens: output,
            timestamp_ms: 1_700_000_000_000,
            cost_usd: Some(0.001),
        }
    }

    #[tokio::test]
    async fn record_accumulates() {
        let meter = TokenMeter::new();
        let agent = Uuid::new_v4();
        meter.record(make_call(agent, "gpt-4o", 100, 50)).await;
        meter.record(make_call(agent, "gpt-4o", 200, 100)).await;
        meter.record(make_call(agent, "claude-3.5", 50, 25)).await;

        let g = meter.global_usage().await;
        assert_eq!(g.input_tokens, 350);
        assert_eq!(g.output_tokens, 175);
        assert_eq!(g.call_count, 3);

        let a = meter.agent_usage(agent).await.unwrap();
        assert_eq!(a.usage.input_tokens, 350);
        assert_eq!(a.by_model.get("gpt-4o").unwrap().input_tokens, 300);

        let m = meter.model_usage("gpt-4o").await.unwrap();
        assert_eq!(m.input_tokens, 300);
    }

    #[tokio::test]
    async fn records_append_only() {
        let meter = TokenMeter::new();
        for _ in 0..5 {
            meter
                .record(make_call(Uuid::new_v4(), "gpt-4o", 10, 5))
                .await;
        }
        assert_eq!(meter.record_count().await, 5);
    }
}
