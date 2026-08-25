//! 强类型 ID 宏(`define_uuid_id!`)
//!
//! 本 crate 内部独立定义(避免跨 crate 共享 `macro_rules!`)。

/// 定义一个 UUID newtype 强类型 ID。
#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(uuid::Uuid);

        impl $name {
            /// 创建新的强类型 ID(包装 `Uuid::new_v4`)。
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
            /// 从已有 UUID 构造。
            pub fn from_uuid(id: uuid::Uuid) -> Self {
                Self(id)
            }
            /// 取内部 UUID 引用。
            pub fn as_uuid(&self) -> &uuid::Uuid {
                &self.0
            }
            /// 取出内部 UUID(consume)。
            pub fn into_uuid(self) -> uuid::Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::ops::Deref for $name {
            type Target = uuid::Uuid;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<uuid::Uuid> for $name {
            fn from(id: uuid::Uuid) -> Self {
                Self(id)
            }
        }
    };
}
