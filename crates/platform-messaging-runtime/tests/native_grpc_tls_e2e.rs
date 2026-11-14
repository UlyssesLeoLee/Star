// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=4b49a9173716c33da2a27a61c459c1bc31e0c322b4aabc85920af2d81842992f
MERGE (self:File {path:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs"})
MERGE (trusted_handshake_test:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::trusted_client_completes_the_mtls_handshake_and_reaches_the_service",kind:"test"})
MERGE (no_certificate_test:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::client_without_a_certificate_is_rejected",kind:"test"})
MERGE (wrong_ca_test:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::client_trusting_an_unrelated_ca_is_rejected",kind:"test"})
MERGE (wrong_name_test:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::client_pinning_a_mismatched_server_name_is_rejected",kind:"test"})
MERGE (pg_revocation_test:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::trusted_append_succeeds_and_sql_revocation_fails_the_followup",kind:"test"})
MERGE (certificates:Type {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::TestCertificates",kind:"struct"})
MERGE (fixture:Type {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::TestFixture",kind:"struct"})
MERGE (server_handle:Type {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::TestServerHandle",kind:"struct"})
MERGE (make_certificates:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::make_test_certificates",kind:"function"})
MERGE (spawn_server:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::spawn_test_server",kind:"function"})
MERGE (spawn_server_pool:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::spawn_test_server_with_pool",kind:"function"})
MERGE (write_pem:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::write_pem",kind:"function"})
MERGE (client_for:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::client_for",kind:"function"})
MERGE (append_probe:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::append_probe",kind:"function"})
MERGE (append_request:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::append_request",kind:"function"})
MERGE (prepare_fixture:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::prepare_fixture",kind:"function"})
MERGE (provision_fixture:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::provision_fixture",kind:"function"})
MERGE (set_owner_scope:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::set_owner_scope",kind:"function"})
MERGE (lane_database:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::require_lane_database_name",kind:"function"})
MERGE (require_schema:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::require_d20_schema",kind:"function"})
MERGE (revoke:Symbol {id:"crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs::revoke_certificate",kind:"function"})
MERGE (client_tls:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::client_tls_config",kind:"function"})
MERGE (server_tls:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::server_mtls_config",kind:"function"})
MERGE (service:Type {id:"crates/platform-messaging-runtime/src/native_grpc_server.rs::NativeGrpcTransportServer",kind:"struct"})
MERGE (profile_master:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileMaster",kind:"struct"})
MERGE (rpc:ExternalService {id:"platform_messaging_proto::wire::DurableTransportClient::append",kind:"method"})
MERGE (database:ExternalService {id:"sqlx::PgPool",kind:"type"})
MERGE (current_database:ExternalService {id:"PostgreSQL.current_database",kind:"function"})
MERGE (tonic_server:ExternalService {id:"tonic::transport::Server::tls_config",kind:"method"})
MERGE (certificate_table:Table {id:"platform_transport.service_certificate_identity",class:"Master"})
MERGE (log_table:Table {id:"platform_transport.log_master",class:"Master"})
MERGE (binding_table:Table {id:"platform_transport.profile_log_binding",class:"Master"})
MERGE (self)-[:DEFINES]->(trusted_handshake_test)
MERGE (self)-[:DEFINES]->(no_certificate_test)
MERGE (self)-[:DEFINES]->(wrong_ca_test)
MERGE (self)-[:DEFINES]->(wrong_name_test)
MERGE (self)-[:DEFINES]->(pg_revocation_test)
MERGE (self)-[:DEFINES]->(certificates)
MERGE (self)-[:DEFINES]->(fixture)
MERGE (self)-[:DEFINES]->(server_handle)
MERGE (self)-[:DEFINES]->(make_certificates)
MERGE (self)-[:DEFINES]->(spawn_server)
MERGE (self)-[:DEFINES]->(spawn_server_pool)
MERGE (self)-[:DEFINES]->(write_pem)
MERGE (self)-[:DEFINES]->(client_for)
MERGE (self)-[:DEFINES]->(append_probe)
MERGE (self)-[:DEFINES]->(append_request)
MERGE (self)-[:DEFINES]->(prepare_fixture)
MERGE (self)-[:DEFINES]->(provision_fixture)
MERGE (self)-[:DEFINES]->(set_owner_scope)
MERGE (self)-[:DEFINES]->(lane_database)
MERGE (self)-[:DEFINES]->(require_schema)
MERGE (self)-[:DEFINES]->(revoke)
MERGE (trusted_handshake_test)-[:CALLS]->(make_certificates)
MERGE (trusted_handshake_test)-[:CALLS]->(spawn_server)
MERGE (trusted_handshake_test)-[:CALLS]->(client_tls)
MERGE (trusted_handshake_test)-[:CALLS]->(client_for)
MERGE (trusted_handshake_test)-[:CALLS]->(append_probe)
MERGE (trusted_handshake_test)-[:CALLS]->(rpc)
MERGE (no_certificate_test)-[:CALLS]->(make_certificates)
MERGE (no_certificate_test)-[:CALLS]->(spawn_server)
MERGE (no_certificate_test)-[:CALLS]->(client_for)
MERGE (no_certificate_test)-[:CALLS]->(append_probe)
MERGE (no_certificate_test)-[:CALLS]->(rpc)
MERGE (wrong_ca_test)-[:CALLS]->(make_certificates)
MERGE (wrong_ca_test)-[:CALLS]->(spawn_server)
MERGE (wrong_ca_test)-[:CALLS]->(client_tls)
MERGE (wrong_ca_test)-[:CALLS]->(client_for)
MERGE (wrong_ca_test)-[:CALLS]->(append_probe)
MERGE (wrong_ca_test)-[:CALLS]->(rpc)
MERGE (wrong_name_test)-[:CALLS]->(make_certificates)
MERGE (wrong_name_test)-[:CALLS]->(spawn_server)
MERGE (wrong_name_test)-[:CALLS]->(client_tls)
MERGE (wrong_name_test)-[:CALLS]->(client_for)
MERGE (wrong_name_test)-[:CALLS]->(append_probe)
MERGE (wrong_name_test)-[:CALLS]->(rpc)
MERGE (pg_revocation_test)-[:CALLS]->(make_certificates)
MERGE (pg_revocation_test)-[:CALLS]->(prepare_fixture)
MERGE (pg_revocation_test)-[:CALLS]->(lane_database)
MERGE (pg_revocation_test)-[:CALLS]->(require_schema)
MERGE (pg_revocation_test)-[:CALLS]->(spawn_server_pool)
MERGE (pg_revocation_test)-[:CALLS]->(client_tls)
MERGE (pg_revocation_test)-[:CALLS]->(client_for)
MERGE (pg_revocation_test)-[:CALLS]->(append_request)
MERGE (pg_revocation_test)-[:CALLS]->(rpc)
MERGE (pg_revocation_test)-[:CALLS]->(revoke)
MERGE (pg_revocation_test)-[:TESTS]->(certificate_table)
MERGE (pg_revocation_test)-[:TESTS]->(log_table)
MERGE (pg_revocation_test)-[:TESTS]->(binding_table)
MERGE (prepare_fixture)-[:USES_TYPE]->(profile_master)
MERGE (prepare_fixture)-[:CALLS]->(provision_fixture)
MERGE (provision_fixture)-[:WRITES]->(certificate_table)
MERGE (provision_fixture)-[:WRITES]->(log_table)
MERGE (provision_fixture)-[:WRITES]->(binding_table)
MERGE (lane_database)-[:CALLS]->(current_database)
MERGE (revoke)-[:WRITES]->(certificate_table)
MERGE (spawn_server)-[:CALLS]->(spawn_server_pool)
MERGE (spawn_server_pool)-[:CALLS]->(server_tls)
MERGE (spawn_server_pool)-[:USES_TYPE]->(service)
MERGE (spawn_server_pool)-[:CALLS]->(tonic_server)
MERGE (spawn_server_pool)-[:USES_TYPE]->(database)
@endcypher
*/

//! Live Tonic mTLS coverage for the PostgreSQL-backed native transport service.
//!
//! Four handshake-level cases run on every `cargo test` invocation with no external services:
//! a trusted client completes the mandatory mTLS handshake and reaches the real
//! [`NativeGrpcTransportServer`] service, and clients without a certificate, with an unrelated
//! CA, or with a mismatched pinned server name are rejected. The bounded-append success and the
//! SQL-layer certificate revocation proof are PostgreSQL-backed and remain opt-in behind the
//! `STAR_PLATFORM_PROFILE_TEST_DATABASE_URL` environment variable; without a live disposable
//! database they are reported as ignored, never as passed.
#![cfg(all(feature = "native-grpc-adapter", feature = "durable-quarantine"))]

use std::{
    env,
    error::Error,
    fs,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    time::Duration,
};

use platform_messaging_proto::wire::{self, durable_transport_client::DurableTransportClient};
use platform_messaging_runtime::{
    native_grpc::{client_tls_config, server_mtls_config},
    native_grpc_server::NativeGrpcTransportServer,
    transport_profile::{
        OpaqueSecretReference, TransportMessageKind, TransportProfileDefinition,
        TransportProfileLifecycle,
    },
    transport_profile_pg::{PgTransportProfileMaster, ProfileChangeRequest},
};
use rcgen::{BasicConstraints, CertificateParams, DnType, IsCa, Issuer, KeyPair};
use sha2::{Digest, Sha256};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool, Postgres, Transaction,
};
use tempfile::TempDir;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    transport::{Certificate as TonicCertificate, Channel, ClientTlsConfig, Endpoint, Server},
    Code,
};
use uuid::Uuid;

const DATABASE_URL_ENV: &str = "STAR_PLATFORM_PROFILE_TEST_DATABASE_URL";
const PROVIDER_ID: &str = "star-rust-log";
const PROVIDER_VERSION: &str = "1.0.0";
const MESSAGE_KIND: &str = "request";
const GENERATION_POLICY: &str = "native-grpc-mtls-e2e-v1";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

struct TestCertificates {
    _directory: TempDir,
    ca_certificate: PathBuf,
    wrong_ca_certificate: PathBuf,
    server_certificate: PathBuf,
    server_private_key: PathBuf,
    client_certificate: PathBuf,
    client_private_key: PathBuf,
    client_sha256: [u8; 32],
}

struct TestFixture {
    deployment_id: Uuid,
    tenant_id: Uuid,
    actor_service_id: Uuid,
    sender_service_id: Uuid,
    recipient_service_id: Uuid,
    profile_id: Uuid,
    profile_digest: Vec<u8>,
    log_id: String,
    generation: Uuid,
    client_sha256: [u8; 32],
}

struct TestServerHandle {
    address: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<Result<(), tonic::transport::Error>>,
}

impl TestServerHandle {
    async fn stop(mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.task
            .await
            .expect("join the loopback gRPC server task")
            .expect("gracefully stop the Tonic server");
    }
}

/// The trusted handshake must reach the real service. The SQL layer is the only stage that can
/// still fail here, and its failure message is distinct from any TLS-level transport error.
#[tokio::test]
async fn trusted_client_completes_the_mtls_handshake_and_reaches_the_service() {
    let certificates = make_test_certificates().expect("create isolated test-only TLS material");
    let server = spawn_test_server(&certificates).await;
    let mut trusted_client = client_for(
        server.address,
        client_tls_config(
            "localhost",
            &certificates.ca_certificate,
            &certificates.client_certificate,
            &certificates.client_private_key,
        )
        .expect("build the production client TLS configuration"),
    );
    let error = trusted_client
        .append(append_probe(b"trusted mTLS handshake probe"))
        .await
        .expect_err(
            "this executed case has no PostgreSQL instance, so the durable append must fail",
        );
    // A TLS rejection would never reach the SQL dispatch: the exact database-error mapping is
    // produced only by the real service after `authenticated_peer` accepted the client leaf.
    assert_eq!(error.code(), Code::Unavailable);
    assert_eq!(
        error.message(),
        "native transport durable store is unavailable"
    );
    drop(trusted_client);
    server.stop().await;
}

/// The server must reject a client that presents no certificate during the mandatory handshake.
#[tokio::test]
async fn client_without_a_certificate_is_rejected() {
    let certificates = make_test_certificates().expect("create isolated test-only TLS material");
    let server = spawn_test_server(&certificates).await;
    let no_certificate_tls = ClientTlsConfig::new()
        .domain_name("localhost")
        .ca_certificate(TonicCertificate::from_pem(
            fs::read(&certificates.ca_certificate).expect("read the test server CA"),
        ));
    let mut no_certificate_client = client_for(server.address, no_certificate_tls);
    assert!(
        no_certificate_client
            .append(append_probe(b"missing client identity"))
            .await
            .is_err(),
        "the server must reject a client without a certificate"
    );
    drop(no_certificate_client);
    server.stop().await;
}

/// The client must reject a server whose certificate chains to an unrelated CA.
#[tokio::test]
async fn client_trusting_an_unrelated_ca_is_rejected() {
    let certificates = make_test_certificates().expect("create isolated test-only TLS material");
    let server = spawn_test_server(&certificates).await;
    let mut wrong_ca_client = client_for(
        server.address,
        client_tls_config(
            "localhost",
            &certificates.wrong_ca_certificate,
            &certificates.client_certificate,
            &certificates.client_private_key,
        )
        .expect("build a client with an untrusted server CA"),
    );
    assert!(
        wrong_ca_client
            .append(append_probe(b"untrusted server CA"))
            .await
            .is_err(),
        "the client must reject a server outside its configured CA"
    );
    drop(wrong_ca_client);
    server.stop().await;
}

/// The client must reject a server whose certificate name does not match the pinned domain.
#[tokio::test]
async fn client_pinning_a_mismatched_server_name_is_rejected() {
    let certificates = make_test_certificates().expect("create isolated test-only TLS material");
    let server = spawn_test_server(&certificates).await;
    let mut wrong_name_client = client_for(
        server.address,
        client_tls_config(
            "wrong.example",
            &certificates.ca_certificate,
            &certificates.client_certificate,
            &certificates.client_private_key,
        )
        .expect("build a client with a mismatched server name"),
    );
    assert!(
        wrong_name_client
            .append(append_probe(b"wrong server name"))
            .await
            .is_err(),
        "the client must reject a server whose certificate name does not match"
    );
    drop(wrong_name_client);
    server.stop().await;
}

/// PostgreSQL-backed proof: a trusted client reaches a bounded append, and SQL-layer revocation
/// of the exact peer certificate makes the next authenticated operation fail closed.
#[tokio::test]
#[ignore = "requires STAR_PLATFORM_PROFILE_TEST_DATABASE_URL pointing at a loopback disposable PostgreSQL database named star_d19_test on a non-default port, with the four D20 migrations applied and login memberships in star_platform_profile_owner, star_platform_transport_owner, and star_platform_transport_runtime"]
async fn trusted_append_succeeds_and_sql_revocation_fails_the_followup() {
    let database_url = env::var(DATABASE_URL_ENV).expect(
        "set STAR_PLATFORM_PROFILE_TEST_DATABASE_URL to a disposable PostgreSQL database; never use a production database",
    );
    let database_options = database_url
        .parse::<PgConnectOptions>()
        .expect("parse the disposable PostgreSQL connection options");
    let database_host = database_options.get_host();
    let is_loopback = database_host == "localhost"
        || database_host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback());
    assert!(
        is_loopback && database_options.get_port() != 5432,
        "the opt-in integration test must use a loopback PostgreSQL instance on a non-default port"
    );
    let admin_pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect to the disposable PostgreSQL database");
    require_lane_database_name(&admin_pool).await;
    require_d20_schema(&admin_pool).await;

    let certificates = make_test_certificates().expect("create isolated test-only TLS material");
    let fixture = prepare_fixture(&admin_pool, &certificates.client_sha256).await;
    let runtime_pool = PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|connection, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE star_platform_transport_runtime")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect(&database_url)
        .await
        .expect("connect a runtime-role database pool for the real gRPC service");
    let server = spawn_test_server_with_pool(&certificates, runtime_pool.clone()).await;

    let mut trusted_client = client_for(
        server.address,
        client_tls_config(
            "localhost",
            &certificates.ca_certificate,
            &certificates.client_certificate,
            &certificates.client_private_key,
        )
        .expect("build the production client TLS configuration"),
    );
    let first_wire = b"live native gRPC mTLS append".to_vec();
    let accepted = trusted_client
        .append(append_request(&fixture, &first_wire))
        .await
        .expect("trusted client certificate should reach the PostgreSQL-backed RPC")
        .into_inner();
    assert_eq!(accepted.log_id, fixture.log_id);
    assert_eq!(accepted.log_generation, fixture.generation.as_bytes());
    assert!(accepted.sequence > 0);
    assert_eq!(accepted.wire_sha256, Sha256::digest(&first_wire).as_slice());

    revoke_certificate(&admin_pool, &fixture)
        .await
        .expect("revoke the exact client certificate identity in PostgreSQL");
    let revoked = trusted_client
        .append(append_request(&fixture, b"revoked certificate follow-up"))
        .await
        .expect_err("the established TLS connection must recheck revocation on each RPC");
    assert_eq!(revoked.code(), Code::PermissionDenied);

    drop(trusted_client);
    server.stop().await;
    runtime_pool.close().await;
    admin_pool.close().await;
}

async fn spawn_test_server(certificates: &TestCertificates) -> TestServerHandle {
    // The handshake-level cases never reach the database, so a lazy pool that is never
    // connected is enough: no PostgreSQL instance is required for them to run.
    let disconnected_pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy("postgres://127.0.0.1:1/native-grpc-tls-e2e-disconnected")
        .expect("construct a lazy pool that is never connected for the TLS-only cases");
    spawn_test_server_with_pool(certificates, disconnected_pool).await
}

async fn spawn_test_server_with_pool(
    certificates: &TestCertificates,
    pool: PgPool,
) -> TestServerHandle {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind a loopback-only ephemeral gRPC port");
    let address = listener.local_addr().expect("read the bound gRPC address");
    let server_tls = server_mtls_config(
        &certificates.server_certificate,
        &certificates.server_private_key,
        &certificates.ca_certificate,
    )
    .expect("build the production mandatory-mTLS server configuration");
    let server_service = NativeGrpcTransportServer::new(pool).into_service();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        Server::builder()
            .tls_config(server_tls)
            .expect("configure Tonic TLS with the production mTLS settings")
            .add_service(server_service)
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                let _ = shutdown_rx.await;
            })
            .await
    });
    TestServerHandle {
        address,
        shutdown: Some(shutdown_tx),
        task,
    }
}

async fn require_lane_database_name(pool: &PgPool) {
    let current_database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await
        .expect("read the connected database identity before creating fixture facts");
    assert_eq!(
        current_database, "star_d19_test",
        "this lane may write fixtures only to its dedicated star_d19_test database"
    );
}

fn client_for(address: SocketAddr, tls: ClientTlsConfig) -> DurableTransportClient<Channel> {
    let endpoint = Endpoint::from_shared(format!("https://localhost:{}", address.port()))
        .expect("construct a valid loopback gRPC endpoint")
        .timeout(REQUEST_TIMEOUT)
        .tls_config(tls)
        .expect("configure the client TLS identity");
    DurableTransportClient::new(endpoint.connect_lazy())
}

/// A well-formed append for the TLS-only cases; the profile digest is shape-valid but the
/// request never reaches the SQL layer without a live database.
fn append_probe(bytes: &[u8]) -> wire::AppendRequest {
    wire::AppendRequest {
        profile: Some(wire::ProfilePin {
            profile_digest: vec![0u8; 32],
        }),
        recipient_service_id: Uuid::new_v4().as_bytes().to_vec(),
        message_kind: MESSAGE_KIND.to_owned(),
        message_id: Uuid::new_v4().as_bytes().to_vec(),
        wire: bytes.to_vec(),
        wire_sha256: Sha256::digest(bytes).to_vec(),
    }
}

fn append_request(fixture: &TestFixture, bytes: &[u8]) -> wire::AppendRequest {
    wire::AppendRequest {
        profile: Some(wire::ProfilePin {
            profile_digest: fixture.profile_digest.clone(),
        }),
        recipient_service_id: fixture.recipient_service_id.as_bytes().to_vec(),
        message_kind: MESSAGE_KIND.to_owned(),
        message_id: Uuid::new_v4().as_bytes().to_vec(),
        wire: bytes.to_vec(),
        wire_sha256: Sha256::digest(bytes).to_vec(),
    }
}

fn make_test_certificates() -> Result<TestCertificates, Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let ca_key = KeyPair::generate()?;
    let mut ca_parameters = CertificateParams::default();
    ca_parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_parameters
        .distinguished_name
        .push(DnType::CommonName, "Star D19 disposable test CA");
    let ca = ca_parameters.self_signed(&ca_key)?;

    let wrong_ca_key = KeyPair::generate()?;
    let mut wrong_ca_parameters = CertificateParams::default();
    wrong_ca_parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    wrong_ca_parameters
        .distinguished_name
        .push(DnType::CommonName, "Star D19 unrelated test CA");
    let wrong_ca = wrong_ca_parameters.self_signed(&wrong_ca_key)?;

    let server_key = KeyPair::generate()?;
    let mut server_parameters = CertificateParams::new(vec!["localhost".to_owned()])?;
    server_parameters
        .distinguished_name
        .push(DnType::CommonName, "localhost");
    let server =
        server_parameters.signed_by(&server_key, &Issuer::from_params(&ca_parameters, &ca_key))?;

    let client_key = KeyPair::generate()?;
    let mut client_parameters = CertificateParams::default();
    client_parameters
        .distinguished_name
        .push(DnType::CommonName, "Star D19 disposable RPC client");
    let client =
        client_parameters.signed_by(&client_key, &Issuer::from_params(&ca_parameters, &ca_key))?;
    let client_sha256 = Sha256::digest(client.der().as_ref()).into();

    let ca_certificate = directory.path().join("ca.pem");
    let wrong_ca_certificate = directory.path().join("wrong-ca.pem");
    let server_certificate = directory.path().join("server.pem");
    let server_private_key = directory.path().join("server-key.pem");
    let client_certificate = directory.path().join("client.pem");
    let client_private_key = directory.path().join("client-key.pem");
    write_pem(&ca_certificate, ca.pem())?;
    write_pem(&wrong_ca_certificate, wrong_ca.pem())?;
    write_pem(&server_certificate, server.pem())?;
    write_pem(&server_private_key, server_key.serialize_pem())?;
    write_pem(&client_certificate, client.pem())?;
    write_pem(&client_private_key, client_key.serialize_pem())?;

    Ok(TestCertificates {
        _directory: directory,
        ca_certificate,
        wrong_ca_certificate,
        server_certificate,
        server_private_key,
        client_certificate,
        client_private_key,
        client_sha256,
    })
}

fn write_pem(path: &Path, pem: String) -> Result<(), Box<dyn Error>> {
    fs::write(path, pem)?;
    Ok(())
}

async fn require_d20_schema(pool: &PgPool) {
    let ready: bool = sqlx::query_scalar(
        r#"
        SELECT to_regclass('platform_messaging.transport_profile_revision') IS NOT NULL
           AND to_regclass('platform_transport.service_certificate_identity') IS NOT NULL
           AND to_regclass('platform_transport.profile_log_binding') IS NOT NULL
           AND to_regclass('platform_transport.log_master') IS NOT NULL
           AND to_regprocedure('platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)') IS NOT NULL
           AND pg_has_role(current_user, 'star_platform_profile_owner', 'MEMBER')
           AND pg_has_role(current_user, 'star_platform_transport_owner', 'MEMBER')
           AND pg_has_role(current_user, 'star_platform_transport_runtime', 'MEMBER')
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("inspect the isolated D20 PostgreSQL prerequisites");
    assert!(
        ready,
        "apply the four D20 migrations and grant the disposable database login membership in profile_owner, transport_owner, and transport_runtime"
    );
}

async fn prepare_fixture(pool: &PgPool, client_sha256: &[u8; 32]) -> TestFixture {
    let fixture = TestFixture {
        deployment_id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        actor_service_id: Uuid::new_v4(),
        sender_service_id: Uuid::new_v4(),
        recipient_service_id: Uuid::new_v4(),
        profile_id: Uuid::new_v4(),
        profile_digest: Vec::new(),
        log_id: format!("native-mtls-{}", Uuid::new_v4().simple()),
        generation: Uuid::new_v4(),
        client_sha256: *client_sha256,
    };
    let definition = TransportProfileDefinition::new(
        PROVIDER_ID.to_owned(),
        PROVIDER_VERSION.to_owned(),
        "native-grpc-mtls-e2e".to_owned(),
        GENERATION_POLICY.to_owned(),
        vec![fixture.sender_service_id],
        vec![fixture.recipient_service_id],
        vec![TransportMessageKind::Request],
        Some(
            OpaqueSecretReference::new("secretref:test/native-grpc-mtls-e2e".to_owned())
                .expect("valid opaque test secret reference"),
        ),
    )
    .expect("construct a bounded test transport profile");
    let command = ProfileChangeRequest::new(
        fixture.profile_id,
        None,
        TransportProfileLifecycle::Active,
        definition,
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        format!("native-mtls-create-{}", Uuid::new_v4().simple()),
    )
    .expect("construct the test profile publication request");
    let profile_master = PgTransportProfileMaster::new(
        pool.clone(),
        fixture.deployment_id,
        fixture.actor_service_id,
    )
    .expect("construct the scoped test profile owner");
    let revision = profile_master
        .publish_revision(fixture.tenant_id, &command)
        .await
        .expect("publish the disposable test profile");
    let mut fixture = fixture;
    fixture.profile_digest = revision.digest().to_vec();
    provision_fixture(pool, &fixture, revision.revision()).await;
    fixture
}

async fn provision_fixture(pool: &PgPool, fixture: &TestFixture, revision: i64) {
    let mut tx = pool.begin().await.expect("begin test owner transaction");
    set_owner_scope(&mut tx, fixture).await;
    sqlx::query("SET LOCAL ROLE star_platform_transport_owner")
        .execute(&mut *tx)
        .await
        .expect("assume the restricted transport owner role");
    sqlx::query(
        r#"
        INSERT INTO platform_transport.service_certificate_identity (
            certificate_sha256, deployment_id, tenant_id, service_id,
            provider_id, provider_version, endpoint_identity, lifecycle_state,
            registered_by_service_id, registration_request_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8, $9)
        "#,
    )
    .bind(fixture.client_sha256.as_slice())
    .bind(fixture.deployment_id)
    .bind(fixture.tenant_id)
    .bind(fixture.sender_service_id)
    .bind(PROVIDER_ID)
    .bind(PROVIDER_VERSION)
    .bind("native-grpc-mtls-e2e")
    .bind(fixture.actor_service_id)
    .bind(Uuid::new_v4())
    .execute(&mut *tx)
    .await
    .expect("register the exact disposable client-certificate fingerprint");
    sqlx::query(
        r#"
        INSERT INTO platform_transport.log_master (
            deployment_id, tenant_id, log_id, generation, sender_service_id,
            recipient_service_id, message_kind, payload_budget_bytes,
            terminal_retention_hours, max_delivery_attempts, lifecycle
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 65536, 24, 6, 'active')
        "#,
    )
    .bind(fixture.deployment_id)
    .bind(fixture.tenant_id)
    .bind(&fixture.log_id)
    .bind(fixture.generation)
    .bind(fixture.sender_service_id)
    .bind(fixture.recipient_service_id)
    .bind(MESSAGE_KIND)
    .execute(&mut *tx)
    .await
    .expect("create the isolated bounded message log");
    sqlx::query(
        r#"
        INSERT INTO platform_transport.profile_log_binding (
            deployment_id, tenant_id, profile_id, revision, profile_digest,
            log_generation_policy, sender_service_id, recipient_service_id,
            message_kind, log_id, generation, provisioned_by_service_id, request_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
    )
    .bind(fixture.deployment_id)
    .bind(fixture.tenant_id)
    .bind(fixture.profile_id)
    .bind(revision)
    .bind(&fixture.profile_digest)
    .bind(GENERATION_POLICY)
    .bind(fixture.sender_service_id)
    .bind(fixture.recipient_service_id)
    .bind(MESSAGE_KIND)
    .bind(&fixture.log_id)
    .bind(fixture.generation)
    .bind(fixture.actor_service_id)
    .bind(Uuid::new_v4())
    .execute(&mut *tx)
    .await
    .expect("bind the profile to the test log generation");
    tx.commit()
        .await
        .expect("commit the isolated route fixture");
}

async fn set_owner_scope(tx: &mut Transaction<'_, Postgres>, fixture: &TestFixture) {
    for (setting, value) in [
        (
            "app.platform_deployment_id",
            fixture.deployment_id.to_string(),
        ),
        ("app.platform_tenant_id", fixture.tenant_id.to_string()),
        (
            "app.platform_profile_tenant_id",
            fixture.tenant_id.to_string(),
        ),
        (
            "app.platform_actor_service_id",
            fixture.actor_service_id.to_string(),
        ),
    ] {
        sqlx::query("SELECT set_config($1, $2, true)")
            .bind(setting)
            .bind(value)
            .execute(&mut **tx)
            .await
            .expect("set the owner scope for the isolated fixture");
    }
}

async fn revoke_certificate(pool: &PgPool, fixture: &TestFixture) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_owner_scope(&mut tx, fixture).await;
    sqlx::query("SET LOCAL ROLE star_platform_transport_owner")
        .execute(&mut *tx)
        .await?;
    let updated = sqlx::query(
        r#"
        UPDATE platform_transport.service_certificate_identity
           SET lifecycle_state = 'revoked', revoked_by_service_id = $1,
               revocation_request_id = $2, revoked_at = transaction_timestamp()
         WHERE certificate_sha256 = $3
        "#,
    )
    .bind(fixture.actor_service_id)
    .bind(Uuid::new_v4())
    .bind(fixture.client_sha256.as_slice())
    .execute(&mut *tx)
    .await?;
    if updated.rows_affected() != 1 {
        tx.rollback().await?;
        return Err(sqlx::Error::RowNotFound);
    }
    tx.commit().await
}
