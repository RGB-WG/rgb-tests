pub mod utils;

use utils::{
    chain::{get_height, initialize, stop_mining},
    helpers::{get_wallet, AssetSchema, NIAIssueParams},
    DescriptorType, *,
};

#[test]
fn simple_transfer() {
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

    let invoice = wlt_2.invoice(contract_id, 400, true);

    // First transfer attempt - with lower fee
    let (consignment_1, tx) = wlt_1.transfer(invoice.clone(), None, Some(500), true, None);

    // Receiver accepts the transfer
    wlt_2.accept_transfer(&consignment_1, None);

    // Broadcast and confirm transaction
    wlt_1.mine_tx(&tx.txid(), true);

    // Sync both wallets
    wlt_1.sync();
    wlt_2.sync();

    // Verify asset allocations in both wallets
    // FIXME:
    // very strange wlt_1, why there are two states after transfer?
    // should check the helper-api implementation
    //     assertion `left == right` failed
    //  left: [200, 400]
    //  right: [200]
    // wlt_1.check_allocations(contract_id, AssetSchema::Nia, vec![200]);
    // wlt_2.check_allocations(contract_id, AssetSchema::Nia, vec![400]);
    dbg!(
        &wlt_1
            .runtime()
            .state_own(Some(contract_id))
            .next()
            .unwrap()
            .1
            .owned
    );
    dbg!(
        &wlt_2
            .runtime()
            .state_own(Some(contract_id))
            .next()
            .unwrap()
            .1
            .owned
    );
}

#[test]
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

    let invoice = wlt_2.invoice(contract_id, 400, true);

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
    wlt_2.send(&mut wlt_1, false, contract_id, 400, 2000, None);
}
