// RGB v0.12 Migration Notes:
// 1. Seal Type Unification (RFC: https://github.com/RGB-WG/RFC/issues/16)
//    - Unified seal type replaces distinct opret and tapret seals
//    - Seal type is automatically determined by wallet type (Taproot/WPKH)
//    - CloseMethod parameter has been removed from contract genesis
//    - Contract no longer commits to specific seal types
//
// 2. API Changes and Migration Strategy:
//    - Removed APIs:
//      * update_witnesses: Will be replaced with new rollback procedure
//      * CloseMethod related parameters: No longer needed due to seal unification
//    - Test Case Handling:
//      * Existing tests dependent on removed APIs: Marked as #[ignore] with tracking issues
//      * New tests: Focus on wallet type interactions rather than seal types
//      * Complex test cases will be implemented after evaluating:
//        - RGB protocol documentation and examples
//        - Implementation approaches for Lightning Network, multi-sig and interactive transactions
//
// 3. Implementation Notes:
//    - Test cases focus on wallet type (Taproot/WPKH) interactions
//    - Complex test scenarios are defined but implementation deferred
//    - Ignored tests will be updated once new APIs are available

pub mod utils;

use rgb::WitnessStatus;
use rstest_reuse::{self, *};
use serial_test::serial;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::str::FromStr;
use utils::chain::{get_tx_height, tx_status};
use utils::helper::wallet::{
    broadcast_tx_and_mine, get_mainnet_wallet, get_wallet, get_wallet_custom, AssetSchema,
};
use utils::{
    chain::{
        connect_reorg_nodes, disconnect_reorg_nodes, get_height, get_height_custom, initialize,
        mine_custom, stop_mining,
    },
    DescriptorType, INSTANCE_2, INSTANCE_3, *,
};

use crate::utils::helper::wallet::{HistoryType, ReorgType};

type TT = TransferType;
type DT = DescriptorType;
type AS = AssetSchema;

#[template]
#[rstest]
#[case(true)]
#[case(false)]
fn wout(#[case] wout: bool) {}
#[apply(wout)]
fn simple_transfer(wout: bool) {
    initialize();

    // Create two wallet instances
    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);
    let supply = 600;
    let asset_name = "TestAsset";

    // Create and issue NIA asset
    let mut params = NIAIssueParams::new(asset_name, "RBF", "centiMilli", supply);
    let outpoint = wlt_1.get_utxo(None);
    params.add_allocation(outpoint, supply);
    let contract_id = wlt_1.issue_nia_with_params(params);
    wlt_1.send_contract(asset_name, &mut wlt_2);
    // TODO: Because the RGB mound currently cannot dynamically load contracts,
    // It needs to be reloaded at a special time, and consider submitting a PR to RGB
    wlt_2.reload_runtime();

    let assign = 400;
    // recive asset by utxo
    let invoice = wlt_2.invoice(contract_id, assign, wout, Some(0), None);

    // send asset to wlt2
    // if `wout` is true (WitnessOut),
    // wlt2 will have a 3000 Sats UTXO, which will be spent to transfer assets to wlt1 in the next step
    let (consignment_1, tx, _) = wlt_1.transfer(invoice, Some(3000), Some(500), true, None);

    // Receiver accepts the transfer
    wlt_2.accept_transfer(&consignment_1, None).unwrap();

    // Broadcast and confirm transaction
    wlt_1.mine_tx(&tx.txid(), false);

    // Sync both wallets
    wlt_1.sync();
    wlt_2.sync();

    wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![supply - assign]);
    wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![assign]);

    let assign_wlt1 = 200;
    let invoice = wlt_1.invoice(contract_id, assign_wlt1, wout, Some(0), None);
    dbg!(
        "wlt2",
        wlt_2.runtime().wallet.balance(),
        wlt_2.runtime().wallet.coins().collect::<Vec<_>>()
    );
    // Sats cost: 500 fee + 2000 sats(default) = 2500
    let (consignment_2, tx, _) = wlt_2.transfer(invoice, None, Some(500), true, None);
    wlt_1.accept_transfer(&consignment_2, None).unwrap();
    wlt_2.mine_tx(&tx.txid(), false);

    // // Sync both wallets
    wlt_1.sync();
    wlt_2.sync();

    // owned state
    dbg!(wlt_1.runtime().state_own(contract_id).owned);
    dbg!(wlt_2.runtime().state_own(contract_id).owned);

    wlt_1.check_allocations(
        contract_id,
        AssetSchema::RGB20,
        vec![supply - assign, assign_wlt1],
    );
    wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![assign - assign_wlt1]);
}

#[test]
fn rbf_transfer() {
    initialize();

    // Create two wallet instances
    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    // Create and issue NIA asset
    let mut params = NIAIssueParams::new("RBFTestAsset", "RBF", "centiMilli", 600);
    let outpoint = wlt_1.get_utxo(None);
    params.add_allocation(outpoint, 600);
    let contract_id = wlt_1.issue_nia_with_params(params);
    wlt_1.send_contract("RBFTestAsset", &mut wlt_2);
    wlt_2.reload_runtime();

    let invoice = wlt_2.invoice(contract_id, 400, false, Some(0), None);

    // Stop mining to test RBF
    stop_mining();
    let initial_height = get_height();

    // First transfer attempt - with a lower fee
    let (consignment_1, _tx, payment) =
        wlt_1.transfer(invoice.clone(), None, Some(500), true, None);
    let first_txid = _tx.txid();
    dbg!(first_txid, tx_status(first_txid, wlt_1.instance));

    // Receiver accepts the transfer
    wlt_2.accept_transfer(&consignment_1, None).unwrap();

    // Verify block height hasn't changed (transaction not confirmed)
    let mid_height = get_height();
    assert_eq!(initial_height, mid_height);

    // Second transfer attempt - with a higher fee for RBF
    let (consignment_2, tx) = wlt_1.transfer_rbf(contract_id, payment, 1000, None);
    let second_txid = tx.txid();
    // Verify block height still hasn't changed
    let final_height = get_height();
    assert_eq!(initial_height, final_height);

    // Broadcast and confirm transaction
    wlt_1.mine_tx(&tx.txid(), true);
    dbg!(first_txid, tx_status(first_txid, wlt_1.instance));
    dbg!(second_txid, tx_status(second_txid, wlt_1.instance));

    // Receiver accepts final transfer
    wlt_2.accept_transfer(&consignment_2, None).unwrap();

    // Sync both wallets
    wlt_1.sync();
    wlt_2.sync();

    // Verify asset allocations in both wallets
    wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![200]);
    wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![400]);

    // Transfer assets back to sender
    wlt_2.send(
        &mut wlt_1,
        false,
        contract_id,
        400,
        2000,
        None,
        Some(0),
        None,
    );
}

#[rstest]
// blinded: nia - nia
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::RGB20, AS::RGB20)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::RGB20, AS::RGB20)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::RGB20, AS::RGB20)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::RGB20, AS::RGB20)]
// blinded: nia - cfa
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::RGB20, AS::RGB25)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::RGB20, AS::RGB25)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::RGB20, AS::RGB25)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::RGB20, AS::RGB25)]
// blinded: cfa - cfa
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::RGB25, AS::RGB25)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::RGB25, AS::RGB25)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::RGB25, AS::RGB25)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::RGB25, AS::RGB25)]
// TODO: UDA related asset feature, RGB core library is being improved,
// And the test case for UDA assets will be added later

fn transfer_loop(
    #[case] transfer_type: TransferType,
    #[case] wlt_1_desc: DescriptorType,
    #[case] wlt_2_desc: DescriptorType,
    #[case] asset_schema_1: AssetSchema,
    #[case] asset_schema_2: AssetSchema,
) {
    println!(
        "transfer_type {transfer_type:?} wlt_1_desc {wlt_1_desc:?} wlt_2_desc {wlt_2_desc:?} \
        asset_schema_1 {asset_schema_1:?} asset_schema_2 {asset_schema_2:?}"
    );

    initialize();

    let mut wlt_1 = get_wallet(&wlt_1_desc);
    let mut wlt_2 = get_wallet(&wlt_2_desc);

    let issued_supply_1 = 999;
    let issued_supply_2 = 666;

    let mut sats = 9000;

    // wlt_1 issues 2 assets on the same UTXO
    let utxo = wlt_1.get_utxo(None);

    // Issue first asset
    let contract_id_1 = match asset_schema_1 {
        AssetSchema::RGB20 => {
            let mut params =
                NIAIssueParams::new("TestAsset1", "TEST1", "centiMilli", issued_supply_1);
            params.add_allocation(utxo, issued_supply_1);
            wlt_1.issue_nia_with_params(params)
        }
        AssetSchema::RGB25 => {
            let mut params =
                FUAIssueParams::new("TestAsset1", "details", "centiMilli", issued_supply_1);
            params.add_allocation(utxo, issued_supply_1);
            wlt_1.issue_fua_with_params(params)
        }
        AssetSchema::RGB21 => {
            // TODO: UDA is not supported yet
            panic!("UDA is not supported yet");
        }
    };

    // Issue second asset
    let contract_id_2 = match asset_schema_2 {
        AssetSchema::RGB20 => {
            let mut params =
                NIAIssueParams::new("TestAsset2", "TEST2", "centiMilli", issued_supply_2);
            params.add_allocation(utxo, issued_supply_2);
            wlt_1.issue_nia_with_params(params)
        }
        AssetSchema::RGB25 => {
            let mut params =
                FUAIssueParams::new("TestAsset2", "details", "centiMilli", issued_supply_2);
            params.add_allocation(utxo, issued_supply_2);
            wlt_1.issue_fua_with_params(params)
        }
        AssetSchema::RGB21 => {
            // TODO: UDA is not supported yet
            panic!("UDA is not supported yet");
        }
    };

    // Share contract info with wallet 2
    wlt_1.send_contract("TestAsset1", &mut wlt_2);
    wlt_1.send_contract("TestAsset2", &mut wlt_2);
    wlt_2.reload_runtime();

    // Verify initial allocations
    wlt_1.check_allocations(contract_id_1, asset_schema_1, vec![issued_supply_1]);
    wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![issued_supply_2]);

    // wlt_1 spends asset 1
    let amount_1 = if asset_schema_1 != AssetSchema::RGB21 {
        99
    } else {
        1
    };
    let wout = match transfer_type {
        TransferType::Blinded => false,
        TransferType::Witness => true,
    };
    wlt_1.send(
        &mut wlt_2,
        wout,
        contract_id_1,
        amount_1,
        sats,
        None,
        Some(0),
        None,
    );

    // Verify allocations after first transfer
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1],
    );
    wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![issued_supply_2]);
    wlt_2.check_allocations(contract_id_1, asset_schema_1, vec![amount_1]);

    // wlt_1 spends asset 1 change (only if possible)
    if asset_schema_1 != AssetSchema::RGB21 {
        let amount_2 = 33;
        wlt_1.send(
            &mut wlt_2,
            wout,
            contract_id_1,
            amount_2,
            sats,
            None,
            Some(0),
            None,
        );
        wlt_1.check_allocations(
            contract_id_1,
            asset_schema_1,
            vec![issued_supply_1 - amount_1 - amount_2],
        );
        wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![issued_supply_2]);
        wlt_2.check_allocations(contract_id_1, asset_schema_1, vec![amount_1, amount_2]);
    }

    // wlt_1 spends asset 2
    let amount_3 = if asset_schema_2 != AssetSchema::RGB21 {
        22
    } else {
        1
    };
    wlt_1.send(
        &mut wlt_2,
        wout,
        contract_id_2,
        amount_3,
        sats,
        None,
        None,
        None,
    );

    // Verify final allocations
    if asset_schema_1 != AssetSchema::RGB21 {
        let amount_2 = 33;
        wlt_1.check_allocations(
            contract_id_1,
            asset_schema_1,
            vec![issued_supply_1 - amount_1 - amount_2],
        );
    } else {
        wlt_1.check_allocations(
            contract_id_1,
            asset_schema_1,
            vec![issued_supply_1 - amount_1],
        );
    }
    wlt_1.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3],
    );
    wlt_2.check_allocations(contract_id_2, asset_schema_2, vec![amount_3]);

    // wlt_2 spends received allocation(s) of asset 1
    let amount_4 = if asset_schema_1 != AssetSchema::RGB21 {
        111
    } else {
        1
    };
    let amount_2 = if asset_schema_1 != AssetSchema::RGB21 {
        33
    } else {
        0
    };
    sats -= 1000;
    wlt_2.send(
        &mut wlt_1,
        wout,
        contract_id_1,
        amount_4,
        sats,
        None,
        None,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2, amount_4],
    );
    wlt_1.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3],
    );
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4],
    );
    wlt_2.check_allocations(contract_id_2, asset_schema_2, vec![amount_3]);

    // wlt_2 spends asset 2
    let amount_5 = if asset_schema_2 != AssetSchema::RGB21 {
        11
    } else {
        1
    };
    sats -= 1000;
    wlt_2.send(
        &mut wlt_1,
        wout,
        contract_id_2,
        amount_5,
        sats,
        None,
        None,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2, amount_4],
    );
    wlt_1.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3, amount_5],
    );

    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4],
    );
    wlt_2.check_allocations(contract_id_2, asset_schema_2, vec![amount_3 - amount_5]);

    // wlt_1 spends asset 1, received back
    let amount_6 = if asset_schema_1 != AssetSchema::RGB21 {
        issued_supply_1 - amount_1 - amount_2 + amount_4
    } else {
        1
    };
    sats -= 1000;
    wlt_1.send(
        &mut wlt_2,
        wout,
        contract_id_1,
        amount_6,
        sats,
        None,
        None,
        None,
    );
    wlt_1.check_allocations(contract_id_1, asset_schema_1, vec![]);

    // Theoretically, there should be two outputs, one for the change UTXO and one for the income UTXO.
    // But because the change UTXO is associated with two assets (asset 1 and asset 2), asset 1 has been fully transferred to the UTXO of wlt2.
    // So there will only be one UTXO, which combines the change and income of asset 2.
    //
    // In most cases, it will be merged into one UTXO,
    // And in a few cases, there will be two UTXOs.
    if let Err(_) = catch_unwind(AssertUnwindSafe(|| {
        wlt_1.check_allocations(
            contract_id_2,
            asset_schema_2,
            vec![issued_supply_2 - amount_3 + amount_5],
        );
    })) {
        wlt_1.check_allocations(
            contract_id_2,
            asset_schema_2,
            vec![issued_supply_2 - amount_3, amount_5],
        );
    }

    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4, amount_6],
    );
    wlt_2.check_allocations(contract_id_2, asset_schema_2, vec![amount_3 - amount_5]);

    // wlt_1 spends asset 2, received back
    let amount_7 = if asset_schema_2 != AssetSchema::RGB21 {
        issued_supply_2 - amount_3 + amount_5
    } else {
        1
    };
    sats -= 1000;
    wlt_1.send(
        &mut wlt_2,
        wout,
        contract_id_2,
        amount_7,
        sats,
        None,
        None,
        None,
    );
    wlt_1.check_allocations(contract_id_1, asset_schema_1, vec![]);
    wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![]);
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4, amount_6],
    );
    wlt_2.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![amount_3 - amount_5, amount_7],
    );
}

#[rstest]
#[case(TransferType::Blinded)]
#[case(TransferType::Witness)]
fn same_transfer_twice_update_witnesses(#[case] transfer_type: TransferType) {
    println!("transfer_type {transfer_type:?}");

    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 2000;
    // Create and issue NIA asset
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", issue_supply);
    let outpoint = wlt_1.get_utxo(None);
    params.add_allocation(outpoint, issue_supply);
    let contract_id = wlt_1.issue_nia_with_params(params);
    wlt_1.send_contract("TestAsset", &mut wlt_2);
    wlt_2.reload_runtime();

    let amount = 100;
    let wout = match transfer_type {
        TransferType::Blinded => false,
        TransferType::Witness => true,
    };

    let invoice = wlt_2.invoice(contract_id, amount, wout, Some(0), None);
    let _ = wlt_1.transfer(invoice.clone(), None, Some(500), false, None);

    wlt_1.sync();

    let (consignment, tx, _) = wlt_1.transfer(invoice, None, Some(1000), true, None);
    wlt_1.mine_tx(&tx.txid(), false);
    wlt_2.accept_transfer(&consignment, None).unwrap();
    wlt_1.sync();
    wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![issue_supply - amount]);
    wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![amount]);

    wlt_2.send(
        &mut wlt_1,
        wout,
        contract_id,
        amount,
        1000,
        None,
        None,
        None,
    );
}

// Complex test cases - Implementation deferred to final phase
// These test cases will be implemented last, after evaluating:
// 1. Available documentation and examples from RGB protocol
// 2. If no official examples exist, Bitlight will explore implementation approaches for:
//    - Lightning Network test-cases integration
//    - Multi-signature operations
//    - Interactive transaction construction
// Reference: https://github.com/RGB-WG/rgb/blob/v0.12/doc/Payments.md

#[test]
#[ignore = "Pending Lightning Network integration documentation"]
fn ln_transfers() {
    // TODO: Implement Lightning Network transfer tests
}

#[test]
#[ignore = "Pending multi-signature workflow documentation"]
fn collaborative_transfer() {
    // TODO: Implement multi-signature transfer tests
}

#[rstest]
#[should_panic(expected = "Fulfill(StateInsufficient)")]
#[case(TransferType::Blinded)]
#[should_panic(expected = "Fulfill(StateInsufficient)")]
#[case(TransferType::Witness)]
fn same_transfer_twice_no_update_witnesses(#[case] transfer_type: TransferType) {
    println!("transfer_type {transfer_type:?}");

    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 2000;
    // Create and issue NIA asset
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", issue_supply);
    let outpoint = wlt_1.get_utxo(None);
    params.add_allocation(outpoint, issue_supply);
    let contract_id = wlt_1.issue_nia_with_params(params);
    wlt_1.send_contract("TestAsset", &mut wlt_2);
    wlt_2.reload_runtime();

    let amount = 100;
    let wout = match transfer_type {
        TransferType::Blinded => false,
        TransferType::Witness => true,
    };
    let invoice = wlt_2.invoice(contract_id, amount, wout, Some(0), None);
    let (_, _tx, payment) = wlt_1.transfer(invoice.clone(), None, Some(500), false, None);

    let (consignment, _) = wlt_1.transfer_rbf(contract_id, payment, 1000, None);

    wlt_2.accept_transfer(&consignment, None).unwrap();

    if transfer_type == TransferType::Blinded {
        wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![amount]);
    } else {
        // Since the receiver state is not updated, it treats the witness transaction as not mined and thus has no state
        wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
    }

    wlt_2.send(
        &mut wlt_1,
        wout,
        contract_id,
        amount,
        1000,
        None,
        None,
        None,
    );

    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);
    wlt_1.send_contract("TestAsset", &mut wlt_3);
    wlt_3.reload_runtime();

    // The receiver will fail to accept the consignment
    wlt_1.send(
        &mut wlt_3,
        wout,
        contract_id,
        issue_supply,
        1000,
        None,
        None,
        None,
    );
}

#[test]
fn accept_0conf() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 600;
    // Create and issue NIA asset
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", issue_supply);
    let outpoint = wlt_1.get_utxo(None);
    params.add_allocation(outpoint, issue_supply);
    let contract_id = wlt_1.issue_nia_with_params(params);
    wlt_1.send_contract("TestAsset", &mut wlt_2);
    wlt_2.reload_runtime();

    let amt = 200;
    let invoice = wlt_2.invoice(contract_id, amt, true, Some(0), None);
    let (consignment, tx, _) = wlt_1.transfer(invoice.clone(), None, None, true, None);
    let txid = tx.txid();

    wlt_2.accept_transfer(&consignment, None).unwrap();

    // wlt_2 sees the allocation even if TX has not been mined
    wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![amt]);

    wlt_1.sync();

    let wlt_1_change_amt = issue_supply - amt;

    // after mining, wlt_1 doesn't need to get tentative allocations to see the change
    wlt_1.mine_tx(&txid, false);
    wlt_1.sync();
    wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_1_change_amt]);
}

#[test]
#[serial]
fn tapret_wlt_receiving_opret() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Tr);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", 600);
    let outpoint = wlt_1.get_utxo(None);
    params.add_allocation(outpoint, 600);
    let contract_id = wlt_1.issue_nia_with_params(params);
    wlt_1.send_contract("TestAsset", &mut wlt_2);
    wlt_2.reload_runtime();

    // First transfer: wlt_1 -> wlt_2, transfer 400
    wlt_1.send(&mut wlt_2, false, contract_id, 400, 5000, None, None, None);

    // Second transfer: wlt_2 -> wlt_1, transfer 100
    let invoice = wlt_1.invoice(contract_id, 100, true, Some(0), None);
    wlt_2.send_to_invoice(&mut wlt_1, invoice, None, None, None);

    // Third transfer: wlt_1 -> wlt_2, transfer 290
    wlt_1.send(&mut wlt_2, true, contract_id, 290, 1000, None, None, None);

    // Fourth transfer: wlt_2 -> wlt_1, transfer 560
    wlt_2.send(&mut wlt_1, false, contract_id, 560, 1000, None, None, None);

    // Fifth transfer: wlt_1 -> wlt_2, transfer 570
    wlt_1.send(&mut wlt_2, false, contract_id, 570, 1000, None, None, None);

    wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
    wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![30, 570]);
}

#[test]
#[serial]
fn check_fungible_history() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 600;
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", issue_supply);
    let outpoint = wlt_1.get_utxo(None);
    params.add_allocation(outpoint, issue_supply);
    let contract_id = wlt_1.issue_nia_with_params(params);
    wlt_1.send_contract("TestAsset", &mut wlt_2);
    wlt_2.reload_runtime();

    // debug contract info
    dbg!(wlt_1.contracts_info());
    dbg!(wlt_1.runtime().state_own(contract_id).owned);

    // transfer
    let amt = 200;
    let (_, tx, _) = wlt_1.send(&mut wlt_2, true, contract_id, amt, 1000, None, None, None);
    let _txid = tx.txid();

    // debug contract state
    dbg!(wlt_1.runtime().state_own(contract_id).owned);
    dbg!(wlt_2.runtime().state_own(contract_id).owned);

    // check allocations
    wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![issue_supply - amt]);
    wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![amt]);
}

#[test]
fn send_to_oneself() {
    initialize();

    let mut wlt = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 600;
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", issue_supply);
    let outpoint = wlt.get_utxo(None);
    params.add_allocation(outpoint, issue_supply);
    let contract_id = wlt.issue_nia_with_params(params);

    // Transfer 200 to yourself
    let amt = 200;
    let invoice = wlt.invoice(contract_id, amt, true, Some(0), None);
    let (consignment, tx, _) = wlt.transfer(invoice.clone(), None, None, true, None);
    wlt.mine_tx(&tx.txid(), false);
    wlt.accept_transfer(&consignment, None).unwrap();
    wlt.sync();

    // debug contract state
    dbg!(wlt.runtime().state_own(contract_id).owned);

    // check allocations
    wlt.check_allocations(
        contract_id,
        AssetSchema::RGB20,
        vec![amt, issue_supply - amt],
    );
}

#[rstest]
#[case(DescriptorType::Tr, DescriptorType::Tr)]
#[case(DescriptorType::Tr, DescriptorType::Wpkh)]
#[case(DescriptorType::Wpkh, DescriptorType::Tr)]
#[case(DescriptorType::Wpkh, DescriptorType::Wpkh)]
fn blank_tapret_opret(
    #[case] descriptor_type_0: DescriptorType,
    #[case] descriptor_type_1: DescriptorType,
) {
    initialize();

    let mut wlt_1 = get_wallet(&descriptor_type_0);
    let mut wlt_2 = get_wallet(&descriptor_type_1);

    // Create and issue first NIA asset
    let mut params_0 = NIAIssueParams::new("TestAsset1", "TEST1", "centiMilli", 200);
    let outpoint = wlt_1.get_utxo(None);
    params_0.add_allocation(outpoint, 200);
    let contract_id_0 = wlt_1.issue_nia_with_params(params_0);
    wlt_1.send_contract("TestAsset1", &mut wlt_2);
    wlt_2.reload_runtime();

    // Create and issue second NIA asset (to be moved in blank)
    let mut params_1 = NIAIssueParams::new("TestAsset2", "TEST2", "centiMilli", 100);
    params_1.add_allocation(outpoint, 100);
    let contract_id_1 = wlt_1.issue_nia_with_params(params_1);
    wlt_1.send_contract("TestAsset2", &mut wlt_2);
    wlt_2.reload_runtime();

    // First transfer: wlt_1 -> wlt_2, transfer 200 of first asset
    wlt_1.send(
        &mut wlt_2,
        false,
        contract_id_0,
        200,
        1000,
        None,
        None,
        None,
    );

    // Second transfer: wlt_1 -> wlt_2, transfer 100 of second asset
    // This tests the blank transfer functionality with different descriptor types
    wlt_1.send(
        &mut wlt_2,
        false,
        contract_id_1,
        100,
        1000,
        None,
        None,
        None,
    );

    // Verify final allocations
    wlt_1.check_allocations(contract_id_0, AssetSchema::RGB20, vec![]);
    wlt_1.check_allocations(contract_id_1, AssetSchema::RGB20, vec![]);
    wlt_2.check_allocations(contract_id_0, AssetSchema::RGB20, vec![200]);
    wlt_2.check_allocations(contract_id_1, AssetSchema::RGB20, vec![100]);
}

#[rstest]
#[case(HistoryType::Linear, ReorgType::ChangeOrder)]
// FIXME: Hope to receive solutions or suggestions from the doctor
#[case(HistoryType::Linear, ReorgType::Revert)]
#[case(HistoryType::Branching, ReorgType::ChangeOrder)]
// FIXME: Hope to receive solutions or suggestions from the doctor
#[case(HistoryType::Branching, ReorgType::Revert)]
#[case(HistoryType::Merging, ReorgType::ChangeOrder)]
// FIXME: Hope to receive solutions or suggestions from the doctor
#[case(HistoryType::Merging, ReorgType::Revert)]
#[serial]
fn reorg_history(#[case] history_type: HistoryType, #[case] reorg_type: ReorgType) {
    println!("history_type {history_type:?} reorg_type {reorg_type:?}");

    initialize();
    connect_reorg_nodes();

    let mut wlt_1 = get_wallet_custom(&DescriptorType::Wpkh, INSTANCE_2);
    let mut wlt_2 = get_wallet_custom(&DescriptorType::Wpkh, INSTANCE_2);
    let mut wlt_3 = get_wallet_custom(&DescriptorType::Wpkh, INSTANCE_2);

    let issued_supply = 600;

    // Initialize contract based on history type
    let contract_id = match history_type {
        HistoryType::Linear | HistoryType::Branching => {
            let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", issued_supply);
            let outpoint = wlt_1.get_utxo(None);
            params.add_allocation(outpoint, issued_supply);
            wlt_1.issue_nia_with_params(params)
        }
        HistoryType::Merging => {
            // For merging, we create a contract with multiple allocations
            let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", 600);
            params.add_allocation(wlt_1.get_utxo(None), 400);
            // Adding a second allocation to the same outpoint
            params.add_allocation(wlt_1.get_utxo(None), 200);
            wlt_1.issue_nia_with_params(params)
        }
    };

    wlt_1.send_contract("TestAsset", &mut wlt_2);
    wlt_2.reload_runtime();
    wlt_1.send_contract("TestAsset", &mut wlt_3);
    wlt_3.reload_runtime();

    // Generate UTXOs before asset transfer to avoid mining blocks during transfer, affecting the test
    let utxo_wlt_1_1 = wlt_1.get_utxo(None);
    let utxo_wlt_1_2 = wlt_1.get_utxo(None);
    let utxo_wlt_2_1 = wlt_2.get_utxo(None);
    let utxo_wlt_2_2 = wlt_2.get_utxo(None);
    mine_custom(false, INSTANCE_2, 6);

    dbg!(get_height_custom(INSTANCE_2));
    dbg!(get_height_custom(INSTANCE_3));

    disconnect_reorg_nodes();

    // Create transactions based on history type
    let txs = match history_type {
        HistoryType::Linear => {
            // Set the coin selection strategy to true small size
            // This setting is very important, it avoids selecting the output of the revert transaction as input
            wlt_1.set_coinselect_strategy(CustomCoinselectStrategy::TrueSmallSize);
            let amt_0 = 590;
            // Create blinded invoice with specific UTXO
            let invoice = wlt_2.invoice(contract_id, amt_0, false, Some(0), Some(utxo_wlt_2_1));
            let (_, tx_0, _) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            let amt_1 = 100;
            let invoice = wlt_1.invoice(contract_id, amt_1, false, Some(0), Some(utxo_wlt_1_1));
            let (_, tx_1, _) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            let amt_2 = 80;
            let invoice = wlt_2.invoice(contract_id, amt_2, false, Some(0), Some(utxo_wlt_2_2));
            let (_, tx_2, _) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            vec![tx_0, tx_1, tx_2]
        }
        HistoryType::Branching => {
            let amt_0 = 600;
            let invoice = wlt_2.invoice(contract_id, amt_0, false, Some(0), Some(utxo_wlt_2_1));
            let (_, tx_0, _) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            let amt_1 = 200;
            let invoice = wlt_1.invoice(contract_id, amt_1, false, Some(0), Some(utxo_wlt_1_1));
            let (_, tx_1, _) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            let amt_2 = amt_0 - amt_1 - 1;
            let invoice = wlt_1.invoice(contract_id, amt_2, false, Some(0), Some(utxo_wlt_1_2));
            let (_, tx_2, _) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            vec![tx_0, tx_1, tx_2]
        }
        HistoryType::Merging => {
            let amt_0 = 400;
            let invoice = wlt_2.invoice(contract_id, amt_0, false, Some(0), Some(utxo_wlt_2_1));
            let (_, tx_0, _) = wlt_1.send_to_invoice(&mut wlt_2, invoice, None, None, None);

            let amt_1 = 200;
            let invoice = wlt_2.invoice(contract_id, amt_1, false, Some(0), Some(utxo_wlt_2_2));
            let (_, tx_1, _) = wlt_1.send_to_invoice(&mut wlt_2, invoice, None, None, None);

            let amt_2 = amt_0 + amt_1 - 1;
            let invoice = wlt_1.invoice(contract_id, amt_2, false, Some(0), Some(utxo_wlt_1_1));
            let (_, tx_2, _) = wlt_2.send_to_invoice(&mut wlt_1, invoice, None, None, None);

            vec![tx_0, tx_1, tx_2]
        }
    };

    dbg!(
        "before switch",
        wlt_1.runtime().state_own(contract_id).owned
    );
    dbg!(
        "before switch",
        wlt_2.runtime().state_own(contract_id).owned
    );

    let tx_0_instance_2_height = get_tx_height(txs[0].txid(), INSTANCE_2);
    let tx_1_instance_2_height = get_tx_height(txs[1].txid(), INSTANCE_2);
    let tx_2_instance_2_height = get_tx_height(txs[2].txid(), INSTANCE_2);

    // Test different reorg scenarios
    match (history_type, reorg_type) {
        (HistoryType::Linear, ReorgType::ChangeOrder) => {
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[0], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 10;
            let wlt_1_alloc_2 = 20;
            let wlt_2_alloc_1 = 490;
            let wlt_2_alloc_2 = 80;
            wlt_1.check_allocations(
                contract_id,
                AssetSchema::RGB20,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
            );
            wlt_2.check_allocations(
                contract_id,
                AssetSchema::RGB20,
                vec![wlt_2_alloc_1, wlt_2_alloc_2],
            );
        }
        (HistoryType::Linear | HistoryType::Branching, ReorgType::Revert) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 600;
            dbg!(tx_status(txs[0].txid(), INSTANCE_3));
            dbg!(wlt_1.runtime().state_own(contract_id).owned);
            dbg!(wlt_2.runtime().state_own(contract_id).owned);
            wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_1_alloc_1]);
            wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
        }
        (HistoryType::Branching, ReorgType::ChangeOrder) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            broadcast_tx_and_mine(&txs[0], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 200;
            let wlt_1_alloc_2 = 399;
            let wlt_2_alloc_1 = 1;
            wlt_1.check_allocations(
                contract_id,
                AssetSchema::RGB20,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
            );
            wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_2_alloc_1]);
        }
        (HistoryType::Merging, ReorgType::ChangeOrder) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[0], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 599;
            let wlt_2_alloc_1 = 1;
            wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_1_alloc_1]);
            wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_2_alloc_1]);
        }
        (HistoryType::Merging, ReorgType::Revert) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 400;
            let wlt_2_alloc_1 = 200;
            wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_1_alloc_1]);
            wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_2_alloc_1]);
        }
    }

    let tx_0_instance_3_height = get_tx_height(txs[0].txid(), INSTANCE_3);
    let tx_1_instance_3_height = get_tx_height(txs[1].txid(), INSTANCE_3);
    let tx_2_instance_3_height = get_tx_height(txs[2].txid(), INSTANCE_3);

    dbg!(
        &txs[0].txid(),
        tx_0_instance_2_height,
        tx_0_instance_3_height
    );
    dbg!(
        &txs[1].txid(),
        tx_1_instance_2_height,
        tx_1_instance_3_height
    );
    dbg!(
        &txs[2].txid(),
        tx_2_instance_2_height,
        tx_2_instance_3_height
    );
    mine_custom(false, INSTANCE_3, 3);
    connect_reorg_nodes();
    dbg!("final");
    wlt_1.switch_to_instance(INSTANCE_2);
    wlt_2.switch_to_instance(INSTANCE_2);

    dbg!(wlt_1.runtime().state_own(contract_id).owned);
    dbg!(wlt_2.runtime().state_own(contract_id).owned);

    // Verify final state based on history type
    match history_type {
        HistoryType::Linear => {
            let wlt_1_alloc_1 = 10;
            let wlt_1_alloc_2 = 20;
            let wlt_1_amt = wlt_1_alloc_1 + wlt_1_alloc_2;
            let wlt_2_alloc_1 = 490;
            let wlt_2_alloc_2 = 80;
            let wlt_2_amt = wlt_2_alloc_1 + wlt_2_alloc_2;
            wlt_1.check_allocations(
                contract_id,
                AssetSchema::RGB20,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
            );
            wlt_2.check_allocations(
                contract_id,
                AssetSchema::RGB20,
                vec![wlt_2_alloc_1, wlt_2_alloc_2],
            );

            // Test spending the final allocations
            wlt_1.send(
                &mut wlt_3,
                false,
                contract_id,
                wlt_1_amt,
                1000,
                None,
                None,
                None,
            );
            wlt_2.send(
                &mut wlt_3,
                false,
                contract_id,
                wlt_2_amt,
                1000,
                None,
                None,
                None,
            );
            wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
            wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
            wlt_3.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_1_amt, wlt_2_amt]);
        }
        HistoryType::Branching => {
            let wlt_1_alloc_1 = 200;
            let wlt_1_alloc_2 = 399;
            let wlt_1_amt = wlt_1_alloc_1 + wlt_1_alloc_2;
            let wlt_2_alloc_1 = 1;
            wlt_1.check_allocations(
                contract_id,
                AssetSchema::RGB20,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
            );
            wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_2_alloc_1]);

            wlt_1.send(
                &mut wlt_3,
                false,
                contract_id,
                wlt_1_amt,
                1000,
                None,
                None,
                None,
            );
            wlt_2.send(
                &mut wlt_3,
                false,
                contract_id,
                wlt_2_alloc_1,
                1000,
                None,
                None,
                None,
            );
            wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
            wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
            wlt_3.check_allocations(
                contract_id,
                AssetSchema::RGB20,
                vec![wlt_1_amt, wlt_2_alloc_1],
            );
        }
        HistoryType::Merging => {
            let wlt_1_alloc_1 = 599;
            let wlt_2_alloc_1 = 1;
            wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_1_alloc_1]);
            wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![wlt_2_alloc_1]);

            wlt_1.send(
                &mut wlt_3,
                false,
                contract_id,
                wlt_1_alloc_1,
                1000,
                None,
                None,
                None,
            );
            wlt_2.send(
                &mut wlt_3,
                false,
                contract_id,
                wlt_2_alloc_1,
                1000,
                None,
                None,
                None,
            );
            wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
            wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
            wlt_3.check_allocations(
                contract_id,
                AssetSchema::RGB20,
                vec![wlt_1_alloc_1, wlt_2_alloc_1],
            );
        }
    }
}

#[test]
#[serial]
fn revert_transfer_state() {
    initialize();
    // connecting before disconnecting since disconnect is not idempotent
    connect_reorg_nodes();
    disconnect_reorg_nodes();

    let mut wlt = get_wallet_custom(&DescriptorType::Wpkh, INSTANCE_2);
    let mut recv_wlt = get_wallet_custom(&DescriptorType::Wpkh, INSTANCE_2);

    let issued_supply = 600;
    let utxo = wlt.get_utxo(None);

    // Create and issue NIA asset
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", issued_supply);
    params.add_allocation(utxo, issued_supply);
    let contract_id = wlt.issue_nia_with_params(params);
    wlt.send_contract("TestAsset", &mut recv_wlt);
    recv_wlt.reload_runtime();

    wlt.check_allocations(contract_id, AssetSchema::RGB20, vec![issued_supply]);

    let amt = 200;

    wlt.send(
        &mut recv_wlt,
        false,
        contract_id,
        amt,
        1000,
        None,
        None,
        None,
    );
    wlt.check_allocations(contract_id, AssetSchema::RGB20, vec![issued_supply - amt]);

    mine_custom(false, INSTANCE_2, 1);
    wlt.sync();
    recv_wlt.sync();

    let state = recv_wlt.runtime().state_own(contract_id).owned;
    let witness_status = state
        .values()
        .next()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .status;
    dbg!(state, witness_status);
    assert!(matches!(witness_status, WitnessStatus::Mined(_)));

    recv_wlt.switch_to_instance(INSTANCE_3);
    let state = recv_wlt.runtime().state_own(contract_id).owned;
    let archived = state.values().next().unwrap().is_empty();
    dbg!(state, archived);
    assert!(archived);
    recv_wlt.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
}

#[test]
#[ignore = "fix needed"]
fn mainnet_wlt_receiving_test_asset() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    // FIXME: Because the latest `Mound` structure in rgb does not support setting the mainnet,
    // The default `Mound.testnet` is eq true, which cannot correctly initialize the mainnet wallet,
    // So this test case cannot be executed temporarily
    let mut wlt_2 = get_mainnet_wallet();

    // Create and issue NIA asset
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", 700);
    let outpoint = wlt_1.get_utxo(None);
    params.add_allocation(outpoint, 700);
    let contract_id = wlt_1.issue_nia_with_params(params);

    let utxo =
        Outpoint::from_str("bebcfcb200a17763f6932a6d6fca9448a4b46c5b737cc3810769a7403ef79ce6:0")
            .unwrap();
    let invoice = wlt_2.invoice(contract_id, 150, false, None, Some(utxo));

    let (consignment, tx, _) = wlt_1.transfer(invoice.clone(), None, Some(500), true, None);
    wlt_1.mine_tx(&tx.txid(), false);
    match wlt_2.accept_transfer(&consignment, None) {
        Err(e) => {
            dbg!(e.to_string());
        }
        _ => panic!("validation must fail"),
    }
}

#[rstest]
#[case(TT::Blinded)]
#[case(TT::Witness)]
#[serial]
fn invoice_reuse(#[case] transfer_type: TransferType) {
    println!("transfer_type {transfer_type:?}");

    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    wlt_1.set_coinselect_strategy(CustomCoinselectStrategy::TrueSmallSize);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    // Create and issue assets
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", 900);
    params.add_allocation(wlt_1.get_utxo(None), 500);
    params.add_allocation(wlt_1.get_utxo(None), 400);
    let contract_id = wlt_1.issue_nia_with_params(params);
    wlt_1.send_contract("TestAsset", &mut wlt_2);
    wlt_2.reload_runtime();

    let amount = 300;
    // Create invoice
    let invoice = wlt_2.invoice(contract_id, amount, false, None, None);

    // First use invoice
    wlt_1.send_to_invoice(&mut wlt_2, invoice.clone(), Some(500), None, None);

    // Second use same invoice
    let (_, _, _) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(600), None, None);

    // Check asset allocations
    wlt_1.check_allocations(contract_id, AssetSchema::RGB20, vec![100, 200]);
    wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![amount, amount]);
}

#[rstest]
#[case(TransferType::Blinded)]
#[case(TransferType::Witness)]
fn pay_one_invoice_twice(#[case] transfer_type: TransferType) {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 2000;
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", issue_supply);
    let outpoint = wlt_1.get_utxo(None);
    params.add_allocation(outpoint, issue_supply);

    let contract_id = wlt_1.issue_nia_with_params(params);

    let amount = 100;
    let wout = match transfer_type {
        TransferType::Blinded => false,
        TransferType::Witness => true,
    };

    wlt_1.send_contract("TestAsset", &mut wlt_2);
    wlt_2.reload_runtime();
    let invoice = wlt_2.invoice(contract_id, amount, wout, None, None);

    wlt_1.sync();

    let (consignment, tx, _payment) = wlt_1.transfer(invoice.clone(), None, Some(1000), true, None);
    let (consignment2, tx2, _paytment2) = wlt_1.transfer(invoice, None, Some(1000), true, None);

    wlt_1.sync();

    wlt_1.mine_tx(&tx.txid(), false);
    let _ = wlt_2.accept_transfer(&consignment, None);
    wlt_1.sync();

    wlt_1.check_allocations(
        contract_id,
        AssetSchema::RGB20,
        vec![issue_supply - amount - amount],
    );
    wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![amount]);

    println!("first transfer complete. sending second one");

    wlt_1.sync();

    wlt_1.mine_tx(&tx2.txid(), false);
    wlt_2.sync();
    let _ = wlt_2.accept_transfer(&consignment2, None);
    wlt_1.sync();

    wlt_1.check_allocations(
        contract_id,
        AssetSchema::RGB20,
        vec![issue_supply - amount - amount],
    );
    wlt_2.check_allocations(contract_id, AssetSchema::RGB20, vec![amount, amount]);
}

#[test]
#[ignore = "fix needed"]
#[serial]
fn sync_mainnet_wlt() {
    initialize();

    // FIXME: Because the latest `Mound` structure in rgb does not support setting the mainnet,
    // The default `Mound.testnet` is eq true, which cannot correctly initialize the mainnet wallet,
    // So this test case cannot be executed temporarily
    let mut wlt_1 = get_mainnet_wallet();

    wlt_1.sync();
}

#[test]
#[serial]
fn receive_from_unbroadcasted_transfer_to_blinded() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);

    // Create and issue assets
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", 600);
    let utxo = wlt_1.get_utxo(None);
    params.add_allocation(utxo, 600);
    let contract_id = wlt_1.issue_nia_with_params(params);
    wlt_1.send_contract("TestAsset", &mut wlt_2);
    wlt_2.reload_runtime();
    wlt_1.send_contract("TestAsset", &mut wlt_3);
    wlt_3.reload_runtime();

    // Get UTXO and create invoice
    let utxo = wlt_2.get_utxo(None);

    // In RGB v0.12, the invoice API has been changed
    let invoice = wlt_2.invoice(contract_id, 100, false, None, Some(utxo));

    // Create transfer but do not broadcast its TX
    let (consignment, _tx, _) = wlt_1.transfer(invoice.clone(), None, Some(500), false, None);
    wlt_2.accept_transfer(&consignment, None).unwrap();

    // The following three lines are debug code,
    // used to debug the transfer under normal broadcast logic

    // wlt_1.broadcast_tx(&tx);
    // wlt_1.mine_tx(&tx.txid(), false);
    // wlt_1.sync();

    dbg!(wlt_2.runtime().state_own(contract_id).owned);

    let invoice = wlt_3.invoice(contract_id, 50, true, None, None);
    // force stop sync, because the asset of wlt1 to wlt2 has not been on-chain,
    // the transfer will execute sync, so it is impossible to transfer assets to wlt3
    wlt_2.set_force_stop_sync(true);
    let (consignment, tx, _) = wlt_2.transfer(invoice, Some(2000), None, true, None);
    wlt_2.mine_tx(&tx.txid(), false);
    wlt_3.accept_transfer(&consignment, None).unwrap();
    let wlt_3_states = wlt_3.runtime().state_own(contract_id).owned;
    let wlt_2_states = wlt_2.runtime().state_own(contract_id).owned;
    dbg!(wlt_3_states, wlt_2_states);
    dbg!(wlt_2.runtime().state_all(contract_id).owned);
    dbg!(wlt_1.runtime().state_own(contract_id).owned);
    dbg!(wlt_1.runtime().state_all(contract_id).owned);

    wlt_3.check_allocations(contract_id, AssetSchema::RGB20, vec![50]);
    // This is the key point:
    // Since the asset transfer from wlt1 to wlt2 was not broadcasted on-chain,
    // when wlt2 transfers part of this asset to wlt3 and broadcasts it on-chain,
    // after sync the transaction is invalid because the original witness information
    // cannot be traced back to its source. The asset becoming invalid is expected behavior.
    wlt_3.sync();
    wlt_3.check_allocations(contract_id, AssetSchema::RGB20, vec![]);
}
