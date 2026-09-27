//! application integration tests (per ULYS-154 PR-2).

use application::WorkItemId;
use uuid::Uuid;

#[test]
fn work_item_id_is_uuid() {
    let id: WorkItemId = Uuid::new_v4();
    assert!(!id.is_nil());
}

#[test]
fn work_item_id_from_uuid_string() {
    let s = "00000000-0000-0000-0000-000000000001";
    let id: WorkItemId = Uuid::parse_str(s).unwrap();
    assert_eq!(id.to_string(), s);
}
