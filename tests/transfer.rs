pub mod utils;

use rstest_reuse::{self, *};
use std::panic::{catch_unwind, AssertUnwindSafe};
use utils::{
    chain::{get_height, initialize, stop_mining},
    helpers::{get_wallet, AssetSchema, CFAIssueParams, NIAIssueParams, TransferType},
    DescriptorType, *,
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

    // pub type DirMound<SealDef> = Mound<FileSupply, FilePile<SealDef>, DirExcavator<SealDef>>;
    //  let mound: &mut rgb::Mound<rgb::FileSupply, rgb::FilePile<bp::seals::WTxoSeal>, rgb::DirExcavator<bp::seals::WTxoSeal>> = &mut wlt_1.runtime().mound;

    let assign = 400;
    // recive asset by utxo
    let invoice = wlt_2.invoice(contract_id, assign, wout, Some(0));

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
    let invoice = wlt_1.invoice(contract_id, assign_wlt1, wout, Some(0));
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
        .collect::<Vec<_>>());
    dbg!(wlt_2
        .runtime()
        .state_own(Some(contract_id))
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

    let invoice = wlt_2.invoice(contract_id, 400, false, Some(0));

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
