pub mod utils;

use utils::*;

enum MockResolvePubWitness {
    Success(Tx),
    Error(WitnessResolverError),
}

enum MockResolvePubWitnessOrd {
    Success(WitnessOrd),
    Error(WitnessResolverError),
}

struct MockResolver {
    pub_witnesses: HashMap<Txid, MockResolvePubWitness>,
    pub_witness_ords: HashMap<Txid, MockResolvePubWitnessOrd>,
}

impl ResolveWitness for MockResolver {
    fn resolve_pub_witness(&self, witness_id: Txid) -> Result<Tx, WitnessResolverError> {
        if let Some(res) = self.pub_witnesses.get(&witness_id) {
            match res {
                MockResolvePubWitness::Success(tx) => Ok(tx.clone()),
                MockResolvePubWitness::Error(err) => Err(err.clone()),
            }
        } else {
            Err(WitnessResolverError::Unknown(witness_id))
        }
    }

    fn resolve_pub_witness_ord(
        &self,
        witness_id: Txid,
    ) -> Result<WitnessOrd, WitnessResolverError> {
        if let Some(res) = self.pub_witness_ords.get(&witness_id) {
            match res {
                MockResolvePubWitnessOrd::Success(witness_ord) => Ok(*witness_ord),
                MockResolvePubWitnessOrd::Error(err) => Err(err.clone()),
            }
        } else {
            Err(WitnessResolverError::Unknown(witness_id))
        }
    }

    fn check_chain_net(&self, _: ChainNet) -> Result<(), WitnessResolverError> {
        Ok(())
    }
}

#[derive(Debug, EnumIter, Copy, Clone, PartialEq)]
enum Scenario {
    A,
    B,
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Scenario {
    fn resolver(&self) -> MockResolver {
        match self {
            Self::A => {
                let (tx_1, witness_id_1) =
                    get_tx("4fa504106afb626b132a70b97225e4ad7607f5ba19801717c8fa056c990b450e");
                let (tx_2, witness_id_2) =
                    get_tx("c1ffb662bdf8a89f5c50946b4f1f7afce1e4dd5655381101f111f839f0294242");
                let (tx_3, witness_id_3) =
                    get_tx("eee4efa2a7d264efd948dc2d46690c200beb4630b5638d3249c4738e0049fd05");
                MockResolver {
                    pub_witnesses: map![
                        witness_id_1 => MockResolvePubWitness::Success(tx_1),
                        witness_id_2 => MockResolvePubWitness::Success(tx_2),
                        witness_id_3 => MockResolvePubWitness::Success(tx_3),
                    ],
                    pub_witness_ords: map![
                        witness_id_1 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(106).unwrap(), 1726062111).unwrap())),
                        witness_id_2 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(108).unwrap(), 1726062111).unwrap())),
                        witness_id_3 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(110).unwrap(), 1726062112).unwrap())),
                    ],
                }
            }
            Self::B => {
                let (tx_1, witness_id_1) =
                    get_tx("3aa4859506a217c570c817104719367313cb905833c63ae8d4bfd40ca478f3be");
                let (tx_2, witness_id_2) =
                    get_tx("f118b087f59c0964dab787b8d123b301bab6c2edb1192e49c31074c77ff144bd");
                let (tx_3, witness_id_3) =
                    get_tx("6a335f810a3b31f2a5326e64b11fed6d712f6776ae211240166858f8ac274391");
                MockResolver {
                    pub_witnesses: map![
                        witness_id_1 => MockResolvePubWitness::Success(tx_1),
                        witness_id_2 => MockResolvePubWitness::Success(tx_2),
                        witness_id_3 => MockResolvePubWitness::Success(tx_3),
                    ],
                    pub_witness_ords: map![
                        witness_id_1 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(105).unwrap(), 1726062423).unwrap())),
                        witness_id_2 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(106).unwrap(), 1726062423).unwrap())),
                        witness_id_3 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(106).unwrap(), 1726062423).unwrap())),
                    ],
                }
            }
        }
    }
}

fn get_consignment(scenario: Scenario) -> (Transfer, Vec<Tx>) {
    initialize();

    let transfer_type = match scenario {
        Scenario::A => TransferType::Blinded,
        Scenario::B => TransferType::Witness,
    };

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issued_supply_1 = 999;
    let issued_supply_2 = 666;

    let sats = 9000;

    let utxo = wlt_1.get_utxo(None);
    let contract_id_1 = wlt_1.issue_nia(issued_supply_1, Some(&utxo));
    let contract_id_2 = wlt_1.issue_nia(issued_supply_2, Some(&utxo));

    let mut txes = vec![];

    let (_consignment, tx) = wlt_1.send(&mut wlt_2, transfer_type, contract_id_1, 66, sats, None);
    txes.push(tx);

    // spend asset that was moved automatically
    let (_consignment, tx) = wlt_1.send(&mut wlt_2, transfer_type, contract_id_2, 50, sats, None);
    txes.push(tx);

    // spend change of previous send
    let (consignment, tx) = wlt_1.send(&mut wlt_2, transfer_type, contract_id_2, 77, sats, None);
    txes.push(tx);

    (consignment, txes)
}

// run once to generate tests/fixtures/consignemnt_<scenario>.yaml
// for example:
// SCENARIO=B cargo test --test validation validate_consignment_generate -- --ignored --show-output
//
// then copy the generated consignemnt file to tests/fixtures/attack_<n>.yaml
// manually change tests/fixtures/attack_<n>.yaml files to simulate attacks
#[cfg(not(feature = "altered"))]
#[test]
#[ignore = "one-shot"]
fn validate_consignment_generate() {
    let scenario = match std::env::var("SCENARIO") {
        Ok(val) if val.to_uppercase() == Scenario::A.to_string() => Scenario::A,
        Ok(val) if val.to_uppercase() == Scenario::B.to_string() => Scenario::B,
        Err(VarError::NotPresent) => Scenario::A,
        _ => panic!("invalid scenario"),
    };
    let (consignment, txes) = get_consignment(scenario);
    println!();
    let cons_path = format!("tests/fixtures/consignment_{scenario}.yaml");
    let yaml = serde_yaml::to_string(&consignment).unwrap();
    std::fs::write(&cons_path, yaml).unwrap();
    println!("written consignment in: {cons_path}");
    for tx in txes {
        let txid = tx.txid().to_string();
        let yaml = serde_yaml::to_string(&tx).unwrap();
        let yaml_path = format!("tests/fixtures/{txid}.yaml");
        std::fs::write(&yaml_path, yaml).unwrap();
        println!("written tx: {txid}");
    }
}

fn get_consignment_from_yaml(fname: &str) -> Transfer {
    let cons_path = format!("tests/fixtures/{fname}.yaml");
    println!("loading {cons_path}");
    let file = std::fs::File::open(cons_path).unwrap();
    let consignment: Transfer = serde_yaml::from_reader(file).unwrap();
    consignment
}

fn get_tx(txid: &str) -> (Tx, Txid) {
    let normalized_txid = txid.replace(":", "_");
    let yaml_path = format!("tests/fixtures/{normalized_txid}.yaml");
    let file = std::fs::File::open(yaml_path).unwrap();
    let tx: Tx = serde_yaml::from_reader(file).unwrap();
    let txid = Txid::from_str(txid).unwrap();
    (tx, txid)
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_success() {
    for scenario in Scenario::iter() {
        let resolver = scenario.resolver();
        let consignment = get_consignment_from_yaml(&format!("consignment_{scenario}"));
        let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
        assert!(res.is_ok());
        let validation_status = match res {
            Ok(validated_consignment) => validated_consignment.validation_status().clone(),
            Err((status, _consignment)) => status,
        };
        dbg!(&validation_status);
        assert!(validation_status.failures.is_empty());
        assert!(validation_status.warnings.is_empty());
        assert!(validation_status.info.is_empty());
        let validity = validation_status.validity();
        assert_eq!(validity, Validity::Valid);
    }
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_chain_fail() {
    let resolver = Scenario::A.resolver();

    // genesis chainNet: change from bitcoinRegtest to liquidTestnet
    let consignment = get_consignment_from_yaml("attack_chain");
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err((status, _consignment)) => status,
    };
    dbg!(&validation_status);
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_genesis_fail() {
    let resolver = Scenario::B.resolver();

    // schema ID: change genesis[schemaId] with CFA schema ID
    let consignment = get_consignment_from_yaml("attack_genesis_schema_id");
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err((status, _consignment)) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 5);
    assert!(matches!(
        validation_status.failures[0],
        Failure::MpcInvalid(_, _, _)
    ));
    assert!(matches!(
        validation_status.failures[1],
        Failure::OperationAbsent(_)
    ));
    assert!(matches!(
        validation_status.failures[2],
        Failure::MpcInvalid(_, _, _)
    ));
    assert!(matches!(
        validation_status.failures[3],
        Failure::BundleExtraTransition(_, _)
    ));
    assert!(matches!(
        validation_status.failures[4],
        Failure::MpcInvalid(_, _, _)
    ));
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);

    // genesis chainNet: change from bitcoinRegtest to bitcoinMainnet
    let consignment = get_consignment_from_yaml("attack_genesis_testnet");
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err((status, _consignment)) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 1);
    assert!(matches!(
        validation_status.failures[0],
        Failure::ContractChainNetMismatch(_)
    ));
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_bundles_fail() {
    let resolver = Scenario::A.resolver();

    // bundles first in time pubWitness inputs[0] sequence: change from 0 to 1
    let consignment = get_consignment_from_yaml("attack_bundles_pubWitness_data_input_sequence");
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err((status, _consignment)) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 3);
    assert!(matches!(
        validation_status.failures[0],
        Failure::SealNoPubWitness(_, _, _)
    ));
    assert!(matches!(
        validation_status.failures[1],
        Failure::SealsInvalid(_, _, _)
    ));
    assert!(matches!(
        validation_status.failures[2],
        Failure::BundleInvalidCommitment(_, _, _, _)
    ));
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_resolver_error() {
    let scenario = Scenario::A;
    let mut resolver = scenario.resolver();
    let txid =
        Txid::from_str("c1ffb662bdf8a89f5c50946b4f1f7afce1e4dd5655381101f111f839f0294242").unwrap();

    // resolve_pub_witness error
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Error(WitnessResolverError::Other(txid, s!("unexpected error")));
    let consignment = get_consignment_from_yaml("attack_resolver_error");
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err((status, _consignment)) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 1);
    assert!(matches!(
        validation_status.failures[0],
        Failure::SealNoPubWitness(_, _, _)
    ));
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);

    // resolve_pub_witness_ord error
    *resolver.pub_witness_ords.get_mut(&txid).unwrap() =
        MockResolvePubWitnessOrd::Error(WitnessResolverError::Other(txid, s!("unexpected error")));
    let consignment = get_consignment_from_yaml("attack_resolver_error");
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err((status, _consignment)) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 1);
    assert!(matches!(
        validation_status.failures[0],
        Failure::SealNoPubWitness(_, _, _)
    ));
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);
}
