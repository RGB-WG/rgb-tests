pub mod utils;

use rstest_reuse::{self, *};
use utils::*;

const MEDIA_FPATH: &str = "tests/fixtures/rgb_logo.jpeg";

#[template]
#[rstest]
#[case(DescriptorType::Wpkh, CloseMethod::OpretFirst)]
#[case(DescriptorType::Wpkh, CloseMethod::TapretFirst)]
#[case(DescriptorType::Tr, CloseMethod::OpretFirst)]
#[case(DescriptorType::Tr, CloseMethod::TapretFirst)]
fn descriptor_and_close_method(
    #[case] wallet_desc: DescriptorType,
    #[case] close_method: CloseMethod,
) {
}

#[apply(descriptor_and_close_method)]
fn issue_nia(wallet_desc: DescriptorType, close_method: CloseMethod) {
    println!("wallet_desc {wallet_desc:?} close_method {close_method:?}");

    initialize();

    let mut wallet = get_wallet(&wallet_desc);

    let issued_supply = 999;
    let ticker = "TCKR";
    let name = "asset name";
    let precision = 2;
    let details = Some("some details");
    let terms_text = "Ricardian contract";
    let terms_media_fpath = Some(MEDIA_FPATH);
    let asset_info = AssetInfo::nia(
        ticker,
        name,
        precision,
        details,
        terms_text,
        terms_media_fpath,
        issued_supply,
    );
    let (contract_id, iface_type_name) = wallet.issue_with_info(asset_info, close_method, None);

    let contract_iface = wallet.contract_iface(contract_id, &iface_type_name);
    let contract = wallet.contract_iface_class::<Rgb20>(contract_id);
    let spec = contract.spec();
    assert_eq!(spec.ticker.to_string(), ticker.to_string());
    assert_eq!(spec.name.to_string(), name.to_string());
    assert_eq!(spec.precision.decimals(), precision);
    let terms = contract.contract_terms();
    assert_eq!(terms.text.to_string(), terms_text.to_string());
    let terms_media = terms.media.unwrap();
    assert_eq!(terms_media.ty.to_string(), "image/jpeg");
    assert_eq!(
        terms_media.digest.to_string(),
        "02d2cc5d7883885bb7472e4fe96a07344b1d7cf794cb06943e1cdb5c57754d8a"
    );
    assert_eq!(contract.total_issued_supply().value(), issued_supply);

    let allocations = wallet.contract_fungible_allocations(&contract_iface);
    assert_eq!(allocations.len(), 1);
    let allocation = allocations[0];
    assert_eq!(allocation.seal.method(), close_method);
    assert_eq!(allocation.state, Amount::from(issued_supply));
}

#[apply(descriptor_and_close_method)]
fn issue_uda(wallet_desc: DescriptorType, close_method: CloseMethod) {
    println!("wallet_desc {wallet_desc:?} close_method {close_method:?}");

    initialize();

    let mut wallet = get_wallet(&wallet_desc);

    let ticker = "TCKR";
    let name = "asset name";
    let details = Some("some details");
    let terms_text = "Ricardian contract";
    let terms_media_fpath = Some(MEDIA_FPATH);
    let data = vec![1u8, 3u8, 9u8];
    let preview_ty = "image/jpeg";
    let token_data_preview = EmbeddedMedia {
        ty: MediaType::with(preview_ty),
        data: Confined::try_from(data.clone()).unwrap(),
    };
    let proof = vec![2u8, 4u8, 6u8, 10u8];
    let token_data_reserves = ProofOfReserves {
        utxo: Outpoint::from_str(FAKE_TXID).unwrap(),
        proof: Confined::try_from(proof.clone()).unwrap(),
    };
    let token_data_ticker = "TDTCKR";
    let token_data_name = "token data name";
    let token_data_details = "token data details";
    let token_data_attachment = attachment_from_fpath(MEDIA_FPATH);
    let mut token_data_attachments = BTreeMap::new();
    for (idx, attachment_fpath) in ["README.md", "Cargo.toml"].iter().enumerate() {
        token_data_attachments.insert(idx as u8, attachment_from_fpath(attachment_fpath));
    }
    let token_data = uda_token_data(
        token_data_ticker,
        token_data_name,
        token_data_details,
        token_data_preview.clone(),
        token_data_attachment.clone(),
        token_data_attachments.clone(),
        token_data_reserves.clone(),
    );
    let asset_info = AssetInfo::uda(
        ticker,
        name,
        details,
        terms_text,
        terms_media_fpath,
        token_data,
    );
    let (contract_id, iface_type_name) = wallet.issue_with_info(asset_info, close_method, None);

    let contract_iface = wallet.contract_iface(contract_id, &iface_type_name);
    let contract = wallet.contract_iface_class::<Rgb21>(contract_id);
    let spec = contract.spec();
    assert_eq!(spec.ticker.to_string(), ticker.to_string());
    assert_eq!(spec.name.to_string(), name.to_string());
    assert_eq!(spec.precision.decimals(), 0);
    let terms = contract.contract_terms();
    assert_eq!(terms.text.to_string(), terms_text.to_string());
    let terms_media = terms.media.unwrap();
    assert_eq!(terms_media.ty.to_string(), "image/jpeg");
    assert_eq!(
        terms_media.digest.to_string(),
        "02d2cc5d7883885bb7472e4fe96a07344b1d7cf794cb06943e1cdb5c57754d8a"
    );
    let token_data = contract.token_data();
    assert_eq!(token_data.index, TokenIndex::from_inner(0));
    assert_eq!(token_data.ticker.unwrap().to_string(), token_data_ticker);
    assert_eq!(token_data.name.unwrap().to_string(), token_data_name);
    assert_eq!(token_data.details.unwrap().to_string(), token_data_details);
    assert_eq!(token_data.preview.unwrap(), token_data_preview);
    assert_eq!(token_data.media.unwrap(), token_data_attachment);
    assert_eq!(token_data.attachments.to_unconfined(), token_data_attachments);
    assert_eq!(token_data.reserves.unwrap(), token_data_reserves);

    let allocations = wallet.contract_data_allocations(&contract_iface);
    assert_eq!(allocations.len(), 1);
    let allocation = &allocations[0];
    assert_eq!(allocation.seal.method(), close_method);
    assert_eq!(allocation.state.to_string(), "000000000100000000000000");
}

#[apply(descriptor_and_close_method)]
fn issue_cfa(wallet_desc: DescriptorType, close_method: CloseMethod) {
    println!("wallet_desc {wallet_desc:?} close_method {close_method:?}");

    initialize();

    let mut wallet = get_wallet(&wallet_desc);

    let issued_supply = 999;
    let name = "asset name";
    let precision = 2;
    let details = Some("some details");
    let terms_text = "Ricardian contract";
    let terms_media_fpath = Some(MEDIA_FPATH);
    let asset_info = AssetInfo::cfa(
        name,
        precision,
        details,
        terms_text,
        terms_media_fpath,
        issued_supply,
    );
    let (contract_id, iface_type_name) = wallet.issue_with_info(asset_info, close_method, None);

    let contract_iface = wallet.contract_iface(contract_id, &iface_type_name);
    let contract = wallet.contract_iface_class::<Rgb25>(contract_id);
    assert_eq!(contract.name().to_string(), name.to_string());
    assert_eq!(
        contract.details().map(|d| d.to_string()),
        details.map(|d| d.to_string())
    );
    assert_eq!(contract.precision().decimals(), precision);
    let terms = contract.contract_terms();
    assert_eq!(terms.text.to_string(), terms_text.to_string());
    let terms_media = terms.media.unwrap();
    assert_eq!(terms_media.ty.to_string(), "image/jpeg");
    assert_eq!(
        terms_media.digest.to_string(),
        "02d2cc5d7883885bb7472e4fe96a07344b1d7cf794cb06943e1cdb5c57754d8a"
    );
    assert_eq!(contract.total_issued_supply().value(), issued_supply);

    let allocations = wallet.contract_fungible_allocations(&contract_iface);
    assert_eq!(allocations.len(), 1);
    let allocation = allocations[0];
    assert_eq!(allocation.seal.method(), close_method);
    assert_eq!(allocation.state, Amount::from(issued_supply));
}
