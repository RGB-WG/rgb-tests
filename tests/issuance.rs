pub mod utils;

use ifaces::{EmbeddedMedia, MediaType, ProofOfReserves};
use rstest_reuse::{self, *};
use utils::{chain::initialize, helper::wallet::get_wallet, DescriptorType, *};

#[allow(dead_code)]
const MEDIA_FPATH: &str = "tests/fixtures/rgb_logo.jpeg";

#[template]
#[rstest]
#[case(DescriptorType::Wpkh)]
#[case(DescriptorType::Tr)]
fn descriptor(#[case] wallet_desc: DescriptorType) {}

#[apply(descriptor)]
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
    assert_eq!(state.owned.allocations.len(), 2);
    assert!(state
        .owned
        .allocations
        .iter()
        .any(|(outpoint, amount)| outpoint.primary == fake_outpoint_zero && *amount == 500_000));
    assert!(state
        .owned
        .allocations
        .iter()
        .any(|(outpoint, amount)| outpoint.primary == fake_outpoint_one && *amount == 500_000));
}

#[apply(descriptor)]
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
        .any(|(outpoint, amount)| outpoint.primary == fake_outpoint && *amount == 10_000));
}

#[apply(descriptor)]
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

#[apply(descriptor)]
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

#[apply(descriptor)]
#[ignore = "failed to issue contract: Inner(Genesis(Named(TypeName('DigitalCollection')), ScriptUnspecified))"]
fn issue_fac(wallet_desc: DescriptorType) {
    println!("wallet_desc {wallet_desc:?}");

    initialize();

    let mut wallet = get_wallet(&wallet_desc);

    // Create FAC asset parameters
    let mut fac_params = FACIssueParams::new(
        "DigitalCollection",
        "A collection of digital assets",
        10_000,
    );

    let ticker = "TCKR";
    let name = "asset name";
    let details = "some details";
    let data = vec![1u8, 3u8, 9u8];
    let preview_ty = "image/jpeg";
    let token_data_preview = EmbeddedMedia {
        mime: MediaType::with(preview_ty),
        data: Confined::try_from(data.clone()).unwrap(),
    };
    let proof = vec![2u8, 4u8, 6u8, 10u8];
    let token_data_reserves = ProofOfReserves {
        utxo: Outpoint::from_str(FAKE_TXID).unwrap(),
        proof: Confined::try_from(proof.clone()).unwrap(),
    };
    let token_data_attachment = attachment_from_fpath(MEDIA_FPATH);
    let mut token_data_attachments = BTreeMap::new();
    for (idx, attachment_fpath) in ["README.md", "Cargo.toml"].iter().enumerate() {
        token_data_attachments.insert(idx as u8, attachment_from_fpath(attachment_fpath));
    }

    let nft_spec = nft_spec(
        name,
        token_data_preview.clone(),
        token_data_attachment.clone(),
        token_data_reserves.clone(),
    );

    // Create allocation
    let outpoint = wallet.get_utxo(None);
    fac_params.with_allocation(outpoint, 10_000);
    fac_params.with_nft_spec(nft_spec);
    // Issue the contract
    let contract_id = wallet.issue_fac_with_params(fac_params);
    println!("FAC contract issued with ID: {}", contract_id);

    let state = wallet.contract_state_rgb21(contract_id).unwrap();
    let token = state.immutable.token.unwrap();
    assert_eq!(state.immutable.name.as_str(), "DigitalCollection");
    assert_eq!(state.immutable.total_fractions, 10_000);
    assert_eq!(token.ticker.unwrap().to_string(), ticker);
    assert_eq!(token.name.unwrap().to_string(), name);
    assert_eq!(token.details.unwrap().to_string(), details);
    assert_eq!(
        token.preview.as_ref().unwrap().media_type.r#type.as_str(),
        token_data_preview.mime.ty.as_str()
    );

    assert_eq!(
        token.preview.as_ref().unwrap().data.as_slice(),
        token_data_preview.data.as_slice()
    );
    assert_eq!(
        token.media.as_ref().unwrap().digest.as_slice(),
        token_data_attachment.digest.as_slice()
    );
    // token_data_attachments
    for (idx, attachment) in token.attachments.iter() {
        assert_eq!(
            attachment.digest.as_slice(),
            token_data_attachments[idx].digest.as_slice()
        );
    }
    assert_eq!(
        token.reserves.as_ref().unwrap().utxo.to_string(),
        token_data_reserves.utxo.to_string()
    );
    assert_eq!(
        token.reserves.as_ref().unwrap().proof.as_slice(),
        token_data_reserves.proof.as_slice()
    );

    assert_eq!(state.owned.fractions.len(), 1);
    assert_eq!(state.owned.fractions[0].0.to_string(), outpoint.to_string());
    assert_eq!(state.owned.fractions[0].1, 10_000);
}
