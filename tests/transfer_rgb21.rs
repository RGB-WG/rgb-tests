pub mod utils;

use ifaces::{EmbeddedMedia, MediaType, ProofOfReserves};

use rstest_reuse::{self, *};
use std::str::FromStr;
use utils::{
    chain::initialize,
    helper::wallet::{get_wallet, AssetSchema},
    DescriptorType, *,
};

#[template]
#[rstest]
#[case(true)]
#[case(false)]
fn wout(#[case] wout: bool) {}

#[allow(dead_code)]
const MEDIA_FPATH: &str = "tests/fixtures/rgb_logo.jpeg";

#[apply(wout)]
#[ignore = "failed to issue contract: Inner(Genesis(Named(TypeName('DigitalCollection')), ScriptUnspecified))"]
fn simple_fac_transfer(wout: bool) {
    initialize();

    // Create two wallet instances
    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);
    let total_fractions = 10_000;
    let asset_name = "DigitalCollection";

    // Create FAC asset parameters
    let mut fac_params = FACIssueParams::new(
        asset_name,
        "A collection of digital assets",
        total_fractions,
    );

    // Prepare NFT data
    let ticker = "DCOLL";
    let name = "Digital Collection #1";
    let details = "First item in the digital collection";

    // Create preview data
    let data = vec![1u8, 3u8, 9u8];
    let preview_ty = "image/jpeg";
    let token_data_preview = EmbeddedMedia {
        ty: MediaType::with(preview_ty),
        data: Confined::try_from(data.clone()).unwrap(),
    };

    // Create reserves proof
    let proof = vec![2u8, 4u8, 6u8, 10u8];
    let token_data_reserves = ProofOfReserves {
        utxo: Outpoint::from_str(FAKE_TXID).unwrap(),
        proof: Confined::try_from(proof.clone()).unwrap(),
    };

    // Create attachment
    let token_data_attachment = attachment_from_fpath(MEDIA_FPATH);

    // Create additional attachments
    let mut token_data_attachments = BTreeMap::new();
    for (idx, attachment_fpath) in ["README.md", "Cargo.toml"].iter().enumerate() {
        token_data_attachments.insert(idx as u8, attachment_from_fpath(attachment_fpath));
    }

    // Create NFT spec
    let nft_spec = nft_spec(
        ticker,
        name,
        details,
        token_data_preview.clone(),
        token_data_attachment.clone(),
        token_data_attachments,
        token_data_reserves.clone(),
    );

    // Get an outpoint for allocation
    let outpoint = wlt_1.get_utxo(None);
    fac_params.with_allocation(outpoint, total_fractions);
    fac_params.with_nft_spec(nft_spec);

    // Issue the contract
    let contract_id = wlt_1.issue_fac_with_params(fac_params);
    println!("FAC contract issued with ID: {}", contract_id);

    // Share contract with wallet 2
    wlt_1.send_contract(asset_name, &mut wlt_2);
    wlt_2.reload_runtime();

    // Verify initial state
    let state = wlt_1.contract_state_rgb21(contract_id).unwrap();
    assert_eq!(state.immutable.name.as_str(), asset_name);
    assert_eq!(state.immutable.total_fractions, total_fractions);
    assert_eq!(state.owned.fractions.len(), 1);
    assert_eq!(state.owned.fractions[0].1, total_fractions);

    dbg!(wlt_1.runtime().state_own(contract_id).owned);
    dbg!(wlt_2.runtime().state_own(contract_id).owned);

    // Transfer some fractions to wallet 2
    let transfer_amount = 1001;
    let invoice = wlt_2.invoice(contract_id, transfer_amount, wout, Some(0), None);
    let (consignment_1, tx, _) = wlt_1.transfer(invoice, Some(9000), Some(500), true, None);

    // Receiver accepts the transfer
    wlt_2.accept_transfer(&consignment_1, None).unwrap();

    // Broadcast and confirm transaction
    wlt_1.mine_tx(&tx.txid(), false);

    // Sync both wallets
    wlt_1.sync();
    wlt_2.sync();

    dbg!(wlt_1.runtime().state_own(contract_id).owned);
    dbg!(wlt_2.runtime().state_own(contract_id).owned);

    // Verify allocations after transfer
    wlt_1.check_allocations(
        contract_id,
        AssetSchema::RGB21,
        vec![total_fractions - transfer_amount],
    );
    wlt_2.check_allocations(contract_id, AssetSchema::RGB21, vec![transfer_amount]);

    // Check that NFT data is preserved in both wallets
    let state_1 = wlt_1.contract_state_rgb21(contract_id).unwrap();
    let token_1 = state_1.immutable.token.unwrap();
    assert_eq!(token_1.ticker.unwrap().to_string(), ticker);
    assert_eq!(token_1.name.unwrap().to_string(), name);

    let state_2 = wlt_2.contract_state_rgb21(contract_id).unwrap();
    let token_2 = state_2.immutable.token.unwrap();
    assert_eq!(token_2.ticker.unwrap().to_string(), ticker);
    assert_eq!(token_2.name.unwrap().to_string(), name);

    // Test transferring some fractions back to wallet 1
    let return_amount = 1000;
    let invoice = wlt_1.invoice(contract_id, return_amount, wout, Some(0), None);
    let (consignment_2, tx, _) = wlt_2.transfer(invoice, Some(3000), Some(500), true, None);

    // Wallet 1 accepts the transfer
    wlt_1.accept_transfer(&consignment_2, None).unwrap();

    // Broadcast and confirm transaction
    wlt_2.mine_tx(&tx.txid(), false);

    // Sync both wallets
    wlt_1.sync();
    wlt_2.sync();

    // Verify final allocations
    wlt_1.check_allocations(
        contract_id,
        AssetSchema::RGB21,
        vec![total_fractions - transfer_amount, return_amount],
    );
    wlt_2.check_allocations(
        contract_id,
        AssetSchema::RGB21,
        vec![transfer_amount - return_amount],
    );
}
