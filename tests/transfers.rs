mod utils;

use rstest::rstest;
use std::path::PathBuf;

use crate::utils::chain::initialize;
use crate::utils::runtime::TestRuntime;
use crate::utils::{AssetSchema, DescriptorType, TransferType};

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
//#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Uda)]
//#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Nia, AS::Uda)]
//#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Nia, AS::Uda)]
//#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Nia, AS::Uda)]
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
// #[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Uda)]
// #[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Cfa, AS::Uda)]
// #[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Cfa, AS::Uda)]
// #[case(TT::Blinded, DT::Tr, DT::Tr, AS::Cfa, AS::Uda)]
// blinded: uda - uda
// #[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Uda)]
// #[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Uda, AS::Uda)]
// #[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Uda, AS::Uda)]
// #[case(TT::Blinded, DT::Tr, DT::Tr, AS::Uda, AS::Uda)]
// blinded: uda - nia
// #[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Nia)]
// #[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Uda, AS::Nia)]
// #[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Uda, AS::Nia)]
// #[case(TT::Blinded, DT::Tr, DT::Tr, AS::Uda, AS::Nia)]
// blinded: uda - cfa
// #[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Cfa)]
// #[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Uda, AS::Cfa)]
// #[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Uda, AS::Cfa)]
// #[case(TT::Blinded, DT::Tr, DT::Tr, AS::Uda, AS::Cfa)]
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
//#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Uda)]
//#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Nia, AS::Uda)]
//#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Nia, AS::Uda)]
//#[case(TT::Witness, DT::Tr, DT::Tr, AS::Nia, AS::Uda)]
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
// #[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Uda)]
// #[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Cfa, AS::Uda)]
// #[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Cfa, AS::Uda)]
// #[case(TT::Witness, DT::Tr, DT::Tr, AS::Cfa, AS::Uda)]
// witness: uda - uda
// #[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Uda)]
// #[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Uda, AS::Uda)]
// #[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Uda, AS::Uda)]
// #[case(TT::Witness, DT::Tr, DT::Tr, AS::Uda, AS::Uda)]
// witness: uda - nia
// #[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Nia)]
// #[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Uda, AS::Nia)]
// #[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Uda, AS::Nia)]
// #[case(TT::Witness, DT::Tr, DT::Tr, AS::Uda, AS::Nia)]
// witness: uda - cfa
// #[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Cfa)]
// #[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Uda, AS::Cfa)]
// #[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Uda, AS::Cfa)]
// #[case(TT::Witness, DT::Tr, DT::Tr, AS::Uda, AS::Cfa)]
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

    let mut wlt_1 = TestRuntime::new(&wlt_1_desc, "wlt1");
    let mut wlt_2 = TestRuntime::new(&wlt_2_desc, "wlt2");

    let issued_supply_1 = 999;
    let issued_supply_2 = 666;

    let wout = transfer_type == TT::Witness;
    let mut sats = 9000;

    println!("-- DEBUG: wlt_1 issues 2 assets on the same UTXO");
    let utxo = wlt_1.get_utxo(None);
    let contract_id_1 = match asset_schema_1 {
        AssetSchema::Nia => wlt_1.issue_nia("Nia1", issued_supply_1, utxo),
        AssetSchema::Uda => todo!(), // wlt_1.issue_uda(utxo),
        AssetSchema::Cfa => wlt_1.issue_cfa("Cfa1", issued_supply_1, utxo),
    };
    let contract_id_2 = match asset_schema_2 {
        AssetSchema::Nia => wlt_1.issue_nia("Nia2", issued_supply_2, utxo),
        AssetSchema::Uda => todo!(), // wlt_1.issue_uda(utxo),
        AssetSchema::Cfa => wlt_1.issue_cfa("Cfa2", issued_supply_2, utxo),
    };
    wlt_1.check_allocations(contract_id_1, asset_schema_1, vec![issued_supply_1], true); // A contract 1, [999]
    wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![issued_supply_2], true); // A contract 2, [666]

    println!("-- DEBUG: wlt_1 spends asset 1, moving the other with a blank transition");
    let amount_1 = if asset_schema_1 == AssetSchema::Uda {
        1
    } else {
        99
    };
    println!("send 1");

    let path1 = PathBuf::from("tests")
        .join("test-data")
        .with_file_name("contract1.rgb");
    let path2 = PathBuf::from("tests")
        .join("test-data")
        .with_file_name("contract2.rgb");
    wlt_1
        .contracts
        .consign_to_file(&path1, contract_id_1, &[])
        .unwrap();
    wlt_1
        .contracts
        .consign_to_file(&path2, contract_id_2, &[])
        .unwrap();
    wlt_2
        .rt
        .consume_from_file(path1)
        .unwrap_or_else(|e| panic!("{e}"));
    wlt_2
        .rt
        .consume_from_file(path2)
        .unwrap_or_else(|e| panic!("{e}"));
    wlt_2.get_utxo(None); // make sure wlt_2 has some coins so it can invoice
    wlt_1.send(&mut wlt_2, wout, contract_id_1, amount_1, sats, None);
    println!("-- DEBUG: wlt_1 checks allocations after spending asset 1");
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1], // contract 1, [900]
        false,
    );
    wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![issued_supply_2], true); // A contract 2, [666]
    println!("-- DEBUG: wlt_2 checks allocations after receiving asset 1");
    wlt_2.check_allocations(contract_id_1, asset_schema_1, vec![amount_1], true); // B contract 1, [100]

    println!("-- DEBUG: wlt_1 spends asset 1 change (only if possible)");
    let amount_2 = 33;
    if asset_schema_1 != AssetSchema::Uda {
        println!("send 2");
        wlt_1.send(&mut wlt_2, wout, contract_id_1, amount_2, sats, None);
        wlt_1.check_allocations(
            contract_id_1,
            asset_schema_1,
            vec![issued_supply_1 - amount_1 - amount_2],
            false,
        );
        wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![issued_supply_2], true);
        wlt_2.check_allocations(
            contract_id_1,
            asset_schema_1,
            vec![amount_1, amount_2],
            true,
        );
    }

    println!("-- DEBUG: wlt_1 spends asset 2");
    let amount_3 = if asset_schema_2 == AssetSchema::Uda {
        1
    } else {
        22
    };
    println!("send 3");
    wlt_1.send(&mut wlt_2, wout, contract_id_2, amount_3, sats, None);
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2],
        false,
    );
    wlt_1.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3],
        false,
    );
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1, amount_2],
        true,
    );
    wlt_2.check_allocations(contract_id_2, asset_schema_2, vec![amount_3], true);

    println!("-- DEBUG: wlt_2 spends received allocation(s) of asset 1");
    let amount_4 = if asset_schema_1 == AssetSchema::Uda {
        1
    } else {
        111
    };
    sats -= 1000;
    wlt_2.send(&mut wlt_1, wout, contract_id_1, amount_4, sats, None);
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2, amount_4],
        true,
    );
    wlt_1.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3],
        false,
    );
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4],
        false,
    );
    wlt_2.check_allocations(contract_id_2, asset_schema_2, vec![amount_3], true);

    println!("-- DEBUG: wlt_2 spends asset 2");
    let amount_5 = if asset_schema_2 == AssetSchema::Uda {
        1
    } else {
        11
    };
    sats -= 1000;
    wlt_2.send(&mut wlt_1, wout, contract_id_2, amount_5, sats, None);
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2, amount_4],
        true,
    );
    wlt_1.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3, amount_5],
        true,
    );
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4],
        false,
    );
    wlt_2.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![amount_3 - amount_5],
        false,
    );

    println!("-- DEBUG: wlt_1 spends asset 1, received back");
    let amount_6 = if asset_schema_1 == AssetSchema::Uda {
        1
    } else {
        issued_supply_1 - amount_1 - amount_2 + amount_4
    };
    sats -= 1000;
    // println!("before send, you can dump and debug now...");
    // std::thread::sleep(std::time::Duration::from_secs(30));
    wlt_1.send(&mut wlt_2, wout, contract_id_1, amount_6, sats, None);

    wlt_1.check_allocations(contract_id_1, asset_schema_1, vec![], false);
    // wlt_1.check_allocations(
    //     contract_id_2,
    //     asset_schema_2,
    //     vec![issued_supply_2 - amount_3, amount_5],
    //     true,
    // );
    // wlt_1.check_allocations(
    //     contract_id_2,
    //     asset_schema_2,
    //     vec![issued_supply_2 - amount_3 + amount_5],
    //     true,
    // );
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4, amount_6],
        true,
    );
    wlt_2.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![amount_3 - amount_5],
        false,
    );

    println!("-- DEBUG: wlt_1 spends asset 2, received back");
    let amount_7 = if asset_schema_2 == AssetSchema::Uda {
        1
    } else {
        issued_supply_2 - amount_3 + amount_5
    };
    sats -= 1000;
    wlt_1.send(&mut wlt_2, wout, contract_id_2, amount_7, sats, None);
    wlt_1.check_allocations(contract_id_1, asset_schema_1, vec![], false);
    wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![], false);
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4, amount_6],
        true,
    );
    wlt_2.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![amount_3 - amount_5, amount_7],
        true,
    );
}
