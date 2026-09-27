//! application integration tests (per ULYS-154 PR-2).
//!
//! application 当前 17% (46/261 lines). 加 coverage_test.rs 覆盖
//! ApplicationService trait + WorkItemId + Runtime 等公开 API.

use application::{ApplicationQueryService, ApplicationService, WorkItemId};
use uuid::Uuid;

/// WorkItemId type alias 是 Uuid (per application:124).
#[test]
fn work_item_id_is_uuid() {
    let id: WorkItemId = Uuid::new_v4();
    // WorkItemId = Uuid, 所以可以 直接调用 Uuid 方法.
    assert!(!id.is_nil());
}

/// WorkItemId 可以从 string 解析.
#[test]
fn work_item_id_from_uuid_string() {
    let s = "00000000-0000-0000-0000-000000000001";
    let id: WorkItemId = Uuid::parse_str(s).unwrap();
    assert_eq!(id.to_string(), s);
}

/// ApplicationService trait Send + Sync (compile-time check).
fn _assert_send_sync<T: ApplicationService + ?Sized>() {
    fn assert<T: Send + Sync>() {}
    assert::<T>();
}

/// ApplicationQueryService trait Send + Sync (compile-time check).
fn _assert_query_send_sync<T: ApplicationQueryService + ?Sized>() {
    fn assert<T: Send + Sync>() {}
    assert::<T>();
}

#[test]
fn application_traits_are_send_sync_compile_check() {
    // fn body 仅做编译期断言, 实际跑 void.
    _assert_send_sync::<dyn ApplicationService>();
    _assert_query_send_sync::<dyn ApplicationQueryService>();
}
