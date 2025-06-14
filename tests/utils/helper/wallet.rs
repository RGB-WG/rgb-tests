use super::*;

enum WalletAccount {
    Private(XprivAccount),
    Public(XpubAccount),
}
/// Test wallet structure type
pub struct TestWallet {
    /// RGB runtime for wallet operations
    pub runtime: RgbpRuntimeDir<Owner>,
    /// RGB descriptor for wallet
    pub descriptor: RgbDescr,
    /// Signer for transaction signing
    pub signer: Option<TestnetSigner>,
    /// Wallet directory path
    pub wallet_dir: PathBuf,
    /// Bitcoin node instance number
    pub instance: u8,
    /// Custom coinselection strategy
    pub coinselect_strategy: CustomCoinselectStrategy,
    /// Optional wallet identifier for reporting purposes
    pub wallet_id: Option<String>,
    /// Whether to force stop sync
    pub force_stop_sync: bool,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    RGB20,
    RGB21,
    RGB25,
}

impl fmt::Display for AssetSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

/// Create a new wallet with the specified descriptor type
pub fn get_wallet(descriptor_type: &DescriptorType) -> TestWallet {
    get_wallet_custom(descriptor_type, INSTANCE_1)
}

/// Create a new wallet with the specified descriptor type and instance
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

/// Create a wallet for mainnet (using predefined public keys)
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

/// Internal wallet creation function
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
        coinselect_strategy: CustomCoinselectStrategy::default(),
        wallet_id: None,
        force_stop_sync: false,
    };

    // Import all issuer files from the schemata directory
    let issuer_dir = PathBuf::from(SCHEMATA_DIR);
    if issuer_dir.exists() {
        for entry in std::fs::read_dir(issuer_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "issuer") {
                // println!("Auto-importing issuer file: {}", path.display());
                if let Err(err) = test_wallet.import(path) {
                    println!("Warning: Failed to import issuer: {}", err);
                }
            }
        }
    }

    test_wallet.sync();

    test_wallet
}

pub fn contracts(network: Network, wallet_dir: PathBuf) -> Contracts<StockpileDir<TxoSeal>> {
    if !network.is_testnet() {
        panic!("Non-testnet networks are not yet supported");
    }
    let stockpile = StockpileDir::load(wallet_dir, Consensus::Bitcoin, true)
        .expect("Invalid contracts directory");
    Contracts::load(stockpile)
}

/// Create a runtime for the wallet
fn make_runtime(
    descriptor: &RgbDescr,
    network: Network,
    wallet_dir: &PathBuf,
) -> RgbpRuntimeDir<Owner> {
    let name = "bp_wallet.wallet";
    let provider = FsTextStore::new(wallet_dir.join(name)).unwrap();

    // Create wallet using Owner::create
    let wallet = Owner::create(provider, descriptor.clone(), network, true)
        .expect("Unable to create wallet");

    let contracts = contracts(network, wallet_dir.clone());
    // Create runtime with wallet and contracts
    let mut runtime = RgbpRuntimeDir::from(RgbWallet::with(wallet, contracts));
    let indexer = get_indexer(&indexer_url(INSTANCE_1, network));
    runtime
        .sync(&indexer)
        .expect("Unable to synchronize wallet");
    runtime
}

/// Get an indexer instance
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

/// Broadcast a transaction
fn broadcast_tx(tx: &Tx, indexer_url: &str) {
    match get_indexer(indexer_url) {
        AnyIndexer::Electrum(inner) => {
            inner.transaction_broadcast(tx).unwrap();
        }
        AnyIndexer::Esplora(inner) => {
            inner
                .broadcast(tx)
                .inspect_err(|e| {
                    dbg!(
                        tx.inputs.iter().map(|i| i.prev_output).collect::<Vec<_>>(),
                        e
                    );
                })
                .unwrap();
        }
        _ => unreachable!("unsupported indexer"),
    }
}

/// Broadcast a transaction and mine a block
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

    pub fn force_stop_sync(&self) -> bool {
        self.force_stop_sync
    }

    pub fn set_force_stop_sync(&mut self, force_stop_sync: bool) {
        self.force_stop_sync = force_stop_sync;
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
        self.sync();
    }

    pub fn change_instance(&mut self, instance: u8) {
        self.instance = instance;
    }

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

    /// Issue a FUA contract with custom parameters
    pub fn issue_fua_with_params(&mut self, params: FUAIssueParams) -> ContractId {
        let mut builder = AssetParamsBuilder::default_fua()
            .name(params.name.as_str())
            .update_name_state(params.name.as_str())
            .update_details_state(params.details.as_str())
            .update_precision_state(params.precision.as_str())
            .update_circulating_state(params.circulating_supply)
            .clear_owned_state();

        // Add initial allocations
        for (outpoint, amount) in params.initial_allocations {
            builder = builder.add_owned_state(outpoint, amount);
        }

        self.issue_with_params(builder.build())
    }

    pub fn switch_to_instance(&mut self, instance: u8) {
        self.change_instance(instance);
        self.sync();
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
        if self.force_stop_sync {
            return;
        }
        let indexer = self.get_indexer();
        self.runtime.sync(&indexer).expect("Failed to sync wallet");
    }

    pub fn runtime(&mut self) -> &mut RgbpRuntimeDir<Owner> {
        &mut self.runtime
    }

    pub fn issue_with_params(&mut self, params: CreateParams<Outpoint>) -> ContractId {
        let contract_id = self
            .runtime
            .issue(params)
            .expect("failed to issue a contract");
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
        let src_consensus_dir = self.wallet_dir.clone();
        let dst_consensus_dir = to_wallet.wallet_dir.clone();

        let contract_id = self
            .runtime
            .contracts
            .find_contract_id(TypeName::from_str(contract_name).unwrap())
            .expect("Contract not found");
        let contract_file_name = format!("{}.{:-}.contract", contract_name, contract_id);
        let src_contract_dir = src_consensus_dir.join(&contract_file_name);
        let dst_contract_dir = dst_consensus_dir.join(&contract_file_name);
        std::fs::create_dir_all(&dst_contract_dir).unwrap();
        let read_dir = std::fs::read_dir(&src_contract_dir).unwrap();
        for entry in read_dir {
            let entry = entry.unwrap();
            let path = entry.path();
            let dst_path = dst_contract_dir.join(path.file_name().unwrap());
            std::fs::copy(&path, &dst_path).unwrap();
        }
    }

    /// Generates a noise engine for seal randomization
    /// This is a clone of the internal noise_engine implementation from rgb-std,
    /// since the original is not public and we need it for custom UTXO selection
    fn noise_engine(&self) -> Sha256 {
        let noise_seed = self.runtime.wallet.noise_seed();
        let mut noise_engine = Sha256::new();
        noise_engine.input_raw(noise_seed.as_ref());
        noise_engine
    }

    /// Creates an auth token for a specific UTXO
    ///
    /// This is a custom implementation that allows specifying the UTXO to use,
    /// unlike the standard rgb-std auth_token which automatically selects a UTXO.
    /// We need this to support custom UTXO selection for auth tokens.
    pub fn create_auth_token_with_utxo(
        &mut self,
        nonce: Option<u64>,
        outpoint: Outpoint,
    ) -> Option<AuthToken> {
        let nonce = nonce.unwrap_or_else(|| self.runtime.wallet.next_nonce());
        let seal = WTxoSeal::no_fallback(outpoint, self.noise_engine(), nonce);
        let auth = seal.auth_token();
        self.runtime.wallet.register_seal(seal);
        Some(auth)
    }

    /// Creates an RGB invoice with either a witness output or auth token beneficiary
    ///
    /// # Arguments
    /// * `contract_id` - ID of the RGB contract
    /// * `amount` - Amount of RGB asset to transfer
    /// * `wout` - Whether to use witness output (true) or auth token (false)
    /// * `nonce` - Optional nonce for seal generation
    /// * `utxo` - Optional UTXO to use for auth token. If None and wout=false, a new UTXO will be created
    pub fn invoice(
        &mut self,
        contract_id: ContractId,
        amount: u64,
        wout: bool,
        nonce: Option<u64>,
        mut utxo: Option<Outpoint>,
    ) -> RgbInvoice<ContractId> {
        let beneficiary = if wout {
            let wout = self.runtime.wout(nonce);
            RgbBeneficiary::WitnessOut(wout)
        } else {
            if utxo.is_none() {
                // Create new UTXO for auth token if none provided
                utxo = Some(self.get_utxo(None));
            }

            let auth = self.create_auth_token_with_utxo(nonce, utxo.unwrap());

            RgbBeneficiary::Token(auth.unwrap())
        };
        let value = StrictVal::num(amount);
        RgbInvoice::new(
            contract_id,
            Consensus::Bitcoin,
            true,
            beneficiary,
            Some(value),
        )
    }

    /// Set the coin selection strategy
    pub fn set_coinselect_strategy(&mut self, strategy: CustomCoinselectStrategy) -> &mut Self {
        self.coinselect_strategy = strategy;
        self
    }

    /// Get the current coin selection strategy
    pub fn coinselect_strategy(&self) -> CustomCoinselectStrategy {
        self.coinselect_strategy
    }

    /// Set wallet identifier for reporting purposes
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.wallet_id = Some(id.into());
        self
    }

    /// Get wallet identifier, or a default if not set
    pub fn wallet_id(&self) -> String {
        self.wallet_id
            .clone()
            .unwrap_or_else(|| format!("wallet_{}", self.instance))
    }

    pub fn sign_finalize(&self, psbt: &mut Psbt) {
        let _sig_count = psbt.sign(self.signer.as_ref().unwrap()).unwrap();
        psbt.finalize(self.runtime.wallet.descriptor());
    }

    pub fn sign_finalize_extract(&self, psbt: &mut Psbt) -> Tx {
        self.sign_finalize(psbt);
        psbt.extract().unwrap()
    }

    pub fn check_allocations(
        &mut self,
        contract_id: ContractId,
        asset_schema: AssetSchema,
        mut expected_fungible_allocations: Vec<u64>,
    ) {
        let allocation_field = match asset_schema {
            AssetSchema::RGB20 | AssetSchema::RGB25 => {
                // For fungible assets, we need to check the "amount" allocation
                "balance"
            }
            AssetSchema::RGB21 => {
                // for RGB21, we need to check the "fractions" allocation
                "balance"
            }
        };

        let state = self.runtime.state_own(contract_id);

        let mut actual_fungible_allocations = state
            .owned
            .get(allocation_field)
            .unwrap()
            .iter()
            .filter(|state| state.status.is_valid())
            .map(|state| state.assignment.data.unwrap_num().unwrap_uint::<u64>())
            .collect::<Vec<_>>();
        actual_fungible_allocations.sort();
        expected_fungible_allocations.sort();
        assert_eq!(actual_fungible_allocations, expected_fungible_allocations);
    }

    pub fn get_allocation_sum(&mut self, contract_id: ContractId) -> u64 {
        let allocation_field = "balance";
        let state = self.runtime.state_own(contract_id);
        state
            .owned
            .get(allocation_field)
            .unwrap()
            .iter()
            .filter(|state| state.status.is_valid())
            .map(|state| state.assignment.data.unwrap_num().unwrap_uint::<u64>())
            .sum::<u64>()
    }

    pub fn check_allocation_sum(&mut self, contract_id: ContractId, expected_sum: u64) {
        let actual_allocation_sum = self.get_allocation_sum(contract_id);
        assert_eq!(actual_allocation_sum, expected_sum);
    }

    pub fn send(
        &mut self,
        recv_wallet: &mut TestWallet,
        wout: bool,
        contract_id: ContractId,
        amount: u64,
        sats: u64,
        fee: Option<u64>,
        nonce: Option<u64>,
        report: Option<&mut Report>,
    ) -> (PathBuf, Tx, Payment) {
        let invoice = recv_wallet.invoice(contract_id, amount, wout, nonce, None);
        self.send_to_invoice(recv_wallet, invoice, Some(sats), fee, report)
    }

    pub fn send_to_invoice(
        &mut self,
        recv_wallet: &mut TestWallet,
        invoice: RgbInvoice<ContractId>,
        sats: Option<u64>,
        fee: Option<u64>,
        mut report: Option<&mut Report>,
    ) -> (PathBuf, Tx, Payment) {
        // We need to handle the report parameter carefully to avoid moving it
        let transfer_report = report.as_deref_mut();

        let (consignment, tx, payment) = self.transfer(invoice, sats, fee, true, transfer_report);
        self.mine_tx(&tx.txid(), false);
        // Now use the original report parameter
        recv_wallet.accept_transfer(&consignment, report).unwrap();
        self.sync();
        (consignment, tx, payment)
    }

    pub fn transfer(
        &mut self,
        invoice: RgbInvoice<ContractId>,
        sats: Option<u64>,
        fee: Option<u64>,
        broadcast: bool,
        report: Option<&mut Report>,
    ) -> (PathBuf, Tx, Payment) {
        self.sync();

        let fee = Sats::from_sats(fee.unwrap_or(DEFAULT_FEE_ABS));
        let sats = Sats::from_sats(sats.unwrap_or(2000));

        let strategy = self.coinselect_strategy;
        let pay_start = Instant::now();
        let params = TxParams::with(fee);
        let (psbt, payment) = self
            .runtime
            .pay_invoice(&invoice, strategy, params, Some(sats))
            .unwrap();

        let pay_duration = pay_start.elapsed();

        let (consignment, tx) = self.consign(
            invoice.scope,
            psbt.clone(),
            &payment.terminals,
            pay_duration,
            broadcast,
            report,
        );
        (consignment, tx, payment)
    }

    pub fn transfer_rbf(
        &mut self,
        contract_id: ContractId,
        payment: Payment,
        fee: u64,
        report: Option<&mut Report>,
    ) -> (PathBuf, Tx) {
        let pay_start = Instant::now();
        let pay_duration = pay_start.elapsed();

        // broadcast the transaction immediately after rbf execution,
        // Wait for the old transaction to be archived, and then actively sync the wallet
        let mut psbt = self.runtime.rbf(&payment, fee).unwrap();

        let tx = self.sign_finalize_extract(&mut psbt);

        self.broadcast_tx(&tx);
        std::thread::sleep(Duration::from_secs(10));
        self.sync();

        let (consignment, tx) = self.consign(
            contract_id,
            psbt,
            &payment.terminals,
            pay_duration,
            false,
            report,
        );
        (consignment, tx)
    }

    fn consign<'a>(
        &mut self,
        contract_id: ContractId,
        mut psbt: Psbt,
        terminals: impl IntoIterator<Item = &'a AuthToken>,
        pay_duration: Duration,
        broadcast: bool,
        report: Option<&mut Report>,
    ) -> (PathBuf, Tx) {
        static COUNTER: OnceLock<AtomicU32> = OnceLock::new();
        let counter = COUNTER.get_or_init(|| AtomicU32::new(0));
        counter.fetch_add(1, Ordering::SeqCst);
        let consignment_no = counter.load(Ordering::SeqCst);

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
            .contracts
            .consign_to_file(&consignment, contract_id, terminals)
            .unwrap();

        if let Some(report) = report {
            let wallet_id = self.wallet_id();
            let column_name = format!("{}_pay", wallet_id);
            report.add_duration(&column_name, pay_duration).unwrap();

            let consigment_column_name = format!("{}_pay_consignment_size", wallet_id);
            let file_size = std::fs::metadata(&consignment).unwrap().len();
            report
                .add_bytes(&consigment_column_name, file_size)
                .unwrap();

            let txin_column_name = format!("{}_pay_txin_count", wallet_id);
            report
                .add_integer(&txin_column_name, tx.inputs.len() as u64)
                .unwrap();

            let txout_column_name = format!("{}_pay_txout_count", wallet_id);
            report
                .add_integer(&txout_column_name, tx.outputs.len() as u64)
                .unwrap();
        }

        (consignment, tx)
    }

    pub fn accept_transfer(
        &mut self,
        consignment: &Path,
        report: Option<&mut Report>,
    ) -> Result<(), String> {
        self.sync();
        let accept_start = Instant::now();
        self.runtime
            .consume_from_file(
                false,
                consignment,
                |_, _, _| Result::<_, Infallible>::Ok(()),
            )
            .map_err(|e| format!("consume_from_file error: {}", e))?;
        let accept_duration = accept_start.elapsed();
        if let Some(report) = report {
            let column_name = format!("{}_accept", self.wallet_id());
            report.add_duration(&column_name, accept_duration).unwrap();
        }
        Ok(())
    }

    /// Import an RGB schema(.issuer) file
    ///
    /// # Arguments
    /// * `schema_path` - Path to the schema file to import
    pub fn import(&mut self, schema_path: impl AsRef<Path>) -> Result<(), String> {
        let schema_path = schema_path.as_ref();

        // Check if file exists
        if !schema_path.exists() {
            return Err(format!(
                "Schema file '{}' does not exist",
                schema_path.display()
            ));
        }

        // print!(
        //     "Processing '{}' ... ",
        //     schema_path.file_name().unwrap().to_string_lossy()
        // );

        // Load schema and get codex ID
        let issuer = Issuer::load(schema_path, |_, _, _| Result::<_, Infallible>::Ok(()))?;
        let codex_id = issuer.codex_id();

        // print!("codex id {} ... ", codex_id);

        // Import the schema into contracts
        if self.runtime.contracts.has_issuer(codex_id) {
            println!("already known, skipping");
            return Ok(());
        }

        self.runtime
            .contracts
            .import_issuer(issuer)
            .map_err(|e| format!("import error: {}", e))?;

        Ok(())
    }
}

impl TestWallet {
    /// Get contract state with parsed data structures
    pub fn contract_state(&mut self, contract_id: ContractId) -> Option<ContractState> {
        self.contract_state_internal(contract_id)
            .map(|(immutable, owned, _)| {
                // Parse immutable state
                let name = immutable
                    .get(&VariantName::from_str("name").unwrap())
                    .and_then(|m| m.iter().next())
                    .map(|v| v.data.verified.unwrap_string())
                    .unwrap_or_default();

                let ticker = immutable
                    .get(&VariantName::from_str("ticker").unwrap())
                    .and_then(|m| m.iter().next())
                    .map(|v| v.data.verified.unwrap_string())
                    .unwrap_or_default();

                let precision = immutable
                    .get(&VariantName::from_str("precision").unwrap())
                    .and_then(|m| m.iter().next())
                    .map(|v| {
                        let tag = v.data.verified.unwrap_enum_tag();
                        if let EnumTag::Name(name) = tag {
                            name.to_string()
                        } else {
                            "".to_string()
                        }
                    })
                    .unwrap_or_default();

                let circulating_supply = immutable
                    .get(&VariantName::from_str("issued").unwrap())
                    .and_then(|m: &Vec<ImmutableState>| m.iter().next())
                    .map(|v| v.data.verified.unwrap_num().unwrap_uint::<u64>())
                    .unwrap_or_default();

                // Parse ownership state
                let mut allocations = vec![];
                if let Some(owned_map) = owned.get(&VariantName::from_str("balance").unwrap()) {
                    for state in owned_map {
                        allocations.push((
                            state.assignment.seal,
                            state.assignment.data.unwrap_num().unwrap_uint::<u64>(),
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
    /// Get contract state (internal implementation)
    fn contract_state_internal(
        &mut self,
        contract_id: ContractId,
    ) -> Option<(
        BTreeMap<StateName, Vec<ImmutableState>>,
        BTreeMap<StateName, Vec<OwnedState<TxoSeal>>>,
        BTreeMap<StateName, StrictVal>,
    )> {
        let rgb_contract_state = self.runtime().state_all(contract_id);

        Some((
            rgb_contract_state.immutable,
            rgb_contract_state.owned,
            rgb_contract_state.aggregated,
        ))
    }
}

/// Immutable state part of RGB21 contract
#[derive(Debug, Clone)]
pub struct RGB21ContractImmutableState {
    pub name: String,
    pub total_fractions: u64,
    pub token: Option<NFTMetadata>,
}

/// NFT metadata in RGB21 contract
#[derive(Debug, Clone)]
pub struct NFTMetadata {
    pub index: u32,
    pub amount: u64,
    pub ticker: Option<String>,
    pub name: Option<String>,
    pub details: Option<String>,
    pub preview: Option<MediaData>,
    pub media: Option<MediaDigest>,
    pub attachments: BTreeMap<u8, MediaDigest>,
    pub reserves: Option<ReserveData>,
}

/// Media data with full content
#[derive(Debug, Clone)]
pub struct MediaData {
    pub media_type: MediaTypeData,
    pub data: Vec<u8>,
}

/// Media type information
#[derive(Debug, Clone)]
pub struct MediaTypeData {
    pub r#type: String,
    pub subtype: Option<String>,
    pub charset: Option<String>,
}

/// Media digest (reference only)
#[derive(Debug, Clone)]
pub struct MediaDigest {
    pub media_type: MediaTypeData,
    pub digest: Vec<u8>,
}

/// Reserve proof data
#[derive(Debug, Clone)]
pub struct ReserveData {
    pub utxo: Outpoint,
    pub proof: Vec<u8>,
}

/// Owned state part of RGB21 contract
#[derive(Debug, Clone)]
pub struct RGB21ContractOwnedState {
    pub fractions: Vec<(TxoSeal, u64)>, // (outpoint, amount)
}

/// Complete RGB21 contract state
#[derive(Debug, Clone)]
pub struct RGB21ContractState {
    pub immutable: RGB21ContractImmutableState,
    pub owned: RGB21ContractOwnedState,
}

/// Extract the first element from a tuple
fn extract_from_tuple(v: &StrictVal) -> Option<StrictVal> {
    if let StrictVal::Tuple(s) = v {
        if !s.is_empty() {
            Some(s[0].clone())
        } else {
            None
        }
    } else {
        None
    }
}

/// Extract the first element from a two-layer tuple
fn extract_from_2_layer_tuple(v: &StrictVal) -> Option<StrictVal> {
    if let StrictVal::Tuple(t) = v {
        if let Some(inner) = t.first() {
            extract_from_tuple(inner)
        } else {
            None
        }
    } else {
        None
    }
}

/// Extract value from Union (for Option type)
fn extract_from_union(v: &StrictVal) -> Option<StrictVal> {
    if let StrictVal::Union(tag, t) = v {
        if let EnumTag::Name(name) = tag {
            if ***name == "some" {
                Some(t.as_ref().clone())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

/// Extract value from Union and single-layer Tuple
fn extract_from_union_and_tuple(v: &StrictVal) -> Option<StrictVal> {
    extract_from_union(v).and_then(|s| extract_from_tuple(&s))
}

/// Extract value from Union and two-layer Tuple
fn extract_from_union_and_2_layer_tuple(v: &StrictVal) -> Option<StrictVal> {
    extract_from_union(v).and_then(|s| extract_from_2_layer_tuple(&s))
}

impl TestWallet {
    // TODO: Need to optimize the following code
    /// Issue a FAC contract with custom parameters
    pub fn issue_fac_with_params(&mut self, params: FACIssueParams) -> ContractId {
        let mut create_params = AssetParamsBuilder::default_fac()
            .name(params.name.as_str())
            .update_name_state(params.name.as_str())
            .update_details_state(params.details.as_str())
            .build();

        for name_state in create_params.global.iter_mut() {
            let name = VariantName::from_str("token").unwrap();
            if name_state.name == name {
                let token = &mut name_state.state.verified;
                if let StrictVal::Struct(s) = token {
                    let reserved_name = FieldName::from_str("align").unwrap();
                    let reserved = s.get_mut(&reserved_name).unwrap();
                    *reserved = StrictVal::Bytes(Blob(vec![0; 26]));

                    let index_name = FieldName::from_str("tokenIndex").unwrap();
                    let index = s.get_mut(&index_name).unwrap();
                    *index = StrictVal::Number(StrictNum::from(params.index));

                    let amount_name = FieldName::from_str("fraction").unwrap();
                    let amount = s.get_mut(&amount_name).unwrap();
                    *amount = StrictVal::Number(StrictNum::from(params.total_fractions));
                }

                if let Some(ref nft_spec) = params.nft_spec {
                    let nft_spec_strict_val = name_state.state.unverified.as_mut().unwrap();

                    match nft_spec_strict_val {
                        StrictVal::Struct(s) => {
                            let index_name = FieldName::from_str("index").unwrap();
                            let index = s.get_mut(&index_name).unwrap();
                            *index = StrictVal::Number(StrictNum::from(params.index));

                            if let Some(ref name_params) = nft_spec.name {
                                let name_name = FieldName::from_str("name").unwrap();
                                let name = s.get_mut(&name_name).unwrap();
                                *name = StrictVal::String(name_params.to_string());
                            }

                            let preview_params = &nft_spec.embedded;
                            let preview_name = FieldName::from_str("preview").unwrap();
                            let preview = s.get_mut(&preview_name).unwrap();
                            match preview {
                                StrictVal::Struct(s) => {
                                    let preview_type_name = FieldName::from_str("type").unwrap();
                                    let preview_type = s.get_mut(&preview_type_name).unwrap();
                                    match preview_type {
                                        StrictVal::Struct(ty) => {
                                            let ty_name = FieldName::from_str("type").unwrap();
                                            let ty_type = ty.get_mut(&ty_name).unwrap();
                                            *ty_type = StrictVal::String(
                                                preview_params.mime.ty.to_string(),
                                            );

                                            let subtype_name =
                                                FieldName::from_str("subtype").unwrap();
                                            let subtype = ty.get_mut(&subtype_name).unwrap();

                                            if let Some(ref subtype_params) =
                                                preview_params.mime.subtype
                                            {
                                                *subtype =
                                                    StrictVal::String(subtype_params.to_string());
                                            } else {
                                                *subtype = StrictVal::Unit;
                                            }

                                            let charset_name =
                                                FieldName::from_str("charset").unwrap();
                                            let charset = ty.get_mut(&charset_name).unwrap();
                                            if let Some(ref charset_params) =
                                                preview_params.mime.charset
                                            {
                                                *charset =
                                                    StrictVal::String(charset_params.to_string());
                                            } else {
                                                *charset = StrictVal::Unit;
                                            }
                                        }
                                        _ => {
                                            panic!("Invalid preview type");
                                        }
                                    }

                                    let data_name = FieldName::from_str("data").unwrap();
                                    let data = s.get_mut(&data_name).unwrap();
                                    *data = StrictVal::Bytes(Blob(preview_params.data.to_vec()));
                                }
                                _ => {
                                    panic!("Invalid preview");
                                }
                            }

                            if let Some(ref media_params) = nft_spec.external {
                                let media_name = FieldName::from_str("media").unwrap();
                                let media = s.get_mut(&media_name).unwrap();
                                match media {
                                    StrictVal::Struct(s) => {
                                        let media_type_name = FieldName::from_str("type").unwrap();
                                        let media_type = s.get_mut(&media_type_name).unwrap();
                                        match media_type {
                                            StrictVal::Struct(ty) => {
                                                let ty_name = FieldName::from_str("type").unwrap();
                                                let ty_type = ty.get_mut(&ty_name).unwrap();
                                                *ty_type = StrictVal::String(
                                                    media_params.mime.ty.to_string(),
                                                );

                                                let subtype_name =
                                                    FieldName::from_str("subtype").unwrap();
                                                let subtype = ty.get_mut(&subtype_name).unwrap();
                                                if let Some(ref subtype_params) =
                                                    media_params.mime.subtype
                                                {
                                                    *subtype = StrictVal::String(
                                                        subtype_params.to_string(),
                                                    );
                                                } else {
                                                    *subtype = StrictVal::Unit;
                                                }

                                                let charset_name =
                                                    FieldName::from_str("charset").unwrap();
                                                let charset = ty.get_mut(&charset_name).unwrap();
                                                if let Some(ref charset_params) =
                                                    media_params.mime.charset
                                                {
                                                    *charset = StrictVal::String(
                                                        charset_params.to_string(),
                                                    );
                                                } else {
                                                    *charset = StrictVal::Unit;
                                                }
                                            }
                                            _ => {
                                                panic!("Invalid media type");
                                            }
                                        }

                                        let data_name = FieldName::from_str("digest").unwrap();
                                        let data = s.get_mut(&data_name).unwrap();
                                        *data =
                                            StrictVal::Bytes(Blob(media_params.digest.to_vec()));
                                    }
                                    _ => {
                                        panic!("Invalid media");
                                    }
                                }
                            }

                            if let Some(ref reserves_params) = nft_spec.reserves {
                                let reserves_name = FieldName::from_str("reserves").unwrap();
                                let reserves = s.get_mut(&reserves_name).unwrap();
                                match reserves {
                                    StrictVal::Struct(s) => {
                                        let utxo_name = FieldName::from_str("utxo").unwrap();
                                        let utxo = s.get_mut(&utxo_name).unwrap();
                                        match utxo {
                                            StrictVal::Struct(s) => {
                                                let txid_name =
                                                    FieldName::from_str("txid").unwrap();
                                                let txid = s.get_mut(&txid_name).unwrap();
                                                *txid = StrictVal::Bytes(Blob(
                                                    reserves_params.utxo.txid.to_inner().to_vec(),
                                                ));

                                                let vout_name =
                                                    FieldName::from_str("vout").unwrap();
                                                let vout = s.get_mut(&vout_name).unwrap();
                                                *vout = StrictVal::Number(StrictNum::from(
                                                    reserves_params.utxo.vout.into_u32(),
                                                ));
                                            }
                                            _ => {
                                                panic!("Invalid utxo");
                                            }
                                        }

                                        let proof_name = FieldName::from_str("proof").unwrap();
                                        let proof = s.get_mut(&proof_name).unwrap();
                                        *proof =
                                            StrictVal::Bytes(Blob(reserves_params.proof.to_vec()));
                                    }
                                    _ => {
                                        panic!("Invalid reserves");
                                    }
                                }
                            }
                        }
                        _ => {
                            panic!("Invalid NFT spec");
                        }
                    }
                }
            }
        }

        for name_state in create_params.owned.iter_mut() {
            let name = VariantName::from_str("fractions").unwrap();
            if name_state.name == name {
                let fractions = &mut name_state.state;
                fractions.seal = EitherSeal::Alt(params.initial_allocation.as_ref().unwrap().0);
                let fractions_data = &mut fractions.data;

                let nft = &params.initial_allocation.as_ref().unwrap().1;
                let nft_data = StrictVal::Struct(IndexMap::from([
                    (
                        FieldName::from_str("tokenIndex").unwrap(),
                        StrictVal::Number(StrictNum::from(nft.token_no.into_inner())),
                    ),
                    (
                        FieldName::from_str("fraction").unwrap(),
                        StrictVal::Number(StrictNum::from(nft.fractions.into_inner())),
                    ),
                    (
                        FieldName::from_str("align").unwrap(),
                        StrictVal::Bytes(Blob(vec![0; 26])),
                    ),
                ]));
                *fractions_data = nft_data;
            }
        }

        self.issue_with_params(create_params)
    }

    /// Get RGB21 contract state
    pub fn contract_state_rgb21(&mut self, contract_id: ContractId) -> Option<RGB21ContractState> {
        self.contract_state_internal(contract_id)
            .map(|(immutable, owned, _)| {
                // Parse immutable state
                let name = immutable
                    .get(&VariantName::from_str("name").unwrap())
                    .and_then(|m| m.iter().next())
                    .map(|v| v.data.verified.unwrap_string())
                    .unwrap_or_default();

                let total_fractions = immutable
                    .get(&VariantName::from_str("fractions").unwrap())
                    .and_then(|m| m.iter().next())
                    .map(|v| v.data.verified.unwrap_num().unwrap_uint::<u64>())
                    .unwrap_or_default();

                // Parse token/NFT metadata
                let token = immutable
                    .get(&VariantName::from_str("token").unwrap())
                    .and_then(|m| m.iter().next())
                    .map(|v| {
                        let mut index = 0u32;
                        let mut amount = 0u64;

                        // Parse verified token data
                        if let StrictVal::Struct(ref s) = v.data.verified {
                            if let Some(StrictVal::Number(n)) =
                                s.get(&FieldName::from_str("index").unwrap())
                            {
                                index = n.unwrap_uint::<u32>();
                            }
                            if let Some(StrictVal::Number(n)) =
                                s.get(&FieldName::from_str("balance").unwrap())
                            {
                                amount = n.unwrap_uint::<u64>();
                            }
                        }

                        // Parse unverified token metadata
                        let mut ticker = None;
                        let mut name = None;
                        let mut details = None;
                        let mut preview = None;
                        let mut media = None;
                        let mut attachments = BTreeMap::new();
                        let mut reserves = None;

                        if let Some(ref unverified) = v.data.unverified {
                            if let StrictVal::Struct(ref s) = unverified {
                                if let Some(StrictVal::Union(_, t)) =
                                    s.get(&FieldName::from_str("ticker").unwrap())
                                {
                                    ticker =
                                        extract_from_2_layer_tuple(t).map(|s| s.unwrap_string());
                                }

                                if let Some(StrictVal::Union(_, t)) =
                                    s.get(&FieldName::from_str("name").unwrap())
                                {
                                    name = extract_from_2_layer_tuple(t).map(|s| s.unwrap_string());
                                }

                                if let Some(StrictVal::Union(_, t)) =
                                    s.get(&FieldName::from_str("details").unwrap())
                                {
                                    details =
                                        extract_from_2_layer_tuple(t).map(|s| s.unwrap_string());
                                }

                                // Parse preview
                                let preview_struct = s
                                    .get(&FieldName::from_str("preview").unwrap())
                                    .and_then(extract_from_union_and_tuple);

                                if let Some(StrictVal::Struct(ref p)) = preview_struct {
                                    let media_type = if let Some(StrictVal::Struct(ref t)) =
                                        p.get(&FieldName::from_str("type").unwrap())
                                    {
                                        let type_value = t
                                            .get(&FieldName::from_str("type").unwrap())
                                            .and_then(|v| {
                                                extract_from_tuple(v).map(|s| s.unwrap_string())
                                            })
                                            .unwrap_or_default();

                                        let subtype = t
                                            .get(&FieldName::from_str("subtype").unwrap())
                                            .and_then(|v| {
                                                extract_from_union_and_2_layer_tuple(v)
                                                    .map(|s| s.unwrap_string())
                                            });

                                        let charset = t
                                            .get(&FieldName::from_str("charset").unwrap())
                                            .and_then(|v| {
                                                extract_from_union_and_2_layer_tuple(v)
                                                    .map(|s| s.unwrap_string())
                                            });

                                        MediaTypeData {
                                            r#type: type_value,
                                            subtype,
                                            charset,
                                        }
                                    } else {
                                        MediaTypeData {
                                            r#type: "".to_string(),
                                            subtype: None,
                                            charset: None,
                                        }
                                    };

                                    let data = if let Some(StrictVal::Bytes(Blob(d))) =
                                        p.get(&FieldName::from_str("data").unwrap())
                                    {
                                        d.clone()
                                    } else {
                                        vec![]
                                    };

                                    preview = Some(MediaData { media_type, data });
                                }

                                // Parse media
                                let media_struct = s
                                    .get(&FieldName::from_str("media").unwrap())
                                    .and_then(extract_from_union_and_tuple);

                                if let Some(StrictVal::Struct(ref m)) = media_struct {
                                    let media_type = if let Some(StrictVal::Struct(ref t)) =
                                        m.get(&FieldName::from_str("type").unwrap())
                                    {
                                        let type_value = t
                                            .get(&FieldName::from_str("type").unwrap())
                                            .and_then(|v| {
                                                extract_from_tuple(v).map(|s| s.unwrap_string())
                                            })
                                            .unwrap_or_default();

                                        let subtype = t
                                            .get(&FieldName::from_str("subtype").unwrap())
                                            .and_then(|v| {
                                                extract_from_union_and_2_layer_tuple(v)
                                                    .map(|s| s.unwrap_string())
                                            });

                                        let charset = t
                                            .get(&FieldName::from_str("charset").unwrap())
                                            .and_then(|v| {
                                                extract_from_union_and_2_layer_tuple(v)
                                                    .map(|s| s.unwrap_string())
                                            });

                                        MediaTypeData {
                                            r#type: type_value,
                                            subtype,
                                            charset,
                                        }
                                    } else {
                                        MediaTypeData {
                                            r#type: "".to_string(),
                                            subtype: None,
                                            charset: None,
                                        }
                                    };

                                    let digest = if let Some(StrictVal::Bytes(Blob(d))) =
                                        m.get(&FieldName::from_str("digest").unwrap())
                                    {
                                        d.clone()
                                    } else {
                                        vec![]
                                    };

                                    media = Some(MediaDigest { media_type, digest });
                                }

                                // Parse attachments
                                if let Some(StrictVal::Map(ref atts)) =
                                    s.get(&FieldName::from_str("attachments").unwrap())
                                {
                                    for (key, value) in atts {
                                        if let (
                                            StrictVal::Number(idx),
                                            StrictVal::Struct(ref att),
                                        ) = (key, value)
                                        {
                                            let media_type = if let Some(StrictVal::Struct(ref t)) =
                                                att.get(&FieldName::from_str("type").unwrap())
                                            {
                                                let type_value = t
                                                    .get(&FieldName::from_str("type").unwrap())
                                                    .and_then(|v| {
                                                        extract_from_tuple(v)
                                                            .map(|s| s.unwrap_string())
                                                    })
                                                    .unwrap_or_default();

                                                let subtype = t
                                                    .get(&FieldName::from_str("subtype").unwrap())
                                                    .and_then(|v| {
                                                        extract_from_union_and_2_layer_tuple(v)
                                                            .map(|s| s.unwrap_string())
                                                    });

                                                let charset = t
                                                    .get(&FieldName::from_str("charset").unwrap())
                                                    .and_then(|v| {
                                                        extract_from_union_and_2_layer_tuple(v)
                                                            .map(|s| s.unwrap_string())
                                                    });

                                                MediaTypeData {
                                                    r#type: type_value,
                                                    subtype,
                                                    charset,
                                                }
                                            } else {
                                                MediaTypeData {
                                                    r#type: "".to_string(),
                                                    subtype: None,
                                                    charset: None,
                                                }
                                            };

                                            let digest = if let Some(StrictVal::Bytes(Blob(d))) =
                                                att.get(&FieldName::from_str("digest").unwrap())
                                            {
                                                d.clone()
                                            } else {
                                                vec![]
                                            };

                                            let attachment = MediaDigest { media_type, digest };

                                            attachments.insert(idx.unwrap_uint::<u8>(), attachment);
                                        }
                                    }
                                }

                                // Parse reserves
                                let reserves_struct = s
                                    .get(&FieldName::from_str("reserves").unwrap())
                                    .and_then(extract_from_union_and_tuple);

                                if let Some(StrictVal::Struct(ref r)) = reserves_struct {
                                    if let Some(StrictVal::Struct(ref u)) =
                                        r.get(&FieldName::from_str("utxo").unwrap())
                                    {
                                        let txid_bytes = u
                                            .get(&FieldName::from_str("txid").unwrap())
                                            .and_then(extract_from_tuple)
                                            .and_then(|v| {
                                                if let StrictVal::Bytes(Blob(tx)) = v {
                                                    Some(tx)
                                                } else {
                                                    None
                                                }
                                            })
                                            .unwrap_or_default();

                                        let vout = u
                                            .get(&FieldName::from_str("vout").unwrap())
                                            .and_then(extract_from_tuple)
                                            .and_then(|v| {
                                                if let StrictVal::Number(n) = v {
                                                    Some(n.unwrap_uint::<u32>())
                                                } else {
                                                    None
                                                }
                                            })
                                            .unwrap_or_default();

                                        let txid = if txid_bytes.len() == 32 {
                                            let mut arr = [0u8; 32];
                                            arr.copy_from_slice(&txid_bytes);
                                            Txid::from_byte_array(arr)
                                        } else {
                                            // default txid
                                            Txid::coinbase()
                                        };

                                        let outpoint = Outpoint::new(txid, vout);

                                        let proof = if let Some(StrictVal::Bytes(Blob(p))) =
                                            r.get(&FieldName::from_str("proof").unwrap())
                                        {
                                            p.clone()
                                        } else {
                                            vec![]
                                        };

                                        reserves = Some(ReserveData {
                                            utxo: outpoint,
                                            proof,
                                        });
                                    }
                                }
                            }
                        }

                        NFTMetadata {
                            index,
                            amount,
                            ticker,
                            name,
                            details,
                            preview,
                            media,
                            attachments,
                            reserves,
                        }
                    });

                // Parse ownership state (fractions)
                let mut fractions = vec![];
                if let Some(owned_map) = owned.get(&VariantName::from_str("fractions").unwrap()) {
                    for state in owned_map {
                        let amt_val = state.assignment.data.unwrap_num().unwrap_uint::<u64>();
                        fractions.push((state.assignment.seal, amt_val));
                    }
                }

                RGB21ContractState {
                    immutable: RGB21ContractImmutableState {
                        name,
                        total_fractions,
                        token,
                    },
                    owned: RGB21ContractOwnedState { fractions },
                }
            })
    }
}
