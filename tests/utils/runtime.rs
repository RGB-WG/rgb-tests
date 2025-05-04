use std::fs::{self, OpenOptions};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use std::{io, thread};

use bpstd::psbt::{PsbtConstructor, TxParams};
use bpstd::signers::TestnetSigner;
use bpstd::{
    h, Address, Keychain, Network, Psbt, Sats, Tx, Txid, Vout, Wpkh, XprivAccount, XpubDerivable,
};
use bpwallet::fs::FsTextStore;
use bpwallet::AnyIndexer;
use rand::RngCore;
use rgb::invoice::{RgbBeneficiary, RgbInvoice};
use rgb::popls::bp::RgbWallet;
use rgb::{
    Assignment, CodexId, Consensus, ContractId, Contracts, CreateParams, EitherSeal, NamedState,
    Outpoint, Schema, StateAtom, StockpileDir,
};
use rgbp::descriptor::RgbDescr;
use rgbp::{CoinselectStrategy, Owner, RgbpRuntimeDir};
use strict_types::{svenum, svnum, svstr, tn, vname, StrictVal};

use crate::utils::chain::{
    broadcast_tx, fund_wallet, get_indexer, indexer_url, is_tx_mined, mine_custom, INSTANCE_1,
};
use crate::utils::report::Report;
use crate::utils::{AssetSchema, DescriptorType, DEFAULT_FEE_ABS};
struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    fn new() -> io::Result<Self> {
        let path = PathBuf::from("./tmp/test.lock");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        Ok(LockGuard { path })
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub struct TestRuntime {
    pub rt: RgbpRuntimeDir,
    signer: TestnetSigner,
    instance: u8,
    alias: String,
}

impl Deref for TestRuntime {
    type Target = RgbpRuntimeDir;
    fn deref(&self) -> &Self::Target {
        &self.rt
    }
}
impl DerefMut for TestRuntime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rt
    }
}

impl TestRuntime {
    pub fn new(descriptor_type: &DescriptorType, alias: &str) -> Self {
        Self::with(descriptor_type, INSTANCE_1, alias)
    }

    pub fn with(descriptor_type: &DescriptorType, instance: u8, alias: &str) -> Self {
        let mut seed = vec![0u8; 128];
        rand::thread_rng().fill_bytes(&mut seed);

        let xpriv_account = XprivAccount::with_seed(true, &seed).derive(h![86, 1, 0]);

        let fingerprint = xpriv_account.account_fp().to_string();
        let wallet_dir = PathBuf::from("tests")
            .join("test-data")
            .join(fingerprint.to_string());
        let alias = format!("{}-{}", alias, fingerprint);

        Self::_init(descriptor_type, wallet_dir, xpriv_account, instance, &alias)
    }

    fn _init(
        descriptor_type: &DescriptorType,
        wallet_dir: PathBuf,
        account: XprivAccount,
        instance: u8,
        alias: &str,
    ) -> Self {
        fs::create_dir_all(&wallet_dir).unwrap();
        println!("wallet dir: {wallet_dir:?}");

        let xpub = account.to_xpub_account();
        let xpub = XpubDerivable::with(xpub, &[Keychain::OUTER, Keychain::INNER]);
        let signer = TestnetSigner::new(account);

        let stockpile = StockpileDir::load(wallet_dir.clone(), Consensus::Bitcoin, true)
            .expect("Invalid contracts directory");
        let mut contracts = Contracts::load(stockpile);
        let issuer = Schema::load("tests/fixtures/RGB20-NFA.issuer").unwrap();
        contracts.import(issuer).unwrap();
        let issuer = Schema::load("tests/fixtures/RGB25-FUA.issuer").unwrap();
        contracts.import(issuer).unwrap();

        let provider = FsTextStore::new(wallet_dir).expect("Broken directory structure");
        let noise = xpub.xpub().chain_code().to_byte_array();
        let descr = match descriptor_type {
            DescriptorType::Wpkh => RgbDescr::new_unfunded(Wpkh::from(xpub), noise),
            DescriptorType::Tr => RgbDescr::key_only_unfunded(xpub, noise),
        };
        let wallet = Owner::create(provider, descr, Network::Regtest, true).unwrap();
        let rt = RgbpRuntimeDir::from(RgbWallet::with(wallet, contracts));

        let mut me = Self {
            rt,
            signer,
            instance,
            alias: alias.to_string(),
        };
        me.sync();
        me
    }

    pub fn get_address(&self) -> Address {
        self.wallet
            .addresses(Keychain::OUTER)
            .next()
            .expect("no addresses left")
            .addr
    }

    pub fn get_utxo(&mut self, sats: Option<u64>) -> Outpoint {
        let address = self.get_address();
        let txid = Txid::from_str(&fund_wallet(address.to_string(), sats, self.instance)).unwrap();
        self.sync();
        let mut vout = None;
        let coins = self.wallet.address_coins();
        assert!(!coins.is_empty());
        for (_derived_addr, utxos) in coins {
            for utxo in utxos {
                if utxo.outpoint.txid == txid {
                    vout = Some(utxo.outpoint.vout_u32());
                }
            }
        }
        Outpoint {
            txid,
            vout: Vout::from_u32(vout.unwrap()),
        }
    }

    pub fn issue_nia(
        &mut self,
        name: &'static str,
        issued_supply: u64,
        outpoint: Outpoint,
    ) -> ContractId {
        let params = CreateParams {
            codex_id: CodexId::from_str(
                "qaeakTdk-FccgZC9-4yYpoHa-quPSbQL-XmyBxtn-2CpD~38#jackson-couple-oberon",
            )
            .unwrap(),
            consensus: Consensus::Bitcoin,
            testnet: true,
            method: vname!("issue"),
            name: tn!(name),
            timestamp: None,
            global: vec![
                // TODO: simplify API for named state creation
                NamedState {
                    name: vname!("name"),
                    state: StateAtom {
                        verified: svstr!(name),
                        unverified: None,
                    },
                },
                NamedState {
                    name: vname!("ticker"),
                    state: StateAtom {
                        verified: svstr!("NIA"),
                        unverified: None,
                    },
                },
                NamedState {
                    name: vname!("precision"),
                    state: StateAtom {
                        verified: svenum!(centiMilli),
                        unverified: None,
                    },
                },
                NamedState {
                    name: vname!("circulating"),
                    state: StateAtom {
                        verified: svnum!(issued_supply),
                        unverified: None,
                    },
                },
            ],
            owned: vec![NamedState {
                name: vname!("owned"),
                state: Assignment {
                    seal: EitherSeal::Alt(outpoint),
                    data: svnum!(issued_supply),
                },
            }],
        };
        self.rt.issue(params).unwrap()
    }

    pub fn issue_cfa(
        &mut self,
        name: &'static str,
        issued_supply: u64,
        outpoint: Outpoint,
    ) -> ContractId {
        let params = CreateParams {
            codex_id: CodexId::from_str(
                "6bl9LdZ_-BU8Skh9-f~4UazR-TFwyotq-ac4yebi-zodXJnw#weather-motif-patriot",
            )
            .unwrap(),
            consensus: Consensus::Bitcoin,
            testnet: true,
            method: vname!("issue"),
            name: tn!(name),
            timestamp: None,
            global: vec![
                // TODO: simplify API for named state creation
                NamedState {
                    name: vname!("name"),
                    state: StateAtom {
                        verified: svstr!(name),
                        unverified: None,
                    },
                },
                NamedState {
                    name: vname!("details"),
                    state: StateAtom {
                        verified: StrictVal::Unit,
                        unverified: Some(svstr!("Demo CFA asset")),
                    },
                },
                NamedState {
                    name: vname!("precision"),
                    state: StateAtom {
                        verified: svenum!(centiMilli),
                        unverified: None,
                    },
                },
                NamedState {
                    name: vname!("circulating"),
                    state: StateAtom {
                        verified: svnum!(issued_supply),
                        unverified: None,
                    },
                },
            ],
            owned: vec![NamedState {
                name: vname!("owned"),
                state: Assignment {
                    seal: EitherSeal::Alt(outpoint),
                    data: svnum!(issued_supply),
                },
            }],
        };
        self.rt.issue(params).unwrap()
    }

    pub fn issue_cfa_with_allocations(
        &mut self,
        name: &'static str,
        allocations: Vec<(Outpoint, u64)>,
    ) -> ContractId {
        let total_supply: u64 = allocations.iter().map(|(_, amt)| amt).sum();
        let params = CreateParams {
            codex_id: CodexId::from_str(
                "6bl9LdZ_-BU8Skh9-f~4UazR-TFwyotq-ac4yebi-zodXJnw#weather-motif-patriot",
            )
            .unwrap(),
            consensus: Consensus::Bitcoin,
            testnet: true,
            method: vname!("issue"),
            name: tn!(name),
            timestamp: None,
            global: vec![
                NamedState {
                    name: vname!("name"),
                    state: StateAtom {
                        verified: svstr!(name),
                        unverified: None,
                    },
                },
                NamedState {
                    name: vname!("details"),
                    state: StateAtom {
                        verified: StrictVal::Unit,
                        unverified: Some(svstr!("Demo CFA asset")),
                    },
                },
                NamedState {
                    name: vname!("precision"),
                    state: StateAtom {
                        verified: svenum!(centiMilli),
                        unverified: None,
                    },
                },
                NamedState {
                    name: vname!("circulating"),
                    state: StateAtom {
                        verified: svnum!(total_supply),
                        unverified: None,
                    },
                },
            ],
            owned: allocations
                .into_iter()
                .map(|(outpoint, amount)| NamedState {
                    name: vname!("owned"),
                    state: Assignment {
                        seal: EitherSeal::Alt(outpoint),
                        data: svnum!(amount),
                    },
                })
                .collect(),
        };
        self.rt.issue(params).unwrap()
    }

    pub fn issue_nia_with_allocations(
        &mut self,
        name: &'static str,
        allocations: Vec<(Outpoint, u64)>,
    ) -> ContractId {
        let total_supply: u64 = allocations.iter().map(|(_, amt)| amt).sum();
        let params = CreateParams {
            codex_id: CodexId::from_str(
                "qaeakTdk-FccgZC9-4yYpoHa-quPSbQL-XmyBxtn-2CpD~38#jackson-couple-oberon",
            )
            .unwrap(),
            consensus: Consensus::Bitcoin,
            testnet: true,
            method: vname!("issue"),
            name: tn!(name),
            timestamp: None,
            global: vec![
                NamedState {
                    name: vname!("name"),
                    state: StateAtom {
                        verified: svstr!(name),
                        unverified: None,
                    },
                },
                NamedState {
                    name: vname!("ticker"),
                    state: StateAtom {
                        verified: svstr!("NIA"),
                        unverified: None,
                    },
                },
                NamedState {
                    name: vname!("precision"),
                    state: StateAtom {
                        verified: svenum!(centiMilli),
                        unverified: None,
                    },
                },
                NamedState {
                    name: vname!("circulating"),
                    state: StateAtom {
                        verified: svnum!(total_supply),
                        unverified: None,
                    },
                },
            ],
            owned: allocations
                .into_iter()
                .map(|(outpoint, amount)| NamedState {
                    name: vname!("owned"),
                    state: Assignment {
                        seal: EitherSeal::Alt(outpoint),
                        data: svnum!(amount),
                    },
                })
                .collect(),
        };
        self.rt.issue(params).unwrap()
    }

    pub fn invoice(
        &mut self,
        contract_id: ContractId,
        amount: u64,
        wout: bool,
    ) -> RgbInvoice<ContractId> {
        let beneficiary = if wout {
            let wout = self.rt.wout(None);
            RgbBeneficiary::WitnessOut(wout)
        } else {
            self.sync();
            let auth = self
                .rt
                .auth_token(None)
                .expect("no auth token, you need to generate some utxos");
            RgbBeneficiary::Token(auth)
        };
        let value = StrictVal::num(amount);
        RgbInvoice::new(contract_id, beneficiary, Some(value))
    }

    pub fn send(
        &mut self,
        recv_wlt: &mut TestRuntime,
        wout: bool,
        contract_id: ContractId,
        amount: u64,
        sats: u64,
        report: Option<&Report>,
    ) -> (PathBuf, Tx) {
        let invoice: rgb::CallRequest<ContractId, RgbBeneficiary> =
            recv_wlt.invoice(contract_id, amount, wout);
        self.send_to_invoice(recv_wlt, invoice, Some(sats), None, report)
    }

    pub fn send_to_invoice(
        &mut self,
        recv_wlt: &mut TestRuntime,
        invoice: RgbInvoice<ContractId>,
        sats: Option<u64>,
        fee: Option<u64>,
        report: Option<&Report>,
    ) -> (PathBuf, Tx) {
        let (consignment, tx) = self.transfer(invoice, sats, fee, true, report);
        self.mine_tx(tx.txid(), false);
        recv_wlt.accept_transfer(&consignment, report);
        self.sync();
        (consignment, tx)
    }

    pub fn transfer(
        &mut self,
        invoice: RgbInvoice<ContractId>,
        sats: Option<u64>,
        fee: Option<u64>,
        broadcast: bool,
        report: Option<&Report>,
    ) -> (PathBuf, Tx) {
        static COUNTER: OnceLock<AtomicU32> = OnceLock::new();

        let counter = COUNTER.get_or_init(|| AtomicU32::new(0));
        counter.fetch_add(1, Ordering::SeqCst);
        let consignment_no = counter.load(Ordering::SeqCst);

        self.sync();

        let fee = Sats::from_sats(fee.unwrap_or(DEFAULT_FEE_ABS));
        let sats = Sats::from_sats(sats.unwrap_or(2000));
        let strategy = CoinselectStrategy::Aggregate;
        let pay_start = Instant::now();
        let params = TxParams::with(fee);
        let (mut psbt, payment) = self
            .pay_invoice(&invoice, strategy, params, Some(sats))
            .unwrap();

        let pay_duration = pay_start.elapsed();
        if let Some(report) = report {
            report.write_duration(pay_duration);
        }

        let tx = self.sign_finalize_extract(&mut psbt);

        println!(
            "transfer txid: {}, consignment: {consignment_no}",
            tx.txid()
        );

        if broadcast {
            self.broadcast_tx(&tx);
        }

        let consignment = PathBuf::new()
            .join("tests")
            .join("test-data")
            .join(format!("consignment-{consignment_no}"))
            .with_extension("rgb");
        self.rt
            .contracts
            .consign_to_file(&consignment, invoice.scope, payment.terminals)
            .unwrap();

        (consignment, tx)
    }

    pub fn accept_transfer(&mut self, consignment: &Path, report: Option<&Report>) {
        self.sync();
        let accept_start = Instant::now();
        self.consume_from_file(consignment)
            .unwrap_or_else(|e| panic!("{e}"));
        let accept_duration = accept_start.elapsed();
        if let Some(report) = report {
            report.write_duration(accept_duration);
        }
    }

    pub fn check_allocations(
        &mut self,
        contract_id: ContractId,
        asset_schema: AssetSchema,
        mut expected_fungible_allocations: Vec<u64>,
        nonfungible_allocation: bool,
    ) {
        match asset_schema {
            AssetSchema::Nia | AssetSchema::Cfa => {
                let state = self.rt.state_own(contract_id);
                let mut actual_fungible_allocations = state
                    .owned
                    .get("owned")
                    .unwrap()
                    .iter()
                    .map(|(_, owned)| owned.assignment.data.unwrap_num().unwrap_uint::<u64>())
                    .collect::<Vec<_>>();
                actual_fungible_allocations.sort();
                expected_fungible_allocations.sort();
                println!(
                    "{}: actual fungible allocations: {:?}, expected fungible allocations: {:?}",
                    self.alias, actual_fungible_allocations, expected_fungible_allocations
                );
                assert_eq!(actual_fungible_allocations, expected_fungible_allocations);
            }
            AssetSchema::Uda => {
                todo!()
            }
        }
    }
    pub fn sync(&mut self) {
        let _lock_guard = loop {
            match LockGuard::new() {
                Ok(guard) => break guard,
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    thread::sleep(Duration::from_secs(1));
                }
                Err(e) => panic!("cannot get lock: {}", e),
            }
        };

        let indexer = self.get_indexer();
        self.wallet.update(&indexer).into_result().unwrap();
    }

    pub fn network(&self) -> Network {
        self.wallet.network()
    }

    fn get_indexer(&self) -> AnyIndexer {
        get_indexer(&self.indexer_url())
    }

    pub fn indexer_url(&self) -> String {
        indexer_url(self.instance, self.network())
    }

    pub fn sign_finalize(&self, psbt: &mut Psbt) {
        let _sig_count = psbt.sign(&self.signer).unwrap();
        psbt.finalize(self.wallet.descriptor());
    }

    pub fn sign_finalize_extract(&self, psbt: &mut Psbt) -> Tx {
        self.sign_finalize(psbt);
        psbt.extract().unwrap()
    }

    pub fn mine_tx(&self, txid: Txid, resume: bool) {
        let mut attempts = 10;
        loop {
            mine_custom(resume, self.instance, 1);
            if is_tx_mined(txid, &self.get_indexer()) {
                break;
            }
            attempts -= 1;
            if attempts == 0 {
                panic!("TX is not getting mined");
            }
        }
    }

    pub fn broadcast_tx(&self, tx: &Tx) {
        broadcast_tx(tx, &self.indexer_url());
    }
}
