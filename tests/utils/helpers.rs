use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

use bp::{seals::WTxoSeal, Outpoint};
use rand::RngCore;
use rgb::invoice::{RgbBeneficiary, RgbInvoice};
use rgb::popls::bp::file::{BpDirMound, DirBarrow};
use rgb::{
    Assignment, CallScope, CodexId, Consensus, ContractId, ContractInfo, CreateParams, EitherSeal,
    MethodName, NamedState, StateAtom,
};
use rgbp::{descriptor::RgbDescr, RgbDirRuntime, RgbWallet};
use strict_types::TypeName;

use crate::utils::chain::fund_wallet;

use super::{
    chain::{indexer_url, mine_custom, Indexer, INDEXER},
    *,
};

/// RGB Asset creation parameters builder
#[derive(Clone)]
pub struct AssetParamsBuilder {
    params: CreateParams<Outpoint>,
}

impl Default for AssetParamsBuilder {
    fn default() -> Self {
        Self {
            params: Self::from_file("./tests/DemoToken.yaml"),
        }
    }
}

impl AssetParamsBuilder {
    /// Create a new builder instance
    pub fn new() -> Self {
        Self::default()
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
    pub fn method(mut self, method: impl Into<MethodName>) -> Self {
        self.params.method = method.into();
        self
    }

    /// Set the contract name
    pub fn name(mut self, name: impl Into<TypeName>) -> Self {
        self.params.name = name.into();
        self
    }

    /// Update name state in global states
    pub fn update_name_state(mut self, value: impl Into<StrictVal>) -> Self {
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
    pub fn update_ticker_state(mut self, value: impl Into<StrictVal>) -> Self {
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
    pub fn update_precision_state(mut self, value: impl Into<StrictVal>) -> Self {
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
    pub fn update_circulating_state(mut self, value: impl Into<StrictVal>) -> Self {
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

    /// Add or update owned state
    pub fn add_owned_state(mut self, seal: Outpoint, val: impl Into<StrictVal>) -> Self {
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

    /// Build CreateParams instance
    pub fn build(self) -> CreateParams<Outpoint> {
        self.params
    }
}

pub struct TestWallet {
    // FIXME: should store runtime instead of wallet
    // when need wallet, use runtime.defer.wallet
    wallet: RgbWallet,
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

impl AssetSchema {
    // fn iface_type_name(&self) -> TypeName {
    //     tn!(match self {
    //         Self::Nia => "RGB20Fixed",
    //         Self::Uda => "RGB21Unique",
    //         Self::Cfa => "RGB25Base",
    //     })
    // }

    // fn schema(&self) -> Schema {
    //     match self {
    //         Self::Nia => NonInflatableAsset::schema(),
    //         Self::Uda => UniqueDigitalAsset::schema(),
    //         Self::Cfa => CollectibleFungibleAsset::schema(),
    //     }
    // }

    // fn issue_impl(&self) -> IfaceImpl {
    //     match self {
    //         Self::Nia => NonInflatableAsset::issue_impl(),
    //         Self::Uda => UniqueDigitalAsset::issue_impl(),
    //         Self::Cfa => CollectibleFungibleAsset::issue_impl(),
    //     }
    // }

    // fn scripts(&self) -> Scripts {
    //     match self {
    //         Self::Nia => NonInflatableAsset::scripts(),
    //         Self::Uda => UniqueDigitalAsset::scripts(),
    //         Self::Cfa => CollectibleFungibleAsset::scripts(),
    //     }
    // }

    // fn types(&self) -> TypeSystem {
    //     match self {
    //         Self::Nia => NonInflatableAsset::types(),
    //         Self::Uda => UniqueDigitalAsset::types(),
    //         Self::Cfa => CollectibleFungibleAsset::types(),
    //     }
    // }

    // fn iface(&self) -> Iface {
    //     match self {
    //         Self::Nia => Rgb20::iface(&Rgb20::FIXED),
    //         Self::Uda => Rgb21::iface(&Rgb21::NONE),
    //         Self::Cfa => Rgb25::iface(&Rgb25::NONE),
    //     }
    // }

    // fn get_valid_kit(&self) -> ValidKit {
    //     let mut kit = Kit::default();
    //     kit.schemata.push(self.schema()).unwrap();
    //     kit.ifaces.push(self.iface()).unwrap();
    //     kit.iimpls.push(self.issue_impl()).unwrap();
    //     kit.scripts.extend(self.scripts().into_values()).unwrap();
    //     kit.types = self.types();
    //     kit.validate().unwrap()
    // }
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

    let xpub_account = match wallet_account {
        WalletAccount::Private(ref xpriv_account) => xpriv_account.to_xpub_account(),
        WalletAccount::Public(ref xpub_account) => xpub_account.clone(),
    };
    const OPRET_KEYCHAINS: [Keychain; 3] = [
        Keychain::INNER,
        Keychain::OUTER,
        Keychain::with(KEY_CHAIN_RGB),
    ];
    const TAPRET_KEYCHAINS: [Keychain; 4] = [
        Keychain::INNER,
        Keychain::OUTER,
        Keychain::with(KEY_CHAIN_RGB),
        Keychain::with(KEY_CHAIN_TAPRET),
    ];
    let keychains: &[Keychain] = match *descriptor_type {
        DescriptorType::Tr => &TAPRET_KEYCHAINS[..],
        DescriptorType::Wpkh => &OPRET_KEYCHAINS[..],
    };
    let xpub_derivable = XpubDerivable::with(xpub_account.clone(), keychains);
    let noise = xpub_derivable.xpub().chain_code().to_byte_array();

    let descriptor = match descriptor_type {
        DescriptorType::Wpkh => RgbDescr::new_unfunded(Wpkh::from(xpub_derivable), noise),
        DescriptorType::Tr => RgbDescr::key_only_unfunded(xpub_derivable, noise),
    };

    let name = "bp_wallet_name";
    let provider = FsTextStore::new(wallet_dir.join(name)).unwrap();
    let wallet = RgbWallet::create(provider, descriptor.clone(), network, true)
        .expect("Unable to create wallet");

    // for asset_schema in AssetSchema::iter() {
    //     let valid_kit = asset_schema.get_valid_kit();
    //     wallet.stock_mut().import_kit(valid_kit).unwrap();
    // }

    let signer = match wallet_account {
        WalletAccount::Private(xpriv_account) => Some(TestnetSigner::new(xpriv_account)),
        WalletAccount::Public(_) => None,
    };

    let mut wallet = TestWallet {
        wallet,
        descriptor,
        signer,
        wallet_dir,
        instance,
    };

    // TODO: remove if once found solution for esplora 'Too many requests' error
    if network.is_testnet() {
        wallet.sync();
    }

    wallet
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
            inner.broadcast(tx).unwrap();
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
        self.wallet.network()
    }

    pub fn testnet(&self) -> bool {
        self.network().is_testnet()
    }

    pub fn keychain(&self) -> Keychain {
        // RgbKeychain::for_method(self.close_method())
        todo!()
    }

    pub fn get_derived_address(&self) -> DerivedAddr {
        self.wallet
            .addresses(self.keychain())
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

    pub fn change_instance(&mut self, instance: u8) {
        self.instance = instance;
    }

    pub fn switch_to_instance(&mut self, instance: u8) {
        self.change_instance(instance);
        // self.sync_and_update_witnesses(None);
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
        self.wallet.update(&indexer).into_result().unwrap();
    }

    pub fn wallet_provider(&self) -> FsTextStore {
        let name: &str = "bp_wallet_name";
        let provider = FsTextStore::new(self.wallet_dir.join(name)).unwrap();
        provider
    }

    pub fn wallet(&self) -> RgbWallet {
        let provider = self.wallet_provider();
        RgbWallet::load(provider, true).unwrap_or_else(|_| {
            panic!(
                "Error: unable to load wallet from path `{}`",
                self.wallet_dir.display()
            )
        })
    }

    pub fn runtime(&self) -> RgbDirRuntime {
        let wallet = self.wallet();
        let mound = self.mound();
        dbg!(&mound.schemata().collect::<Vec<_>>());
        let runtime = RgbDirRuntime::from(DirBarrow::with(wallet, mound));
        runtime
    }

    pub fn mound(&self) -> BpDirMound {
        if !self.network().is_testnet() {
            panic!("Non-testnet networks are not yet supported");
        }
        BpDirMound::load_testnet(Consensus::Bitcoin, "./tests", false)
    }

    pub fn contracts_info(&self) -> Vec<ContractInfo> {
        self.mound().contracts_info().collect()
    }

    pub fn contract_states(&self, contract_id: Option<ContractId>) {
        let mut runtime = self.runtime();
        let contract_states = runtime.state_all(contract_id).collect::<Vec<_>>();
        dbg!(contract_states);
    }

    pub fn issue_nia(&mut self) -> ContractId {
        let params = AssetParamsBuilder::default().name("USDT");
        // .update_circulating_state("1000000");
        self.issue_with_params(params.build())
    }

    pub fn issue_with_params(&mut self, params: CreateParams<Outpoint>) -> ContractId {
        let mut runtime = self.runtime();
        let contract_id = runtime
            .issue_to_file(params)
            .expect("failed to issue contract");
        println!("A new contract issued with ID {contract_id}");
        contract_id
    }

    pub fn issue_from_file(&mut self, params_path: impl AsRef<Path>) -> ContractId {
        let params = AssetParamsBuilder::from_file(params_path);
        self.issue_with_params(params)
    }
}
