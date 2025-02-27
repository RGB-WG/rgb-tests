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

use rstest_reuse::{self, *};
use serial_test::serial;
use std::panic::{catch_unwind, AssertUnwindSafe};
use utils::{
    chain::{
        connect_reorg_nodes, disconnect_reorg_nodes, get_height, get_height_custom, initialize,
        mine_custom, stop_mining,
    },
    helpers::{
        broadcast_tx_and_mine, get_wallet, get_wallet_custom, AssetSchema, CFAIssueParams,
        HistoryType, NIAIssueParams, ReorgType, TransferType,
    },
    DescriptorType, INSTANCE_2, INSTANCE_3, *,
};

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
    let invoice = wlt_2.invoice(contract_id, assign, wout, Some(0), true);

    // send asset to wlt2
    // if `wout` is true (WitnessOut),
    // wlt2 will have a 3000 Sats UTXO, which will be spent to transfer assets to wlt1 in the next step
    let (consignment_1, tx) = wlt_1.transfer(invoice, Some(3000), Some(500), true, None);

    // Receiver accepts the transfer
    wlt_2.accept_transfer(&consignment_1, None);

    // Broadcast and confirm transaction
    wlt_1.mine_tx(&tx.txid(), true);

    // Sync both wallets
    wlt_1.sync();
    wlt_2.sync();

    wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![supply - assign]);
    wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![assign]);

    let assign_wlt1 = 200;
    let invoice = wlt_1.invoice(contract_id, assign_wlt1, wout, Some(0), true);
    dbg!(
        "wlt2",
        wlt_2.runtime().wallet.balance(),
        wlt_2.runtime().wallet.coins().collect::<Vec<_>>()
    );
    // Sats cost: 500 fee + 2000 sats(default) = 2500
    let (consignment_2, tx) = wlt_2.transfer(invoice, None, Some(500), true, None);
    wlt_1.accept_transfer(&consignment_2, None);
    wlt_2.mine_tx(&tx.txid(), true);

    // // Sync both wallets
    wlt_1.sync();
    wlt_2.sync();

    // owned state
    dbg!(wlt_1
        .runtime()
        .state_own(Some(contract_id))
        .map(|s| { s.1.owned })
        .collect::<Vec<_>>());
    dbg!(wlt_2
        .runtime()
        .state_own(Some(contract_id))
        .map(|s| { s.1.owned })
        .collect::<Vec<_>>());

    wlt_1.check_allocations(
        contract_id,
        AssetSchema::Nia,
        vec![supply - assign, assign_wlt1],
    );
    wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![assign - assign_wlt1]);
}

#[test]
// FIXME:
// called `Result::unwrap()` on an `Err` value: Fulfill(StateInsufficient)
//
// RBF transfer fails in the second asset transfer,
// Likely due to the inability to spend the same asset balance twice.
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

    let invoice = wlt_2.invoice(contract_id, 400, false, Some(0), true);

    // Stop mining to test RBF
    stop_mining();
    let initial_height = get_height();

    // First transfer attempt - with lower fee
    let (consignment_1, _) = wlt_1.transfer(invoice.clone(), None, Some(500), true, None);

    // Receiver accepts the transfer
    wlt_2.accept_transfer(&consignment_1, None);

    // Verify block height hasn't changed (transaction not confirmed)
    let mid_height = get_height();
    assert_eq!(initial_height, mid_height);

    // Second transfer attempt - with higher fee for RBF
    let (consignment_2, tx) = wlt_1.transfer(invoice, None, Some(1000), true, None);

    // Verify block height still hasn't changed
    let final_height = get_height();
    assert_eq!(initial_height, final_height);

    // Broadcast and confirm transaction
    wlt_1.mine_tx(&tx.txid(), true);

    // Receiver accepts final transfer
    wlt_2.accept_transfer(&consignment_2, None);

    // Sync both wallets
    wlt_1.sync();
    wlt_2.sync();

    // Verify asset allocations in both wallets
    wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![200]);
    wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![400]);

    // Transfer assets back to sender
    wlt_2.send(&mut wlt_1, false, contract_id, 400, 2000, Some(0), None);
}

#[rstest]
// blinded: nia - nia
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Nia)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Nia, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Nia, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Nia, AS::Nia)]
// blinded: nia - cfa
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Cfa)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Nia, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Nia, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Nia, AS::Cfa)]
// blinded: cfa - cfa
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Cfa)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Cfa, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Cfa, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Cfa, AS::Cfa)]
// FIXME: `calling to method absent in Codex API`
// When using the same utxo for issue, the transfer will report an error
//
// There is also a strange phenomenon that when all assets issued using the same utxo are CFA types, no errors are reported.
// rgb-test cmd: cargo test transfer_loop::case_09
// If both are NIA or the first asset NIA, an error will occur.
// rgb-test cmd: cargo test transfer_loop::case_01
//
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
        AssetSchema::Nia => {
            let mut params =
                NIAIssueParams::new("TestAsset1", "TEST1", "centiMilli", issued_supply_1);
            params.add_allocation(utxo, issued_supply_1);
            wlt_1.issue_nia_with_params(params)
        }
        AssetSchema::Cfa => {
            let mut params = CFAIssueParams::new("TestAsset1", "centiMilli", issued_supply_1);
            params.add_allocation(utxo, issued_supply_1);
            wlt_1.issue_cfa_with_params(params)
        }
        AssetSchema::Uda => {
            // TODO: UDA is not supported yet
            panic!("UDA is not supported yet");
        }
    };

    // Issue second asset
    let contract_id_2 = match asset_schema_2 {
        AssetSchema::Nia => {
            let mut params =
                NIAIssueParams::new("TestAsset2", "TEST2", "centiMilli", issued_supply_2);
            params.add_allocation(utxo, issued_supply_2);
            wlt_1.issue_nia_with_params(params)
        }
        AssetSchema::Cfa => {
            let mut params = CFAIssueParams::new("TestAsset2", "centiMilli", issued_supply_2);
            params.add_allocation(utxo, issued_supply_2);
            wlt_1.issue_cfa_with_params(params)
        }
        AssetSchema::Uda => {
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
    let amount_1 = if asset_schema_1 != AssetSchema::Uda {
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
    if asset_schema_1 != AssetSchema::Uda {
        let amount_2 = 33;
        wlt_1.send(
            &mut wlt_2,
            wout,
            contract_id_1,
            amount_2,
            sats,
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
    let amount_3 = if asset_schema_2 != AssetSchema::Uda {
        22
    } else {
        1
    };
    wlt_1.send(&mut wlt_2, wout, contract_id_2, amount_3, sats, None, None);

    // Verify final allocations
    if asset_schema_1 != AssetSchema::Uda {
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
    let amount_4 = if asset_schema_1 != AssetSchema::Uda {
        111
    } else {
        1
    };
    let amount_2 = if asset_schema_1 != AssetSchema::Uda {
        33
    } else {
        0
    };
    sats -= 1000;
    wlt_2.send(&mut wlt_1, wout, contract_id_1, amount_4, sats, None, None);
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
    let amount_5 = if asset_schema_2 != AssetSchema::Uda {
        11
    } else {
        1
    };
    sats -= 1000;
    wlt_2.send(&mut wlt_1, wout, contract_id_2, amount_5, sats, None, None);
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

    // for debug
    {
        let wlt_1_contract_2_state = wlt_1.runtime().state_own(None).map(|s| s.1.owned);
        dbg!(wlt_1_contract_2_state.collect::<Vec<_>>());
    }

    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4],
    );
    wlt_2.check_allocations(contract_id_2, asset_schema_2, vec![amount_3 - amount_5]);

    // wlt_1 spends asset 1, received back
    let amount_6 = if asset_schema_1 != AssetSchema::Uda {
        issued_supply_1 - amount_1 - amount_2 + amount_4
    } else {
        1
    };
    sats -= 1000;
    wlt_1.send(&mut wlt_2, wout, contract_id_1, amount_6, sats, None, None);
    wlt_1.check_allocations(contract_id_1, asset_schema_1, vec![]);
    // for debug
    {
        let wlt_1_contract_2_state = wlt_1.runtime().state_own(None).map(|s| s.1.owned);
        dbg!(wlt_1_contract_2_state.collect::<Vec<_>>());
    }

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
    let amount_7 = if asset_schema_2 != AssetSchema::Uda {
        issued_supply_2 - amount_3 + amount_5
    } else {
        1
    };
    sats -= 1000;
    wlt_1.send(&mut wlt_2, wout, contract_id_2, amount_7, sats, None, None);
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

// Test case pending new rollback procedure API
// Will be updated once the high-level API for rollback handling is available
#[rstest]
#[ignore = "Awaiting new rollback procedure API in RGB v0.12"]
#[case(TransferType::Blinded)]
#[case(TransferType::Witness)]
fn same_transfer_twice_update_witnesses(#[case] transfer_type: TransferType) {}

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
#[case(TransferType::Blinded)]
#[case(TransferType::Witness)]
fn same_transfer_twice_no_update_witnesses(#[case] transfer_type: TransferType) {
    println!("transfer_type {transfer_type:?}");

    // TODO: This test case aims to verify if asset transfers are handled correctly when using RBF (Replace-By-Fee) with the same invoice
    // In RGB V0.11, there was an inflation attack vulnerability where using RBF with the same invoice would cause the receiver to record two receive states,
    // while the sender only paid the assets once. This resulted in the total circulating assets exceeding the issued amount.
    //
    // In RGB V0.12, since it's not possible to use RBF with the same invoice, we cannot test for this inflation attack.
    // NOTE: When paying the same invoice for the second time, the error `called `Result::unwrap()` on an `Err` value: Fulfill(StateInsufficient)` occurs
    // Need to consult with Dr. Maxim on how to construct RBF asset transfer examples using the current API.

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
    let invoice = wlt_2.invoice(contract_id, amount, wout, Some(0), true);
    let _ = wlt_1.transfer(invoice.clone(), None, Some(500), false, None);

    dbg!(wlt_1
        .runtime()
        .state_all(None)
        .map(|(c, s)| { s.owned })
        .collect::<Vec<_>>());
    dbg!(wlt_1
        .runtime()
        .state_own(None)
        .map(|(c, s)| { s.owned })
        .collect::<Vec<_>>());

    // dbg!(wlt_2
    //     .runtime()
    //     .state_all(None)
    //     .map(|(c, s)| { s.owned })
    //     .collect::<Vec<_>>());
    // dbg!(wlt_2
    //     .runtime()
    //     .state_own(None)
    //     .map(|(c, s)| { s.owned })
    //     .collect::<Vec<_>>());

    // TODO: called `Result::unwrap()` on an `Err` value: Fulfill(StateInsufficient)
    let (consignment, _) = wlt_1.transfer(invoice, None, Some(1000), true, None);

    wlt_2.accept_transfer(&consignment, None);

    let wlt_2_contract_state = wlt_2.runtime().state_own(None).map(|s| s.1.owned);
    dbg!(wlt_2_contract_state.collect::<Vec<_>>());

    wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![amount]);

    wlt_2.send(&mut wlt_1, wout, contract_id, amount, 1000, None, None);

    let wlt_1_contract_state = wlt_1.runtime().state_own(None).map(|s| s.1.owned);
    dbg!(wlt_1_contract_state.collect::<Vec<_>>());

    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);
    wlt_1.send_contract("TestAsset", &mut wlt_3);
    wlt_3.reload_runtime();

    wlt_1.send(
        &mut wlt_3,
        wout,
        contract_id,
        issue_supply,
        1000,
        None,
        None,
    );
    let wlt_3_contract_state = wlt_3.runtime().state_own(None).map(|s| s.1.owned);
    dbg!(wlt_3_contract_state.collect::<Vec<_>>());
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
    let invoice = wlt_2.invoice(contract_id, amt, true, Some(0), true);
    let (consignment, tx) = wlt_1.transfer(invoice.clone(), None, None, true, None);
    let txid = tx.txid();

    wlt_2.accept_transfer(&consignment, None);

    // wlt_2 sees the allocation even if TX has not been mined
    wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![amt]);

    wlt_1.sync();

    let wlt_1_change_amt = issue_supply - amt;

    // wlt_1 needs to get tentative allocations to see its change from the unmined TX
    let wlt_1_contract_state = wlt_1.runtime().state_own(None).map(|s| s.1.owned);
    dbg!(wlt_1_contract_state.collect::<Vec<_>>());

    // after mining, wlt_1 doesn't need to get tentative allocations to see the change
    wlt_1.mine_tx(&txid, false);
    wlt_1.sync();
    wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_1_change_amt]);
}

#[test]
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
    wlt_1.send(&mut wlt_2, false, contract_id, 400, 5000, None, None);

    // Second transfer: wlt_2 -> wlt_1, transfer 100
    let invoice = wlt_1.invoice(contract_id, 100, true, Some(0), true);
    wlt_2.send_to_invoice(&mut wlt_1, invoice, None, None, None);

    // Third transfer: wlt_1 -> wlt_2, transfer 290
    wlt_1.send(&mut wlt_2, true, contract_id, 290, 1000, None, None);

    // Fourth transfer: wlt_2 -> wlt_1, transfer 560
    wlt_2.send(&mut wlt_1, false, contract_id, 560, 1000, None, None);

    // Fifth transfer: wlt_1 -> wlt_2, transfer 570
    wlt_1.send(&mut wlt_2, false, contract_id, 570, 1000, None, None);

    wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![]);
    wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![30, 570]);
}

#[test]
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
    dbg!(wlt_1
        .runtime()
        .state_own(Some(contract_id))
        .map(|s| { s.1.owned })
        .collect::<Vec<_>>());

    // transfer
    let amt = 200;
    let (_, tx) = wlt_1.send(&mut wlt_2, true, contract_id, amt, 1000, None, None);
    let _txid = tx.txid();

    // debug contract state
    dbg!(wlt_1
        .runtime()
        .state_own(Some(contract_id))
        .map(|s| { s.1.owned })
        .collect::<Vec<_>>());
    dbg!(wlt_2
        .runtime()
        .state_own(Some(contract_id))
        .map(|s| { s.1.owned })
        .collect::<Vec<_>>());

    // check allocations
    wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![issue_supply - amt]);
    wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![amt]);
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
    let invoice = wlt.invoice(contract_id, amt, true, Some(0), true);
    let (consignment, tx) = wlt.transfer(invoice.clone(), None, None, true, None);
    wlt.mine_tx(&tx.txid(), false);
    wlt.accept_transfer(&consignment, None);
    wlt.sync();

    // debug contract state
    dbg!(wlt
        .runtime()
        .state_own(None)
        .map(|s| s.1.owned)
        .collect::<Vec<_>>());

    // check allocations
    wlt.check_allocations(contract_id, AssetSchema::Nia, vec![amt, issue_supply - amt]);
}

#[rstest]
#[ignore = "fix needed (calling to method absent in Codex API)"]
#[case(DescriptorType::Tr, DescriptorType::Tr)]
#[ignore = "fix needed (calling to method absent in Codex API)"]
#[case(DescriptorType::Tr, DescriptorType::Wpkh)]
#[ignore = "fix needed (calling to method absent in Codex API)"]
#[case(DescriptorType::Wpkh, DescriptorType::Tr)]
#[ignore = "fix needed (calling to method absent in Codex API)"]
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
    wlt_1.send(&mut wlt_2, false, contract_id_0, 200, 1000, None, None);

    // Second transfer: wlt_1 -> wlt_2, transfer 100 of second asset
    // This tests the blank transfer functionality with different descriptor types
    wlt_1.send(&mut wlt_2, false, contract_id_1, 100, 1000, None, None);

    // Verify final allocations
    wlt_1.check_allocations(contract_id_0, AssetSchema::Nia, vec![]);
    wlt_1.check_allocations(contract_id_1, AssetSchema::Nia, vec![]);
    wlt_2.check_allocations(contract_id_0, AssetSchema::Nia, vec![200]);
    wlt_2.check_allocations(contract_id_1, AssetSchema::Nia, vec![100]);
}

#[rstest]
// Unable to accept a consignment: unknown seal definition for cell address qMWtQjXCWjJAXdrg7npyI2KZz3vXNVyZhoomqF7v8z4:0.
#[case(HistoryType::Linear, ReorgType::ChangeOrder)]
// #[ignore = "fix needed"]
#[case(HistoryType::Linear, ReorgType::Revert)]
#[case(HistoryType::Branching, ReorgType::ChangeOrder)]
// #[ignore = "fix needed"]
#[case(HistoryType::Branching, ReorgType::Revert)]
#[case(HistoryType::Merging, ReorgType::ChangeOrder)]
// #[ignore = "fix needed"]
#[case(HistoryType::Merging, ReorgType::Revert)]
#[serial]
fn reorg_history(#[case] history_type: HistoryType, #[case] reorg_type: ReorgType) {
    println!("history_type {history_type:?} reorg_type {reorg_type:?}");

    initialize();
    connect_reorg_nodes();

    let mut wlt_1 = get_wallet_custom(&DescriptorType::Wpkh, INSTANCE_2);
    let mut wlt_2 = get_wallet_custom(&DescriptorType::Wpkh, INSTANCE_2);

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

    // Generate UTXOs before asset transfer to avoid mining blocks during transfer, affecting the test
    let _utxo_wlt_1_1 = wlt_1.get_utxo(None);
    let _utxo_wlt_1_2 = wlt_1.get_utxo(None);
    let _utxo_wlt_2_1 = wlt_2.get_utxo(None);
    let _utxo_wlt_2_2 = wlt_2.get_utxo(None);
    mine_custom(false, INSTANCE_2, 6);

    dbg!(get_height_custom(INSTANCE_2));
    dbg!(get_height_custom(INSTANCE_3));

    disconnect_reorg_nodes();

    // Create transactions based on history type
    let txs = match history_type {
        HistoryType::Linear => {
            let amt_0 = 590;
            // Create blinded invoice with specific UTXO
            let invoice = wlt_2.invoice(contract_id, amt_0, false, Some(0), false);
            let (_, tx_0) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);
            dbg!(wlt_1
                .runtime()
                .state_own(Some(contract_id))
                .map(|s| { s.1.owned })
                .collect::<Vec<_>>());

            let amt_1 = 100;
            let invoice = wlt_1.invoice(contract_id, amt_1, false, Some(0), false);
            let (_, tx_1) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);
            dbg!(wlt_1
                .runtime()
                .state_own(Some(contract_id))
                .map(|s| { s.1.owned })
                .collect::<Vec<_>>());

            let amt_2 = 80;
            let invoice = wlt_2.invoice(contract_id, amt_2, false, Some(0), false);
            let (_, tx_2) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            vec![tx_0, tx_1, tx_2]
        }
        HistoryType::Branching => {
            let amt_0 = 600;
            let invoice = wlt_2.invoice(contract_id, amt_0, false, Some(0), false);
            let (_, tx_0) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            let amt_1 = 200;
            let invoice = wlt_1.invoice(contract_id, amt_1, false, Some(0), false);
            let (_, tx_1) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            let amt_2 = amt_0 - amt_1 - 1;
            let invoice = wlt_1.invoice(contract_id, amt_2, false, Some(0), false);
            let (_, tx_2) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            vec![tx_0, tx_1, tx_2]
        }
        HistoryType::Merging => {
            let amt_0 = 400;
            let invoice = wlt_2.invoice(contract_id, amt_0, false, Some(0), false);
            let (_, tx_0) = wlt_1.send_to_invoice(&mut wlt_2, invoice, None, None, None);

            let amt_1 = 200;
            let invoice = wlt_2.invoice(contract_id, amt_1, false, Some(0), false);
            let (_, tx_1) = wlt_1.send_to_invoice(&mut wlt_2, invoice, None, None, None);

            let amt_2 = amt_0 + amt_1 - 1;
            let invoice = wlt_1.invoice(contract_id, amt_2, false, Some(0), false);
            let (_, tx_2) = wlt_2.send_to_invoice(&mut wlt_1, invoice, None, None, None);

            vec![tx_0, tx_1, tx_2]
        }
    };

    dbg!(wlt_1
        .runtime()
        .state_own(Some(contract_id))
        .map(|s| { s.1.owned })
        .collect::<Vec<_>>());
    dbg!(wlt_2
        .runtime()
        .state_own(Some(contract_id))
        .map(|s| { s.1.owned })
        .collect::<Vec<_>>());

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
                AssetSchema::Nia,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
            );
            wlt_2.check_allocations(
                contract_id,
                AssetSchema::Nia,
                vec![wlt_2_alloc_1, wlt_2_alloc_2],
            );
        }
        (HistoryType::Linear | HistoryType::Branching, ReorgType::Revert) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 600;
            wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_1_alloc_1]);
            wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![]);
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
                AssetSchema::Nia,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
            );
            wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_2_alloc_1]);
        }
        (HistoryType::Merging, ReorgType::ChangeOrder) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[0], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 599;
            let wlt_2_alloc_1 = 1;
            wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_1_alloc_1]);
            wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_2_alloc_1]);
        }
        (HistoryType::Merging, ReorgType::Revert) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 400;
            let wlt_2_alloc_1 = 200;
            wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_1_alloc_1]);
            wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_2_alloc_1]);
        }
    }

    mine_custom(false, INSTANCE_3, 3);
    connect_reorg_nodes();
    wlt_1.switch_to_instance(INSTANCE_2);
    wlt_2.switch_to_instance(INSTANCE_2);

    let mut wlt_3 = get_wallet_custom(&DescriptorType::Wpkh, INSTANCE_2);

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
                AssetSchema::Nia,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
            );
            wlt_2.check_allocations(
                contract_id,
                AssetSchema::Nia,
                vec![wlt_2_alloc_1, wlt_2_alloc_2],
            );

            // Test spending the final allocations
            wlt_1.send_contract("TestAsset", &mut wlt_3);
            wlt_3.reload_runtime();
            wlt_1.send(&mut wlt_3, false, contract_id, wlt_1_amt, 1000, None, None);
            wlt_2.send(&mut wlt_3, false, contract_id, wlt_2_amt, 1000, None, None);
            wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![]);
            wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![]);
            wlt_3.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_1_amt, wlt_2_amt]);
        }
        HistoryType::Branching => {
            let wlt_1_alloc_1 = 200;
            let wlt_1_alloc_2 = 399;
            let wlt_1_amt = wlt_1_alloc_1 + wlt_1_alloc_2;
            let wlt_2_alloc_1 = 1;
            wlt_1.check_allocations(
                contract_id,
                AssetSchema::Nia,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
            );
            wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_2_alloc_1]);

            // Test spending the final allocations
            wlt_1.send_contract("TestAsset", &mut wlt_3);
            wlt_3.reload_runtime();
            wlt_1.send(&mut wlt_3, false, contract_id, wlt_1_amt, 1000, None, None);
            wlt_2.send(
                &mut wlt_3,
                false,
                contract_id,
                wlt_2_alloc_1,
                1000,
                None,
                None,
            );
            wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![]);
            wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![]);
            wlt_3.check_allocations(
                contract_id,
                AssetSchema::Nia,
                vec![wlt_1_amt, wlt_2_alloc_1],
            );
        }
        HistoryType::Merging => {
            let wlt_1_alloc_1 = 599;
            let wlt_2_alloc_1 = 1;
            wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_1_alloc_1]);
            wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![wlt_2_alloc_1]);

            // Test spending the final allocations
            wlt_1.send_contract("TestAsset", &mut wlt_3);
            wlt_3.reload_runtime();
            wlt_1.send(
                &mut wlt_3,
                false,
                contract_id,
                wlt_1_alloc_1,
                1000,
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
            );
            wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![]);
            wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![]);
            wlt_3.check_allocations(
                contract_id,
                AssetSchema::Nia,
                vec![wlt_1_alloc_1, wlt_2_alloc_1],
            );
        }
    }
}
