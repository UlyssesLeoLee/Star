/*
@cypher schema=1 source_sha256=59d206c144c17f21360fc23c4b0c3d1ec37ef62e358f13f0ef7862e404afb9a3
MERGE (self:File {path:"crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs"})
MERGE (module:Symbol {id:"crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs::trusted_profile_endpoint",kind:"module"})
MERGE (resolver:Type {id:"crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs::TrustedProfileEndpointResolver",kind:"struct"})
MERGE (error:Type {id:"crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs::TrustedProfileEndpointError",kind:"enum"})
MERGE (new:Symbol {id:"crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs::TrustedProfileEndpointResolver.new",kind:"method"})
MERGE (resolve:Symbol {id:"crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs::TrustedProfileEndpointResolver.resolve",kind:"method"})
MERGE (profile_resolver:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileResolver",kind:"struct"})
MERGE (endpoint_resolver:Type {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointResolver",kind:"struct"})
MERGE (session:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::AuthenticatedTransportSession",kind:"trait"})
MERGE (message_kind:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportMessageKind",kind:"enum"})
MERGE (profile_revision:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileRevision",kind:"struct"})
MERGE (resolved_endpoint:Type {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::ResolvedTransportEndpoint",kind:"struct"})
MERGE (profile_error:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::TransportProfileStoreError",kind:"enum"})
MERGE (endpoint_error:Type {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::EndpointResolutionError",kind:"enum"})
MERGE (profile_file:File {path:"crates/platform-messaging-runtime/src/transport_profile.rs"})
MERGE (profile_pg_file:File {path:"crates/platform-messaging-runtime/src/transport_profile_pg.rs"})
MERGE (endpoint_file:File {path:"crates/platform-messaging-runtime/src/transport_endpoint.rs"})
MERGE (profile_resolve:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileResolver.resolve",kind:"method"})
MERGE (endpoint_resolve:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointResolver.resolve",kind:"method"})
MERGE (result_map_err:Function {name:"std::result::Result::map_err",type:"function",language:"rust"})
MERGE (to_string:Function {name:"std::string::ToString::to_string",type:"function",language:"rust"})
MERGE (tests:Symbol {id:"crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs::tests",kind:"module"})
MERGE (test_errors:Symbol {id:"crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs::tests.public_errors_are_constant_and_bounded",kind:"test"})
MERGE (profiles_arg:Variable {name:"profiles",type:"variable",language:"rust"})
MERGE (endpoints_arg:Variable {name:"endpoints",type:"variable",language:"rust"})
MERGE (session_arg:Variable {name:"session",type:"variable",language:"rust"})
MERGE (profile_id_arg:Variable {name:"profile_id",type:"variable",language:"rust"})
MERGE (kind_arg:Variable {name:"kind",type:"variable",language:"rust"})
MERGE (profile_local:Variable {name:"profile",type:"variable",language:"rust"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(resolver)
MERGE (module)-[:DEFINES]->(error)
MERGE (module)-[:DEFINES]->(new)
MERGE (module)-[:DEFINES]->(resolve)
MERGE (module)-[:DEFINES]->(tests)
MERGE (tests)-[:DEFINES]->(test_errors)
MERGE (new)-[:USES_TYPE]->(profile_resolver)
MERGE (new)-[:USES_TYPE]->(endpoint_resolver)
MERGE (resolve)-[:USES_TYPE]->(session)
MERGE (resolve)-[:USES_TYPE]->(message_kind)
MERGE (resolve)-[:USES_TYPE]->(profile_revision)
MERGE (resolve)-[:USES_TYPE]->(resolved_endpoint)
MERGE (resolve)-[:USES_TYPE]->(error)
MERGE (resolve)-[:USES_TYPE]->(profile_error)
MERGE (resolve)-[:USES_TYPE]->(endpoint_error)
MERGE (resolve)-[:CALLS]->(profile_resolve)
MERGE (resolve)-[:CALLS]->(endpoint_resolve)
MERGE (resolve)-[:CALLS]->(result_map_err)
MERGE (resolve)-[:USES]->(profile_local)
MERGE (test_errors)-[:CALLS]->(to_string)
MERGE (test_errors)-[:TESTS]->(error)
MERGE (new)-[:USES]->(profiles_arg)
MERGE (new)-[:USES]->(endpoints_arg)
MERGE (resolve)-[:USES]->(session_arg)
MERGE (resolve)-[:USES]->(profile_id_arg)
MERGE (resolve)-[:USES]->(kind_arg)
MERGE (self)-[:IMPORTS]->(profile_file)
MERGE (self)-[:IMPORTS]->(profile_pg_file)
MERGE (self)-[:IMPORTS]->(endpoint_file)
@endcypher
*/

use thiserror::Error;
use uuid::Uuid;

use crate::{
    transport_endpoint::{ResolvedTransportEndpoint, TrustedEndpointResolver},
    transport_profile::{AuthenticatedTransportSession, TransportMessageKind},
    transport_profile_pg::PgTransportProfileResolver,
};

/// Resolves the current authorized profile before selecting its deployment-trusted endpoint.
///
/// The endpoint registry is fixed when this resolver is constructed. Profile revisions are
/// deliberately read from PostgreSQL on every call so a later disable or revocation is observed.
#[derive(Clone)]
pub struct TrustedProfileEndpointResolver {
    profiles: PgTransportProfileResolver,
    endpoints: TrustedEndpointResolver,
}

impl TrustedProfileEndpointResolver {
    /// Compose the uncached profile reader with the deployment-owned endpoint registry.
    pub fn new(profiles: PgTransportProfileResolver, endpoints: TrustedEndpointResolver) -> Self {
        Self {
            profiles,
            endpoints,
        }
    }

    /// Resolve one current profile for trusted application routing and map its endpoint pin.
    ///
    /// `profile_id` and `kind` must come from trusted server-side routing, not request payload
    /// data. This method accepts no destination, TLS name, or endpoint identity from the caller.
    pub async fn resolve<S: AuthenticatedTransportSession>(
        &self,
        session: &S,
        profile_id: Uuid,
        kind: TransportMessageKind,
    ) -> Result<ResolvedTransportEndpoint, TrustedProfileEndpointError> {
        let profile = self
            .profiles
            .resolve(session, profile_id, kind)
            .await
            .map_err(|_| TrustedProfileEndpointError::CurrentProfileUnavailable)?;

        self.endpoints
            .resolve(&profile)
            .map_err(|_| TrustedProfileEndpointError::TrustedEndpointUnavailable)
    }
}

/// Bounded failures that do not expose database diagnostics, endpoint URIs, or secret references.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum TrustedProfileEndpointError {
    /// The current profile could not be read or authorized.
    #[error("current transport profile resolution failed")]
    CurrentProfileUnavailable,
    /// The profile pin could not be mapped to the trusted endpoint registry.
    #[error("trusted transport endpoint resolution failed")]
    TrustedEndpointUnavailable,
}

#[cfg(test)]
mod tests {
    use super::TrustedProfileEndpointError;

    #[test]
    fn public_errors_are_constant_and_bounded() {
        assert_eq!(
            TrustedProfileEndpointError::CurrentProfileUnavailable.to_string(),
            "current transport profile resolution failed"
        );
        assert_eq!(
            TrustedProfileEndpointError::TrustedEndpointUnavailable.to_string(),
            "trusted transport endpoint resolution failed"
        );
    }
}
