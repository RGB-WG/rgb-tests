pub mod utils;

use utils::*;

type TT = TransferType;
type DT = DescriptorType;
type AS = AssetSchema;

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
// blinded: nia - uda
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Uda)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Nia, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Nia, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Nia, AS::Uda)]
// blinded: cfa - cfa
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Cfa)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Cfa, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Cfa, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Cfa, AS::Cfa)]
// blinded: cfa - nia
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Nia)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Cfa, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Cfa, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Cfa, AS::Nia)]
// blinded: cfa - uda
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Uda)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Cfa, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Cfa, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Cfa, AS::Uda)]
// blinded: uda - uda
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Uda)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Uda, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Uda, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Uda, AS::Uda)]
// blinded: uda - nia
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Nia)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Uda, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Uda, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Uda, AS::Nia)]
// blinded: uda - cfa
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Cfa)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Uda, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Uda, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Uda, AS::Cfa)]
// witness: nia - nia
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Nia)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Nia, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Nia, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Nia, AS::Nia)]
// witness: nia - cfa
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Cfa)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Nia, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Nia, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Nia, AS::Cfa)]
// witness: nia - uda
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Uda)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Nia, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Nia, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Nia, AS::Uda)]
// witness: cfa - cfa
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Cfa)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Cfa, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Cfa, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Cfa, AS::Cfa)]
// witness: cfa - nia
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Nia)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Cfa, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Cfa, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Cfa, AS::Nia)]
// witness: cfa - uda
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Uda)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Cfa, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Cfa, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Cfa, AS::Uda)]
// witness: uda - uda
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Uda)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Uda, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Uda, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Uda, AS::Uda)]
// witness: uda - nia
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Nia)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Uda, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Uda, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Uda, AS::Nia)]
// witness: uda - cfa
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Cfa)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Uda, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Uda, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Uda, AS::Cfa)]
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
    let (contract_id_1, iface_type_name_1) = match asset_schema_1 {
        AssetSchema::Nia => wlt_1.issue_nia(issued_supply_1, wlt_1.close_method(), Some(&utxo)),
        AssetSchema::Uda => wlt_1.issue_uda(wlt_1.close_method(), Some(&utxo)),
        AssetSchema::Cfa => wlt_1.issue_cfa(issued_supply_1, wlt_1.close_method(), Some(&utxo)),
    };
    let (contract_id_2, iface_type_name_2) = match asset_schema_2 {
        AssetSchema::Nia => wlt_1.issue_nia(issued_supply_2, wlt_1.close_method(), Some(&utxo)),
        AssetSchema::Uda => wlt_1.issue_uda(wlt_1.close_method(), Some(&utxo)),
        AssetSchema::Cfa => wlt_1.issue_cfa(issued_supply_2, wlt_1.close_method(), Some(&utxo)),
    };
    wlt_1.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![issued_supply_1],
        true,
    );
    wlt_1.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![issued_supply_2],
        true,
    );

    // wlt_1 spends asset 1, moving the other with a blank transition
    let amount_1 = if asset_schema_1 == AssetSchema::Uda {
        1
    } else {
        99
    };
    wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_1,
        &iface_type_name_1,
        amount_1,
        sats,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1],
        false,
    );
    wlt_1.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![issued_supply_2],
        true,
    );
    wlt_2.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![amount_1],
        true,
    );

    // wlt_1 spends asset 1 change (only if possible)
    let amount_2 = 33;
    if asset_schema_1 != AssetSchema::Uda {
        wlt_1.send(
            &mut wlt_2,
            transfer_type,
            contract_id_1,
            &iface_type_name_1,
            amount_2,
            sats,
            None,
        );
        wlt_1.check_allocations(
            contract_id_1,
            &iface_type_name_1,
            asset_schema_1,
            vec![issued_supply_1 - amount_1 - amount_2],
            false,
        );
        wlt_1.check_allocations(
            contract_id_2,
            &iface_type_name_2,
            asset_schema_2,
            vec![issued_supply_2],
            true,
        );
        wlt_2.check_allocations(
            contract_id_1,
            &iface_type_name_1,
            asset_schema_1,
            vec![amount_1, amount_2],
            true,
        );
    }

    // wlt_1 spends asset 2
    let amount_3 = if asset_schema_2 == AssetSchema::Uda {
        1
    } else {
        22
    };
    wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_2,
        &iface_type_name_2,
        amount_3,
        sats,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2],
        false,
    );
    wlt_1.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3],
        false,
    );
    wlt_2.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![amount_1, amount_2],
        true,
    );
    wlt_2.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![amount_3],
        true,
    );

    // wlt_2 spends received allocation(s) of asset 1
    let amount_4 = if asset_schema_1 == AssetSchema::Uda {
        1
    } else {
        111
    };
    sats -= 1000;
    wlt_2.send(
        &mut wlt_1,
        transfer_type,
        contract_id_1,
        &iface_type_name_1,
        amount_4,
        sats,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2, amount_4],
        true,
    );
    wlt_1.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3],
        false,
    );
    wlt_2.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4],
        false,
    );
    wlt_2.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![amount_3],
        true,
    );

    // wlt_2 spends asset 2
    let amount_5 = if asset_schema_2 == AssetSchema::Uda {
        1
    } else {
        11
    };
    sats -= 1000;
    wlt_2.send(
        &mut wlt_1,
        transfer_type,
        contract_id_2,
        &iface_type_name_2,
        amount_5,
        sats,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2, amount_4],
        true,
    );
    wlt_1.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3, amount_5],
        true,
    );
    wlt_2.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4],
        false,
    );
    wlt_2.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![amount_3 - amount_5],
        false,
    );

    // wlt_1 spends asset 1, received back
    let amount_6 = if asset_schema_1 == AssetSchema::Uda {
        1
    } else {
        issued_supply_1 - amount_1 - amount_2 + amount_4
    };
    sats -= 1000;
    wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_1,
        &iface_type_name_1,
        amount_6,
        sats,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![],
        false,
    );
    wlt_1.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3, amount_5],
        true,
    );
    wlt_2.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4, amount_6],
        true,
    );
    wlt_2.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![amount_3 - amount_5],
        false,
    );

    // wlt_1 spends asset 2, received back
    let amount_7 = if asset_schema_2 == AssetSchema::Uda {
        1
    } else {
        issued_supply_2 - amount_3 + amount_5
    };
    sats -= 1000;
    wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_2,
        &iface_type_name_2,
        amount_7,
        sats,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![],
        false,
    );
    wlt_1.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![],
        false,
    );
    wlt_2.check_allocations(
        contract_id_1,
        &iface_type_name_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4, amount_6],
        true,
    );
    wlt_2.check_allocations(
        contract_id_2,
        &iface_type_name_2,
        asset_schema_2,
        vec![amount_3 - amount_5, amount_7],
        true,
    );
}

#[test]
fn same_transfer_twice() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let amount = 600;

    let (contract_id, iface_type_name) = wlt_1.issue_nia(amount, wlt_1.close_method(), None);

    stop_mining();
    let initial_height = get_height();

    let invoice = wlt_2.invoice(
        contract_id,
        &iface_type_name,
        amount,
        wlt_2.close_method(),
        InvoiceType::Witness,
    );
    let _ = wlt_1.transfer(invoice.clone(), None, Some(500), true, None);

    // retry with higher fees, TX hasn't been mined
    let mid_height = get_height();
    assert_eq!(initial_height, mid_height);

    let _ = wlt_1.transfer(invoice, None, Some(1000), true, None);

    let final_height = get_height();
    assert_eq!(initial_height, final_height);
    resume_mining();
}

#[test]
fn accept_0conf() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 600;
    let (contract_id, iface_type_name) = wlt_1.issue_nia(issue_supply, wlt_1.close_method(), None);

    let amt = 200;
    let invoice = wlt_2.invoice(
        contract_id,
        &iface_type_name,
        amt,
        wlt_2.close_method(),
        InvoiceType::Witness,
    );
    let (consignment, _) = wlt_1.transfer(invoice.clone(), None, None, true, None);

    wlt_2.accept_transfer(consignment.clone(), None);

    // TODO: check if it's correct that sender sees 2 allocations
    /*
    wlt_1.sync();
    wlt_1.check_allocations(
        contract_id,
        &iface_type_name,
        AssetSchema::Nia,
        vec![issue_supply - amt],
        false,
    );
    */
    wlt_2.check_allocations(
        contract_id,
        &iface_type_name,
        AssetSchema::Nia,
        vec![amt],
        false,
    );
}

#[test]
fn ln_transfers() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let wlt_2 = get_wallet(&DescriptorType::Wpkh);
    let pre_funding_height = get_height();

    let utxo = wlt_1.get_utxo(Some(10_000));
    let (contract_id, iface_type_name) = wlt_1.issue_nia(600, wlt_1.close_method(), Some(&utxo));

    println!("\n1. fake commitment TX (no HTLCs)");
    let beneficiaries = vec![
        (wlt_2.get_address(), Some(2000)),
        (wlt_1.get_address(), None),
    ];
    let (mut psbt, _meta) = wlt_1.construct_psbt(vec![utxo], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                iface: iface_type_name.clone(),
                input_outpoints: vec![utxo],
                output_map: HashMap::from([(0, 100), (1, 500)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info.clone());
    wlt_1.consume_fascia(fascia.clone(), psbt.txid());
    wlt_1.debug_logs(contract_id, &iface_type_name, AllocationFilter::WalletAll);

    let htlc_vout = 2;
    let htlc_rgb_amt = 200;
    let htlc_btc_amt = 4000;
    let htlc_derived_addr = wlt_1.get_derived_address();

    println!("\n2. fake commitment TX (1 HTLC)");
    let beneficiaries = vec![
        (wlt_2.get_address(), Some(2000)),
        (wlt_1.get_address(), None),
        (htlc_derived_addr.addr, Some(htlc_btc_amt)),
    ];
    let (mut psbt, _meta) = wlt_1.construct_psbt(vec![utxo], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                iface: iface_type_name.clone(),
                input_outpoints: vec![utxo],
                output_map: HashMap::from([(0, 100), (1, 300), (htlc_vout, htlc_rgb_amt)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info);
    wlt_1.consume_fascia(fascia.clone(), psbt.txid());
    wlt_1.debug_logs(contract_id, &iface_type_name, AllocationFilter::WalletAll);

    println!("\n3. fake HTLC TX");
    let witness_id = fascia.witness_id();
    let txid = witness_id.as_reduced_unsafe();
    let input_outpoint = Outpoint::new(*txid, htlc_vout);
    let beneficiaries = vec![(wlt_1.get_address(), None)];
    let (mut psbt, _meta) = wlt_1.construct_psbt_offchain(
        vec![(input_outpoint, htlc_btc_amt, htlc_derived_addr.terminal)],
        beneficiaries,
        None,
    );
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                iface: iface_type_name.clone(),
                input_outpoints: vec![input_outpoint],
                output_map: HashMap::from([(0, htlc_rgb_amt)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info);
    wlt_1.consume_fascia(fascia.clone(), psbt.txid());
    wlt_1.debug_logs(contract_id, &iface_type_name, AllocationFilter::WalletAll);

    println!("\n4. fake commitment TX (no HTLCs)");
    let beneficiaries = vec![
        (wlt_2.get_address(), Some(3000)),
        (wlt_1.get_address(), None),
    ];
    let (mut psbt, _meta) = wlt_1.construct_psbt(vec![utxo], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                iface: iface_type_name.clone(),
                input_outpoints: vec![utxo],
                output_map: HashMap::from([(0, 100), (1, 500)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info);
    wlt_1.consume_fascia(fascia.clone(), psbt.txid());
    wlt_1.debug_logs(contract_id, &iface_type_name, AllocationFilter::WalletAll);
    let mut old_psbt = psbt.clone();

    println!("\n5. fake commitment TX (1 HTLC)");
    let htlc_rgb_amt = 180;
    let beneficiaries = vec![
        (wlt_2.get_address(), Some(2000)),
        (wlt_1.get_address(), None),
        (htlc_derived_addr.addr, Some(htlc_btc_amt)),
    ];
    let (mut psbt, _meta) = wlt_1.construct_psbt(vec![utxo], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                iface: iface_type_name.clone(),
                input_outpoints: vec![utxo],
                output_map: HashMap::from([(0, 122), (1, 298), (htlc_vout, htlc_rgb_amt)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info.clone());
    wlt_1.consume_fascia(fascia.clone(), psbt.txid());
    wlt_1.debug_logs(contract_id, &iface_type_name, AllocationFilter::WalletAll);

    println!("\n6. fake HTLC TX");
    let witness_id = fascia.witness_id();
    let txid = witness_id.as_reduced_unsafe();
    let input_outpoint = Outpoint::new(*txid, htlc_vout);
    let beneficiaries = vec![(wlt_1.get_address(), None)];
    let (mut psbt, _meta) = wlt_1.construct_psbt_offchain(
        vec![(input_outpoint, htlc_btc_amt, htlc_derived_addr.terminal)],
        beneficiaries,
        None,
    );
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                iface: iface_type_name.clone(),
                input_outpoints: vec![input_outpoint],
                output_map: HashMap::from([(0, htlc_rgb_amt)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info);
    wlt_1.consume_fascia(fascia.clone(), psbt.txid());
    wlt_1.debug_logs(contract_id, &iface_type_name, AllocationFilter::WalletAll);

    println!("\n7. broadcast old PSBT");
    let tx = wlt_1.sign_finalize_extract(&mut old_psbt);
    wlt_1.broadcast_tx(&tx);
    wlt_1.mine_tx(&tx.txid(), false);
    wlt_1.sync();
    wlt_1.update_witnesses(pre_funding_height);
    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);
    wlt_1.send(
        &mut wlt_3,
        TransferType::Blinded,
        contract_id,
        &iface_type_name,
        500,
        1000,
        None,
    );
}

#[test]
fn mainnet_wlt_receiving_test_asset() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_mainnet_wallet();

    let (contract_id, iface_type_name) = wlt_1.issue_nia(700, wlt_1.close_method(), None);

    let utxo =
        Outpoint::from_str("bebcfcb200a17763f6932a6d6fca9448a4b46c5b737cc3810769a7403ef79ce6:0")
            .unwrap();
    let invoice = wlt_2.invoice(
        contract_id,
        &iface_type_name,
        150,
        wlt_2.close_method(),
        InvoiceType::Blinded(Some(utxo)),
    );
    let (consignment, tx) = wlt_1.transfer(invoice.clone(), None, Some(500), true, None);
    wlt_1.mine_tx(&tx.txid(), false);
    match consignment.validate(&wlt_2.get_resolver(), wlt_2.testnet()) {
        Err((status, _invalid_consignment)) => {
            assert_eq!(
                status.failures,
                vec![Failure::NetworkMismatch(wlt_2.testnet())]
            )
        }
        _ => panic!("validation must fail"),
    }
}

#[test]
fn tapret_wlt_receiving_opret() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Tr);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let (contract_id, iface_type_name) = wlt_1.issue_nia(600, wlt_1.close_method(), None);

    println!("1st transfer");
    wlt_1.send(
        &mut wlt_2,
        TransferType::Blinded,
        contract_id,
        &iface_type_name,
        400,
        5000,
        None,
    );

    println!("2nd transfer");
    let invoice = wlt_1.invoice(
        contract_id,
        &iface_type_name,
        100,
        CloseMethod::OpretFirst,
        InvoiceType::Witness,
    );
    wlt_2.send_to_invoice(&mut wlt_1, invoice, None, None, None);

    println!("3rd transfer");
    wlt_1.send(
        &mut wlt_2,
        TransferType::Blinded,
        contract_id,
        &iface_type_name,
        290,
        1000,
        None,
    );

    println!("4th transfer");
    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id,
        &iface_type_name,
        560,
        1000,
        None,
    );

    println!("5th transfer");
    wlt_1.send(
        &mut wlt_2,
        TransferType::Blinded,
        contract_id,
        &iface_type_name,
        570,
        1000,
        None,
    );
}

#[test]
fn collaborative_transfer() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);

    let sats = 30_000;

    let utxo_0 = wlt_1.get_utxo(Some(sats));
    let (contract_id, iface_type_name) = wlt_1.issue_nia(600, wlt_1.close_method(), Some(&utxo_0));
    let (_, tx) = wlt_1.send(
        &mut wlt_2,
        TransferType::Witness,
        contract_id,
        &iface_type_name,
        200,
        18_000,
        None,
    );
    let utxo_1 = Outpoint::new(tx.txid(), 1); // change: 11_600 sat
    let utxo_2 = Outpoint::new(tx.txid(), 0); // transfered: 18_000 sat

    let mut psbt = Psbt::default();

    wlt_1.psbt_add_input(&mut psbt, utxo_1);
    wlt_2.psbt_add_input(&mut psbt, utxo_2);

    psbt.construct_output_expect(
        wlt_3.get_address().script_pubkey(),
        Sats::from_sats(sats - 2 * DEFAULT_FEE_ABS),
    );

    let coloring_info_1 = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                iface: iface_type_name.clone(),
                input_outpoints: vec![utxo_1],
                output_map: HashMap::from([(0, 400)]),
                static_blinding: None,
            },
        )]),
        static_blinding: None,
        nonce: None,
    };
    let coloring_info_2 = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                iface: iface_type_name.clone(),
                input_outpoints: vec![utxo_2],
                output_map: HashMap::from([(0, 200)]),
                static_blinding: None,
            },
        )]),
        static_blinding: None,
        nonce: None,
    };
    let beneficiaries_1 = wlt_1.color_psbt_init(&mut psbt, coloring_info_1);

    let (fascia, beneficiaries_2) = wlt_2.color_psbt(&mut psbt, coloring_info_2);

    wlt_1.sign_finalize(&mut psbt);
    let tx = wlt_2.sign_finalize_extract(&mut psbt);
    wlt_1.broadcast_tx(&tx);

    wlt_1.consume_fascia(fascia.clone(), tx.txid());
    wlt_2.consume_fascia(fascia, tx.txid());

    let consignments_1 = wlt_1.create_consignments(beneficiaries_1, tx.txid());
    let consignments_2 = wlt_2.create_consignments(beneficiaries_2, tx.txid());

    println!("Send the whole asset amount back to wlt_1 to check new allocations are spendable");
    for consignment in vec![consignments_1, consignments_2].into_iter().flatten() {
        wlt_3.accept_transfer(consignment, None);
    }
    wlt_3.send(
        &mut wlt_1,
        TransferType::Witness,
        contract_id,
        &iface_type_name,
        600,
        sats - 4 * DEFAULT_FEE_ABS,
        None,
    );
    wlt_1.send(
        &mut wlt_2,
        TransferType::Witness,
        contract_id,
        &iface_type_name,
        600,
        sats - 6 * DEFAULT_FEE_ABS,
        None,
    );
}

#[test]
fn receive_from_unbroadcasted_transfer_to_blinded() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);

    let (contract_id, iface_type_name) = wlt_1.issue_nia(600, wlt_1.close_method(), None);

    let utxo = wlt_2.get_utxo(None);
    mine(false);
    let invoice = wlt_2.invoice(
        contract_id,
        &iface_type_name,
        100,
        wlt_2.close_method(),
        InvoiceType::Blinded(Some(utxo)),
    );
    // create transfer but do not broadcast its TX
    let (consignment, tx) = wlt_1.transfer(invoice.clone(), None, Some(500), false, None);
    let txid = tx.txid();

    struct OffchainResolver<'a, 'cons, const TRANSFER: bool> {
        witness_id: XWitnessId,
        consignment: &'cons IndexedConsignment<'cons, TRANSFER>,
        fallback: &'a AnyResolver,
    }
    impl<'a, 'cons, const TRANSFER: bool> ResolveWitness for OffchainResolver<'a, 'cons, TRANSFER> {
        fn resolve_pub_witness(
            &self,
            witness_id: XWitnessId,
        ) -> Result<XWitnessTx, WitnessResolverError> {
            self.consignment
                .pub_witness(witness_id)
                .and_then(|p| p.map_ref(|pw| pw.tx().cloned()).transpose())
                .ok_or(WitnessResolverError::Unknown(witness_id))
                .or_else(|_| self.fallback.resolve_pub_witness(witness_id))
        }
        fn resolve_pub_witness_ord(
            &self,
            witness_id: XWitnessId,
        ) -> Result<WitnessOrd, WitnessResolverError> {
            if witness_id != self.witness_id {
                return self.fallback.resolve_pub_witness_ord(witness_id);
            }
            Ok(WitnessOrd::Tentative)
        }
    }

    let resolver = OffchainResolver {
        witness_id: XChain::Bitcoin(txid),
        consignment: &IndexedConsignment::new(&consignment),
        fallback: &wlt_2.get_resolver(),
    };

    // wlt_2 use custom resolver to be able to send the assets even if transfer TX sending to
    // blinded UTXO has not been broadcasted
    wlt_2.accept_transfer_custom_resolver(consignment.clone(), None, &resolver);

    let invoice = wlt_3.invoice(
        contract_id,
        &iface_type_name,
        50,
        wlt_2.close_method(),
        InvoiceType::Witness,
    );
    let (consignment, tx) = wlt_2.transfer(invoice, Some(2000), None, true, None);
    wlt_2.mine_tx(&tx.txid(), false);

    // consignemnt validation fails because it notices an unbroadcasted TX in the history
    let res = consignment.validate(&wlt_3.get_resolver(), wlt_3.testnet());
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err((status, _consignment)) => status,
    };
    assert_eq!(validation_status.failures.len(), 1);
    assert!(matches!(
        validation_status.failures[0],
        Failure::SealNoPubWitness(_, _, _)
    ));
}
