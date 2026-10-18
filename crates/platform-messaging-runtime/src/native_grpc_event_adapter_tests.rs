// SPDX-License-Identifier: Apache-2.0
/*
@cypher schema=1 source_sha256=ec7e581530cab5aa4ddebc88b5d557849e932f99ceba766f951a5e416f6f88c9
MERGE (self:File {path:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs"})
MERGE (native_grpc_file:File {path:"crates/platform-messaging-runtime/src/native_grpc.rs"})
MERGE (admission_delivery_file:File {path:"crates/platform-messaging-runtime/src/admission_delivery.rs"})
MERGE (native_provider_id:Config {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NATIVE_PROVIDER_ID"})
MERGE (native_provider_identity:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::native_provider_identity",kind:"function"})
MERGE (profile_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportProfile.new",kind:"method"})
MERGE (sha256:ExternalService {id:"sha2::Sha256::digest",kind:"method"})
MERGE (uuid_from_u128:ExternalService {id:"uuid::Uuid::from_u128",kind:"method"})
MERGE (profile_helper:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs::profile",kind:"function"})
MERGE (scope_helper:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs::scope",kind:"function"})
MERGE (delivery_helper:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs::delivery",kind:"function"})
MERGE (verification_pins_helper:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs::verification_pins",kind:"function"})
MERGE (event_verification_pins:Type {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeEventOriginVerificationPins",kind:"struct"})
MERGE (scope_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs::event_scope_rejects_invalid_identities",kind:"test"})
MERGE (pull_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs::pull_request_pins_profile_sender_event_kind_and_lease",kind:"test"})
MERGE (delivery_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs::delivery_preserves_exact_wire_and_origin_fence",kind:"test"})
MERGE (reject_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs::delivery_rejects_route_kind_digest_and_size_mismatches",kind:"test"})
MERGE (verify_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs::origin_verification_pins_exact_wire_and_route",kind:"test"})
MERGE (ack_test:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc_event_adapter_tests.rs::acknowledgement_carries_the_full_lease_fence",kind:"test"})
MERGE (event_scope:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcEventReceiveScope.new",kind:"method"})
MERGE (pull_request:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::native_event_pull_request",kind:"function"})
MERGE (validate_delivery:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::validate_native_event_delivery",kind:"function"})
MERGE (verify_request:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::native_event_origin_verification_request",kind:"function"})
MERGE (ack_request:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::native_event_acknowledgement_request",kind:"function"})
MERGE (self)-[:DEFINES]->(scope_test)
MERGE (self)-[:DEFINES]->(profile_helper)
MERGE (self)-[:DEFINES]->(scope_helper)
MERGE (self)-[:DEFINES]->(delivery_helper)
MERGE (self)-[:DEFINES]->(pull_test)
MERGE (self)-[:DEFINES]->(delivery_test)
MERGE (self)-[:DEFINES]->(reject_test)
MERGE (self)-[:DEFINES]->(verify_test)
MERGE (self)-[:DEFINES]->(ack_test)
MERGE (scope_test)-[:TESTS]->(event_scope)
MERGE (pull_test)-[:TESTS]->(pull_request)
MERGE (delivery_test)-[:TESTS]->(validate_delivery)
MERGE (reject_test)-[:TESTS]->(validate_delivery)
MERGE (verify_test)-[:TESTS]->(verify_request)
MERGE (ack_test)-[:TESTS]->(ack_request)
MERGE (pull_test)-[:CALLS]->(profile_helper)
MERGE (pull_test)-[:CALLS]->(scope_helper)
MERGE (delivery_test)-[:CALLS]->(scope_helper)
MERGE (delivery_test)-[:CALLS]->(delivery_helper)
MERGE (delivery_test)-[:USES]->(native_provider_id)
MERGE (reject_test)-[:CALLS]->(scope_helper)
MERGE (reject_test)-[:CALLS]->(delivery_helper)
MERGE (reject_test)-[:CALLS]->(profile_helper)
MERGE (reject_test)-[:USES]->(native_provider_id)
MERGE (verify_test)-[:CALLS]->(profile_helper)
MERGE (verify_test)-[:CALLS]->(scope_helper)
MERGE (verify_test)-[:CALLS]->(delivery_helper)
MERGE (ack_test)-[:CALLS]->(profile_helper)
MERGE (ack_test)-[:CALLS]->(scope_helper)
MERGE (ack_test)-[:CALLS]->(delivery_helper)
MERGE (profile_helper)-[:USES]->(native_provider_id)
MERGE (profile_helper)-[:CALLS]->(profile_new)
MERGE (scope_helper)-[:CALLS]->(event_scope)
MERGE (scope_helper)-[:CALLS]->(uuid_from_u128)
MERGE (delivery_helper)-[:CALLS]->(sha256)
MERGE (delivery_helper)-[:CALLS]->(uuid_from_u128)
MERGE (delivery_test)-[:CALLS]->(native_provider_identity)
MERGE (verify_test)-[:CALLS]->(native_provider_identity)
MERGE (verify_test)-[:CALLS]->(verification_pins_helper)
MERGE (verify_test)-[:CALLS]->(profile_new)
MERGE (verification_pins_helper)-[:USES_TYPE]->(event_verification_pins)
MERGE (self)-[:IMPORTS]->(native_grpc_file)
MERGE (self)-[:IMPORTS]->(admission_delivery_file)
@endcypher
*/

use super::*;

fn profile() -> AdmissionTransportProfile {
    AdmissionTransportProfile::new(
        NATIVE_PROVIDER_ID.to_owned(),
        "1".to_owned(),
        [7; 32],
        "native-endpoint-a".to_owned(),
    )
    .expect("test profile is valid")
}

fn scope() -> NativeGrpcEventReceiveScope {
    NativeGrpcEventReceiveScope::new(Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(3))
        .expect("test event route is valid")
}

fn delivery(scope: NativeGrpcEventReceiveScope, payload: Vec<u8>) -> wire::Delivery {
    let wire_sha256: [u8; 32] = Sha256::digest(&payload).into();
    wire::Delivery {
        log_id: "native-log-1".to_owned(),
        log_generation: Uuid::from_u128(4).as_bytes().to_vec(),
        sequence: 9,
        sender_service_id: scope.sender_service_id().as_bytes().to_vec(),
        message_kind: TransportMessageKind::Event.as_str().to_owned(),
        message_id: Uuid::from_u128(5).as_bytes().to_vec(),
        wire: payload,
        wire_sha256: wire_sha256.to_vec(),
        lease_id: Uuid::from_u128(6).as_bytes().to_vec(),
        lease_expires_unix_ms: 1_000_000,
        attempt_count: 1,
    }
}

fn verification_pins<'a>(
    profile: &'a AdmissionTransportProfile,
    scope: NativeGrpcEventReceiveScope,
    provider_identity: &'a str,
) -> NativeEventOriginVerificationPins<'a> {
    NativeEventOriginVerificationPins {
        profile,
        provider_identity,
        scope,
        expected_log_id: "native-log-1",
        expected_log_generation: Uuid::from_u128(4),
    }
}

#[test]
fn event_scope_rejects_invalid_identities() {
    assert!(
        NativeGrpcEventReceiveScope::new(Uuid::nil(), Uuid::from_u128(2), Uuid::from_u128(3))
            .is_err()
    );
    assert!(
        NativeGrpcEventReceiveScope::new(Uuid::from_u128(1), Uuid::nil(), Uuid::from_u128(3))
            .is_err()
    );
    assert!(NativeGrpcEventReceiveScope::new(
        Uuid::from_u128(1),
        Uuid::from_u128(2),
        Uuid::from_u128(2),
    )
    .is_err());
}

#[test]
fn pull_request_pins_profile_sender_event_kind_and_lease() {
    let profile = profile();
    let scope = scope();

    let request = native_event_pull_request(&profile, scope, 45).unwrap();

    assert_eq!(
        request.profile.unwrap().profile_digest.as_slice(),
        &profile.profile_digest[..],
    );
    assert_eq!(
        request.sender_service_id.as_slice(),
        scope.sender_service_id().as_bytes().as_slice(),
    );
    assert_eq!(request.message_kind, "event");
    assert_eq!(request.lease_seconds, 45);
    assert!(native_event_pull_request(&profile, scope, 29).is_err());
    assert!(native_event_pull_request(&profile, scope, 121).is_err());

    let mut invalid_profile = profile;
    invalid_profile.provider_version.clear();
    assert!(native_event_pull_request(&invalid_profile, scope, 45).is_err());
}

#[test]
fn delivery_preserves_exact_wire_and_origin_fence() {
    let profile = profile();
    let scope = scope();
    let exact_wire = b"owner event bytes remain exact".to_vec();
    let expected_digest: [u8; 32] = Sha256::digest(&exact_wire).into();
    let provider_identity = native_provider_identity(&profile);

    let validated = validate_native_event_delivery(
        scope,
        &profile,
        "native-log-1",
        Uuid::from_u128(4),
        delivery(scope, exact_wire.clone()),
    )
    .unwrap();

    assert_eq!(validated.wire, exact_wire);
    assert_eq!(validated.origin.scope(), scope);
    assert_eq!(validated.origin.provider_identity(), provider_identity);
    assert_eq!(validated.origin.log_id(), "native-log-1");
    assert_eq!(validated.origin.log_generation(), Uuid::from_u128(4));
    assert_eq!(validated.origin.sequence(), 9);
    assert_eq!(validated.origin.message_id(), Uuid::from_u128(5));
    assert_eq!(validated.origin.wire_sha256(), expected_digest);
    assert_eq!(validated.lease_id, Uuid::from_u128(6));

    let max_wire = vec![0; MAX_NATIVE_APPEND_WIRE_BYTES];
    assert!(validate_native_event_delivery(
        scope,
        &profile,
        "native-log-1",
        Uuid::from_u128(4),
        delivery(scope, max_wire),
    )
    .is_ok());
}

#[test]
fn delivery_rejects_route_kind_digest_and_size_mismatches() {
    let profile = profile();
    let scope = scope();

    let mut wrong_sender = delivery(scope, b"event".to_vec());
    wrong_sender.sender_service_id = Uuid::from_u128(99).as_bytes().to_vec();
    assert!(validate_native_event_delivery(
        scope,
        &profile,
        "native-log-1",
        Uuid::from_u128(4),
        wrong_sender,
    )
    .is_err());

    let mut wrong_log = delivery(scope, b"event".to_vec());
    wrong_log.log_id = "other-log".to_owned();
    assert!(validate_native_event_delivery(
        scope,
        &profile,
        "native-log-1",
        Uuid::from_u128(4),
        wrong_log,
    )
    .is_err());

    let mut wrong_generation = delivery(scope, b"event".to_vec());
    wrong_generation.log_generation = Uuid::from_u128(44).as_bytes().to_vec();
    assert!(validate_native_event_delivery(
        scope,
        &profile,
        "native-log-1",
        Uuid::from_u128(4),
        wrong_generation,
    )
    .is_err());

    let mut wrong_kind = delivery(scope, b"event".to_vec());
    wrong_kind.message_kind = "request".to_owned();
    assert!(validate_native_event_delivery(
        scope,
        &profile,
        "native-log-1",
        Uuid::from_u128(4),
        wrong_kind,
    )
    .is_err());

    let mut wrong_digest = delivery(scope, b"event".to_vec());
    wrong_digest.wire_sha256[0] ^= 0xff;
    assert!(validate_native_event_delivery(
        scope,
        &profile,
        "native-log-1",
        Uuid::from_u128(4),
        wrong_digest,
    )
    .is_err());

    let too_large = delivery(scope, vec![0; MAX_NATIVE_APPEND_WIRE_BYTES + 1]);
    assert!(validate_native_event_delivery(
        scope,
        &profile,
        "native-log-1",
        Uuid::from_u128(4),
        too_large,
    )
    .is_err());
}

#[test]
fn origin_verification_pins_exact_wire_and_route() {
    let profile = profile();
    let scope = scope();
    let exact_wire = b"verify the exact bytes".to_vec();
    let provider_identity = native_provider_identity(&profile);
    let validated = validate_native_event_delivery(
        scope,
        &profile,
        "native-log-1",
        Uuid::from_u128(4),
        delivery(scope, exact_wire.clone()),
    )
    .unwrap();
    let pins = verification_pins(&profile, scope, &provider_identity);

    let request =
        native_event_origin_verification_request(pins, scope, &validated.origin, &exact_wire)
            .unwrap();
    assert_eq!(
        request.profile.unwrap().profile_digest.as_slice(),
        &profile.profile_digest[..],
    );
    assert_eq!(
        request.sender_service_id.as_slice(),
        scope.sender_service_id().as_bytes().as_slice(),
    );
    assert_eq!(request.message_kind, "event");
    assert_eq!(request.log_id, "native-log-1");
    assert_eq!(
        request.log_generation.as_slice(),
        Uuid::from_u128(4).as_bytes().as_slice(),
    );
    assert_eq!(request.sequence, 9);
    assert_eq!(
        request.message_id.as_slice(),
        Uuid::from_u128(5).as_bytes().as_slice(),
    );
    let expected_digest: [u8; 32] = Sha256::digest(&exact_wire).into();
    assert_eq!(request.wire_sha256.as_slice(), &expected_digest[..]);

    let altered_wire = b"changed event bytes";
    assert!(
        native_event_origin_verification_request(pins, scope, &validated.origin, altered_wire,)
            .is_err()
    );

    let changed_endpoint = AdmissionTransportProfile::new(
        profile.provider_id.clone(),
        profile.provider_version.clone(),
        profile.profile_digest,
        "native-endpoint-b".to_owned(),
    )
    .unwrap();
    let changed_pins = verification_pins(&changed_endpoint, scope, &provider_identity);
    assert!(native_event_origin_verification_request(
        changed_pins,
        scope,
        &validated.origin,
        &exact_wire,
    )
    .is_err());
    let drifted_scope = NativeGrpcEventReceiveScope::new(
        scope.tenant_id(),
        scope.sender_service_id(),
        Uuid::from_u128(30),
    )
    .unwrap();
    assert!(native_event_origin_verification_request(
        pins,
        drifted_scope,
        &validated.origin,
        &exact_wire,
    )
    .is_err());
}

#[test]
fn acknowledgement_carries_the_full_lease_fence() {
    let profile = profile();
    let scope = scope();
    let validated = validate_native_event_delivery(
        scope,
        &profile,
        "native-log-1",
        Uuid::from_u128(4),
        delivery(scope, b"committed owner event".to_vec()),
    )
    .unwrap();

    let request = native_event_acknowledgement_request(
        &profile,
        scope,
        &validated.origin,
        validated.lease_id,
    )
    .unwrap();

    assert_eq!(
        request.profile.unwrap().profile_digest.as_slice(),
        &profile.profile_digest[..],
    );
    assert_eq!(
        request.sender_service_id.as_slice(),
        scope.sender_service_id().as_bytes().as_slice(),
    );
    assert_eq!(request.message_kind, "event");
    assert_eq!(request.log_id, "native-log-1");
    assert_eq!(
        request.log_generation.as_slice(),
        Uuid::from_u128(4).as_bytes().as_slice(),
    );
    assert_eq!(request.sequence, 9);
    assert_eq!(
        request.lease_id.as_slice(),
        Uuid::from_u128(6).as_bytes().as_slice(),
    );
    assert_eq!(
        request.message_id.as_slice(),
        Uuid::from_u128(5).as_bytes().as_slice(),
    );
    assert_eq!(
        request.wire_sha256.as_slice(),
        validated.origin.wire_sha256().as_slice(),
    );

    let mut changed_profile = profile;
    changed_profile.provider_version.push_str(".next");
    assert!(native_event_acknowledgement_request(
        &changed_profile,
        scope,
        &validated.origin,
        validated.lease_id,
    )
    .is_err());
}
