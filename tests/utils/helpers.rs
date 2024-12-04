use super::*;

pub struct TestWallet {
    wallet: RgbWallet<Wallet<XpubDerivable, RgbDescr>>,
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

enum Filter<'w> {
    NoWallet,
    Wallet(&'w RgbWallet<Wallet<XpubDerivable, RgbDescr>>),
    WalletAll(&'w RgbWallet<Wallet<XpubDerivable, RgbDescr>>),
    WalletTentative(&'w RgbWallet<Wallet<XpubDerivable, RgbDescr>>),
}

impl AssignmentsFilter for Filter<'_> {
    fn should_include(&self, outpoint: impl Into<XOutpoint>, id: Option<XWitnessId>) -> bool {
        match self {
            Filter::Wallet(wallet) => wallet
                .wallet()
                .filter_unspent()
                .should_include(outpoint, id),
            Filter::WalletTentative(wallet) => wallet
                .wallet()
                .filter_outpoints()
                .should_include(outpoint, id),
            _ => true,
        }
    }
}
impl Filter<'_> {
    fn comment(&self, outpoint: XOutpoint) -> &'static str {
        let outpoint = outpoint
            .into_bp()
            .into_bitcoin()
            .expect("liquid is not yet supported");
        match self {
            Filter::Wallet(rgb) if rgb.wallet().is_unspent(outpoint) => "",
            Filter::WalletAll(rgb) | Filter::WalletTentative(rgb)
                if rgb.wallet().is_unspent(outpoint) =>
            {
                "-- unspent"
            }
            Filter::WalletAll(rgb) | Filter::WalletTentative(rgb)
                if rgb.wallet().has_outpoint(outpoint) =>
            {
                "-- spent"
            }
            _ => "-- third-party",
        }
    }
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

/// Map of contract ID and list of its beneficiaries
pub type AssetBeneficiariesMap = BTreeMap<ContractId, Vec<BuilderSeal<GraphSeal>>>;

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
    fn iface_type_name(&self) -> TypeName {
        tn!(match self {
            Self::Nia => "RGB20Fixed",
            Self::Uda => "RGB21Unique",
            Self::Cfa => "RGB25Base",
        })
    }

    fn schema(&self) -> Schema {
        match self {
            Self::Nia => NonInflatableAsset::schema(),
            Self::Uda => UniqueDigitalAsset::schema(),
            Self::Cfa => CollectibleFungibleAsset::schema(),
        }
    }

    fn issue_impl(&self) -> IfaceImpl {
        match self {
            Self::Nia => NonInflatableAsset::issue_impl(),
            Self::Uda => UniqueDigitalAsset::issue_impl(),
            Self::Cfa => CollectibleFungibleAsset::issue_impl(),
        }
    }

    fn scripts(&self) -> Scripts {
        match self {
            Self::Nia => NonInflatableAsset::scripts(),
            Self::Uda => UniqueDigitalAsset::scripts(),
            Self::Cfa => CollectibleFungibleAsset::scripts(),
        }
    }

    fn types(&self) -> TypeSystem {
        match self {
            Self::Nia => NonInflatableAsset::types(),
            Self::Uda => UniqueDigitalAsset::types(),
            Self::Cfa => CollectibleFungibleAsset::types(),
        }
    }

    fn iface(&self) -> Iface {
        match self {
            Self::Nia => Rgb20::iface(&Rgb20::FIXED),
            Self::Uda => Rgb21::iface(&Rgb21::NONE),
            Self::Cfa => Rgb25::iface(&Rgb25::NONE),
        }
    }

    fn get_valid_kit(&self) -> ValidKit {
        let mut kit = Kit::default();
        kit.schemata.push(self.schema()).unwrap();
        kit.ifaces.push(self.iface()).unwrap();
        kit.iimpls.push(self.issue_impl()).unwrap();
        kit.scripts.extend(self.scripts().into_values()).unwrap();
        kit.types = self.types();
        kit.validate().unwrap()
    }
}

#[derive(Debug)]
pub enum AssetInfo {
    Nia {
        spec: AssetSpec,
        terms: ContractTerms,
        issue_amounts: Vec<u64>,
    },
    Uda {
        spec: AssetSpec,
        terms: ContractTerms,
        token_data: TokenData,
    },
    Cfa {
        name: Name,
        precision: Precision,
        details: Option<Details>,
        terms: ContractTerms,
        issue_amounts: Vec<u64>,
    },
}

impl AssetInfo {
    fn asset_schema(&self) -> AssetSchema {
        match self {
            Self::Nia { .. } => AssetSchema::Nia,
            Self::Uda { .. } => AssetSchema::Uda,
            Self::Cfa { .. } => AssetSchema::Cfa,
        }
    }

    pub fn iface_type_name(&self) -> TypeName {
        self.asset_schema().iface_type_name()
    }

    pub fn schema(&self) -> Schema {
        self.asset_schema().schema()
    }

    pub fn issue_impl(&self) -> IfaceImpl {
        self.asset_schema().issue_impl()
    }

    pub fn scripts(&self) -> Scripts {
        self.asset_schema().scripts()
    }

    pub fn types(&self) -> TypeSystem {
        self.asset_schema().types()
    }

    pub fn iface(&self) -> Iface {
        self.asset_schema().iface()
    }

    pub fn default_cfa(issue_amounts: Vec<u64>) -> Self {
        AssetInfo::cfa("CFA asset name", 0, None, "CFA terms", None, issue_amounts)
    }

    pub fn default_nia(issue_amounts: Vec<u64>) -> Self {
        AssetInfo::nia(
            "NIATCKR",
            "NIA asset name",
            2,
            None,
            "NIA terms",
            None,
            issue_amounts,
        )
    }

    pub fn default_uda() -> Self {
        AssetInfo::uda(
            "UDATCKR",
            "UDA asset name",
            None,
            "NIA terms",
            None,
            uda_token_data_minimal(),
        )
    }

    pub fn nia(
        ticker: &str,
        name: &str,
        precision: u8,
        details: Option<&str>,
        terms_text: &str,
        terms_media_fpath: Option<&str>,
        issue_amounts: Vec<u64>,
    ) -> Self {
        let spec = AssetSpec::with(
            ticker,
            name,
            Precision::try_from(precision).unwrap(),
            details,
        )
        .unwrap();
        let text = RicardianContract::from_str(terms_text).unwrap();
        let attachment = terms_media_fpath.map(attachment_from_fpath);
        let terms = ContractTerms {
            text,
            media: attachment,
        };
        Self::Nia {
            spec,
            terms,
            issue_amounts,
        }
    }

    pub fn uda(
        ticker: &str,
        name: &str,
        details: Option<&str>,
        terms_text: &str,
        terms_media_fpath: Option<&str>,
        token_data: TokenData,
    ) -> AssetInfo {
        let spec = AssetSpec::with(ticker, name, Precision::try_from(0).unwrap(), details).unwrap();
        let text = RicardianContract::from_str(terms_text).unwrap();
        let attachment = terms_media_fpath.map(attachment_from_fpath);
        let terms = ContractTerms {
            text,
            media: attachment.clone(),
        };
        Self::Uda {
            spec,
            terms,
            token_data,
        }
    }

    pub fn cfa(
        name: &str,
        precision: u8,
        details: Option<&str>,
        terms_text: &str,
        terms_media_fpath: Option<&str>,
        issue_amounts: Vec<u64>,
    ) -> AssetInfo {
        let text = RicardianContract::from_str(terms_text).unwrap();
        let attachment = terms_media_fpath.map(attachment_from_fpath);
        let terms = ContractTerms {
            text,
            media: attachment,
        };
        Self::Cfa {
            name: Name::try_from(name.to_owned()).unwrap(),
            precision: Precision::try_from(precision).unwrap(),
            details: details.map(|d| Details::try_from(d.to_owned()).unwrap()),
            terms,
            issue_amounts,
        }
    }

    pub fn add_global_state(&self, mut builder: ContractBuilder) -> ContractBuilder {
        match self {
            Self::Nia {
                spec,
                terms,
                issue_amounts,
            } => builder
                .add_global_state("spec", spec.clone())
                .unwrap()
                .add_global_state("terms", terms.clone())
                .unwrap()
                .add_global_state(
                    "issuedSupply",
                    Amount::from(issue_amounts.iter().sum::<u64>()),
                )
                .unwrap(),
            Self::Uda {
                spec,
                terms,
                token_data,
            } => builder
                .add_global_state("spec", spec.clone())
                .unwrap()
                .add_global_state("terms", terms.clone())
                .unwrap()
                .add_global_state("tokens", token_data.clone())
                .unwrap(),
            Self::Cfa {
                name,
                precision,
                details,
                terms,
                issue_amounts: issued_supply,
            } => {
                builder = builder
                    .add_global_state("name", name.clone())
                    .unwrap()
                    .add_global_state("precision", *precision)
                    .unwrap()
                    .add_global_state("terms", terms.clone())
                    .unwrap()
                    .add_global_state(
                        "issuedSupply",
                        Amount::from(issued_supply.iter().sum::<u64>()),
                    )
                    .unwrap();
                if let Some(details) = details {
                    builder = builder
                        .add_global_state("details", details.clone())
                        .unwrap()
                }
                builder
            }
        }
    }

    pub fn add_asset_owner(
        &self,
        mut builder: ContractBuilder,
        close_method: CloseMethod,
        outpoints: Vec<Outpoint>,
        layer_1: Layer1,
    ) -> ContractBuilder {
        match self {
            Self::Nia { issue_amounts, .. } | Self::Cfa { issue_amounts, .. } => {
                for (amt, outpoint) in issue_amounts.iter().zip(outpoints.iter().cycle()) {
                    builder = builder
                        .add_fungible_state(
                            "assetOwner",
                            get_genesis_seal(close_method, *outpoint, layer_1),
                            *amt,
                        )
                        .unwrap();
                }
                builder
            }
            Self::Uda { token_data, .. } => {
                let fraction = OwnedFraction::from(1);
                let allocation = Allocation::with(token_data.index, fraction);
                builder
                    .add_data(
                        "assetOwner",
                        get_genesis_seal(close_method, outpoints[0], layer_1),
                        allocation,
                    )
                    .unwrap()
            }
        }
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

pub fn get_genesis_seal(
    close_method: CloseMethod,
    outpoint: Outpoint,
    layer_1: Layer1,
) -> BuilderSeal<BlindSeal<Txid>> {
    let blind_seal = match close_method {
        CloseMethod::TapretFirst => BlindSeal::tapret_first_rand(outpoint.txid, outpoint.vout),
        CloseMethod::OpretFirst => BlindSeal::opret_first_rand(outpoint.txid, outpoint.vout),
    };
    let genesis_seal = GenesisSeal::from(blind_seal);
    let seal: XChain<BlindSeal<Txid>> = XChain::with(layer_1, genesis_seal);
    BuilderSeal::from(seal)
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
        Keychain::with(RgbKeychain::Rgb as u8),
    ];
    const TAPRET_KEYCHAINS: [Keychain; 4] = [
        Keychain::INNER,
        Keychain::OUTER,
        Keychain::with(RgbKeychain::Rgb as u8),
        Keychain::with(RgbKeychain::Tapret as u8),
    ];
    let keychains: &[Keychain] = match *descriptor_type {
        DescriptorType::Tr => &TAPRET_KEYCHAINS[..],
        DescriptorType::Wpkh => &OPRET_KEYCHAINS[..],
    };
    let xpub_derivable = XpubDerivable::with(xpub_account.clone(), keychains);

    let descriptor = match descriptor_type {
        DescriptorType::Wpkh => RgbDescr::Wpkh(Wpkh::from(xpub_derivable)),
        DescriptorType::Tr => RgbDescr::TapretKey(TapretKey::from(xpub_derivable)),
    };

    let name = "bp_wallet_name";
    let mut bp_wallet = Wallet::new_layer1(descriptor.clone(), network);
    bp_wallet.set_name(name.to_string());
    let bp_dir = wallet_dir.join(name);
    let bp_wallet_provider = FsTextStore::new(bp_dir).unwrap();
    bp_wallet.make_persistent(bp_wallet_provider, true).unwrap();

    let stock_provider = FsBinStore::new(wallet_dir.clone()).unwrap();
    let mut stock = Stock::in_memory();
    stock.make_persistent(stock_provider, true).unwrap();
    let mut wallet = RgbWallet::new(stock, bp_wallet);

    for asset_schema in AssetSchema::iter() {
        let valid_kit = asset_schema.get_valid_kit();
        wallet.stock_mut().import_kit(valid_kit).unwrap();
    }

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

fn get_resolver(indexer_url: &str) -> AnyResolver {
    match INDEXER.get().unwrap() {
        Indexer::Electrum => AnyResolver::electrum_blocking(indexer_url, None).unwrap(),
        Indexer::Esplora => AnyResolver::esplora_blocking(indexer_url, None).unwrap(),
    }
}

fn broadcast_tx(tx: &Tx, indexer_url: &str) {
    match get_indexer(indexer_url) {
        AnyIndexer::Electrum(inner) => {
            inner.transaction_broadcast(tx).unwrap();
        }
        AnyIndexer::Esplora(inner) => {
            inner.publish(tx).unwrap();
        }
        _ => unreachable!("unsupported indexer"),
    }
}

pub fn broadcast_tx_and_mine(tx: &Tx, instance: u8) {
    broadcast_tx(tx, &indexer_url(instance, Network::Regtest));
    mine_custom(false, instance, 1);
}

pub fn attachment_from_fpath(fpath: &str) -> Attachment {
    let file_bytes = std::fs::read(fpath).unwrap();
    let file_hash: sha256::Hash = Hash::hash(&file_bytes[..]);
    let digest = file_hash.to_byte_array().into();
    let mime = FileFormat::from_file(fpath)
        .unwrap()
        .media_type()
        .to_string();
    let media_ty: &'static str = Box::leak(mime.clone().into_boxed_str());
    let media_type = MediaType::with(media_ty);
    Attachment {
        ty: media_type,
        digest,
    }
}

fn uda_token_data_minimal() -> TokenData {
    TokenData {
        index: TokenIndex::from(UDA_FIXED_INDEX),
        ..Default::default()
    }
}

pub fn uda_token_data(
    ticker: &str,
    name: &str,
    details: &str,
    preview: EmbeddedMedia,
    media: Attachment,
    attachments: BTreeMap<u8, Attachment>,
    reserves: ProofOfReserves,
) -> TokenData {
    let mut token_data = uda_token_data_minimal();
    token_data.preview = Some(preview);
    token_data.media = Some(media);
    token_data.attachments = Confined::try_from(attachments.clone()).unwrap();
    token_data.reserves = Some(reserves);
    token_data.ticker = Some(Ticker::try_from(ticker.to_string()).unwrap());
    token_data.name = Some(Name::try_from(name.to_string()).unwrap());
    token_data.details = Some(Details::try_from(details.to_string()).unwrap());
    token_data
}

impl TestWallet {
    pub fn network(&self) -> Network {
        self.wallet.wallet().network()
    }

    pub fn testnet(&self) -> bool {
        self.network().is_testnet()
    }

    pub fn keychain(&self) -> RgbKeychain {
        RgbKeychain::for_method(self.close_method())
    }

    pub fn get_derived_address(&self) -> DerivedAddr {
        self.wallet
            .wallet()
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
        let coins = self.wallet.wallet().address_coins();
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

    pub fn sync_and_update_witnesses(&mut self, after_height: Option<u32>) {
        self.sync();
        self.update_witnesses(after_height.unwrap_or(1));
    }

    pub fn switch_to_instance(&mut self, instance: u8) {
        self.change_instance(instance);
        self.sync_and_update_witnesses(None);
    }

    pub fn indexer_url(&self) -> String {
        indexer_url(self.instance, self.network())
    }

    fn get_indexer(&self) -> AnyIndexer {
        get_indexer(&self.indexer_url())
    }

    pub fn get_resolver(&self) -> AnyResolver {
        get_resolver(&self.indexer_url())
    }

    pub fn broadcast_tx(&self, tx: &Tx) {
        broadcast_tx(tx, &self.indexer_url());
    }

    pub fn get_witness_ord(&self, txid: &Txid) -> WitnessOrd {
        self.get_resolver()
            .resolve_pub_witness_ord(XWitnessId::Bitcoin(*txid))
            .unwrap()
    }

    pub fn get_tx_height(&self, txid: &Txid) -> Option<u32> {
        match self.get_witness_ord(txid) {
            WitnessOrd::Mined(witness_pos) => Some(witness_pos.height().get()),
            _ => None,
        }
    }

    pub fn sync(&mut self) {
        let indexer = self.get_indexer();
        self.wallet
            .wallet_mut()
            .update(&indexer)
            .into_result()
            .unwrap();
    }

    pub fn close_method(&self) -> CloseMethod {
        self.wallet.wallet().seal_close_method()
    }

    pub fn mine_tx(&self, txid: &Txid, resume: bool) {
        let mut attempts = 10;
        loop {
            mine_custom(resume, self.instance, 1);
            if self.get_tx_height(txid).is_some() {
                break;
            }
            attempts -= 1;
            if attempts == 0 {
                panic!("TX is not getting mined");
            }
        }
    }

    pub fn issue_with_info(
        &mut self,
        asset_info: AssetInfo,
        close_method: CloseMethod,
        outpoints: Vec<Option<Outpoint>>,
    ) -> (ContractId, TypeName) {
        let outpoints = if outpoints.is_empty() {
            vec![self.get_utxo(None)]
        } else {
            outpoints
                .into_iter()
                .map(|o| o.unwrap_or_else(|| self.get_utxo(None)))
                .collect()
        };

        let layer_1 = Layer1::Bitcoin;

        let mut builder = ContractBuilder::with(
            Identity::default(),
            asset_info.iface(),
            asset_info.schema(),
            asset_info.issue_impl(),
            asset_info.types(),
            asset_info.scripts(),
            layer_1,
        );

        builder = asset_info.add_global_state(builder);

        builder = asset_info.add_asset_owner(builder, close_method, outpoints, layer_1);

        let contract = builder.issue_contract().expect("failure issuing contract");
        let resolver = self.get_resolver();
        self.wallet
            .stock_mut()
            .import_contract(contract.clone(), resolver)
            .unwrap();

        (contract.contract_id(), asset_info.iface_type_name())
    }

    pub fn issue_nia(
        &mut self,
        issued_supply: u64,
        close_method: CloseMethod,
        outpoint: Option<&Outpoint>,
    ) -> (ContractId, TypeName) {
        let asset_info = AssetInfo::default_nia(vec![issued_supply]);
        self.issue_with_info(asset_info, close_method, vec![outpoint.copied()])
    }

    pub fn issue_uda(
        &mut self,
        close_method: CloseMethod,
        outpoint: Option<&Outpoint>,
    ) -> (ContractId, TypeName) {
        let asset_info = AssetInfo::default_uda();
        self.issue_with_info(asset_info, close_method, vec![outpoint.copied()])
    }

    pub fn issue_cfa(
        &mut self,
        issued_supply: u64,
        close_method: CloseMethod,
        outpoint: Option<&Outpoint>,
    ) -> (ContractId, TypeName) {
        let asset_info = AssetInfo::default_cfa(vec![issued_supply]);
        self.issue_with_info(asset_info, close_method, vec![outpoint.copied()])
    }

    pub fn invoice(
        &mut self,
        contract_id: ContractId,
        iface_type_name: &TypeName,
        amount: u64,
        close_method: CloseMethod,
        invoice_type: InvoiceType,
    ) -> RgbInvoice {
        let network = self.wallet.wallet().network();
        let beneficiary = match invoice_type {
            InvoiceType::Blinded(outpoint) => {
                let outpoint = if let Some(outpoint) = outpoint {
                    outpoint
                } else {
                    self.get_utxo(None)
                };
                let seal = XChain::Bitcoin(GraphSeal::new_random(
                    close_method,
                    outpoint.txid,
                    outpoint.vout,
                ));
                self.wallet.stock_mut().store_secret_seal(seal).unwrap();
                Beneficiary::BlindedSeal(*seal.to_secret_seal().as_reduced_unsafe())
            }
            InvoiceType::Witness => {
                let address = self.get_address();
                Beneficiary::WitnessVout(Pay2Vout {
                    address: address.payload,
                    method: close_method,
                })
            }
        };

        let mut builder = RgbInvoiceBuilder::new(XChainNet::bitcoin(network, beneficiary))
            .set_contract(contract_id)
            .set_interface(iface_type_name.clone());
        if *iface_type_name == AssetSchema::Uda.iface_type_name() {
            if amount != 1 {
                panic!("UDA amount must be 1");
            }
            builder = builder
                .clone()
                .set_allocation(UDA_FIXED_INDEX, amount)
                .unwrap();
        } else {
            builder = builder.clone().set_amount_raw(amount);
        }
        builder.finish()
    }

    pub fn sign_finalize(&self, psbt: &mut Psbt) {
        let _sig_count = psbt.sign(self.signer.as_ref().unwrap()).unwrap();
        psbt.finalize(&self.descriptor);
    }

    pub fn sign_finalize_extract(&self, psbt: &mut Psbt) -> Tx {
        self.sign_finalize(psbt);
        psbt.extract().unwrap()
    }

    pub fn transfer(
        &mut self,
        invoice: RgbInvoice,
        sats: Option<u64>,
        fee: Option<u64>,
        broadcast: bool,
        report: Option<&Report>,
    ) -> (Transfer, Tx) {
        self.sync();

        let fee = Sats::from_sats(fee.unwrap_or(DEFAULT_FEE_ABS));
        let sats = Sats::from_sats(sats.unwrap_or(2000));
        let params = TransferParams::with(fee, sats);
        let pay_start = Instant::now();
        let (mut psbt, _psbt_meta, consignment) = self.wallet.pay(&invoice, params).unwrap();
        let pay_duration = pay_start.elapsed();
        if let Some(report) = report {
            report.write_duration(pay_duration);
        }

        let mut cs_path = self.wallet_dir.join("consignments");
        std::fs::create_dir_all(&cs_path).unwrap();
        let consignment_id = consignment.consignment_id();
        cs_path.push(consignment_id.to_string());
        cs_path.set_extension("yaml");
        let mut file = std::fs::File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(cs_path)
            .unwrap();
        serde_yaml::to_writer(&mut file, &consignment).unwrap();

        let tx = self.sign_finalize_extract(&mut psbt);

        let txid = tx.txid().to_string();
        println!("transfer txid: {txid}, consignment: {consignment_id}");

        let mut tx_path = self.wallet_dir.join("transactions");
        std::fs::create_dir_all(&tx_path).unwrap();
        tx_path.push(&txid);
        tx_path.set_extension("yaml");
        let mut file = std::fs::File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(tx_path)
            .unwrap();
        serde_yaml::to_writer(&mut file, &tx).unwrap();
        writeln!(file, "\n---\n").unwrap();
        serde_yaml::to_writer(&mut file, &psbt).unwrap();

        if broadcast {
            self.broadcast_tx(&tx);
        }

        (consignment, tx)
    }

    pub fn accept_transfer(&mut self, consignment: Transfer, report: Option<&Report>) {
        let mut resolver = self.get_resolver();
        resolver.add_terminals(&consignment);
        self.accept_transfer_custom_resolver(consignment, report, &resolver);
    }

    pub fn accept_transfer_custom_resolver(
        &mut self,
        consignment: Transfer,
        report: Option<&Report>,
        resolver: &impl ResolveWitness,
    ) {
        self.sync();
        let validate_start = Instant::now();
        let validated_consignment = consignment
            .validate(&resolver, self.testnet())
            .map_err(|(status, _)| status)
            .unwrap();
        let validate_duration = validate_start.elapsed();
        if let Some(report) = report {
            report.write_duration(validate_duration);
        }

        let validation_status = validated_consignment.clone().into_validation_status();
        let validity = validation_status.validity();
        assert_eq!(validity, Validity::Valid);
        let accept_start = Instant::now();
        self.wallet
            .stock_mut()
            .accept_transfer(validated_consignment.clone(), &resolver)
            .unwrap();
        let accept_duration = accept_start.elapsed();
        if let Some(report) = report {
            report.write_duration(accept_duration);
        }
    }

    pub fn contract_iface(
        &self,
        contract_id: ContractId,
        iface_type_name: &TypeName,
    ) -> ContractIface<MemContract<&MemContractState>> {
        self.wallet
            .stock()
            .contract_iface(contract_id, iface_type_name.clone())
            .unwrap()
    }

    pub fn contract_iface_class<C: IfaceClass>(
        &self,
        contract_id: ContractId,
    ) -> C::Wrapper<MemContract<&MemContractState>> {
        self.wallet
            .stock()
            .contract_iface_class::<C>(contract_id)
            .unwrap()
    }

    pub fn contract_fungible_allocations(
        &self,
        contract_id: ContractId,
        iface_type_name: &TypeName,
        show_tentative: bool,
    ) -> Vec<FungibleAllocation> {
        let filter = if show_tentative {
            Filter::WalletTentative(&self.wallet)
        } else {
            Filter::Wallet(&self.wallet)
        };
        self.contract_iface(contract_id, iface_type_name)
            .fungible(fname!("assetOwner"), filter)
            .unwrap()
            .collect()
    }

    pub fn contract_data_allocations(
        &self,
        contract_id: ContractId,
        iface_type_name: &TypeName,
    ) -> Vec<DataAllocation> {
        self.contract_iface(contract_id, iface_type_name)
            .data(fname!("assetOwner"), Filter::Wallet(&self.wallet))
            .unwrap()
            .collect()
    }

    pub fn history(&self, contract_id: ContractId, iface_type_name: &TypeName) -> Vec<ContractOp> {
        self.wallet
            .history(contract_id, iface_type_name.clone())
            .unwrap()
    }

    pub fn list_contracts(&self) -> Vec<ContractInfo> {
        self.wallet.stock().contracts().unwrap().collect()
    }

    pub fn utxos(&self) -> Vec<WalletUtxo> {
        self.wallet.wallet().utxos().collect()
    }

    pub fn debug_contracts(&self) {
        println!("Contracts:");
        for info in self.list_contracts() {
            println!("{}", info.to_string().replace("\n", "\t"));
        }
    }

    pub fn debug_logs(
        &self,
        contract_id: ContractId,
        iface_type_name: &TypeName,
        filter: AllocationFilter,
    ) {
        let filter = match filter {
            AllocationFilter::WalletAll => Filter::WalletAll(&self.wallet),
            AllocationFilter::WalletTentative => Filter::WalletTentative(&self.wallet),
            AllocationFilter::Wallet => Filter::Wallet(&self.wallet),
            AllocationFilter::Stock => Filter::NoWallet,
        };

        let contract = self.contract_iface(contract_id, iface_type_name);

        println!("Global:");
        for global in &contract.iface.global_state {
            if let Ok(values) = contract.global(global.name.clone()) {
                for val in values {
                    println!("  {} := {}", global.name, val);
                }
            }
        }

        println!("\nOwned:");
        fn witness<S: KnownState>(
            allocation: &OutputAssignment<S>,
            contract: &ContractIface<MemContract<&MemContractState>>,
        ) -> String {
            allocation
                .witness
                .and_then(|w| contract.witness_info(w))
                .map(|info| format!("{} ({})", info.id, info.ord))
                .unwrap_or_else(|| s!("~"))
        }
        for owned in &contract.iface.assignments {
            println!("  State      \t{:78}\tWitness", "Seal");
            println!("  {}:", owned.name);
            if let Ok(allocations) = contract.fungible(owned.name.clone(), &filter) {
                for allocation in allocations {
                    println!(
                        "    {: >9}\t{}\t{} {}",
                        allocation.state.value(),
                        allocation.seal,
                        witness(&allocation, &contract),
                        filter.comment(allocation.seal.to_outpoint())
                    );
                }
            }
            if let Ok(allocations) = contract.data(owned.name.clone(), &filter) {
                for allocation in allocations {
                    println!(
                        "    {: >9}\t{}\t{} {}",
                        allocation.state,
                        allocation.seal,
                        witness(&allocation, &contract),
                        filter.comment(allocation.seal.to_outpoint())
                    );
                }
            }
            if let Ok(allocations) = contract.attachments(owned.name.clone(), &filter) {
                for allocation in allocations {
                    println!(
                        "    {: >9}\t{}\t{} {}",
                        allocation.state,
                        allocation.seal,
                        witness(&allocation, &contract),
                        filter.comment(allocation.seal.to_outpoint())
                    );
                }
            }
            if let Ok(allocations) = contract.rights(owned.name.clone(), &filter) {
                for allocation in allocations {
                    println!(
                        "    {: >9}\t{}\t{} {}",
                        "right",
                        allocation.seal,
                        witness(&allocation, &contract),
                        filter.comment(allocation.seal.to_outpoint())
                    );
                }
            }
        }

        let bp_runtime = self.wallet.wallet();
        println!("\nHeight\t{:>12}\t{:68}", "Amount, ṩ", "Outpoint");
        for (derived_addr, utxos) in bp_runtime.address_coins() {
            println!("{}\t{}", derived_addr.addr, derived_addr.terminal);
            for row in utxos {
                println!("{}\t{: >12}\t{:68}", row.height, row.amount, row.outpoint);
            }
            println!()
        }

        println!("\nWallet total balance: {} ṩ", bp_runtime.balance());
    }

    pub fn debug_history(
        &self,
        contract_id: ContractId,
        iface_type_name: &TypeName,
        details: bool,
    ) {
        let mut history = self.history(contract_id, iface_type_name);
        history.sort_by_key(|op| op.witness.map(|w| w.ord).unwrap_or(WitnessOrd::Archived));
        if details {
            println!("Operation\tValue    \tState\t{:78}\tWitness", "Seal");
        } else {
            println!("Operation\tValue    \t{:78}\tWitness", "Seal");
        }
        for ContractOp {
            direction,
            ty,
            opids,
            state,
            to,
            witness,
        } in history
        {
            print!("{:9}\t", direction.to_string());
            if let AllocatedState::Amount(amount) = state {
                print!("{: >9}", amount.value());
            } else {
                print!("{state:>9}");
            }
            if details {
                print!("\t{ty}");
            }
            println!(
                "\t{}\t{}",
                to.first().expect("at least one receiver is always present"),
                witness
                    .map(|info| format!("{} ({})", info.id, info.ord))
                    .unwrap_or_else(|| s!("~"))
            );
            if details {
                println!(
                    "\topid={}",
                    opids
                        .iter()
                        .map(OpId::to_string)
                        .collect::<Vec<_>>()
                        .join("\n\topid=")
                )
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn send(
        &mut self,
        recv_wlt: &mut TestWallet,
        transfer_type: TransferType,
        contract_id: ContractId,
        iface_type_name: &TypeName,
        amount: u64,
        sats: u64,
        report: Option<&Report>,
    ) -> (Transfer, Tx) {
        let invoice = recv_wlt.invoice(
            contract_id,
            iface_type_name,
            amount,
            recv_wlt.close_method(),
            transfer_type.into(),
        );
        self.send_to_invoice(recv_wlt, invoice, Some(sats), None, report)
    }

    pub fn send_to_invoice(
        &mut self,
        recv_wlt: &mut TestWallet,
        invoice: RgbInvoice,
        sats: Option<u64>,
        fee: Option<u64>,
        report: Option<&Report>,
    ) -> (Transfer, Tx) {
        let (consignment, tx) = self.transfer(invoice, sats, fee, true, report);
        self.mine_tx(&tx.txid(), false);
        recv_wlt.accept_transfer(consignment.clone(), report);
        self.sync();
        (consignment, tx)
    }

    pub fn check_allocations(
        &self,
        contract_id: ContractId,
        iface_type_name: &TypeName,
        asset_schema: AssetSchema,
        expected_fungible_allocations: Vec<u64>,
        nonfungible_allocation: bool,
    ) {
        match asset_schema {
            AssetSchema::Nia | AssetSchema::Cfa => {
                let allocations =
                    self.contract_fungible_allocations(contract_id, iface_type_name, false);
                let mut actual_fungible_allocations = allocations
                    .iter()
                    .map(|a| a.state.value())
                    .collect::<Vec<_>>();
                let mut expected_fungible_allocations = expected_fungible_allocations.clone();
                actual_fungible_allocations.sort();
                expected_fungible_allocations.sort();
                assert_eq!(actual_fungible_allocations, expected_fungible_allocations);
            }
            AssetSchema::Uda => {
                let allocations = self.contract_data_allocations(contract_id, iface_type_name);
                let expected_allocations = if nonfungible_allocation {
                    assert_eq!(
                        allocations
                            .iter()
                            .filter(|a| a.state.to_string() == "000000000100000000000000")
                            .count(),
                        1
                    );
                    1
                } else {
                    0
                };
                assert_eq!(allocations.len(), expected_allocations);
            }
        }
    }

    pub fn check_history_operation(
        &self,
        contract_id: &ContractId,
        iface_type_name: &TypeName,
        txid: Option<&Txid>,
        direction: OpDirection,
        amount: u64,
    ) {
        let operation = self
            .history(*contract_id, iface_type_name)
            .into_iter()
            .find(|co| {
                co.direction == direction
                    && co
                        .witness
                        .map_or(true, |w| Some(w.id.as_reduced_unsafe()) == txid)
            })
            .unwrap();
        assert!(matches!(operation.state, AllocatedState::Amount(amt) if amt.value() == amount));
    }

    fn _construct_psbt_offchain(
        &mut self,
        input_outpoints: Vec<(Outpoint, u64, Terminal)>,
        beneficiaries: Vec<&PsbtBeneficiary>,
        tx_params: TxParams,
    ) -> (Psbt, PsbtMeta) {
        let mut psbt = Psbt::create(PsbtVer::V2);

        for (outpoint, value, terminal) in input_outpoints {
            psbt.construct_input_expect(
                Prevout::new(outpoint, Sats::from(value)),
                self.wallet.wallet().descriptor(),
                terminal,
                tx_params.seq_no,
            );
        }
        if psbt.inputs().count() == 0 {
            panic!("no inputs");
        }

        let input_value = psbt.input_sum();
        let mut max = Vec::new();
        let mut output_value = Sats::ZERO;
        for beneficiary in beneficiaries {
            let amount = beneficiary.amount.unwrap_or(Sats::ZERO);
            output_value.checked_add_assign(amount).unwrap();
            let out = psbt.construct_output_expect(beneficiary.script_pubkey(), amount);
            if beneficiary.amount.is_max() {
                max.push(out.index());
            }
        }
        let mut remaining_value = input_value
            .checked_sub(output_value)
            .unwrap()
            .checked_sub(tx_params.fee)
            .unwrap();
        if !max.is_empty() {
            let portion = remaining_value / max.len();
            for out in psbt.outputs_mut() {
                if max.contains(&out.index()) {
                    out.amount = portion;
                }
            }
            remaining_value = Sats::ZERO;
        }

        let (change_vout, change_terminal) = if remaining_value > Sats::from(546u64) {
            let change_index = self
                .wallet
                .wallet_mut()
                .next_derivation_index(tx_params.change_keychain, tx_params.change_shift);
            let change_terminal = Terminal::new(tx_params.change_keychain, change_index);
            let change_vout = psbt
                .construct_change_expect(
                    self.wallet.wallet().descriptor(),
                    change_terminal,
                    remaining_value,
                )
                .index();
            (
                Some(Vout::from_u32(change_vout as u32)),
                Some(change_terminal),
            )
        } else {
            (None, None)
        };

        (
            psbt,
            PsbtMeta {
                change_vout,
                change_terminal,
            },
        )
    }

    fn _construct_beneficiaries(
        &self,
        beneficiaries: Vec<(Address, Option<u64>)>,
    ) -> Vec<PsbtBeneficiary> {
        beneficiaries
            .into_iter()
            .map(|(addr, amt)| {
                let payment = if let Some(amt) = amt {
                    Payment::Fixed(Sats::from_sats(amt))
                } else {
                    Payment::Max
                };
                PsbtBeneficiary::new(addr, payment)
            })
            .collect()
    }

    pub fn construct_psbt_offchain(
        &mut self,
        input_outpoints: Vec<(Outpoint, u64, Terminal)>,
        beneficiaries: Vec<(Address, Option<u64>)>,
        fee: Option<u64>,
    ) -> (Psbt, PsbtMeta) {
        let tx_params = TxParams::with(Sats::from_sats(fee.unwrap_or(DEFAULT_FEE_ABS)));
        let beneficiaries = self._construct_beneficiaries(beneficiaries);
        let beneficiaries: Vec<&PsbtBeneficiary> = beneficiaries.iter().collect();

        self._construct_psbt_offchain(input_outpoints, beneficiaries, tx_params)
    }

    pub fn construct_psbt(
        &mut self,
        input_outpoints: Vec<Outpoint>,
        beneficiaries: Vec<(Address, Option<u64>)>,
        fee: Option<u64>,
    ) -> (Psbt, PsbtMeta) {
        let tx_params = TxParams::with(Sats::from_sats(fee.unwrap_or(DEFAULT_FEE_ABS)));
        let beneficiaries = self._construct_beneficiaries(beneficiaries);
        let beneficiaries: Vec<&PsbtBeneficiary> = beneficiaries.iter().collect();

        self.wallet
            .wallet_mut()
            .construct_psbt(input_outpoints, beneficiaries, tx_params)
            .unwrap()
    }

    pub fn psbt_add_input(&self, psbt: &mut Psbt, utxo: Outpoint) {
        for account in self.descriptor.xpubs() {
            psbt.xpubs.insert(*account.xpub(), account.origin().clone());
        }
        let input = self.wallet.wallet().utxo(utxo).unwrap();
        psbt.construct_input_expect(
            input.to_prevout(),
            self.wallet.wallet().descriptor(),
            input.terminal,
            SeqNo::ZERO,
        );
    }

    pub fn color_psbt(
        &self,
        psbt: &mut Psbt,
        coloring_info: ColoringInfo,
    ) -> (Fascia, AssetBeneficiariesMap) {
        let asset_beneficiaries = self.color_psbt_init(psbt, coloring_info);
        psbt.complete_construction();
        let fascia = psbt.rgb_commit().unwrap();
        (fascia, asset_beneficiaries)
    }

    pub fn color_psbt_init(
        &self,
        psbt: &mut Psbt,
        coloring_info: ColoringInfo,
    ) -> AssetBeneficiariesMap {
        if !psbt.outputs().any(|o| o.script.is_op_return()) {
            let _output = psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO);
        }

        let prev_outputs = psbt
            .to_unsigned_tx()
            .inputs
            .iter()
            .map(|txin| txin.prev_output)
            .map(|outpoint| XOutpoint::from(XChain::Bitcoin(outpoint)))
            .collect::<HashSet<XOutpoint>>();

        let mut all_transitions: HashMap<ContractId, Transition> = HashMap::new();
        let mut asset_beneficiaries: AssetBeneficiariesMap = bmap![];
        let assignment_name = FieldName::from("assetOwner");

        for (contract_id, asset_coloring_info) in coloring_info.asset_info_map.clone() {
            let mut asset_transition_builder = self
                .wallet
                .stock()
                .transition_builder(contract_id, asset_coloring_info.iface, None::<&str>)
                .unwrap();
            let assignment_id = asset_transition_builder
                .assignments_type(&assignment_name)
                .unwrap();

            let mut asset_available_amt = 0;
            for (_, opout_state_map) in self
                .wallet
                .stock()
                .contract_assignments_for(
                    contract_id,
                    prev_outputs
                        .iter()
                        // only retrieve assignments for owned prevouts using coloring_info
                        .filter(|xop| {
                            coloring_info.asset_info_map[&contract_id]
                                .input_outpoints
                                .contains(xop.as_reduced_unsafe())
                        })
                        .copied(),
                )
                .unwrap()
            {
                for (opout, state) in opout_state_map {
                    if let PersistedState::Amount(amt, _, _) = &state {
                        asset_available_amt += amt.value();
                    }
                    asset_transition_builder =
                        asset_transition_builder.add_input(opout, state).unwrap();
                }
            }

            let mut beneficiaries = vec![];
            let mut sending_amt = 0;
            for (vout, amount) in asset_coloring_info.output_map {
                if amount == 0 {
                    continue;
                }
                sending_amt += amount;
                if vout as usize > psbt.outputs().count() {
                    panic!("invalid vout in output_map, does not exist in the given PSBT");
                }
                let graph_seal = if let Some(blinding) = asset_coloring_info.static_blinding {
                    GraphSeal::with_blinded_vout(CloseMethod::OpretFirst, vout, blinding)
                } else {
                    GraphSeal::new_random_vout(CloseMethod::OpretFirst, vout)
                };
                let seal = BuilderSeal::Revealed(XChain::with(Layer1::Bitcoin, graph_seal));
                beneficiaries.push(seal);

                let blinding_factor = if let Some(blinding) = asset_coloring_info.static_blinding {
                    let mut blinding_32_bytes: [u8; 32] = [0; 32];
                    blinding_32_bytes[0..8].copy_from_slice(&blinding.to_le_bytes());
                    BlindingFactor::try_from(blinding_32_bytes).unwrap()
                } else {
                    BlindingFactor::random()
                };
                asset_transition_builder = asset_transition_builder
                    .add_fungible_state_raw(assignment_id, seal, amount, blinding_factor)
                    .unwrap();
            }
            if sending_amt > asset_available_amt {
                panic!("total amount in output_map greater than available ({asset_available_amt})");
            }

            if let Some(nonce) = coloring_info.nonce {
                asset_transition_builder = asset_transition_builder.set_nonce(nonce);
            }

            let transition = asset_transition_builder.complete_transition().unwrap();
            all_transitions.insert(contract_id, transition);
            asset_beneficiaries.insert(contract_id, beneficiaries);
        }

        let (opreturn_index, _) = psbt
            .to_unsigned_tx()
            .outputs
            .iter()
            .enumerate()
            .find(|(_, o)| o.script_pubkey.is_op_return())
            .expect("psbt should have an op_return output");
        let (_, opreturn_output) = psbt
            .outputs_mut()
            .enumerate()
            .find(|(i, _)| i == &opreturn_index)
            .unwrap();
        opreturn_output.set_opret_host().unwrap();
        if let Some(blinding) = coloring_info.static_blinding {
            opreturn_output.set_mpc_entropy(blinding).unwrap();
        }

        let tx_inputs = psbt.clone().to_unsigned_tx().inputs;
        for (contract_id, transition) in all_transitions {
            for (input, txin) in psbt.inputs_mut().zip(&tx_inputs) {
                let prevout = txin.prev_output;
                let outpoint = Outpoint::new(prevout.txid.to_byte_array().into(), prevout.vout);
                if coloring_info
                    .asset_info_map
                    .clone()
                    .get(&contract_id)
                    .unwrap()
                    .input_outpoints
                    .contains(&outpoint)
                {
                    input
                        .set_rgb_consumer(contract_id, transition.id())
                        .unwrap();
                }
            }
            psbt.push_rgb_transition(transition, CloseMethod::OpretFirst)
                .unwrap();
        }

        asset_beneficiaries
    }

    pub fn consume_fascia(&mut self, fascia: Fascia, witness_txid: Txid) {
        struct FasciaResolver {
            witness_id: XWitnessId,
        }
        impl ResolveWitness for FasciaResolver {
            fn resolve_pub_witness(
                &self,
                _: XWitnessId,
            ) -> Result<XWitnessTx, WitnessResolverError> {
                unreachable!()
            }
            fn resolve_pub_witness_ord(
                &self,
                witness_id: XWitnessId,
            ) -> Result<WitnessOrd, WitnessResolverError> {
                assert_eq!(witness_id, self.witness_id);
                Ok(WitnessOrd::Tentative)
            }
        }

        let resolver = FasciaResolver {
            witness_id: XChain::Bitcoin(witness_txid),
        };

        self.wallet
            .stock_mut()
            .consume_fascia(fascia, resolver)
            .unwrap();
    }

    pub fn update_witnesses(&mut self, after_height: u32) {
        let resolver = self.get_resolver();
        self.wallet
            .stock_mut()
            .update_witnesses(resolver, after_height)
            .unwrap();
    }

    pub fn create_consignments(
        &self,
        asset_beneficiaries: AssetBeneficiariesMap,
        witness_txid: Txid,
    ) -> Vec<Transfer> {
        let mut transfers = vec![];
        let stock = self.wallet.stock();

        for (contract_id, beneficiaries) in asset_beneficiaries {
            for beneficiary in beneficiaries {
                match beneficiary {
                    BuilderSeal::Revealed(seal) => {
                        let explicit_seal = XChain::Bitcoin(ExplicitSeal::new(
                            seal.method(),
                            Outpoint::new(witness_txid, seal.as_reduced_unsafe().vout),
                        ));
                        transfers.push(stock.transfer(contract_id, [explicit_seal], None).unwrap());
                    }
                    BuilderSeal::Concealed(seal) => {
                        transfers.push(stock.transfer(contract_id, vec![], Some(seal)).unwrap());
                    }
                }
            }
        }
        transfers
    }
}
