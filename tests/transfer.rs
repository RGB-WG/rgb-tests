pub mod utils;

use utils::{
    chain::{get_height, initialize, stop_mining},
    helpers::{get_wallet, AssetSchema, NIAIssueParams},
    DescriptorType, *,
};

// FIXME: 
// If invoice: wout is false, the asset transfer succeeds; if true, the asset transfer fails
#[test]
fn simple_transfer() {
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
    let invoice = wlt_2.invoice(contract_id, assign, false, Some(0));

    // First transfer attempt - with lower fee
    let (consignment_1, tx) = wlt_1.transfer(invoice, None, Some(500), true, None);

    // Receiver accepts the transfer
    wlt_2.accept_transfer(&consignment_1, None);

    // Broadcast and confirm transaction
    wlt_1.mine_tx(&tx.txid(), true);

    // Sync both wallets
    wlt_1.sync();
    wlt_2.sync();

    wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![supply - assign]);
    wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![assign]);

    // let assign_wlt1 = 200;
    // let invoice = wlt_1.invoice(contract_id, assign_wlt1, false, Some(0));
    // let (consignment_2, tx) = wlt_2.transfer(invoice, None, Some(500), true, None);
    // wlt_1.accept_transfer(&consignment_2, None);
    // wlt_2.mine_tx(&tx.txid(), true);

    // // Sync both wallets
    // wlt_1.sync();
    // wlt_2.sync();

    // wlt_1.check_allocations(
    //     contract_id,
    //     AssetSchema::Nia,
    //     vec![supply - assign + assign_wlt1],
    // );
    // wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![assign - assign_wlt1]);
}

#[test]
#[ignore]
// FIXME:
// Invalid operation data: operation references immutable memory cell VMlnpvOd~VSldT4BePvgcH4oM68W2noeTXgTXbtBNoU:0 which was not defined.
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

    // Stop mining to test RBF
    stop_mining();
    let initial_height = get_height();

    let invoice = wlt_2.invoice(contract_id, 400, false, Some(0));

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
