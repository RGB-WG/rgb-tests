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
    pub_witnesses: HashMap<XWitnessId, MockResolvePubWitness>,
    pub_witness_ords: HashMap<XWitnessId, MockResolvePubWitnessOrd>,
}

impl ResolveWitness for MockResolver {
    fn resolve_pub_witness(
        &self,
        witness_id: XWitnessId,
    ) -> Result<XWitnessTx, WitnessResolverError> {
        if let Some(res) = self.pub_witnesses.get(&witness_id) {
            match res {
                MockResolvePubWitness::Success(tx) => Ok(XChain::Bitcoin(tx.clone())),
                MockResolvePubWitness::Error(err) => Err(err.clone()),
            }
        } else {
            Err(WitnessResolverError::Unknown(witness_id))
        }
    }

    fn resolve_pub_witness_ord(
        &self,
        witness_id: XWitnessId,
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
                    get_tx("bc:97b4c906fafad952e7b68784d38f56dec505a719c2564743b1bebe0289c7d893");
                let (tx_2, witness_id_2) =
                    get_tx("bc:f93edffa684d53146d5bc11122eb129960f2ca86bbdbca338df3b3c7234544dc");
                let (tx_3, witness_id_3) =
                    get_tx("bc:3b9c8836e09cdf08d225b8e9f7862e23a6ede1276c12ccd426c530cd531e4cfa");
                MockResolver {
                    pub_witnesses: map![
                        witness_id_1 => MockResolvePubWitness::Success(tx_1),
                        witness_id_2 => MockResolvePubWitness::Success(tx_2),
                        witness_id_3 => MockResolvePubWitness::Success(tx_3),
                    ],
                    pub_witness_ords: map![
                        witness_id_1 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::new(106, 1726062111).unwrap())),
                        witness_id_2 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::new(108, 1726062111).unwrap())),
                        witness_id_3 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::new(110, 1726062112).unwrap())),
                    ],
                }
            }
            Self::B => {
                let (tx_1, witness_id_1) =
                    get_tx("bc:b9703e716d01de771d443c5cbf06cbba1f132c67a2b6d8c6c88bbcc0f83afe1b");
                let (tx_2, witness_id_2) =
                    get_tx("bc:5abbca66698ef17ab5e5794adc5a73e5988a96235b84e943a0c4e0fd05a8ca66");
                let (tx_3, witness_id_3) =
                    get_tx("bc:222f1a35bb96a02831930dcdd02e46f46daca1fdd7199dd043e04c7841b2ff40");
                MockResolver {
                    pub_witnesses: map![
                        witness_id_1 => MockResolvePubWitness::Success(tx_1),
                        witness_id_2 => MockResolvePubWitness::Success(tx_2),
                        witness_id_3 => MockResolvePubWitness::Success(tx_3),
                    ],
                    pub_witness_ords: map![
                        witness_id_1 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::new(105, 1726062423).unwrap())),
                        witness_id_2 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::new(106, 1726062423).unwrap())),
                        witness_id_3 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::new(106, 1726062423).unwrap())),
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
    let (contract_id_1, iface_type_name_1) =
        wlt_1.issue_nia(issued_supply_1, wlt_1.close_method(), Some(&utxo));
    let (contract_id_2, iface_type_name_2) =
        wlt_1.issue_nia(issued_supply_2, wlt_1.close_method(), Some(&utxo));

    let mut txes = vec![];

    let (_consignment, tx) = wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_1,
        &iface_type_name_1,
        66,
        sats,
        None,
    );
    txes.push(tx);

    // spend asset moved with blank transition
    let (_consignment, tx) = wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_2,
        &iface_type_name_2,
        50,
        sats,
        None,
    );
    txes.push(tx);

    // spend change of previous send
    let (consignment, tx) = wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_2,
        &iface_type_name_2,
        77,
        sats,
        None,
    );
    txes.push(tx);

    (consignment, txes)
}

// run once to generate tests/fixtures/consignemnt_<scenario>.yaml
// for example:
// SCENARIO=B cargo test --test validation validate_consignment_generate -- --ignored --show-output
//
// then copy the generated consignemnt file to tests/fixtures/attack_<n>.yaml
// manually change tests/fixtures/attack_<n>.yaml files to simulate attacks
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
        let witness_id = XWitnessId::from_str(&txid).unwrap();
        let yaml = serde_yaml::to_string(&tx).unwrap();
        let yaml_path = format!("tests/fixtures/{witness_id}.yaml");
        std::fs::write(&yaml_path, yaml).unwrap();
        println!("written tx: {witness_id}");
    }
}

fn get_consignment_from_yaml(fname: &str) -> Transfer {
    let cons_path = format!("tests/fixtures/{fname}.yaml");
    println!("loading {cons_path}");
    let file = std::fs::File::open(cons_path).unwrap();
    let consignment: Transfer = serde_yaml::from_reader(file).unwrap();
    consignment
}

fn get_tx(witness_id: &str) -> (Tx, XWitnessId) {
    let yaml_path = format!("tests/fixtures/{witness_id}.yaml");
    let file = std::fs::File::open(yaml_path).unwrap();
    let tx: Tx = serde_yaml::from_reader(file).unwrap();
    let xwitness_id = XWitnessId::from_str(witness_id).unwrap();
    (tx, xwitness_id)
}

#[test]
fn validate_consignment_success() {
    for scenario in Scenario::iter() {
        let resolver = scenario.resolver();
        let consignment = get_consignment_from_yaml(&format!("consignment_{scenario}"));
        let res = consignment.validate(&resolver, true);
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

#[test]
fn validate_consignment_chain_fail() {
    let resolver = Scenario::A.resolver();

    // liquid
    let consignment = get_consignment_from_yaml("attack_chain");
    let res = consignment.validate(&resolver, true);
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

#[test]
fn validate_consignment_genesis_fail() {
    let resolver = Scenario::B.resolver();

    // schema ID: change genesis[schemaId] with CFA schema ID
    let consignment = get_consignment_from_yaml("attack_genesis_schema_id");
    let res = consignment.validate(&resolver, true);
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

    // genesis testnet: change from true to false
    let consignment = get_consignment_from_yaml("attack_genesis_testnet");
    let res = consignment.validate(&resolver, true);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err((status, _consignment)) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 1);
    assert!(matches!(
        validation_status.failures[0],
        Failure::NetworkMismatch(_)
    ));
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);
}

#[test]
fn validate_consignment_bundles_fail() {
    let resolver = Scenario::A.resolver();

    // bundles pubWitness data inputs[0] sequence: change from 0 to 1
    let consignment = get_consignment_from_yaml("attack_bundles_pubWitness_data_input_sequence");
    let res = consignment.validate(&resolver, true);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err((status, _consignment)) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 2);
    assert!(matches!(
        validation_status.failures[0],
        Failure::SealsInvalid(_, _, _)
    ));
    assert!(matches!(
        validation_status.failures[1],
        Failure::BundleInvalidCommitment(_, _, _, _)
    ));
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);
}

#[test]
fn validate_consignment_resolver_error() {
    let scenario = Scenario::A;
    let mut resolver = scenario.resolver();
    let txid =
        Txid::from_str("f93edffa684d53146d5bc11122eb129960f2ca86bbdbca338df3b3c7234544dc").unwrap();
    let xwitness_id = XChain::Bitcoin(txid);

    // resolve_pub_witness error
    *resolver.pub_witnesses.get_mut(&xwitness_id).unwrap() = MockResolvePubWitness::Error(
        WitnessResolverError::Other(xwitness_id, s!("unexpected error")),
    );
    let consignment = get_consignment_from_yaml("attack_resolver_error");
    let res = consignment.validate(&resolver, true);
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
    *resolver.pub_witness_ords.get_mut(&xwitness_id).unwrap() = MockResolvePubWitnessOrd::Error(
        WitnessResolverError::Other(xwitness_id, s!("unexpected error")),
    );
    let consignment = get_consignment_from_yaml("attack_resolver_error");
    let res = consignment.validate(&resolver, true);
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
