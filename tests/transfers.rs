pub mod utils;

use rstest_reuse::{self, *};
use utils::*;

#[template]
#[rstest]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    TransferType::Blinded,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    TransferType::Blinded,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    TransferType::Blinded,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    TransferType::Blinded,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    TransferType::Witness,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    TransferType::Witness,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    TransferType::Witness,
    AssetSchema::Nia
)]
// TODO: wait for https://github.com/RGB-WG/rgb-std/issues/198 to be fixed
/*#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    TransferType::Witness,
    AssetSchema::Nia
)]*/
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    TransferType::Blinded,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    TransferType::Blinded,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    TransferType::Blinded,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    TransferType::Blinded,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    TransferType::Witness,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    TransferType::Witness,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    TransferType::Witness,
    AssetSchema::Uda
)]
// TODO: wait for https://github.com/RGB-WG/rgb-std/issues/198 to be fixed
/*#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    TransferType::Witness,
    AssetSchema::Uda
)]*/
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    TransferType::Blinded,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    TransferType::Blinded,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    TransferType::Blinded,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    TransferType::Blinded,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    TransferType::Witness,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    TransferType::Witness,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    TransferType::Witness,
    AssetSchema::Cfa
)]
// TODO: wait for https://github.com/RGB-WG/rgb-std/issues/198 to be fixed
/*#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    TransferType::Witness,
    AssetSchema::Cfa
)]*/
fn descriptors_transfer_type_and_asset_schema(
    #[case] wlt_1_desc: DescriptorType,
    #[case] wlt_2_desc: DescriptorType,
    #[case] transfer_type: TransferType,
    #[case] asset_schema: AssetSchema,
) {
}

#[apply(descriptors_transfer_type_and_asset_schema)]
fn send_received(
    wlt_1_desc: DescriptorType,
    wlt_2_desc: DescriptorType,
    transfer_type: TransferType,
    asset_schema: AssetSchema,
) {
    println!("wlt_1_desc {wlt_1_desc:?} wlt_2_desc {wlt_2_desc:?} transfer_type {transfer_type:?} asset_schema {asset_schema:?}");
    initialize();

    let mut wlt_1 = get_wallet(&wlt_1_desc);
    let mut wlt_2 = get_wallet(&wlt_2_desc);

    let issued_supply = 999;
    let amount_1 = 66;

    let (contract_id, iface_type_name) = match asset_schema {
        AssetSchema::Nia => wlt_1.issue_nia(issued_supply, wlt_1.close_method(), None),
        AssetSchema::Uda => wlt_1.issue_uda(wlt_1.close_method(), None),
        AssetSchema::Cfa => wlt_1.issue_cfa(issued_supply, wlt_1.close_method(), None),
    };

    let invoice = match transfer_type {
        TransferType::Blinded => {
            let utxo = wlt_2.get_utxo();
            wlt_2.invoice(
                contract_id,
                &iface_type_name,
                amount_1,
                wlt_2.close_method(),
                Some(&utxo),
            )
        }
        TransferType::Witness => wlt_2.invoice(
            contract_id,
            &iface_type_name,
            amount_1,
            wlt_2.close_method(),
            None,
        ),
    };

    let (consignment, _txid) = wlt_1.transfer(invoice, 3000);

    mine(false);

    wlt_2.accept_transfer(consignment);

    wlt_1.sync();

    let contract_iface = wlt_1.contract_iface(contract_id, &iface_type_name);
    match asset_schema {
        AssetSchema::Nia | AssetSchema::Cfa => {
            let allocations = wlt_1.contract_fungible_allocations(&contract_iface);
            assert_eq!(allocations.len(), 1);
            let allocation = allocations[0];
            assert_eq!(allocation.seal.method(), wlt_1.close_method());
            assert_eq!(allocation.state, Amount::from(issued_supply - amount_1));
        }
        AssetSchema::Uda => {
            let allocations = wlt_1.contract_data_allocations(&contract_iface);
            assert_eq!(allocations.len(), 0);
        }
    }

    let contract_iface = wlt_2.contract_iface(contract_id, &iface_type_name);
    // TODO: remove once https://github.com/RGB-WG/rgb-std/issues/198 gets fixed
    let expected_close_method = match transfer_type {
        TransferType::Blinded => wlt_2.close_method(),
        TransferType::Witness => wlt_1.close_method(),
    };
    match asset_schema {
        AssetSchema::Nia | AssetSchema::Cfa => {
            let allocations = wlt_2.contract_fungible_allocations(&contract_iface);
            assert_eq!(allocations.len(), 1);
            let allocation = allocations[0];
            assert_eq!(allocation.seal.method(), expected_close_method);
            assert_eq!(allocation.state, Amount::from(amount_1));
        }
        AssetSchema::Uda => {
            let allocations = wlt_2.contract_data_allocations(&contract_iface);
            assert_eq!(allocations.len(), 1);
            let allocation = &allocations[0];
            assert_eq!(allocation.seal.method(), expected_close_method);
            assert_eq!(allocation.state.to_string(), "000000000100000000000000");
        }
    }

    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);
    let amount_2 = 46;

    let invoice = match transfer_type {
        TransferType::Blinded => {
            let utxo = wlt_3.get_utxo();
            wlt_3.invoice(
                contract_id,
                &iface_type_name,
                amount_2,
                wlt_3.close_method(),
                Some(&utxo),
            )
        }
        TransferType::Witness => wlt_3.invoice(
            contract_id,
            &iface_type_name,
            amount_2,
            wlt_3.close_method(),
            None,
        ),
    };

    let (consignment, _txid) = wlt_2.transfer(invoice, 1000);

    mine(false);

    wlt_3.accept_transfer(consignment);

    wlt_2.sync();

    let contract_iface = wlt_2.contract_iface(contract_id, &iface_type_name);
    match asset_schema {
        AssetSchema::Nia | AssetSchema::Cfa => {
            let allocations = wlt_2.contract_fungible_allocations(&contract_iface);
            assert_eq!(allocations.len(), 1);
            let allocation = allocations[0];
            assert_eq!(allocation.seal.method(), wlt_2.close_method());
            assert_eq!(allocation.state, Amount::from(amount_1 - amount_2));
        }
        AssetSchema::Uda => {
            let allocations = wlt_2.contract_data_allocations(&contract_iface);
            assert_eq!(allocations.len(), 0);
        }
    }

    let contract_iface = wlt_3.contract_iface(contract_id, &iface_type_name);
    // TODO: remove once https://github.com/RGB-WG/rgb-std/issues/198 gets fixed
    let expected_close_method = match transfer_type {
        TransferType::Blinded => wlt_3.close_method(),
        TransferType::Witness => wlt_2.close_method(),
    };
    match asset_schema {
        AssetSchema::Nia | AssetSchema::Cfa => {
            let allocations = wlt_3.contract_fungible_allocations(&contract_iface);
            assert_eq!(allocations.len(), 1);
            let allocation = allocations[0];
            assert_eq!(allocation.seal.method(), expected_close_method);
            assert_eq!(allocation.state, Amount::from(amount_2));
        }
        AssetSchema::Uda => {
            let allocations = wlt_3.contract_data_allocations(&contract_iface);
            assert_eq!(allocations.len(), 1);
            let allocation = &allocations[0];
            assert_eq!(allocation.seal.method(), expected_close_method);
            assert_eq!(allocation.state.to_string(), "000000000100000000000000");
        }
    }
}

#[rstest]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    AssetSchema::Nia,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    AssetSchema::Nia,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    AssetSchema::Nia,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    AssetSchema::Cfa,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    AssetSchema::Cfa,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    AssetSchema::Cfa,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    AssetSchema::Uda,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    AssetSchema::Uda,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Wpkh,
    AssetSchema::Uda,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    AssetSchema::Nia,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    AssetSchema::Nia,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    AssetSchema::Nia,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    AssetSchema::Cfa,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    AssetSchema::Cfa,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    AssetSchema::Cfa,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    AssetSchema::Uda,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    AssetSchema::Uda,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Tr,
    AssetSchema::Uda,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    AssetSchema::Nia,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    AssetSchema::Nia,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    AssetSchema::Nia,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    AssetSchema::Cfa,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    AssetSchema::Cfa,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    AssetSchema::Cfa,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    AssetSchema::Uda,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    AssetSchema::Uda,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Wpkh,
    DescriptorType::Tr,
    AssetSchema::Uda,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    AssetSchema::Nia,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    AssetSchema::Nia,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    AssetSchema::Nia,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    AssetSchema::Cfa,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    AssetSchema::Cfa,
    AssetSchema::Nia
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    AssetSchema::Cfa,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    AssetSchema::Uda,
    AssetSchema::Uda
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    AssetSchema::Uda,
    AssetSchema::Cfa
)]
#[case(
    DescriptorType::Tr,
    DescriptorType::Wpkh,
    AssetSchema::Uda,
    AssetSchema::Nia
)]
fn with_blank_transitions(
    #[case] wlt_1_desc: DescriptorType,
    #[case] wlt_2_desc: DescriptorType,
    #[case] asset_schema_1: AssetSchema,
    #[case] asset_schema_2: AssetSchema,
) {
    initialize();

    let mut wlt_1 = get_wallet(&wlt_1_desc);
    let mut wlt_2 = get_wallet(&wlt_2_desc);

    let issued_supply = 999;
    let amount = 66;

    let utxo = wlt_1.get_utxo();
    let (contract_id_1, iface_type_name_1) = match asset_schema_1 {
        AssetSchema::Nia => wlt_1.issue_nia(issued_supply, wlt_1.close_method(), Some(&utxo)),
        AssetSchema::Uda => wlt_1.issue_uda(wlt_1.close_method(), Some(&utxo)),
        AssetSchema::Cfa => wlt_1.issue_cfa(issued_supply, wlt_1.close_method(), Some(&utxo)),
    };
    let (contract_id_2, iface_type_name_2) = match asset_schema_2 {
        AssetSchema::Nia => wlt_1.issue_nia(issued_supply, wlt_1.close_method(), Some(&utxo)),
        AssetSchema::Uda => wlt_1.issue_uda(wlt_1.close_method(), Some(&utxo)),
        AssetSchema::Cfa => wlt_1.issue_cfa(issued_supply, wlt_1.close_method(), Some(&utxo)),
    };

    let invoice = wlt_2.invoice(
        contract_id_1,
        &iface_type_name_1,
        amount,
        wlt_2.close_method(),
        None,
    );

    let (consignment, _txid) = wlt_1.transfer(invoice, 1000);

    mine(false);

    wlt_2.accept_transfer(consignment);

    wlt_1.sync();

    let contract_iface_1 = wlt_1.contract_iface(contract_id_1, &iface_type_name_1);
    match asset_schema_1 {
        AssetSchema::Nia | AssetSchema::Cfa => {
            let allocations = wlt_1.contract_fungible_allocations(&contract_iface_1);
            assert_eq!(allocations.len(), 1);
            let allocation = allocations[0];
            assert_eq!(allocation.seal.method(), wlt_1.close_method());
            assert_eq!(allocation.state, Amount::from(issued_supply - amount));
            let allocations = wlt_2.contract_fungible_allocations(&contract_iface_1);
            assert_eq!(allocations.len(), 1);
            let allocation = allocations[0];
            assert_eq!(allocation.seal.method(), wlt_1.close_method());
            assert_eq!(allocation.state, Amount::from(amount));
        }
        AssetSchema::Uda => {
            let allocations = wlt_1.contract_data_allocations(&contract_iface_1);
            assert_eq!(allocations.len(), 0);
            let allocations = wlt_2.contract_data_allocations(&contract_iface_1);
            assert_eq!(allocations.len(), 1);
            let allocation = &allocations[0];
            assert_eq!(allocation.seal.method(), wlt_1.close_method());
            assert_eq!(allocation.state.to_string(), "000000000100000000000000");
        }
    }

    let contract_iface_2 = wlt_1.contract_iface(contract_id_2, &iface_type_name_2);
    match asset_schema_2 {
        AssetSchema::Nia | AssetSchema::Cfa => {
            let allocations = wlt_1.contract_fungible_allocations(&contract_iface_2);
            assert_eq!(allocations.len(), 1);
            let allocation = allocations[0];
            assert_eq!(allocation.seal.method(), wlt_1.close_method());
            assert_eq!(allocation.state, Amount::from(issued_supply));
        }
        AssetSchema::Uda => {
            let allocations = wlt_1.contract_data_allocations(&contract_iface_2);
            assert_eq!(allocations.len(), 1);
            let allocation = &allocations[0];
            assert_eq!(allocation.seal.method(), wlt_1.close_method());
            assert_eq!(allocation.state.to_string(), "000000000100000000000000");
        }
    }
}
