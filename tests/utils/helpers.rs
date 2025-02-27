use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use bp::{seals::WTxoSeal, Outpoint};
use psbt::TxParams;
use rand::RngCore;
use rgb::invoice::{RgbBeneficiary, RgbInvoice};
use rgb::popls::bp::file::{BpDirMound, DirBarrow};
use rgb::{
    Assignment, CallScope, CellAddr, CodexId, Consensus, ContractId, ContractInfo, CreateParams,
    EitherSeal, MethodName, NamedState, StateAtom,
};
use rgbp::CoinselectStrategy;
use rgbp::{descriptor::RgbDescr, RgbDirRuntime, RgbWallet};
use strict_types::value::EnumTag;
use strict_types::{TypeName, VariantName};

use crate::utils::chain::fund_wallet;

use super::chain::is_tx_confirmed;
use super::{
    chain::{indexer_url, mine_custom, Indexer, INDEXER},
    *,
};

/// RGB Asset creation parameters builder
#[derive(Clone)]
pub struct AssetParamsBuilder {
    params: CreateParams<Outpoint>,
}

impl AssetParamsBuilder {
    /// Create a new builder instance for non-inflatable asset
    pub fn default_nia() -> Self {
        Self {
            params: Self::from_file(NON_INFLATABLE_ASSET_TEMPLATE_PATH),
        }
    }

    /// Create a new builder instance for collectible fungible asset
    pub fn default_cfa() -> Self {
        Self {
            params: Self::from_file(COLLECTIBLE_FUNGIBLE_ASSET_TEMPLATE_PATH),
        }
    }

    /// Load parameters from YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> CreateParams<Outpoint> {
        let file = File::open(path).expect("Unable to open file");
        let params: CreateParams<Outpoint> =
            serde_yaml::from_reader::<_, CreateParams<Outpoint>>(file).expect("");
        params
    }

    /// Set the contract template ID
    pub fn codex_id(mut self, codex_id: CodexId) -> Self {
        self.params.codex_id = codex_id;
        self
    }

    /// Set the consensus type
    pub fn consensus(mut self, consensus: Consensus) -> Self {
        self.params.consensus = consensus;
        self
    }

    /// Set whether it is a test network
    pub fn testnet(mut self, testnet: bool) -> Self {
        self.params.testnet = testnet;
        self
    }

    /// Set the contract method name
    pub fn method(mut self, method: &str) -> Self {
        self.params.method = VariantName::from_str(method).unwrap();
        self
    }

    /// Set the contract name
    pub fn name(mut self, name: &str) -> Self {
        self.params.name = TypeName::from_str(name).unwrap();
        self
    }

    /// Update name state in global states
    pub fn update_name_state(mut self, value: &str) -> Self {
        if let Some(state) = self
            .params
            .global
            .iter_mut()
            .find(|s| s.name == "name".into())
        {
            state.state.verified = value.into();
        }
        self
    }

    /// Update ticker state in global states
    pub fn update_ticker_state(mut self, value: &str) -> Self {
        if let Some(state) = self
            .params
            .global
            .iter_mut()
            .find(|s| s.name == "ticker".into())
        {
            state.state.verified = value.into();
        }
        self
    }

    /// Update precision state in global states
    pub fn update_precision_state(mut self, value: &str) -> Self {
        if let Some(state) = self
            .params
            .global
            .iter_mut()
            .find(|s| s.name == "precision".into())
        {
            state.state.verified = value.into();
        }
        self
    }

    /// Update circulating state in global states
    /// circulating type is "RGBContract.Amount" eq u64 in rust
    pub fn update_circulating_state(mut self, value: u64) -> Self {
        if let Some(state) = self
            .params
            .global
            .iter_mut()
            .find(|s| s.name == "circulating".into())
        {
            state.state.verified = value.into();
        }
        self
    }

    /// Update owned state
    pub fn update_owned_state(mut self, seal: Outpoint, val: u64) -> Self {
        // check if owned state exists
        if let Some(state) = self
            .params
            .owned
            .iter_mut()
            .find(|s| s.name == "owned".into())
        {
            // if exists, update seal and data
            state.state.seal = EitherSeal::Alt(seal);
            state.state.data = val.into();
        } else {
            // if not exists, create a new owned state
            self.params.owned.push(NamedState {
                name: "owned".into(),
                state: Assignment {
                    seal: EitherSeal::Alt(seal),
                    data: val.into(),
                },
            });
        }
        self
    }

    pub fn clear_owned_state(mut self) -> Self {
        self.params.owned.clear();
        self
    }

    /// Add owned state
    pub fn add_owned_state(mut self, seal: Outpoint, val: u64) -> Self {
        self.params.owned.push(NamedState {
            name: "owned".into(),
            state: Assignment {
                seal: EitherSeal::Alt(seal),
                data: val.into(),
            },
        });
        self
    }

    /// Build CreateParams instance
    pub fn build(self) -> CreateParams<Outpoint> {
        self.params
    }
}

pub struct TestWallet {
    runtime: RgbDirRuntime,
    descriptor: RgbDescr,
    signer: Option<TestnetSigner>,
    wallet_dir: PathBuf,
    instance: u8,
}

enum WalletAccount {
    Private(XprivAccount),
    Public(XpubAccount),
}

pub enum AllocationFilter {
    Stock,
    Wallet,
    WalletAll,
    WalletTentative,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DescriptorType {
    Wpkh,
    Tr,
}

impl fmt::Display for DescriptorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum HistoryType {
    Linear,
    Branching,
    Merging,
}

#[derive(Debug, Copy, Clone)]
pub enum ReorgType {
    ChangeOrder,
    Revert,
}

#[derive(Debug, Copy, Clone)]
pub enum TransferType {
    Blinded,
    Witness,
}

impl fmt::Display for TransferType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InvoiceType {
    Blinded(Option<Outpoint>),
    Witness,
}

impl From<TransferType> for InvoiceType {
    fn from(transfer_type: TransferType) -> Self {
        match transfer_type {
            TransferType::Blinded => InvoiceType::Blinded(None),
            TransferType::Witness => InvoiceType::Witness,
        }
    }
}

/// RGB asset-specific information to color a transaction
#[derive(Clone, Debug)]
pub struct AssetColoringInfo {
    /// Contract iface
    pub iface: TypeName,
    /// Input outpoints of the assets being spent
    pub input_outpoints: Vec<Outpoint>,
    /// Map of vouts and asset amounts to color the transaction outputs
    pub output_map: HashMap<u32, u64>,
    /// Static blinding to keep the transaction construction deterministic
    pub static_blinding: Option<u64>,
}

/// RGB information to color a transaction
#[derive(Clone, Debug)]
pub struct ColoringInfo {
    /// Asset-specific information
    pub asset_info_map: HashMap<ContractId, AssetColoringInfo>,
    /// Static blinding to keep the transaction construction deterministic
    pub static_blinding: Option<u64>,
    /// Nonce for offchain TXs ordering
    pub nonce: Option<u64>,
}

#[derive(Debug, EnumIter, Copy, Clone, PartialEq)]
pub enum AssetSchema {
    Nia,
    Uda,
    Cfa,
}

impl fmt::Display for AssetSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

pub struct Report {
    pub report_path: PathBuf,
}

impl Report {
    pub fn write_header(&self, fields: &[&str]) {
        let mut file = std::fs::File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&self.report_path)
            .unwrap();
        file.write_all(format!("{}\n", fields.join(";")).as_bytes())
            .unwrap();
    }

    pub fn write_duration(&self, duration: Duration) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.report_path)
            .unwrap();
        file.write_all(format!("{};", duration.as_millis()).as_bytes())
            .unwrap();
    }

    pub fn end_line(&self) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.report_path)
            .unwrap();
        file.write_all("\n".as_bytes()).unwrap();
    }
}

fn _get_wallet(
    descriptor_type: &DescriptorType,
    network: Network,
    wallet_dir: PathBuf,
    wallet_account: WalletAccount,
    instance: u8,
) -> TestWallet {
    std::fs::create_dir_all(&wallet_dir).unwrap();
    println!("wallet dir: {wallet_dir:?}");

    // create consensus_dir for managing RGB smart contracts
    let mut consensus_dir = wallet_dir.join(Consensus::Bitcoin.to_string());
    if network.is_testnet() {
        consensus_dir.set_extension("testnet");
    }
    std::fs::create_dir_all(&consensus_dir).unwrap();

    // copy schema files from template directory to wallet_dir
    let schemata_dir = PathBuf::from(SCHEMATA_DIR);
    if schemata_dir.exists() {
        for entry in std::fs::read_dir(schemata_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                std::fs::copy(&path, wallet_dir.join(path.file_name().unwrap())).unwrap();
            }
        }
    }

    let xpub_account = match wallet_account {
        WalletAccount::Private(ref xpriv_account) => xpriv_account.to_xpub_account(),
        WalletAccount::Public(ref xpub_account) => xpub_account.clone(),
    };
    let keychains: &[Keychain] = &[Keychain::INNER, Keychain::OUTER];
    let xpub_derivable = XpubDerivable::with(xpub_account.clone(), keychains);
    let noise = xpub_derivable.xpub().chain_code().to_byte_array();

    let descriptor = match descriptor_type {
        DescriptorType::Wpkh => RgbDescr::new_unfunded(Wpkh::from(xpub_derivable), noise),
        DescriptorType::Tr => RgbDescr::key_only_unfunded(xpub_derivable, noise),
    };

    let signer = match wallet_account {
        WalletAccount::Private(xpriv_account) => Some(TestnetSigner::new(xpriv_account)),
        WalletAccount::Public(_) => None,
    };

    let runtime = make_runtime(&descriptor, network, &wallet_dir);
    let mut test_wallet = TestWallet {
        runtime,
        descriptor,
        signer,
        wallet_dir,
        instance,
    };

    // TODO: remove if once found solution for esplora 'Too many requests' error
    if network.is_testnet() {
        test_wallet.sync();
    }

    test_wallet
}

fn make_runtime(descriptor: &RgbDescr, network: Network, wallet_dir: &PathBuf) -> RgbDirRuntime {
    let name = "bp_wallet.wallet";
    let provider = FsTextStore::new(wallet_dir.join(name)).unwrap();
    let wallet = RgbWallet::create(provider, descriptor.clone(), network, true)
        .expect("Unable to create wallet");

    let mound = BpDirMound::load_testnet(Consensus::Bitcoin, &wallet_dir, false);
    RgbDirRuntime::from(DirBarrow::with(wallet, mound))
}

pub fn get_wallet(descriptor_type: &DescriptorType) -> TestWallet {
    get_wallet_custom(descriptor_type, INSTANCE_1)
}

pub fn get_wallet_custom(descriptor_type: &DescriptorType, instance: u8) -> TestWallet {
    let mut seed = vec![0u8; 128];
    rand::thread_rng().fill_bytes(&mut seed);

    let xpriv_account = XprivAccount::with_seed(true, &seed).derive(h![86, 1, 0]);

    let fingerprint = xpriv_account.account_fp().to_string();
    let wallet_dir = PathBuf::from(TEST_DATA_DIR)
        .join(INTEGRATION_DATA_DIR)
        .join(fingerprint);

    _get_wallet(
        descriptor_type,
        Network::Regtest,
        wallet_dir,
        WalletAccount::Private(xpriv_account),
        instance,
    )
}

pub fn get_mainnet_wallet() -> TestWallet {
    let xpub_account = XpubAccount::from_str(
        "[c32338a7/86h/0h/0h]xpub6CmiK1xc7YwL472qm4zxeURFX8yMCSasioXujBjVMMzA3AKZr6KLQEmkzDge1Ezn2p43ZUysyx6gfajFVVnhtQ1AwbXEHrioLioXXgj2xW5"
    ).unwrap();

    let wallet_dir = PathBuf::from(TEST_DATA_DIR)
        .join(INTEGRATION_DATA_DIR)
        .join("mainnet");

    _get_wallet(
        &DescriptorType::Wpkh,
        Network::Mainnet,
        wallet_dir,
        WalletAccount::Public(xpub_account),
        INSTANCE_1,
    )
}

fn get_indexer(indexer_url: &str) -> AnyIndexer {
    match INDEXER.get().unwrap() {
        Indexer::Electrum => {
            AnyIndexer::Electrum(Box::new(ElectrumClient::new(indexer_url).unwrap()))
        }
        Indexer::Esplora => {
            AnyIndexer::Esplora(Box::new(EsploraClient::new_esplora(indexer_url).unwrap()))
        }
    }
}

fn broadcast_tx(tx: &Tx, indexer_url: &str) {
    match get_indexer(indexer_url) {
        AnyIndexer::Electrum(inner) => {
            inner.transaction_broadcast(tx).unwrap();
        }
        AnyIndexer::Esplora(inner) => {
            inner
                .broadcast(tx)
                .map_err(|e| {
                    dbg!(
                        tx.inputs.iter().map(|i| i.prev_output).collect::<Vec<_>>(),
                        &e
                    );
                    e
                })
                .unwrap();
        }
        _ => unreachable!("unsupported indexer"),
    }
}

pub fn broadcast_tx_and_mine(tx: &Tx, instance: u8) {
    broadcast_tx(tx, &indexer_url(instance, Network::Regtest));
    mine_custom(false, instance, 1);
}

impl TestWallet {
    pub fn network(&self) -> Network {
        self.runtime.wallet.network()
    }

    pub fn testnet(&self) -> bool {
        self.network().is_testnet()
    }

    pub fn get_derived_address(&self) -> DerivedAddr {
        self.runtime
            .wallet
            .addresses(Keychain::OUTER)
            .next()
            .expect("no addresses left")
    }

    pub fn get_address(&self) -> Address {
        self.get_derived_address().addr
    }

    pub fn get_utxo(&mut self, sats: Option<u64>) -> Outpoint {
        let address = self.get_address();
        let txid = Txid::from_str(&fund_wallet(address.to_string(), sats, self.instance)).unwrap();
        self.sync();
        let mut vout = None;
        let coins = self.runtime.wallet.address_coins();
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

    // TODO: Because the RGB mound currently cannot dynamically load contracts,
    // It needs to be reloaded at a special time, and consider submitting a PR to RGB
    pub fn reload_runtime(&mut self) {
        self.runtime = make_runtime(&self.descriptor, self.network(), &self.wallet_dir);
    }

    pub fn change_instance(&mut self, instance: u8) {
        self.instance = instance;
    }

    pub fn switch_to_instance(&mut self, instance: u8) {
        self.change_instance(instance);
    }

    pub fn indexer_url(&self) -> String {
        indexer_url(self.instance, self.network())
    }

    fn get_indexer(&self) -> AnyIndexer {
        get_indexer(&self.indexer_url())
    }

    pub fn broadcast_tx(&self, tx: &Tx) {
        broadcast_tx(tx, &self.indexer_url());
    }

    pub fn sync(&mut self) {
        let indexer = self.get_indexer();
        self.runtime.wallet.update(&indexer).into_result().unwrap();
    }

    pub fn runtime(&mut self) -> &mut RgbDirRuntime {
        &mut self.runtime
    }

    pub fn contracts_info(&self) -> Vec<ContractInfo> {
        self.runtime.mound.contracts_info().collect()
    }

    pub fn issue_with_params(&mut self, params: CreateParams<Outpoint>) -> ContractId {
        let contract_id = self
            .runtime
            .issue_to_file(params)
            .expect("failed to issue contract");
        println!("A new contract issued with ID {contract_id}");
        contract_id
    }

    pub fn issue_from_file(&mut self, params_path: impl AsRef<Path>) -> ContractId {
        let params = AssetParamsBuilder::from_file(params_path);
        self.issue_with_params(params)
    }

    pub fn mine_tx(&self, txid: &Txid, resume: bool) {
        let mut attempts = 10;
        loop {
            mine_custom(resume, self.instance, 1);
            if is_tx_confirmed(&txid.to_string(), self.instance) {
                break;
            }
            attempts -= 1;
            if attempts == 0 {
                panic!("TX is not getting mined");
            }
        }
    }

    pub fn send_contract(&mut self, contract_name: &str, to_wallet: &mut TestWallet) {
        let mut src_consensus_dir = self.wallet_dir.join(Consensus::Bitcoin.to_string());
        let mut dst_consensus_dir = to_wallet.wallet_dir.join(Consensus::Bitcoin.to_string());
        if self.network().is_testnet() {
            src_consensus_dir.set_extension("testnet");
            dst_consensus_dir.set_extension("testnet");
        }
        let src_contract_dir = src_consensus_dir.join(format!("{contract_name}.contract"));
        let dst_contract_dir = dst_consensus_dir.join(format!("{contract_name}.contract"));
        std::fs::create_dir_all(&dst_contract_dir).unwrap();
        let read_dir = std::fs::read_dir(&src_contract_dir).unwrap();
        for entry in read_dir {
            let entry = entry.unwrap();
            let path = entry.path();
            let dst_path = dst_contract_dir.join(path.file_name().unwrap());
            std::fs::copy(&path, &dst_path).unwrap();
        }
    }

    // force_create_utxo: Whether to force create a new UTXO.
    // Note: This will mine a block.
    pub fn invoice(
        &mut self,
        contract_id: ContractId,
        amount: u64,
        wout: bool,
        nonce: Option<u64>,
        force_create_utxo: bool,
    ) -> RgbInvoice<ContractId> {
        let beneficiary = if wout {
            let wout = self.runtime.wout(nonce);
            RgbBeneficiary::WitnessOut(wout)
        } else {
            if force_create_utxo {
                // Create a new UTXO if requested.
                // Required for auth token generation when wout=false.
                let _ = self.get_utxo(None);
            }
            // dbg!(self.runtime.wallet.utxos().collect::<Vec<_>>());
            // because auth_token needs a UTXO, so we need to fund `(send+mine)` the wallet first
            // FIXME: Design an `auth_token` that can customize the UTXO
            let auth = self.runtime.auth_token(nonce).unwrap();
            RgbBeneficiary::Token(auth)
        };
        let value = StrictVal::num(amount);
        RgbInvoice::new(contract_id, beneficiary, Some(value))
    }

    pub fn send(
        &mut self,
        recv_wallet: &mut TestWallet,
        wout: bool,
        contract_id: ContractId,
        amount: u64,
        sats: u64,
        nonce: Option<u64>,
        report: Option<&Report>,
    ) -> (PathBuf, Tx) {
        let invoice = recv_wallet.invoice(contract_id, amount, wout, nonce, true);
        self.send_to_invoice(recv_wallet, invoice, Some(sats), None, report)
    }

    pub fn send_to_invoice(
        &mut self,
        recv_wallet: &mut TestWallet,
        invoice: RgbInvoice<ContractId>,
        sats: Option<u64>,
        fee: Option<u64>,
        report: Option<&Report>,
    ) -> (PathBuf, Tx) {
        let (consignment, tx) = self.transfer(invoice, sats, fee, true, report);
        broadcast_tx_and_mine(&tx, self.instance);
        recv_wallet.accept_transfer(&consignment, report);
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
        let (mut psbt, terminal) = self
            .runtime
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

        let consignment = self
            .wallet_dir
            .join(format!("consignment-{consignment_no}"))
            .with_extension("rgb");

        self.runtime
            .mound
            .consign_to_file(invoice.scope, [terminal], &consignment)
            .unwrap();

        (consignment, tx)
    }

    pub fn accept_transfer(&mut self, consignment: &Path, report: Option<&Report>) {
        self.sync();
        let accept_start = Instant::now();
        self.runtime.consume_from_file(consignment).unwrap();
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
    ) {
        match asset_schema {
            AssetSchema::Nia | AssetSchema::Cfa => {
                let state = self.runtime.state_own(Some(contract_id)).next().unwrap().1;
                let mut actual_fungible_allocations = state
                    .owned
                    .get("owned")
                    .unwrap()
                    .iter()
                    .map(|(_, assignment)| assignment.data.unwrap_num().unwrap_uint::<u64>())
                    .collect::<Vec<_>>();
                actual_fungible_allocations.sort();
                expected_fungible_allocations.sort();
                assert_eq!(actual_fungible_allocations, expected_fungible_allocations);
            }
            AssetSchema::Uda => {
                todo!()
            }
        }
    }

    pub fn sign_finalize(&self, psbt: &mut Psbt) {
        let _sig_count = psbt.sign(self.signer.as_ref().unwrap()).unwrap();
        psbt.finalize(self.runtime.wallet.descriptor());
    }

    pub fn sign_finalize_extract(&self, psbt: &mut Psbt) -> Tx {
        self.sign_finalize(psbt);
        psbt.extract().unwrap()
    }
}

/// Parameters for NIA (Non-Inflatable Asset) issuance
#[derive(Clone)]
pub struct NIAIssueParams {
    pub name: String,
    pub ticker: String,
    pub precision: String,
    pub circulating_supply: u64,
    pub initial_allocations: Vec<(Outpoint, u64)>,
}

impl Default for NIAIssueParams {
    fn default() -> Self {
        Self {
            name: "USD Tether".to_string(),
            ticker: "USDT".to_string(),
            precision: "centiMilli".to_string(),
            circulating_supply: 1_000_000,
            initial_allocations: vec![],
        }
    }
}

impl NIAIssueParams {
    pub fn new(
        name: impl Into<String>,
        ticker: impl Into<String>,
        precision: impl Into<String>,
        circulating_supply: u64,
    ) -> Self {
        Self {
            name: name.into(),
            ticker: ticker.into(),
            precision: precision.into(),
            circulating_supply,
            initial_allocations: vec![],
        }
    }

    pub fn add_allocation(&mut self, outpoint: Outpoint, amount: u64) -> &mut Self {
        self.initial_allocations.push((outpoint, amount));
        self
    }
}

/// RGB Contract State representation
#[derive(Debug)]
pub struct ContractState {
    /// Immutable state of the contract
    pub immutable: ContractImmutableState,
    /// Ownership state of the contract
    pub owned: ContractOwnedState,
}

/// Contract's immutable state
#[derive(Debug)]
pub struct ContractImmutableState {
    pub name: String,
    pub ticker: String,
    pub precision: String,
    pub circulating_supply: u64,
}

/// Contract's ownership state
#[derive(Debug)]
pub struct ContractOwnedState {
    pub allocations: Vec<(Outpoint, u64)>,
}

impl TestWallet {
    pub fn issue_nia_with_params(&mut self, params: NIAIssueParams) -> ContractId {
        let mut builder: AssetParamsBuilder = AssetParamsBuilder::default_nia()
            .name(params.name.as_str())
            .update_name_state(params.name.as_str())
            .update_ticker_state(params.ticker.as_str())
            .update_precision_state(params.precision.as_str())
            .update_circulating_state(params.circulating_supply)
            .clear_owned_state();

        for (outpoint, amount) in params.initial_allocations {
            builder = builder.add_owned_state(outpoint, amount);
        }

        self.issue_with_params(builder.build())
    }

    /// Get contract state (internal implementation)
    fn contract_state_internal(
        &mut self,
        contract_id: ContractId,
    ) -> Option<(
        BTreeMap<VariantName, BTreeMap<CellAddr, StateAtom>>,
        BTreeMap<VariantName, BTreeMap<CellAddr, Assignment<Outpoint>>>,
        BTreeMap<VariantName, StrictVal>,
    )> {
        self.runtime()
            .state_all(Some(contract_id))
            .next()
            .map(|(_, state)| {
                (
                    state.immutable.clone(),
                    state.owned.clone(),
                    state.computed.clone(),
                )
            })
    }

    /// Get contract state with parsed data structures
    pub fn contract_state(&mut self, contract_id: ContractId) -> Option<ContractState> {
        self.contract_state_internal(contract_id)
            .map(|(immutable, owned, computed)| {
                // Parse immutable state
                let name = immutable
                    .get(&VariantName::from_str("name").unwrap())
                    .and_then(|m| m.values().next())
                    .map(|v| v.verified.unwrap_string())
                    .unwrap_or_default();

                let ticker = immutable
                    .get(&VariantName::from_str("ticker").unwrap())
                    .and_then(|m| m.values().next())
                    .map(|v| v.verified.unwrap_string())
                    .unwrap_or_default();

                let precision = immutable
                    .get(&VariantName::from_str("precision").unwrap())
                    .and_then(|m| m.values().next())
                    .inspect(|v| {
                        dbg!(&v.verified);
                    })
                    .map(|v| {
                        let tag = v.verified.unwrap_enum_tag();
                        if let EnumTag::Name(name) = tag {
                            name.to_string()
                        } else {
                            "".to_string()
                        }
                    })
                    .unwrap_or_default();

                let circulating_supply = immutable
                    .get(&VariantName::from_str("circulating").unwrap())
                    .and_then(|m: &BTreeMap<CellAddr, StateAtom>| m.values().next())
                    .and_then(|v| Some(v.verified.unwrap_num().unwrap_uint::<u64>()))
                    .unwrap_or_default();

                // Parse ownership state
                let mut allocations = vec![];
                if let Some(owned_map) = owned.get(&VariantName::from_str("owned").unwrap()) {
                    for assignment in owned_map.values() {
                        allocations.push((
                            assignment.seal,
                            assignment.data.unwrap_num().unwrap_uint::<u64>(),
                        ));
                    }
                }

                ContractState {
                    immutable: ContractImmutableState {
                        name,
                        ticker,
                        precision,
                        circulating_supply,
                    },
                    owned: ContractOwnedState { allocations },
                }
            })
    }
}

/// Parameters for CFA (Collectible Fungible Asset) issuance
#[derive(Clone)]
pub struct CFAIssueParams {
    /// Asset name
    pub name: String,
    /// Decimal precision for the asset
    pub precision: String,
    /// Total circulating supply
    pub circulating_supply: u64,
    /// Initial token allocations (outpoint, amount)
    pub initial_allocations: Vec<(Outpoint, u64)>,
}

impl Default for CFAIssueParams {
    fn default() -> Self {
        Self {
            name: "Demo CFA".to_string(),
            precision: "centiMilli".to_string(),
            circulating_supply: 10_000,
            initial_allocations: vec![],
        }
    }
}

impl CFAIssueParams {
    /// Create new CFA issuance parameters
    pub fn new(
        name: impl Into<String>,
        precision: impl Into<String>,
        circulating_supply: u64,
    ) -> Self {
        Self {
            name: name.into(),
            precision: precision.into(),
            circulating_supply,
            initial_allocations: vec![],
        }
    }

    /// Add a token allocation
    pub fn add_allocation(&mut self, outpoint: Outpoint, amount: u64) -> &mut Self {
        self.initial_allocations.push((outpoint, amount));
        self
    }
}

impl TestWallet {
    /// Issue a CFA contract with custom parameters
    pub fn issue_cfa_with_params(&mut self, params: CFAIssueParams) -> ContractId {
        let mut builder = AssetParamsBuilder::default_cfa()
            .name(params.name.as_str())
            .update_name_state(params.name.as_str())
            .update_precision_state(params.precision.as_str())
            .update_circulating_state(params.circulating_supply)
            .clear_owned_state();

        // Add initial allocations
        for (outpoint, amount) in params.initial_allocations {
            builder = builder.add_owned_state(outpoint, amount);
        }

        self.issue_with_params(builder.build())
    }
}
