pub mod utils;
use rstest::rstest;

use rstest_reuse::{self, *};
use utils::{chain::initialize, DescriptorType, *};

type TT = TransferType;
type DT = DescriptorType;
type AS = AssetSchema;

const MEDIA_FPATH: &str = "tests/fixtures/rgb_logo.jpeg";

#[template]
#[rstest]
#[case(DescriptorType::Wpkh)]
#[case(DescriptorType::Wpkh)]
#[case(DescriptorType::Tr)]
#[case(DescriptorType::Tr)]
fn descriptor_and_close_method(#[case] wallet_desc: DescriptorType) {}

#[apply(descriptor_and_close_method)]
fn issue_nia(#[case] wallet_desc: DescriptorType) {
    use utils::runtime::TestRuntime;

    initialize();

    let mut wallet = TestRuntime::new(&wallet_desc, "issuer");

    let utxo = wallet.get_utxo(Some(10_000));

    let contract_id = wallet.issue_nia("DemoNIA", 10_000, utxo.clone());

    wallet.check_allocations(contract_id, AS::Nia, vec![10_000], false);

    let articles = wallet.contracts.contract_articles(contract_id);
    assert_eq!(articles.issue.meta.name.to_string(), "DemoNIA");

    let mut found_name = false;
    let mut found_ticker = false;
    let mut found_precision = false;
    let mut found_circulating = false;

    let state = wallet.contracts.contract_state(contract_id);
    let imm_state = &state.immutable;
    for (name, map) in imm_state {
        for (_, state) in map {
            match name.as_str() {
                "name" => {
                    assert_eq!(state.data.verified.to_string(), "\"DemoNIA\"");
                    found_name = true;
                }
                "ticker" => {
                    assert_eq!(state.data.verified.to_string(), "\"NIA\"");
                    found_ticker = true;
                }
                "precision" => {
                    assert_eq!(state.data.verified.to_string(), "centiMilli");
                    found_precision = true;
                }
                "circulating" => {
                    let supply = state.data.verified.unwrap_uint::<u64>();
                    assert_eq!(supply, 10_000);
                    found_circulating = true;
                }
                _ => (),
            }
        }
    }
    assert!(found_name, "Name field not found in global state");
    assert!(found_ticker, "Ticker field not found in global state");
    assert!(found_precision, "Precision field not found");
    assert!(found_circulating, "Circulating supply field not found");

    let owned_states = state
        .owned
        .get("amount")
        .expect("Owned state should exist")
        .clone();

    let mut found = false;
    for (_, state) in owned_states.iter() {
        let utxo_match = state.assignment.seal.primary == utxo;
        if utxo_match && state.assignment.data.unwrap_num().unwrap_uint::<u64>() == 10_000 {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "The specified UTXO with allocation of 10,000 not found in owned states"
    );
}

#[apply(descriptor_and_close_method)]
fn issue_cfa(#[case] wallet_desc: DescriptorType) {
    use utils::runtime::TestRuntime;

    initialize();

    let mut wallet = TestRuntime::new(&wallet_desc, "issuer");

    let utxo = wallet.get_utxo(Some(10_000));

    let contract_id = wallet.issue_cfa("DemoCFA", 10_000, utxo.clone());

    wallet.check_allocations(contract_id, AS::Cfa, vec![10_000], true);

    let articles = wallet.contracts.contract_articles(contract_id);
    assert_eq!(articles.issue.meta.name.to_string(), "DemoCFA");

    let mut found_name = false;
    let mut found_precision = false;
    let mut found_circulating = false;

    let state = wallet.contracts.contract_state(contract_id);
    let imm_state = &state.immutable;
    for (name, map) in imm_state {
        for (_, state) in map {
            match name.as_str() {
                "name" => {
                    assert_eq!(state.data.verified.to_string(), "\"DemoCFA\"");
                    found_name = true;
                }
                "precision" => {
                    assert_eq!(state.data.verified.to_string(), "centiMilli");
                    found_precision = true;
                }
                "circulating" => {
                    let supply = state.data.verified.unwrap_uint::<u64>();
                    assert_eq!(supply, 10_000);
                    found_circulating = true;
                }
                _ => (),
            }
        }
    }
    assert!(found_name, "Name field not found in global state");
    assert!(found_precision, "Precision field not found in global state");
    assert!(found_circulating, "Circulating supply field not found");

    let owned_states = state
        .owned
        .get("amount")
        .expect("Owned state should exist")
        .clone();

    let mut found = false;

    for (_, state) in owned_states.iter() {
        let utxo_match = state.assignment.seal.primary == utxo;
        if utxo_match && state.assignment.data.unwrap_num().unwrap_uint::<u64>() == 10_000 {
            found = true;
            break;
        }
    }

    assert!(
        found,
        "The specified UTXO with assignment of 10,000 not found in owned states"
    );
}

#[apply(descriptor_and_close_method)]
fn issue_cfa_multiple_utxos(#[case] wallet_desc: DescriptorType) {
    use utils::runtime::TestRuntime;

    initialize();

    let mut wallet = TestRuntime::new(&wallet_desc, "issuer");

    // Generate 3 UTXOs
    let utxo1 = wallet.get_utxo(Some(10_000));
    let utxo2 = wallet.get_utxo(Some(10_000));
    let utxo3 = wallet.get_utxo(Some(10_000));

    // Define allocations
    let allocations = vec![(utxo1, 222), (utxo2, 444), (utxo3, 333)];

    // Issue CFA with multiple allocations
    let contract_id = wallet.issue_cfa_with_allocations("MultiCFA", allocations);

    // Verify total supply and allocations
    wallet.check_allocations(contract_id, AS::Cfa, vec![222, 444, 333], true);

    let state = wallet.contracts.contract_state(contract_id);
    let imm_state = &state.immutable;

    // Check global circulating supply
    let circulating = imm_state
        .get("circulating")
        .unwrap()
        .values()
        .next()
        .unwrap()
        .data
        .verified
        .unwrap_uint::<u64>();
    assert_eq!(circulating, 999);

    // Verify allocation amounts
    let owned_states = state.owned.get("amount").unwrap();
    let amounts: Vec<u64> = owned_states
        .iter()
        .map(|(_, state)| state.assignment.data.unwrap_num().unwrap_uint::<u64>())
        .collect();
    let mut sorted_amounts = amounts.clone();
    sorted_amounts.sort();
    assert_eq!(sorted_amounts, vec![222, 333, 444]);
}

#[apply(descriptor_and_close_method)]
fn issue_nia_multiple_utxos(#[case] wallet_desc: DescriptorType) {
    use utils::runtime::TestRuntime;

    initialize();

    let mut wallet = TestRuntime::new(&wallet_desc, "issuer");

    // Generate 3 UTXOs
    let utxo1 = wallet.get_utxo(Some(10_000));
    let utxo2 = wallet.get_utxo(Some(10_000));
    let utxo3 = wallet.get_utxo(Some(10_000));

    // Define allocations
    let allocations = vec![(utxo1, 333), (utxo2, 333), (utxo3, 333)];

    // Issue NIA with multiple allocations
    let contract_id = wallet.issue_nia_with_allocations("MultiNIA", allocations);

    // Verify total supply and allocations
    wallet.check_allocations(contract_id, AS::Nia, vec![333, 333, 333], false);

    let state = wallet.contracts.contract_state(contract_id);
    let imm_state = &state.immutable;

    // Check global circulating supply
    let circulating = imm_state
        .get("circulating")
        .unwrap()
        .values()
        .next()
        .unwrap()
        .data
        .verified
        .unwrap_uint::<u64>();
    assert_eq!(circulating, 999);

    // Verify allocation amounts
    let owned_states = state.owned.get("amount").unwrap();
    let amounts: Vec<u64> = owned_states
        .iter()
        .map(|(_, state)| state.assignment.data.unwrap_num().unwrap_uint::<u64>())
        .collect();
    assert_eq!(amounts, vec![333, 333, 333]);
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
