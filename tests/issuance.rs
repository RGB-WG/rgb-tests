pub mod utils;

use rstest_reuse::{self, *};
use utils::{
    chain::initialize,
    helpers::{get_wallet, CFAIssueParams, FUAIssueParams, NIAIssueParams},
    DescriptorType, *,
};

#[allow(dead_code)]
const MEDIA_FPATH: &str = "tests/fixtures/rgb_logo.jpeg";

#[template]
#[rstest]
#[case(DescriptorType::Wpkh)]
#[case(DescriptorType::Wpkh)]
#[case(DescriptorType::Tr)]
#[case(DescriptorType::Tr)]
fn descriptor_and_close_method(#[case] wallet_desc: DescriptorType) {}

#[apply(descriptor_and_close_method)]
fn issue_nia(wallet_desc: DescriptorType) {
    println!("wallet_desc {wallet_desc:?}");

    initialize();

    let mut wallet = get_wallet(&wallet_desc);

    // Create NIA issuance parameters
    let mut params = NIAIssueParams::new("TestAsset", "TEST", "centiMilli", 1_000_000);

    // Add initial allocations
    let fake_outpoint_zero =
        Outpoint::from_str("0000000000000000000000000000000000000000000000000000000000000000:0")
            .unwrap();
    let fake_outpoint_one =
        Outpoint::from_str("0000000000000000000000000000000000000000000000000000000000000001:0")
            .unwrap();
    params
        .add_allocation(fake_outpoint_zero, 500_000)
        .add_allocation(fake_outpoint_one, 500_000);

    // Issue the contract
    let contract_id = wallet.issue_nia_with_params(params);

    // Verify contract state
    let state = wallet
        .contract_state(contract_id)
        .expect("Contract state does not exist");

    // Verify immutable state
    assert_eq!(state.immutable.name, "TestAsset");
    assert_eq!(state.immutable.ticker, "TEST");
    assert_eq!(state.immutable.precision, "centiMilli");

    // Verify circulating supply
    assert_eq!(state.immutable.circulating_supply, 1_000_000);

    // Verify ownership state
    dbg!(&state.owned.allocations);
    assert_eq!(state.owned.allocations.len(), 2);
    assert!(state
        .owned
        .allocations
        .iter()
        .any(|(outpoint, amount)| *outpoint == fake_outpoint_zero && *amount == 500_000));
    assert!(state
        .owned
        .allocations
        .iter()
        .any(|(outpoint, amount)| *outpoint == fake_outpoint_one && *amount == 500_000));
    dbg!(wallet
        .runtime()
        .state_all(Some(contract_id))
        .collect::<Vec<_>>());
}

#[apply(descriptor_and_close_method)]
fn issue_fua(wallet_desc: DescriptorType) {
    println!("wallet_desc {wallet_desc:?}");

    initialize();

    let mut wallet = get_wallet(&wallet_desc);

    // Create CFA issuance parameters
    let mut params = FUAIssueParams::new("DemoCFA", "details", "centiMilli", 10_000);

    // Add initial allocation
    let fake_outpoint =
        Outpoint::from_str("b7116550736fbe5d3e234d0141c6bc8d1825f94da78514a3cede5674e9a5eae9:1")
            .unwrap();
    params.add_allocation(fake_outpoint, 10_000);

    // Issue the contract
    let contract_id = wallet.issue_fua_with_params(params);

    // Verify contract state
    let state = wallet
        .contract_state(contract_id)
        .expect("Contract state does not exist");

    // Verify immutable state
    assert_eq!(state.immutable.name, "DemoCFA");
    assert_eq!(state.immutable.precision, "centiMilli");
    assert_eq!(state.immutable.circulating_supply, 10_000);

    // Verify ownership state
    assert_eq!(state.owned.allocations.len(), 1);
    assert!(state
        .owned
        .allocations
        .iter()
        .any(|(outpoint, amount)| *outpoint == fake_outpoint && *amount == 10_000));
    dbg!(wallet
        .runtime()
        .state_all(Some(contract_id))
        .collect::<Vec<_>>());
}

#[apply(descriptor_and_close_method)]
fn issue_fua_multiple_utxos(wallet_desc: DescriptorType) {
    println!("wallet_desc {wallet_desc:?}");

    initialize();

    let mut wallet = get_wallet(&wallet_desc);

    // Create FUA issuance parameters with multiple allocations
    let mut params = FUAIssueParams::new("Multi_UTXO_CFA", "details", "centiMilli", 999);

    // Get multiple UTXOs and add allocations
    let amounts = [222, 444, 333];
    for amount in amounts.iter() {
        let outpoint = wallet.get_utxo(None);
        params.add_allocation(outpoint, *amount);
    }

    // Issue the contract
    let contract_id = wallet.issue_fua_with_params(params);

    // Verify contract state
    let state = wallet
        .contract_state(contract_id)
        .expect("Contract state does not exist");

    // Verify immutable state
    assert_eq!(state.immutable.name, "Multi_UTXO_CFA");
    assert_eq!(state.immutable.precision, "centiMilli");
    assert_eq!(state.immutable.circulating_supply, 999);

    // Verify ownership state
    assert_eq!(state.owned.allocations.len(), 3);
    let total_allocated: u64 = state
        .owned
        .allocations
        .iter()
        .map(|(_, amount)| amount)
        .sum();
    assert_eq!(total_allocated, 999);
}

#[apply(descriptor_and_close_method)]
fn issue_nia_multiple_utxos(wallet_desc: DescriptorType) {
    println!("wallet_desc {wallet_desc:?}");

    initialize();

    let mut wallet = get_wallet(&wallet_desc);

    // Create NIA issuance parameters with multiple allocations
    let mut params = NIAIssueParams::new("Multi_UTXO_NIA", "MUTX", "centiMilli", 999);

    // Get multiple UTXOs and add allocations
    let amounts = [333, 333, 333];
    for amount in amounts.iter() {
        let outpoint = wallet.get_utxo(None);
        params.add_allocation(outpoint, *amount);
    }

    // Issue the contract
    let contract_id = wallet.issue_nia_with_params(params);

    // Verify contract state
    let state = wallet
        .contract_state(contract_id)
        .expect("Contract state does not exist");

    // Verify immutable state
    assert_eq!(state.immutable.name, "Multi_UTXO_NIA");
    assert_eq!(state.immutable.ticker, "MUTX");
    assert_eq!(state.immutable.precision, "centiMilli");
    assert_eq!(state.immutable.circulating_supply, 999);

    // Verify ownership state
    assert_eq!(state.owned.allocations.len(), 3);
    let total_allocated: u64 = state
        .owned
        .allocations
        .iter()
        .map(|(_, amount)| amount)
        .sum();
    assert_eq!(total_allocated, 999);
}

// TODO: RGB official is improving the feature of uda asset, will add test after it's ready
// #[apply(descriptor_and_close_method)]
// fn issue_uda(wallet_desc: DescriptorType) {
//     println!("wallet_desc {wallet_desc:?} ");

//     initialize();

//     let mut wallet = get_wallet(&wallet_desc);

//     let ticker = "TCKR";
//     let name = "asset name";
//     let details = Some("some details");
//     let terms_text = "Ricardian contract";
//     let terms_media_fpath = Some(MEDIA_FPATH);
//     let data = vec![1u8, 3u8, 9u8];
//     let preview_ty = "image/jpeg";
//     let token_data_preview = EmbeddedMedia {
//         ty: MediaType::with(preview_ty),
//         data: Confined::try_from(data.clone()).unwrap(),
//     };
//     let proof = vec![2u8, 4u8, 6u8, 10u8];
//     let token_data_reserves = ProofOfReserves {
//         utxo: Outpoint::from_str(FAKE_TXID).unwrap(),
//         proof: Confined::try_from(proof.clone()).unwrap(),
//     };
//     let token_data_ticker = "TDTCKR";
//     let token_data_name = "token data name";
//     let token_data_details = "token data details";
//     let token_data_attachment = attachment_from_fpath(MEDIA_FPATH);
//     let mut token_data_attachments = BTreeMap::new();
//     for (idx, attachment_fpath) in ["README.md", "Cargo.toml"].iter().enumerate() {
//         token_data_attachments.insert(idx as u8, attachment_from_fpath(attachment_fpath));
//     }
//     let token_data = uda_token_data(
//         token_data_ticker,
//         token_data_name,
//         token_data_details,
//         token_data_preview.clone(),
//         token_data_attachment.clone(),
//         token_data_attachments.clone(),
//         token_data_reserves.clone(),
//     );
//     let asset_info = AssetInfo::uda(
//         ticker,
//         name,
//         details,
//         terms_text,
//         terms_media_fpath,
//         token_data,
//     );
//     let (contract_id, iface_type_name) = wallet.issue_with_info(asset_info, close_method, vec![]);

//     let contract = wallet.contract_iface_class::<Rgb21>(contract_id);
//     let spec = contract.spec();
//     assert_eq!(spec.ticker.to_string(), ticker.to_string());
//     assert_eq!(spec.name.to_string(), name.to_string());
//     assert_eq!(spec.precision.decimals(), 0);
//     let terms = contract.contract_terms();
//     assert_eq!(terms.text.to_string(), terms_text.to_string());
//     let terms_media = terms.media.unwrap();
//     assert_eq!(terms_media.ty.to_string(), "image/jpeg");
//     assert_eq!(
//         terms_media.digest.to_string(),
//         "02d2cc5d7883885bb7472e4fe96a07344b1d7cf794cb06943e1cdb5c57754d8a"
//     );
//     let token_data = contract.token_data();
//     assert_eq!(token_data.index, TokenIndex::from(0));
//     assert_eq!(token_data.ticker.unwrap().to_string(), token_data_ticker);
//     assert_eq!(token_data.name.unwrap().to_string(), token_data_name);
//     assert_eq!(token_data.details.unwrap().to_string(), token_data_details);
//     assert_eq!(token_data.preview.unwrap(), token_data_preview);
//     assert_eq!(token_data.media.unwrap(), token_data_attachment);
//     assert_eq!(
//         token_data.attachments.to_unconfined(),
//         token_data_attachments
//     );
//     assert_eq!(token_data.reserves.unwrap(), token_data_reserves);

//     let allocations = wallet.contract_data_allocations(contract_id, &iface_type_name);
//     assert_eq!(allocations.len(), 1);
//     let allocation = &allocations[0];
//     assert_eq!(allocation.seal.method(), close_method);
//     assert_eq!(allocation.state.to_string(), "000000000100000000000000");
// }
