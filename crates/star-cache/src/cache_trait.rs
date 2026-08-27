// KeyBuilder per spec/cache/01 §3 — 统一 key 命名空间
// 格式: cache:v1:{crate_name}:{...}

/// 缓存键构造器 (per spec/cache/01 §3)
/// 所有域统一通过本构造器生成键,避免冲突与运营盲区。
pub struct KeyBuilder;

impl KeyBuilder {
    /// 资源键 — 单个聚合根 (e.g. `cache:v1:agent:<uuid>`)
    pub fn resource(crate_name: &str, id: &str) -> String {
        format!("cache:v1:{}:{}", crate_name, id)
    }

    /// 列表键 — 带过滤哈希 (e.g. `cache:v1:agent:list:<hash>`)
    pub fn list(crate_name: &str, filter_hash: &str) -> String {
        format!("cache:v1:{}:list:{}", crate_name, filter_hash)
    }

    /// 字段键 — 聚合根内子字段 (e.g. `cache:v1:agent:<id>:profile`)
    pub fn field(crate_name: &str, id: &str, field: &str) -> String {
        format!("cache:v1:{}:{}:{}", crate_name, id, field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_format() {
        assert_eq!(KeyBuilder::resource("agent", "x"), "cache:v1:agent:x");
        assert_eq!(KeyBuilder::list("agent", "h1"), "cache:v1:agent:list:h1");
        assert_eq!(
            KeyBuilder::field("agent", "x", "profile"),
            "cache:v1:agent:x:profile"
        );
    }
}
