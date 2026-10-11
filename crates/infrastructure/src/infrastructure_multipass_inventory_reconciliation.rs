//! Read-only comparison of bounded Multipass observations with the Infrastructure owner catalog.
//!
//! A finding is a projection only. It never registers an observed instance, retires a catalog row,
//! proves capacity, or establishes guest/K3s/Runtime readiness.
//!
//! @cypher schema=1 source_sha256=0520799384c00a5f38910196ba1bb3f10feca0ba08435b75b770bcc377d1508c
//! MERGE (self:File {path:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs"})
//! MERGE (reconciler:Type {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::PostgresMultipassInventoryReconciler",kind:"struct"})
//! MERGE (report:Type {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::MultipassInventoryReconciliation",kind:"struct"})
//! MERGE (finding:Type {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::MultipassInventoryFinding",kind:"enum"})
//! MERGE (owner_state:Type {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::CatalogOwnerState",kind:"enum"})
//! MERGE (error:Type {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::InventoryReconciliationError",kind:"enum"})
//! MERGE (reconcile:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::PostgresMultipassInventoryReconciler.reconcile",kind:"method"})
//! MERGE (compare:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::compare_inventory",kind:"function"})
//! MERGE (test_module:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::tests",kind:"module"})
//! MERGE (registered_test:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::tests::registered_and_external_rows_are_classified",kind:"test"})
//! MERGE (missing_test:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::tests::empty_snapshot_reports_missing_without_retiring",kind:"test"})
//! MERGE (duplicate_test:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::tests::duplicate_catalog_names_fail_closed",kind:"test"})
//! MERGE (host_table:Table {id:"infrastructure.host_environment_catalog",class:"Master"})
//! MERGE (instance_table:Table {id:"infrastructure.multipass_instance_catalog",class:"Master"})
//! MERGE (event_table:Table {id:"infrastructure.resource_catalog_event",class:"Transaction"})
//! MERGE (snapshot:Type {id:"multipass_adapter::MultipassInventorySnapshot",kind:"external_type"})
//! MERGE (pool:Type {id:"sqlx::PgPool",kind:"external_type"})
//! MERGE (catalog_instance:Type {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::CatalogInstance",kind:"struct"})
//! MERGE (finding_name:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::MultipassInventoryFinding.provider_instance_name",kind:"method"})
//! MERGE (report_host:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::MultipassInventoryReconciliation.host_environment",kind:"method"})
//! MERGE (report_findings:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::MultipassInventoryReconciliation.findings",kind:"method"})
//! MERGE (present_count:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::MultipassInventoryReconciliation.managed_present_count",kind:"method"})
//! MERGE (missing_count:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::MultipassInventoryReconciliation.managed_missing_count",kind:"method"})
//! MERGE (external_count:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::MultipassInventoryReconciliation.external_observed_count",kind:"method"})
//! MERGE (reconciler_new:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::PostgresMultipassInventoryReconciler.new",kind:"method"})
//! MERGE (parse_owner_state:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::parse_owner_state",kind:"function"})
//! MERGE (catalog_test_helper:Symbol {id:"crates/infrastructure/src/infrastructure_multipass_inventory_reconciliation.rs::tests::catalog_instance",kind:"function"})
//! MERGE (self)-[:DEFINES]->(catalog_instance)
//! MERGE (self)-[:DEFINES]->(test_module)
//! MERGE (self)-[:DEFINES]->(parse_owner_state)
//! MERGE (finding)-[:DEFINES]->(finding_name)
//! MERGE (report)-[:DEFINES]->(report_host)
//! MERGE (report)-[:DEFINES]->(report_findings)
//! MERGE (report)-[:DEFINES]->(present_count)
//! MERGE (report)-[:DEFINES]->(missing_count)
//! MERGE (report)-[:DEFINES]->(external_count)
//! MERGE (reconciler)-[:DEFINES]->(reconciler_new)
//! MERGE (reconcile)-[:CALLS]->(parse_owner_state)
//! MERGE (compare)-[:USES_TYPE]->(catalog_instance)
//! MERGE (compare)-[:USES_TYPE]->(instance)
//! MERGE (compare)-[:USES_TYPE]->(state)
//! MERGE (registered_test)-[:CALLS]->(catalog_test_helper)
//! MERGE (missing_test)-[:CALLS]->(catalog_test_helper)
//! MERGE (duplicate_test)-[:CALLS]->(catalog_test_helper)
//! MERGE (self)-[:DEFINES]->(reconciler)
//! MERGE (self)-[:DEFINES]->(report)
//! MERGE (self)-[:DEFINES]->(finding)
//! MERGE (self)-[:DEFINES]->(owner_state)
//! MERGE (self)-[:DEFINES]->(error)
//! MERGE (self)-[:DEFINES]->(compare)
//! MERGE (reconciler)-[:DEFINES]->(reconcile)
//! MERGE (reconcile)-[:USES_TYPE]->(pool)
//! MERGE (reconcile)-[:USES_TYPE]->(snapshot)
//! MERGE (reconcile)-[:CALLS]->(compare)
//! MERGE (reconcile)-[:READS]->(host_table)
//! MERGE (reconcile)-[:READS]->(instance_table)
//! MERGE (reconcile)-[:READS]->(event_table)
//! MERGE (test_module)-[:DEFINES]->(registered_test)
//! MERGE (test_module)-[:DEFINES]->(missing_test)
//! MERGE (test_module)-[:DEFINES]->(duplicate_test)
//! MERGE (registered_test)-[:TESTS]->(compare)
//! MERGE (missing_test)-[:TESTS]->(compare)
//! MERGE (duplicate_test)-[:TESTS]->(compare)
//! @endcypher

use std::collections::{HashMap, HashSet};

use multipass_adapter::{MultipassInstance, MultipassInstanceState, MultipassInventorySnapshot};
use sqlx::{PgPool, Row};
use thiserror::Error;
use uuid::Uuid;

const OWNER_ROLE: &str = "SET LOCAL ROLE star_infrastructure_owner";
const MAX_CATALOG_ROWS: usize = multipass_adapter::MAX_INVENTORY_ROWS;
const CATALOG_QUERY: &str = r#"
SELECT instance.multipass_instance_id,
       instance.generation,
       instance.provider_instance_name,
       (
           SELECT event.owner_state
             FROM infrastructure.resource_catalog_event AS event
            WHERE event.resource_kind = 'multipass_instance'
              AND event.resource_id = instance.multipass_instance_id
              AND event.resource_generation = instance.generation
            ORDER BY event.recorded_at DESC, event.event_id DESC
            LIMIT 1
       ) AS owner_state
  FROM infrastructure.multipass_instance_catalog AS instance
 WHERE instance.host_environment_id = $1
   AND instance.host_generation = $2
   AND instance.valid_to IS NULL
 ORDER BY instance.provider_instance_name, instance.multipass_instance_id
 LIMIT $3
"#;

/// Infrastructure catalog state for a current Multipass identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogOwnerState {
    /// The resource is eligible for a new binding subject to the other authorization gates.
    Active,
    /// The resource remains registered but is draining.
    Draining,
}

/// One item in a read-only Multipass inventory reconciliation result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MultipassInventoryFinding {
    /// A provider row exactly matches one current Infrastructure-owned identity.
    ManagedPresent {
        /// Stable catalog identity.
        resource_id: Uuid,
        /// Exact SCD2 generation.
        generation: i64,
        /// Registered Multipass name matched byte-for-byte.
        provider_instance_name: String,
        /// Catalog lifecycle state; this does not imply readiness.
        owner_state: CatalogOwnerState,
        /// State returned by the read-only Multipass observation.
        observed_state: MultipassInstanceState,
    },
    /// A registered instance is absent from this complete, valid snapshot.
    ManagedMissing {
        /// Stable catalog identity.
        resource_id: Uuid,
        /// Exact SCD2 generation.
        generation: i64,
        /// Registered Multipass name expected in the inventory.
        provider_instance_name: String,
        /// Catalog lifecycle state; the row is not changed by this finding.
        owner_state: CatalogOwnerState,
    },
    /// An observed row has no matching current Infrastructure-owned identity.
    ExternalObserved {
        /// Provider-reported name; this is not a durable identity or ownership proof.
        provider_instance_name: String,
        /// State returned by the read-only Multipass observation.
        observed_state: MultipassInstanceState,
    },
}

impl MultipassInventoryFinding {
    /// Return the provider name used to order the result deterministically.
    pub fn provider_instance_name(&self) -> &str {
        match self {
            Self::ManagedPresent {
                provider_instance_name,
                ..
            }
            | Self::ManagedMissing {
                provider_instance_name,
                ..
            }
            | Self::ExternalObserved {
                provider_instance_name,
                ..
            } => provider_instance_name,
        }
    }
}

/// Read-only reconciliation output for one exact HostEnvironment generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultipassInventoryReconciliation {
    host_environment_id: Uuid,
    host_generation: i64,
    findings: Vec<MultipassInventoryFinding>,
}

impl MultipassInventoryReconciliation {
    /// Return the exact HostEnvironment identity used for this comparison.
    pub fn host_environment(&self) -> (Uuid, i64) {
        (self.host_environment_id, self.host_generation)
    }

    /// Return the deterministically ordered findings.
    pub fn findings(&self) -> &[MultipassInventoryFinding] {
        &self.findings
    }

    /// Count current catalog identities present in the observed inventory.
    pub fn managed_present_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|finding| matches!(finding, MultipassInventoryFinding::ManagedPresent { .. }))
            .count()
    }

    /// Count current catalog identities missing from the observed inventory.
    pub fn managed_missing_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|finding| matches!(finding, MultipassInventoryFinding::ManagedMissing { .. }))
            .count()
    }

    /// Count observed provider rows that are not registered to this HostEnvironment.
    pub fn external_observed_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|finding| matches!(finding, MultipassInventoryFinding::ExternalObserved { .. }))
            .count()
    }
}

/// PostgreSQL-backed, read-only view of the Infrastructure resource catalog.
#[derive(Clone)]
pub struct PostgresMultipassInventoryReconciler {
    pool: PgPool,
}

impl PostgresMultipassInventoryReconciler {
    /// Create a reconciler using the Infrastructure owner's configured PostgreSQL pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Compare a valid Multipass snapshot to current registered instances for one Host generation.
    ///
    /// The database remains authoritative. This method opens a repeatable-read, read-only
    /// transaction and never registers, retires, starts, stops, or deletes a resource. A database
    /// error is returned directly; callers must not turn it into an empty catalog or an approval.
    pub async fn reconcile(
        &self,
        host_environment_id: Uuid,
        host_generation: i64,
        snapshot: &MultipassInventorySnapshot,
    ) -> Result<MultipassInventoryReconciliation, InventoryReconciliationError> {
        if host_environment_id.is_nil() || host_generation <= 0 {
            return Err(InventoryReconciliationError::InvalidHostIdentity);
        }
        if snapshot.registered_count() > MAX_CATALOG_ROWS {
            return Err(InventoryReconciliationError::InventoryRowLimitExceeded);
        }

        let mut transaction = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *transaction)
            .await?;
        sqlx::query(OWNER_ROLE).execute(&mut *transaction).await?;
        sqlx::query("SET LOCAL statement_timeout = '5000ms'")
            .execute(&mut *transaction)
            .await?;

        let host_is_current: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM infrastructure.host_environment_catalog WHERE host_environment_id = $1 AND generation = $2 AND valid_to IS NULL)",
        )
        .bind(host_environment_id)
        .bind(host_generation)
        .fetch_one(&mut *transaction)
        .await?;
        if !host_is_current {
            transaction.rollback().await?;
            return Err(InventoryReconciliationError::HostEnvironmentNotCurrent);
        }

        let rows = sqlx::query(CATALOG_QUERY)
            .bind(host_environment_id)
            .bind(host_generation)
            .bind(MAX_CATALOG_ROWS as i64 + 1)
            .fetch_all(&mut *transaction)
            .await?;
        if rows.len() > MAX_CATALOG_ROWS {
            transaction.rollback().await?;
            return Err(InventoryReconciliationError::CatalogRowLimitExceeded);
        }

        let catalog = rows
            .into_iter()
            .map(|row| {
                let owner_state: Option<String> = row.try_get("owner_state")?;
                Ok(CatalogInstance {
                    resource_id: row.try_get("multipass_instance_id")?,
                    generation: row.try_get("generation")?,
                    provider_instance_name: row.try_get("provider_instance_name")?,
                    owner_state: parse_owner_state(
                        owner_state.ok_or(InventoryReconciliationError::MissingCatalogState)?,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, InventoryReconciliationError>>();

        transaction.commit().await?;
        compare_inventory(
            host_environment_id,
            host_generation,
            snapshot.instances(),
            catalog?,
        )
    }
}

/// Errors returned by the read-only inventory reconciliation boundary.
#[derive(Debug, Error)]
pub enum InventoryReconciliationError {
    /// The Host identity is nil or has an invalid generation.
    #[error("HostEnvironment identity must be non-nil with a positive generation")]
    InvalidHostIdentity,
    /// The observation exceeded the adapter's hard row bound.
    #[error("Multipass inventory exceeds the reconciliation row limit")]
    InventoryRowLimitExceeded,
    /// The requested Host generation is not the current owner Master row.
    #[error("HostEnvironment generation is not current in the Infrastructure catalog")]
    HostEnvironmentNotCurrent,
    /// The owner catalog contained more current rows than the bounded reconciler accepts.
    #[error("Infrastructure catalog exceeds the reconciliation row limit")]
    CatalogRowLimitExceeded,
    /// A current resource had no matching lifecycle event.
    #[error("current Multipass catalog row has no lifecycle event")]
    MissingCatalogState,
    /// A current catalog row had a duplicate provider name for this Host generation.
    #[error("current Multipass catalog contains an ambiguous provider instance name")]
    DuplicateCatalogName,
    /// A snapshot contains duplicate provider names after parsing.
    #[error("Multipass inventory contains an ambiguous provider instance name")]
    DuplicateObservedName,
    /// A current catalog row had an owner state outside its supported state machine.
    #[error("current Multipass catalog has unsupported owner state: {0}")]
    UnsupportedCatalogState(String),
    /// A PostgreSQL query or transaction failed; no empty-catalog fallback is allowed.
    #[error("Infrastructure catalog query failed: {0}")]
    Database(#[from] sqlx::Error),
}

struct CatalogInstance {
    resource_id: Uuid,
    generation: i64,
    provider_instance_name: String,
    owner_state: CatalogOwnerState,
}

fn parse_owner_state(value: String) -> Result<CatalogOwnerState, InventoryReconciliationError> {
    match value.as_str() {
        "active" => Ok(CatalogOwnerState::Active),
        "draining" => Ok(CatalogOwnerState::Draining),
        other => Err(InventoryReconciliationError::UnsupportedCatalogState(
            other.to_owned(),
        )),
    }
}

fn compare_inventory(
    host_environment_id: Uuid,
    host_generation: i64,
    observed: &[MultipassInstance],
    catalog: Vec<CatalogInstance>,
) -> Result<MultipassInventoryReconciliation, InventoryReconciliationError> {
    let mut observed_by_name = HashMap::with_capacity(observed.len());
    for instance in observed {
        if observed_by_name.insert(instance.name(), instance).is_some() {
            return Err(InventoryReconciliationError::DuplicateObservedName);
        }
    }

    let mut managed_names = HashSet::with_capacity(catalog.len());
    let mut findings = Vec::with_capacity(observed.len().saturating_add(catalog.len()));
    for instance in catalog {
        if !managed_names.insert(instance.provider_instance_name.clone()) {
            return Err(InventoryReconciliationError::DuplicateCatalogName);
        }
        match observed_by_name.remove(instance.provider_instance_name.as_str()) {
            Some(observed) => findings.push(MultipassInventoryFinding::ManagedPresent {
                resource_id: instance.resource_id,
                generation: instance.generation,
                provider_instance_name: instance.provider_instance_name,
                owner_state: instance.owner_state,
                observed_state: observed.state().clone(),
            }),
            None => findings.push(MultipassInventoryFinding::ManagedMissing {
                resource_id: instance.resource_id,
                generation: instance.generation,
                provider_instance_name: instance.provider_instance_name,
                owner_state: instance.owner_state,
            }),
        }
    }
    for (provider_instance_name, instance) in observed_by_name {
        findings.push(MultipassInventoryFinding::ExternalObserved {
            provider_instance_name: provider_instance_name.to_owned(),
            observed_state: instance.state().clone(),
        });
    }
    findings.sort_by(|left, right| {
        left.provider_instance_name()
            .cmp(right.provider_instance_name())
    });

    Ok(MultipassInventoryReconciliation {
        host_environment_id,
        host_generation,
        findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog_instance(
        resource_id: u128,
        provider_instance_name: &str,
        owner_state: CatalogOwnerState,
    ) -> CatalogInstance {
        CatalogInstance {
            resource_id: Uuid::from_u128(resource_id),
            generation: 1,
            provider_instance_name: provider_instance_name.to_owned(),
            owner_state,
        }
    }

    #[test]
    fn registered_and_external_rows_are_classified() {
        let snapshot = MultipassInventorySnapshot::parse_json(
            br#"{"list":[{"name":"ci-test-01","state":"Stopped"},{"name":"external-02","state":"Deleted"}]}"#,
        )
        .expect("two bounded inventory rows should parse");
        let report = compare_inventory(
            Uuid::from_u128(1),
            1,
            snapshot.instances(),
            vec![catalog_instance(2, "ci-test-01", CatalogOwnerState::Active)],
        )
        .expect("the owner name has one exact match");

        assert_eq!(report.managed_present_count(), 1);
        assert_eq!(report.managed_missing_count(), 0);
        assert_eq!(report.external_observed_count(), 1);
        assert_eq!(report.findings()[0].provider_instance_name(), "ci-test-01");
        assert_eq!(report.findings()[1].provider_instance_name(), "external-02");
        assert!(matches!(
            &report.findings()[0],
            MultipassInventoryFinding::ManagedPresent {
                observed_state: MultipassInstanceState::Stopped,
                ..
            }
        ));
        assert!(matches!(
            &report.findings()[1],
            MultipassInventoryFinding::ExternalObserved {
                observed_state: MultipassInstanceState::Deleted,
                ..
            }
        ));
    }

    #[test]
    fn empty_snapshot_reports_missing_without_retiring() {
        let snapshot = MultipassInventorySnapshot::parse_json(br#"{"list":[]}"#)
            .expect("empty inventory is a valid observation");
        let report = compare_inventory(
            Uuid::from_u128(1),
            1,
            snapshot.instances(),
            vec![catalog_instance(
                2,
                "ci-test-01",
                CatalogOwnerState::Draining,
            )],
        )
        .expect("absence is a report, not a write");

        assert_eq!(report.managed_missing_count(), 1);
        assert_eq!(report.external_observed_count(), 0);
        assert!(matches!(
            &report.findings()[0],
            MultipassInventoryFinding::ManagedMissing {
                owner_state: CatalogOwnerState::Draining,
                ..
            }
        ));
    }

    #[test]
    fn duplicate_catalog_names_fail_closed() {
        let snapshot = MultipassInventorySnapshot::parse_json(br#"{"list":[]}"#)
            .expect("empty inventory is a valid observation");
        let result = compare_inventory(
            Uuid::from_u128(1),
            1,
            snapshot.instances(),
            vec![
                catalog_instance(2, "ci-test-01", CatalogOwnerState::Active),
                catalog_instance(3, "ci-test-01", CatalogOwnerState::Active),
            ],
        );

        assert!(matches!(
            result,
            Err(InventoryReconciliationError::DuplicateCatalogName)
        ));
    }
}
