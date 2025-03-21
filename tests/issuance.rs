pub mod utils;

use rstest_reuse::{self, *};
use utils::*;

const MEDIA_FPATH: &str = "tests/fixtures/rgb_logo.jpeg";

#[template]
#[rstest]
#[case(DescriptorType::Wpkh)]
#[case(DescriptorType::Tr)]
fn descriptor(#[case] wallet_desc: DescriptorType) {}

#[cfg(not(feature = "altered"))]
#[apply(descriptor)]
fn issue_nia(wallet_desc: DescriptorType) {
    println!("wallet_desc {wallet_desc:?}");

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
        vec![issued_supply],
    );
    let contract_id = wallet.issue_with_info(asset_info, vec![]);

    let contract = wallet.contract_wrapper::<NonInflatableAsset>(contract_id);
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

    let allocations = wallet.contract_fungible_allocations(contract_id, false);
    assert_eq!(allocations.len(), 1);
    let allocation = allocations[0];
    assert_eq!(allocation.state, Amount::from(issued_supply));
}

#[cfg(not(feature = "altered"))]
#[apply(descriptor)]
fn issue_uda(wallet_desc: DescriptorType) {
    println!("wallet_desc {wallet_desc:?}");

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
    let contract_id = wallet.issue_with_info(asset_info, vec![]);

    let contract = wallet.contract_wrapper::<UniqueDigitalAsset>(contract_id);
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
    assert_eq!(token_data.index, TokenIndex::from(0));
    assert_eq!(token_data.ticker.unwrap().to_string(), token_data_ticker);
    assert_eq!(token_data.name.unwrap().to_string(), token_data_name);
    assert_eq!(token_data.details.unwrap().to_string(), token_data_details);
    assert_eq!(token_data.preview.unwrap(), token_data_preview);
    assert_eq!(token_data.media.unwrap(), token_data_attachment);
    assert_eq!(
        token_data.attachments.to_unconfined(),
        token_data_attachments
    );
    assert_eq!(token_data.reserves.unwrap(), token_data_reserves);

    let allocations = wallet.contract_data_allocations(contract_id);
    assert_eq!(allocations.len(), 1);
    let allocation = &allocations[0];
    assert_eq!(allocation.state.to_string(), "000000000100000000000000");
}

#[cfg(not(feature = "altered"))]
#[apply(descriptor)]
fn issue_cfa(wallet_desc: DescriptorType) {
    println!("wallet_desc {wallet_desc:?}");

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
        vec![issued_supply],
    );
    let contract_id = wallet.issue_with_info(asset_info, vec![]);

    let contract = wallet.contract_wrapper::<CollectibleFungibleAsset>(contract_id);
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

    let allocations = wallet.contract_fungible_allocations(contract_id, false);
    assert_eq!(allocations.len(), 1);
    let allocation = allocations[0];
    assert_eq!(allocation.state, Amount::from(issued_supply));
}

#[cfg(not(feature = "altered"))]
#[apply(descriptor)]
fn issue_nia_multiple_utxos(wallet_desc: DescriptorType) {
    println!("wallet_desc {wallet_desc:?}");

    initialize();

    let mut wallet = get_wallet(&wallet_desc);

    let amounts = vec![222, 444, 333];
    let outpoints: Vec<_> = (0..amounts.len())
        .map(|_| Some(wallet.get_utxo(None)))
        .collect();
    let asset_info = AssetInfo::default_nia(amounts.clone());
    let contract_id = wallet.issue_with_info(asset_info, outpoints.clone());

    let contract = wallet.contract_wrapper::<NonInflatableAsset>(contract_id);
    assert_eq!(
        contract.total_issued_supply().value(),
        amounts.iter().sum::<u64>()
    );

    let allocations = wallet.contract_fungible_allocations(contract_id, false);
    assert_eq!(allocations.len(), amounts.len());
    for (amt, outpoint) in amounts.iter().zip(outpoints.into_iter()) {
        assert!(allocations.iter().any(|a| a.state == Amount::from(*amt)
            && a.seal
                == ExplicitSeal {
                    txid: outpoint.unwrap().txid,
                    vout: outpoint.unwrap().vout
                }))
    }
}

#[cfg(not(feature = "altered"))]
#[apply(descriptor)]
fn issue_cfa_multiple_utxos(wallet_desc: DescriptorType) {
    println!("wallet_desc {wallet_desc:?}");

    initialize();

    let mut wallet = get_wallet(&wallet_desc);

    let amounts = vec![222, 444, 333];
    let outpoints: Vec<_> = (0..amounts.len())
        .map(|_| Some(wallet.get_utxo(None)))
        .collect();
    let asset_info = AssetInfo::default_cfa(amounts.clone());
    let contract_id = wallet.issue_with_info(asset_info, outpoints.clone());

    let contract = wallet.contract_wrapper::<CollectibleFungibleAsset>(contract_id);
    assert_eq!(
        contract.total_issued_supply().value(),
        amounts.iter().sum::<u64>()
    );

    let allocations = wallet.contract_fungible_allocations(contract_id, false);
    assert_eq!(allocations.len(), amounts.len());
    for (amt, outpoint) in amounts.iter().zip(outpoints.into_iter()) {
        assert!(allocations.iter().any(|a| a.state == Amount::from(*amt)
            && a.seal
                == ExplicitSeal {
                    txid: outpoint.unwrap().txid,
                    vout: outpoint.unwrap().vout
                }))
    }
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[should_panic(expected = "InvoiceBeneficiaryWrongChainNet(BitcoinRegtest, LiquidTestnet)")]
#[case("standard_invoice")]
#[should_panic(expected = "NetworkMismatch")]
#[case("liquid_testnet_invoice")]
#[should_panic(expected = "ContractChainNetMismatch(BitcoinMainnet)")]
#[case("liquid_mainnet_invoice")]
fn issue_on_different_layers(#[case] scenario: &str) {
    initialize();

    let mut wlt_1 = if scenario == "liquid_mainnet_invoice" {
        get_mainnet_wallet()
    } else {
        get_wallet(&DescriptorType::Wpkh)
    };

    let issued_amt = 100;
    let amounts = vec![issued_amt];
    let asset_info = AssetInfo::default_nia(amounts.clone());
    let contract_chainnet = if scenario == "liquid_mainnet_invoice" {
        ChainNet::LiquidMainnet
    } else {
        ChainNet::LiquidTestnet
    };
    let mut builder = ContractBuilder::with(
        Identity::default(),
        asset_info.schema(),
        asset_info.types(),
        asset_info.scripts(),
        contract_chainnet,
    );

    builder = asset_info.add_global_state(builder);

    let outpoint = if scenario == "liquid_mainnet_invoice" {
        Outpoint::from_str("bebcfcb200a17763f6932a6d6fca9448a4b46c5b737cc3810769a7403ef79ce6:0")
            .unwrap()
    } else {
        wlt_1.get_utxo(None)
    };

    builder = builder
        .add_fungible_state("assetOwner", get_builder_seal(outpoint), amounts[0])
        .unwrap();

    let contract = builder.issue_contract().expect("failure issuing contract");
    let resolver = wlt_1.get_resolver();
    wlt_1.import_contract(&contract, resolver);

    let mut wlt_2 = if scenario == "liquid_mainnet_invoice" {
        get_mainnet_wallet()
    } else {
        get_wallet(&DescriptorType::Wpkh)
    };
    let contract_id = contract.contract_id();
    let amt = 60;
    let sats = 1000;

    match scenario {
        "standard_invoice" => {
            wlt_1.send(
                &mut wlt_2,
                TransferType::Witness,
                contract_id,
                amt,
                sats,
                None,
            );
        }
        "liquid_testnet_invoice" => {
            let address = wlt_2.get_address();
            let beneficiary = Beneficiary::WitnessVout(Pay2Vout::new(address.payload), None);
            let builder = RgbInvoiceBuilder::new(XChainNet::LiquidTestnet(beneficiary))
                .set_contract(contract_id)
                .set_amount_raw(amt);
            let invoice = builder.finish();
            wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(sats), None, None);
        }
        "liquid_mainnet_invoice" => {
            let address = wlt_2.get_address();
            let beneficiary = Beneficiary::WitnessVout(Pay2Vout::new(address.payload), None);
            let builder = RgbInvoiceBuilder::new(XChainNet::LiquidMainnet(beneficiary))
                .set_contract(contract_id)
                .set_amount_raw(issued_amt);
            let invoice = builder.finish();
            let (_, _, consignment) = wlt_1.pay(invoice, Some(500), Some(100));
            wlt_2.accept_transfer(consignment.clone(), None);
        }
        _ => unreachable!(),
    }
}
