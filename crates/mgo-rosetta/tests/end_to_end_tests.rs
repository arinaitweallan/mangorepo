// Copyright (c) MangoNet Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use serde_json::json;

use rosetta_client::start_rosetta_test_server;
use mgo_json_rpc_types::MgoTransactionBlockResponseOptions;
use mgo_keys::keystore::AccountKeystore;
use mgo_rosetta::operations::Operations;
use mgo_rosetta::types::{
    AccountBalanceRequest, AccountBalanceResponse, AccountIdentifier, NetworkIdentifier,
    SubAccount, SubAccountType, MgoEnv,
};
use mgo_sdk::rpc_types::{MgoExecutionStatus, MgoTransactionBlockEffectsAPI};
use mgo_swarm_config::genesis_config::{DEFAULT_GAS_AMOUNT, DEFAULT_NUMBER_OF_OBJECT_PER_ACCOUNT};
use mgo_types::quorum_driver_types::ExecuteTransactionRequestType;
use mgo_types::utils::to_sender_signed_transaction;
use test_cluster::TestClusterBuilder;

use crate::rosetta_client::RosettaEndpoint;

mod rosetta_client;

#[tokio::test]
async fn test_get_staked_mgo() {
    let test_cluster = TestClusterBuilder::new().build().await;
    let address = test_cluster.get_address_0();
    let client = test_cluster.wallet.get_client().await.unwrap();
    let keystore = &test_cluster.wallet.config.keystore;

    let (rosetta_client, _handle) = start_rosetta_test_server(client.clone()).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let network_identifier = NetworkIdentifier {
        blockchain: "mgo".to_string(),
        network: MgoEnv::LocalNet,
    };
    // Verify initial balance and stake
    let request = AccountBalanceRequest {
        network_identifier: network_identifier.clone(),
        account_identifier: AccountIdentifier {
            address,
            sub_account: None,
        },
        block_identifier: Default::default(),
        currencies: vec![],
    };

    let response: AccountBalanceResponse = rosetta_client
        .call(RosettaEndpoint::Balance, &request)
        .await;
    assert_eq!(1, response.balances.len());
    assert_eq!(
        (DEFAULT_GAS_AMOUNT * DEFAULT_NUMBER_OF_OBJECT_PER_ACCOUNT as u64) as i128,
        response.balances[0].value
    );

    let request = AccountBalanceRequest {
        network_identifier: network_identifier.clone(),
        account_identifier: AccountIdentifier {
            address,
            sub_account: Some(SubAccount {
                account_type: SubAccountType::PendingStake,
            }),
        },
        block_identifier: Default::default(),
        currencies: vec![],
    };
    let response: AccountBalanceResponse = rosetta_client
        .call(RosettaEndpoint::Balance, &request)
        .await;
    assert_eq!(response.balances[0].value, 0);

    // Stake some mgo
    let validator = client
        .governance_api()
        .get_latest_mgo_system_state()
        .await
        .unwrap()
        .active_validators[0]
        .mgo_address;
    let coins = client
        .coin_read_api()
        .get_coins(address, None, None, None)
        .await
        .unwrap()
        .data;
    let delegation_tx = client
        .transaction_builder()
        .request_add_stake(
            address,
            vec![coins[0].coin_object_id],
            Some(1_000_000_000),
            validator,
            None,
            1_000_000_000,
        )
        .await
        .unwrap();
    let tx = to_sender_signed_transaction(delegation_tx, keystore.get_key(&address).unwrap());
    client
        .quorum_driver_api()
        .execute_transaction_block(
            tx,
            MgoTransactionBlockResponseOptions::new(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await
        .unwrap();

    let response = rosetta_client
        .get_balance(
            network_identifier.clone(),
            address,
            Some(SubAccountType::PendingStake),
        )
        .await;
    assert_eq!(1, response.balances.len());
    assert_eq!(1_000_000_000, response.balances[0].value);

    println!("{}", serde_json::to_string_pretty(&response).unwrap());
}

#[tokio::test]
async fn test_stake() {
    let test_cluster = TestClusterBuilder::new().build().await;
    let sender = test_cluster.get_address_0();
    let client = test_cluster.wallet.get_client().await.unwrap();
    let keystore = &test_cluster.wallet.config.keystore;

    let (rosetta_client, _handle) = start_rosetta_test_server(client.clone()).await;

    let validator = client
        .governance_api()
        .get_latest_mgo_system_state()
        .await
        .unwrap()
        .active_validators[0]
        .mgo_address;

    let ops = serde_json::from_value(json!(
        [{
            "operation_identifier":{"index":0},
            "type":"Stake",
            "account": { "address" : sender.to_string() },
            "amount" : { "value": "-1000000000" , "currency": { "symbol": "MGO", "decimals": 9}},
            "metadata": { "Stake" : {"validator": validator.to_string()} }
        }]
    ))
    .unwrap();

    let response = rosetta_client.rosetta_flow(&ops, keystore).await;

    let tx = client
        .read_api()
        .get_transaction_with_options(
            response.transaction_identifier.hash,
            MgoTransactionBlockResponseOptions::new()
                .with_input()
                .with_effects()
                .with_balance_changes()
                .with_events(),
        )
        .await
        .unwrap();

    println!("Mgo TX: {tx:?}");

    assert_eq!(
        &MgoExecutionStatus::Success,
        tx.effects.as_ref().unwrap().status()
    );

    let ops2 = Operations::try_from(tx).unwrap();
    assert!(
        ops2.contains(&ops),
        "Operation mismatch. expecting:{}, got:{}",
        serde_json::to_string(&ops).unwrap(),
        serde_json::to_string(&ops2).unwrap()
    );

    println!("{}", serde_json::to_string_pretty(&ops2).unwrap())
}

#[tokio::test]
async fn test_stake_all() {
    let test_cluster = TestClusterBuilder::new().build().await;
    let sender = test_cluster.get_address_0();
    let client = test_cluster.wallet.get_client().await.unwrap();
    let keystore = &test_cluster.wallet.config.keystore;

    let (rosetta_client, _handle) = start_rosetta_test_server(client.clone()).await;

    let validator = client
        .governance_api()
        .get_latest_mgo_system_state()
        .await
        .unwrap()
        .active_validators[0]
        .mgo_address;

    let ops = serde_json::from_value(json!(
        [{
            "operation_identifier":{"index":0},
            "type":"Stake",
            "account": { "address" : sender.to_string() },
            "metadata": { "Stake" : {"validator": validator.to_string()} }
        }]
    ))
    .unwrap();

    let response = rosetta_client.rosetta_flow(&ops, keystore).await;

    let tx = client
        .read_api()
        .get_transaction_with_options(
            response.transaction_identifier.hash,
            MgoTransactionBlockResponseOptions::new()
                .with_input()
                .with_effects()
                .with_balance_changes()
                .with_events(),
        )
        .await
        .unwrap();

    println!("Mgo TX: {tx:?}");

    assert_eq!(
        &MgoExecutionStatus::Success,
        tx.effects.as_ref().unwrap().status()
    );

    let ops2 = Operations::try_from(tx).unwrap();
    assert!(
        ops2.contains(&ops),
        "Operation mismatch. expecting:{}, got:{}",
        serde_json::to_string(&ops).unwrap(),
        serde_json::to_string(&ops2).unwrap()
    );

    println!("{}", serde_json::to_string_pretty(&ops2).unwrap())
}

#[tokio::test]
async fn test_withdraw_stake() {
    let test_cluster = TestClusterBuilder::new()
        .with_epoch_duration_ms(10000)
        .build()
        .await;
    let sender = test_cluster.get_address_0();
    let client = test_cluster.wallet.get_client().await.unwrap();
    let keystore = &test_cluster.wallet.config.keystore;

    let (rosetta_client, _handle) = start_rosetta_test_server(client.clone()).await;

    // First add some stakes
    let validator = client
        .governance_api()
        .get_latest_mgo_system_state()
        .await
        .unwrap()
        .active_validators[0]
        .mgo_address;

    let ops = serde_json::from_value(json!(
        [{
            "operation_identifier":{"index":0},
            "type":"Stake",
            "account": { "address" : sender.to_string() },
            "amount" : { "value": "-1000000000" , "currency": { "symbol": "MGO", "decimals": 9}},
            "metadata": { "Stake" : {"validator": validator.to_string()} }
        }]
    ))
    .unwrap();

    let response = rosetta_client.rosetta_flow(&ops, keystore).await;

    let tx = client
        .read_api()
        .get_transaction_with_options(
            response.transaction_identifier.hash,
            MgoTransactionBlockResponseOptions::new()
                .with_input()
                .with_effects()
                .with_balance_changes()
                .with_events(),
        )
        .await
        .unwrap();

    println!("Mgo TX: {tx:?}");

    assert_eq!(
        &MgoExecutionStatus::Success,
        tx.effects.as_ref().unwrap().status()
    );
    // verify balance
    let network_identifier = NetworkIdentifier {
        blockchain: "mgo".to_string(),
        network: MgoEnv::LocalNet,
    };
    let response = rosetta_client
        .get_balance(
            network_identifier.clone(),
            sender,
            Some(SubAccountType::PendingStake),
        )
        .await;

    assert_eq!(1, response.balances.len());
    assert_eq!(1000000000, response.balances[0].value);

    // wait for epoch.
    tokio::time::sleep(Duration::from_millis(15000)).await;

    // withdraw all stake
    let ops = serde_json::from_value(json!(
        [{
            "operation_identifier":{"index":0},
            "type":"WithdrawStake",
            "account": { "address" : sender.to_string() }
        }]
    ))
    .unwrap();

    let response = rosetta_client.rosetta_flow(&ops, keystore).await;

    let tx = client
        .read_api()
        .get_transaction_with_options(
            response.transaction_identifier.hash,
            MgoTransactionBlockResponseOptions::new()
                .with_input()
                .with_effects()
                .with_balance_changes()
                .with_events(),
        )
        .await
        .unwrap();

    assert_eq!(
        &MgoExecutionStatus::Success,
        tx.effects.as_ref().unwrap().status()
    );
    println!("Mgo TX: {tx:?}");

    let ops2 = Operations::try_from(tx).unwrap();
    assert!(
        ops2.contains(&ops),
        "Operation mismatch. expecting:{}, got:{}",
        serde_json::to_string(&ops).unwrap(),
        serde_json::to_string(&ops2).unwrap()
    );

    println!("{}", serde_json::to_string_pretty(&ops2).unwrap());

    // stake should be 0
    let response = rosetta_client
        .get_balance(
            network_identifier.clone(),
            sender,
            Some(SubAccountType::PendingStake),
        )
        .await;

    assert_eq!(1, response.balances.len());
    assert_eq!(0, response.balances[0].value);
}

#[tokio::test]
async fn test_pay_mgo() {
    let test_cluster = TestClusterBuilder::new().build().await;
    let sender = test_cluster.get_address_0();
    let recipient = test_cluster.get_address_1();
    let client = test_cluster.wallet.get_client().await.unwrap();
    let keystore = &test_cluster.wallet.config.keystore;

    let (rosetta_client, _handle) = start_rosetta_test_server(client.clone()).await;

    let ops = serde_json::from_value(json!(
        [{
            "operation_identifier":{"index":0},
            "type":"PayMgo",
            "account": { "address" : recipient.to_string() },
            "amount" : { "value": "1000000000" , "currency": { "symbol": "MGO", "decimals": 9}}
        },{
            "operation_identifier":{"index":1},
            "type":"PayMgo",
            "account": { "address" : sender.to_string() },
            "amount" : { "value": "-1000000000" , "currency": { "symbol": "MGO", "decimals": 9}}
        }]
    ))
    .unwrap();

    let response = rosetta_client.rosetta_flow(&ops, keystore).await;

    let tx = client
        .read_api()
        .get_transaction_with_options(
            response.transaction_identifier.hash,
            MgoTransactionBlockResponseOptions::new()
                .with_input()
                .with_effects()
                .with_balance_changes()
                .with_events(),
        )
        .await
        .unwrap();

    assert_eq!(
        &MgoExecutionStatus::Success,
        tx.effects.as_ref().unwrap().status()
    );
    println!("Mgo TX: {tx:?}");

    let ops2 = Operations::try_from(tx).unwrap();
    assert!(
        ops2.contains(&ops),
        "Operation mismatch. expecting:{}, got:{}",
        serde_json::to_string(&ops).unwrap(),
        serde_json::to_string(&ops2).unwrap()
    );
}

#[tokio::test]
async fn test_pay_mgo_multiple_times() {
    let test_cluster = TestClusterBuilder::new()
        .with_epoch_duration_ms(36000000)
        .build()
        .await;
    let sender = test_cluster.get_address_0();
    let recipient = test_cluster.get_address_1();
    let client = test_cluster.wallet.get_client().await.unwrap();
    let keystore = &test_cluster.wallet.config.keystore;

    let (rosetta_client, _handle) = start_rosetta_test_server(client.clone()).await;

    for _ in 1..100 {
        let ops = serde_json::from_value(json!(
            [{
                "operation_identifier":{"index":0},
                "type":"PayMgo",
                "account": { "address" : recipient.to_string() },
                "amount" : { "value": "1000000000" , "currency": { "symbol": "MGO", "decimals": 9}}
            },{
                "operation_identifier":{"index":1},
                "type":"PayMgo",
                "account": { "address" : sender.to_string() },
                "amount" : { "value": "-1000000000" , "currency": { "symbol": "MGO", "decimals": 9}}
            }]
        ))
        .unwrap();

        let response = rosetta_client.rosetta_flow(&ops, keystore).await;

        let tx = client
            .read_api()
            .get_transaction_with_options(
                response.transaction_identifier.hash,
                MgoTransactionBlockResponseOptions::new()
                    .with_input()
                    .with_effects()
                    .with_balance_changes()
                    .with_events(),
            )
            .await
            .unwrap();
        println!("Mgo TX: {tx:?}");
        assert_eq!(
            &MgoExecutionStatus::Success,
            tx.effects.as_ref().unwrap().status()
        );

        let ops2 = Operations::try_from(tx).unwrap();
        assert!(
            ops2.contains(&ops),
            "Operation mismatch. expecting:{}, got:{}",
            serde_json::to_string(&ops).unwrap(),
            serde_json::to_string(&ops2).unwrap()
        );
    }
}
