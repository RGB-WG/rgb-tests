// RGB v0.12 Migration Notes:
// 1. Seal Type Unification (RFC: https://github.com/RGB-WG/RFC/issues/16)
//    - Unified seal type replaces distinct opret and tapret seals
//    - Seal type is automatically determined by wallet type (Taproot/WPKH)
//    - CloseMethod parameter has been removed from contract genesis
//    - Contract no longer commits to specific seal types
//
// 2. API Changes and Migration Strategy:
//    - Removed APIs:
//      * update_witnesses: Will be replaced with new rollback procedure
//      * CloseMethod related parameters: No longer needed due to seal unification
//    - Test Case Handling:
//      * Existing tests dependent on removed APIs: Marked as #[ignore] with tracking issues
//      * New tests: Focus on wallet type interactions rather than seal types
//      * Complex test cases will be implemented after evaluating:
//        - RGB protocol documentation and examples
//        - Implementation approaches for Lightning Network, multi-sig and interactive transactions

#![allow(unused_imports)]

use rstest::rstest;
use serial_test::serial;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use std::{env::VarError, fs::File};
use time::format_description::well_known::Iso8601;
use time::{format_description, OffsetDateTime};
use utils::{
    chain::initialize,
    helpers::{get_wallet, MetricDefinition, MetricType, Report, TransferType},
    DescriptorType, DEFAULT_FEE_ABS, STRESS_DATA_DIR, TEST_DATA_DIR,
};

pub mod utils;

// Aliases for shorter test case definitions
type TT = TransferType;
type DT = DescriptorType;

#[rstest]
#[case(false, DT::Wpkh, DT::Wpkh)]
#[case(false, DT::Wpkh, DT::Tr)]
#[case(false, DT::Tr, DT::Wpkh)]
#[case(false, DT::Tr, DT::Tr)]
#[case(true, DT::Wpkh, DT::Wpkh)]
#[case(true, DT::Wpkh, DT::Tr)]
#[case(true, DT::Tr, DT::Wpkh)]
#[case(true, DT::Tr, DT::Tr)]
#[serial]
fn back_and_forth(
    #[case] wout: bool,
    #[case] wlt_1_desc: DescriptorType,
    #[case] wlt_2_desc: DescriptorType,
) {
    // case_7: `{\"code\":-26,\"message\":\"min relay fee not met, 600 < 671\"}"`
    // Define a fee constant to prevent errors when the number of transaction inputs is too high
    const MIN_RELAY_FEE: u64 = 800;
    println!("wout {wout:?} wlt_1_desc {wlt_1_desc:?} wlt_2_desc {wlt_2_desc:?}");

    initialize();

    let stress_tests_dir = PathBuf::from(TEST_DATA_DIR).join(STRESS_DATA_DIR);
    std::fs::create_dir_all(&stress_tests_dir).unwrap();
    let summary_name = format!("wout_{wout}_wlt_1_{wlt_1_desc}_wlt_2_{wlt_2_desc}");

    let format =
        format_description::parse("[year]_[month]_[day]_[hour]_[minute]_[second]").unwrap();
    let date = OffsetDateTime::now_utc().format(&format).unwrap();
    let filename = format!("{}_{}", summary_name, date);
    let mut fpath = stress_tests_dir.join(&filename);
    fpath.set_extension("csv");
    println!("report path: {}", fpath.to_string_lossy());

    // Create Report instance with metrics
    let metrics = vec![
        MetricDefinition {
            name: "wlt_1_pay".to_string(),
            metric_type: MetricType::Duration,
            description: "Time taken for wallet 1 to pay".to_string(),
        },
        MetricDefinition {
            name: "wlt_1_pay_consignment_size".to_string(),
            metric_type: MetricType::Bytes,
            description: "Size of the consignment file for wallet 1".to_string(),
        },
        MetricDefinition {
            name: "wlt_1_pay_txin_count".to_string(),
            metric_type: MetricType::Integer,
            description: "Number of inputs in the transaction for wallet 1".to_string(),
        },
        MetricDefinition {
            name: "wlt_1_pay_txout_count".to_string(),
            metric_type: MetricType::Integer,
            description: "Number of outputs in the transaction for wallet 1".to_string(),
        },
        MetricDefinition {
            name: "wlt_2_accept".to_string(),
            metric_type: MetricType::Duration,
            description: "Time taken for wallet 2 to accept".to_string(),
        },
        MetricDefinition {
            name: "wlt_2_pay".to_string(),
            metric_type: MetricType::Duration,
            description: "Time taken for wallet 2 to pay".to_string(),
        },
        MetricDefinition {
            name: "wlt_2_pay_consignment_size".to_string(),
            metric_type: MetricType::Bytes,
            description: "Size of the consignment file for wallet 2".to_string(),
        },
        MetricDefinition {
            name: "wlt_2_pay_txin_count".to_string(),
            metric_type: MetricType::Integer,
            description: "Number of inputs in the transaction for wallet 2".to_string(),
        },
        MetricDefinition {
            name: "wlt_2_pay_txout_count".to_string(),
            metric_type: MetricType::Integer,
            description: "Number of outputs in the transaction for wallet 2".to_string(),
        },
        MetricDefinition {
            name: "wlt_1_accept".to_string(),
            metric_type: MetricType::Duration,
            description: "Time taken for wallet 1 to accept".to_string(),
        },
    ];

    let mut report = Report::new(fpath, metrics, Some(10)).unwrap();

    let mut wlt_1 = get_wallet(&wlt_1_desc).with_id("wlt_1");
    let mut wlt_2 = get_wallet(&wlt_2_desc).with_id("wlt_2");

    let issued_supply = u64::MAX;

    // In RGB v0.12, the close_method parameter is no longer required
    // Create and issue assets
    let mut params =
        utils::helpers::NIAIssueParams::new("TestAsset", "TEST", "centiMilli", issued_supply);
    let utxo = wlt_1.get_utxo(None);
    params.add_allocation(utxo, issued_supply);
    let contract_id = wlt_1.issue_nia_with_params(params);
    wlt_1.send_contract("TestAsset", &mut wlt_2);
    wlt_2.reload_runtime();

    let loops = match std::env::var("LOOPS") {
        Ok(val) if u16::from_str(&val).is_ok() => u16::from_str(&val).unwrap(),
        Err(VarError::NotPresent) => 50,
        _ => {
            panic!("invalid loops value: must be a u16 number")
        }
    };

    let sats_base = 3601;
    let mut sats_send = sats_base * loops as u64;
    let now = Instant::now();
    for i in 1..=loops {
        println!("loop {i}/{loops}");
        sats_send -= MIN_RELAY_FEE * 2;

        // In RGB v0.12, the send method parameters have been changed
        wlt_1.send(
            &mut wlt_2,
            wout,
            contract_id,
            issued_supply - i as u64,
            sats_send,
            Some(MIN_RELAY_FEE),
            None,
            Some(&mut report),
        );

        sats_send -= DEFAULT_FEE_ABS * 2;

        wlt_2.send(
            &mut wlt_1,
            wout,
            contract_id,
            issued_supply - i as u64 - 1,
            sats_send,
            Some(MIN_RELAY_FEE),
            None,
            Some(&mut report),
        );

        // End the row for this iteration
        report.end_row().unwrap();
    }
    let elapsed = now.elapsed();
    println!("Total time: {:.2?}", elapsed);
    println!("Average time per transfer: {:.2?}", elapsed / loops as u32);

    // Generate and save test summary
    match report.save_summary(&filename) {
        Ok(summary) => println!("{}", std::fs::read_to_string(summary).unwrap()),
        Err(e) => println!("Failed to save test summary: {}", e),
    }
}
