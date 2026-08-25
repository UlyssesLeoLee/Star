//! 强类型 ID 宏(`define_uuid_id!`)

/// 定义一个 UUID newtype 强类型 ID。
#[macro_export]
macro_rules! define_uuid_id {
    ($name:ident) => {
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(uuid::Uuid);

        impl $name {
            #[allow(dead_code)]
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
            #[allow(dead_code)]
            pub fn from_uuid(id: uuid::Uuid) -> Self {
                Self(id)
            }
            #[allow(dead_code)]
            pub fn as_uuid(&self) -> &uuid::Uuid {
                &self.0
            }
            #[allow(dead_code)]
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
