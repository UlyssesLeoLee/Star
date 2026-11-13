/*
@cypher schema=1 source_sha256=30ad04005f8757f09a934b824181e53733738f2608682f42c0daf0d850847ec9
MERGE (self:File {path:"crates/star-agent-runtime/src/agent_type.rs"})
@endcypher
*/
//! AgentType — 9 SA Type (per LangGraph 9/3 02 §3, 引用不重写)。
//!
//! LangGraph view 的 SA-01..SA-09 是接口而非实现 (per 02-BD §1.3):
//! LangGraph subgraph 实现 SA-XX 业务逻辑, Agent Runtime ECS 提供底层 Runtime,
//! 两者通过 Adapter 连接。本模块只承载类型标识 + 校验。

use std::fmt;

/// 9 SA Type 注册表: code → label (per LangGraph 9/3 02 §3 图示)。
pub const SA_TYPES: [(&str, &str); 9] = [
    ("SA-01", "code-review"),
    ("SA-02", "test-gen"),
    ("SA-03", "5-域-lead-audit"),
    ("SA-04", "git-ops"),
    ("SA-05", "doc-sync"),
    ("SA-06", "refactor"),
    ("SA-07", "db-migration"),
    ("SA-08", "domain-dev"),
    ("SA-09", "free-form"),
];

/// AgentType 校验错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTypeError(pub String);

impl fmt::Display for AgentTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "非法 AgentType: {} (允许 SA-01..SA-09 + SA-N)", self.0)
    }
}

impl std::error::Error for AgentTypeError {}

/// SA-01..SA-09 类型标识 (per LangGraph 9/3 §6.1)。`SA-N` 为新类型占位。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentType {
    code: &'static str,
}

impl AgentType {
    /// SA-01 code-review。
    pub const SA01: AgentType = AgentType { code: "SA-01" };
    /// SA-02 test-gen。
    pub const SA02: AgentType = AgentType { code: "SA-02" };
    /// SA-03 5-域-lead-audit。
    pub const SA03: AgentType = AgentType { code: "SA-03" };
    /// SA-04 git-ops。
    pub const SA04: AgentType = AgentType { code: "SA-04" };
    /// SA-05 doc-sync。
    pub const SA05: AgentType = AgentType { code: "SA-05" };
    /// SA-06 refactor。
    pub const SA06: AgentType = AgentType { code: "SA-06" };
    /// SA-07 db-migration。
    pub const SA07: AgentType = AgentType { code: "SA-07" };
    /// SA-08 domain-dev。
    pub const SA08: AgentType = AgentType { code: "SA-08" };
    /// SA-09 free-form。
    pub const SA09: AgentType = AgentType { code: "SA-09" };
    /// SA-N 新类型占位 (per LangGraph 9/3 02 §3 图示)。
    pub const SA_N: AgentType = AgentType { code: "SA-N" };

    /// 解析 "SA-01".."SA-09" / "SA-N"。
    pub fn parse(s: &str) -> Result<AgentType, AgentTypeError> {
        for (code, _) in SA_TYPES {
            if s == code {
                return Ok(AgentType { code });
            }
        }
        if s == "SA-N" {
            return Ok(AgentType::SA_N);
        }
        Err(AgentTypeError(s.to_string()))
    }

    /// code (如 "SA-01")。
    pub fn code(self) -> &'static str {
        self.code
    }

    /// 业务标签 (如 "code-review"); SA-N → "new-agent-type"。
    pub fn label(self) -> &'static str {
        for (code, label) in SA_TYPES {
            if self.code == code {
                return label;
            }
        }
        "new-agent-type"
    }
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code)
    }
}
