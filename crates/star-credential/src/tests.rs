//! crates/star-credential — 4 e2e tests
//!
//! 真实应用场景: 用户 UI 填入 OpenClaw / Hermes / KMS 凭证 → 后端 CredentialManager
//! 加密存储 + 运行时解密读取 (per V2 阶段 + 守门 #5 env 安全 + 守门 #14 5 域 Lead)

use super::*;

fn make_plaintext(secret: &str) -> CredentialPlaintext {
    CredentialPlaintext {
        secret: secret.to_string(),
        base_url: Some("https://api.example.com/v1".to_string()),
        region: None,
    }
}

/// V2-1 test 1: store + retrieve round-trip
#[tokio::test]
async fn v2_store_and_retrieve_round_trip() {
    let manager = CredentialManager::with_local_mock_kms();
    let metadata = CredentialMetadata {
        display_name: "我的 OpenClaw 账号".into(),
        description: "用于 LangGraph sub-agent 派发".into(),
    };
    let id = manager
        .store(
            "tenant-1",
            "user-1",
            Provider::OpenClaw,
            metadata,
            make_plaintext("oc_live_secret_xxx"),
        )
        .await
        .unwrap();

    let pt = manager
        .retrieve("tenant-1", Provider::OpenClaw)
        .await
        .unwrap();
    assert_eq!(pt.secret, "oc_live_secret_xxx");
    assert_eq!(pt.base_url, Some("https://api.example.com/v1".to_string()));
}

/// V2-1 test 2: 多 provider 隔离 (OpenClaw / Hermes / KMS Vault 各自独立)
#[tokio::test]
async fn v2_multi_provider_isolation() {
    let manager = CredentialManager::with_local_mock_kms();

    manager
        .store(
            "tenant-1",
            "user-1",
            Provider::OpenClaw,
            CredentialMetadata {
                display_name: "OpenClaw".into(),
                description: "".into(),
            },
            make_plaintext("oc_secret"),
        )
        .await
        .unwrap();
    manager
        .store(
            "tenant-1",
            "user-1",
            Provider::Hermes,
            CredentialMetadata {
                display_name: "Hermes".into(),
                description: "".into(),
            },
            make_plaintext("hm_secret"),
        )
        .await
        .unwrap();
    manager
        .store(
            "tenant-1",
            "user-1",
            Provider::KmsVault,
            CredentialMetadata {
                display_name: "Vault".into(),
                description: "".into(),
            },
            make_plaintext("vault_token_xxx"),
        )
        .await
        .unwrap();

    assert_eq!(
        manager
            .retrieve("tenant-1", Provider::OpenClaw)
            .await
            .unwrap()
            .secret,
        "oc_secret"
    );
    assert_eq!(
        manager
            .retrieve("tenant-1", Provider::Hermes)
            .await
            .unwrap()
            .secret,
        "hm_secret"
    );
    assert_eq!(
        manager
            .retrieve("tenant-1", Provider::KmsVault)
            .await
            .unwrap()
            .secret,
        "vault_token_xxx"
    );
}

/// V2-1 test 3: rotate 老凭证标 deprecated, 新凭证 active
#[tokio::test]
async fn v2_rotate_deprecates_old() {
    let manager = CredentialManager::with_local_mock_kms();
    manager
        .store(
            "tenant-1",
            "user-1",
            Provider::OpenClaw,
            CredentialMetadata {
                display_name: "OpenClaw v1".into(),
                description: "".into(),
            },
            make_plaintext("oc_v1_secret"),
        )
        .await
        .unwrap();

    let new_id = manager
        .rotate(
            "tenant-1",
            "user-1",
            Provider::OpenClaw,
            CredentialMetadata {
                display_name: "OpenClaw v2".into(),
                description: "用户轮换".into(),
            },
            make_plaintext("oc_v2_secret"),
        )
        .await
        .unwrap();

    // retrieve 应返回新凭证 (最新 active 优先)
    let pt = manager
        .retrieve("tenant-1", Provider::OpenClaw)
        .await
        .unwrap();
    assert_eq!(pt.secret, "oc_v2_secret");

    // 老凭证标 deprecated, 单独 retrieve 返 Deprecated 错
    let records = manager.list("tenant-1", Some(Provider::OpenClaw)).await;
    assert_eq!(records.len(), 2);
    let old = records.iter().find(|r| r.id != new_id).unwrap();
    assert_eq!(old.status, CredentialStatus::Deprecated);
    assert!(old.deprecated_at_ms.is_some());
}

/// V2-1 test 4: revoke 标 revoked, 不删 (per INV-CR-06)
#[tokio::test]
async fn v2_revoke_marks_not_deletes() {
    let manager = CredentialManager::with_local_mock_kms();
    let id = manager
        .store(
            "tenant-1",
            "user-1",
            Provider::Hermes,
            CredentialMetadata {
                display_name: "Hermes 撤销".into(),
                description: "用户主动撤销".into(),
            },
            make_plaintext("hm_secret_to_revoke"),
        )
        .await
        .unwrap();

    manager.revoke(&id).await.unwrap();

    // retrieve 返 Revoked 错
    let err = manager
        .retrieve("tenant-1", Provider::Hermes)
        .await
        .unwrap_err();
    assert!(matches!(err, CredentialError::Revoked(_)));

    // list 仍能看到 (per INV-CR-06 物理删除禁止)
    let records = manager.list("tenant-1", Some(Provider::Hermes)).await;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].status, CredentialStatus::Revoked);
    assert!(records[0].revoked_at_ms.is_some());
}
