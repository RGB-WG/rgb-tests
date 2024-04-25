use super::*;

pub struct Wallet {
    stored_wallet: StoredWallet<BpWallet<XpubDerivable, RgbDescr>>,
    account: MemorySigningAccount,
}

#[derive(Debug)]
pub enum DescriptorType {
    Wpkh,
    Tr,
}

#[derive(Debug)]
pub enum TransferType {
    Blinded,
    Witness,
}

#[derive(Debug, EnumIter)]
pub enum AssetSchema {
    Nia,
    Uda,
    Cfa,
}

#[derive(Debug)]
pub enum AssetInfo {
    Nia {
        spec: AssetSpec,
        terms: ContractTerms,
        issued_supply: u64,
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
        issued_supply: u64,
    },
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
            Self::Nia => Rgb20::iface(rgb20::Features::FIXED),
            Self::Uda => Rgb21::iface(rgb21::Features::NONE),
            Self::Cfa => Rgb25::iface(rgb25::Features::NONE),
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

impl AssetInfo {
    fn asset_schema(&self) -> AssetSchema {
        match self {
            Self::Nia { .. } => AssetSchema::Nia,
            Self::Uda { .. } => AssetSchema::Uda,
            Self::Cfa { .. } => AssetSchema::Cfa,
        }
    }

    fn iface_type_name(&self) -> TypeName {
        self.asset_schema().iface_type_name()
    }

    fn schema(&self) -> Schema {
        self.asset_schema().schema()
    }

    fn issue_impl(&self) -> IfaceImpl {
        self.asset_schema().issue_impl()
    }

    fn scripts(&self) -> Scripts {
        self.asset_schema().scripts()
    }

    fn types(&self) -> TypeSystem {
        self.asset_schema().types()
    }

    fn iface(&self) -> Iface {
        self.asset_schema().iface()
    }

    pub fn nia(
        ticker: &str,
        name: &str,
        precision: u8,
        details: Option<&str>,
        terms_text: &str,
        terms_media_fpath: Option<&str>,
        issued_supply: u64,
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
            issued_supply,
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
        issued_supply: u64,
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
            issued_supply,
        }
    }

    fn add_global_state(&self, mut builder: ContractBuilder) -> ContractBuilder {
        match self {
            Self::Nia {
                spec,
                terms,
                issued_supply,
            } => builder
                .add_global_state("spec", spec.clone())
                .unwrap()
                .add_global_state("terms", terms.clone())
                .unwrap()
                .add_global_state("issuedSupply", Amount::from(*issued_supply))
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
                issued_supply,
            } => {
                builder = builder
                    .add_global_state("name", name.clone())
                    .unwrap()
                    .add_global_state("precision", *precision)
                    .unwrap()
                    .add_global_state("terms", terms.clone())
                    .unwrap()
                    .add_global_state("issuedSupply", Amount::from(*issued_supply))
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

    fn add_asset_owner(
        &self,
        builder: ContractBuilder,
        builder_seal: BuilderSeal<BlindSeal<Txid>>,
    ) -> ContractBuilder {
        match self {
            Self::Nia { issued_supply, .. } | Self::Cfa { issued_supply, .. } => builder
                .add_fungible_state("assetOwner", builder_seal, *issued_supply)
                .unwrap(),
            Self::Uda { token_data, .. } => {
                let fraction = OwnedFraction::from_inner(1);
                let allocation = Allocation::with(token_data.index, fraction);
                builder
                    .add_data("assetOwner", builder_seal, allocation)
                    .unwrap()
            }
        }
    }
}

fn get_test_data_dir_path() -> PathBuf {
    PathBuf::from("tests").join("tmp")
}

pub fn get_wallet(descriptor_type: &DescriptorType) -> Wallet {
    let mut seed = vec![0u8; 128];
    rand::thread_rng().fill_bytes(&mut seed);

    let secp = Secp256k1::new();

    let master_xpriv = ExtendedPrivKey::new_master(bitcoin::Network::Regtest, &seed).unwrap();

    let master_xpub = ExtendedPubKey::from_priv(&secp, &master_xpriv);

    let derivation: DerivationPath = vec![
        ChildNumber::from_hardened_idx(86).unwrap(),
        ChildNumber::from_hardened_idx(1).unwrap(),
        ChildNumber::from_hardened_idx(0).unwrap(),
    ]
    .into();

    let account_xpriv = master_xpriv.derive_priv(&secp, &derivation).unwrap();

    let account =
        MemorySigningAccount::with(&secp, master_xpub.identifier(), derivation, account_xpriv);

    let derivation_account = account.to_account();
    let derivation_account_rgb = derivation_account
        .to_string()
        .replace("/*/*", "/<0;1;9;10>/*");
    let xpub_derivable = XpubDerivable::from_str(&derivation_account_rgb).unwrap();

    let descriptor = match descriptor_type {
        DescriptorType::Wpkh => RgbDescr::Wpkh(Wpkh::from(xpub_derivable)),
        DescriptorType::Tr => RgbDescr::TapretKey(TapretKey::from(xpub_derivable)),
    };

    let rgb_dir = get_test_data_dir_path().join(account.account_fingerprint().to_string());
    std::fs::create_dir_all(&rgb_dir).unwrap();

    let mut bp_runtime: BpRuntime<RgbDescr> = BpRuntime::new_standard(descriptor, Network::Regtest);

    if bp_runtime.warnings().is_empty() {
        eprintln!("success");
    } else {
        eprintln!("complete with warnings:");
        for warning in bp_runtime.warnings() {
            eprintln!("- {warning}");
        }
        bp_runtime.reset_warnings();
    }

    let name = s!("wallet_name");
    let dir = rgb_dir.join(&name);
    bp_runtime.set_name(name);
    bp_runtime.store(&dir).unwrap();

    let stock = Stock::default();
    stock.store(&rgb_dir).unwrap();
    let mut stored_stock = StoredStock::attach(rgb_dir.clone(), stock.clone());

    let wallet_path = bp_runtime.path().clone();
    let stored_wallet =
        StoredWallet::attach(rgb_dir.clone(), wallet_path, stock, bp_runtime.detach());

    for asset_schema in AssetSchema::iter() {
        let valid_kit = asset_schema.get_valid_kit();
        stored_stock.import_kit(valid_kit).unwrap();
    }

    let mut wallet = Wallet {
        stored_wallet,
        account,
    };

    wallet.sync();

    wallet
}

fn get_indexer() -> AnyIndexer {
    match INDEXER.get().unwrap() {
        Indexer::Electrum => {
            AnyIndexer::Electrum(Box::new(ElectrumClient::new(ELECTRUM_URL).unwrap()))
        }
        Indexer::Esplora => AnyIndexer::Esplora(Box::new(
            EsploraClient::new(ESPLORA_URL).build_blocking().unwrap(),
        )),
    }
}

fn get_resolver() -> AnyResolver {
    match INDEXER.get().unwrap() {
        Indexer::Electrum => AnyResolver::electrum_blocking(ELECTRUM_URL, None).unwrap(),
        Indexer::Esplora => AnyResolver::esplora_blocking(ESPLORA_URL, None).unwrap(),
    }
}

fn broadcast_tx(indexer: &AnyIndexer, tx: &Tx) {
    match indexer {
        AnyIndexer::Electrum(inner) => {
            inner.transaction_broadcast(tx).unwrap();
        }
        AnyIndexer::Esplora(inner) => {
            inner.broadcast(tx).unwrap();
        }
        _ => unreachable!("unsupported indexer"),
    }
}

pub fn attachment_from_fpath(fpath: &str) -> Attachment {
    let file_bytes = std::fs::read(fpath).unwrap();
    let file_hash: sha256::Hash = Hash::hash(&file_bytes[..]);
    let digest = file_hash.into_inner().into();
    let mime = tree_magic::from_filepath(fpath.as_ref());
    let media_ty: &'static str = Box::leak(mime.clone().into_boxed_str());
    let media_type = MediaType::with(media_ty);
    Attachment {
        ty: media_type,
        digest,
    }
}

fn uda_token_data_minimal() -> TokenData {
    TokenData {
        index: TokenIndex::from_inner(UDA_FIXED_INDEX),
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

impl Wallet {
    pub fn get_utxo(&mut self) -> (String, u32) {
        let address = self
            .stored_wallet
            .wallet()
            .addresses(RgbKeychain::Rgb)
            .next()
            .expect("no addresses left")
            .addr
            .to_string();
        let txid = fund_wallet(address);
        self.sync();
        let mut vout = None;
        let bp_runtime = self.stored_wallet.wallet();
        for (_derived_addr, utxos) in bp_runtime.address_coins() {
            for utxo in utxos {
                if utxo.outpoint.txid.to_string() == txid {
                    vout = Some(utxo.outpoint.vout_u32());
                }
            }
        }
        (txid, vout.unwrap())
    }

    pub fn sync(&mut self) {
        let indexer = get_indexer();
        self.stored_wallet
            .wallet_mut()
            .update(&indexer)
            .into_result()
            .unwrap();
        self.stored_wallet.store();
    }

    pub fn close_method(&self) -> CloseMethod {
        self.stored_wallet.wallet().seal_close_method()
    }

    pub fn issue_with_info(
        &mut self,
        asset_info: AssetInfo,
        close_method: CloseMethod,
        outpoint: Option<&(String, u32)>,
    ) -> (ContractId, TypeName) {
        let (txid, vout) = if let Some((txid, vout)) = outpoint {
            (txid.clone(), *vout)
        } else {
            self.get_utxo()
        };

        let blind_seal = match close_method {
            CloseMethod::TapretFirst => {
                BlindSeal::tapret_first_rand(Txid::from_str(&txid).unwrap(), vout)
            }
            CloseMethod::OpretFirst => {
                BlindSeal::opret_first_rand(Txid::from_str(&txid).unwrap(), vout)
            }
        };
        let genesis_seal = GenesisSeal::from(blind_seal);
        let seal: XChain<BlindSeal<Txid>> = XChain::with(Layer1::Bitcoin, genesis_seal);
        let builder_seal = BuilderSeal::from(seal);

        let mut builder = ContractBuilder::with(
            Identity::default(),
            asset_info.iface(),
            asset_info.schema(),
            asset_info.issue_impl(),
            asset_info.types(),
            asset_info.scripts(),
        );

        builder = asset_info.add_global_state(builder);

        builder = asset_info.add_asset_owner(builder, builder_seal);

        let contract = builder.issue_contract().expect("failure issuing contract");
        let mut resolver = get_resolver();
        self.stored_wallet
            .stock_mut()
            .import_contract(contract.clone(), &mut resolver)
            .unwrap();

        (contract.contract_id(), asset_info.iface_type_name())
    }

    pub fn issue_nia(
        &mut self,
        issued_supply: u64,
        close_method: CloseMethod,
        outpoint: Option<&(String, u32)>,
    ) -> (ContractId, TypeName) {
        let asset_info = AssetInfo::nia(
            "NIATCKR",
            "NIA asset name",
            2,
            None,
            "NIA terms",
            None,
            issued_supply,
        );
        self.issue_with_info(asset_info, close_method, outpoint)
    }

    pub fn issue_uda(
        &mut self,
        close_method: CloseMethod,
        outpoint: Option<&(String, u32)>,
    ) -> (ContractId, TypeName) {
        let token_data = uda_token_data_minimal();
        let asset_info = AssetInfo::uda(
            "UDATCKR",
            "UDA asset name",
            None,
            "NIA terms",
            None,
            token_data,
        );
        self.issue_with_info(asset_info, close_method, outpoint)
    }

    pub fn issue_cfa(
        &mut self,
        issued_supply: u64,
        close_method: CloseMethod,
        outpoint: Option<&(String, u32)>,
    ) -> (ContractId, TypeName) {
        let asset_info =
            AssetInfo::cfa("CFA asset name", 0, None, "CFA terms", None, issued_supply);
        self.issue_with_info(asset_info, close_method, outpoint)
    }

    pub fn invoice(
        &mut self,
        contract_id: ContractId,
        iface_type_name: &TypeName,
        amount: u64,
        close_method: CloseMethod,
        outpoint: Option<&(String, u32)>,
    ) -> RgbInvoice {
        let network = self.stored_wallet.wallet().network();
        let beneficiary = if let Some((txid, vout)) = outpoint {
            let seal = XChain::Bitcoin(GraphSeal::new_random(
                close_method,
                Txid::from_str(txid).unwrap(),
                *vout,
            ));
            self.stored_wallet
                .stock_mut()
                .store_secret_seal(seal)
                .unwrap();
            Beneficiary::BlindedSeal(*seal.to_secret_seal().as_reduced_unsafe())
        } else {
            let address = self
                .stored_wallet
                .wallet()
                .addresses(RgbKeychain::Rgb)
                .next()
                .expect("no addresses left")
                .addr;
            Beneficiary::WitnessVout(address.payload)
        };

        let mut builder = RgbInvoiceBuilder::new(XChainNet::bitcoin(network, beneficiary))
            .set_contract(contract_id)
            .set_interface(iface_type_name.clone());
        if *iface_type_name == AssetSchema::Uda.iface_type_name() {
            builder = builder.clone().set_allocation(UDA_FIXED_INDEX, 1).unwrap();
        } else {
            builder = builder.clone().set_amount_raw(amount);
        }
        builder.finish()
    }

    pub fn transfer(&mut self, invoice: RgbInvoice, sats: u64) -> (Transfer, String) {
        self.sync();

        let fee = Sats::from_sats(400u64);
        let sats = Sats::from_sats(sats);
        let params = TransferParams::with(fee, sats);
        let (psbt, _psbt_meta, consignment) = self.stored_wallet.pay(&invoice, params).unwrap();

        let secp = Secp256k1::new();

        let mut key_provider = MemoryKeyProvider::with(&secp, true);
        key_provider.add_account(self.account.clone());

        let mut psbt = DwPsbt::deserialize(&psbt.serialize(PsbtVer::V0)).unwrap();

        let _sig_count = psbt.sign_all(&key_provider).unwrap();

        let mut psbt = consensus::encode::deserialize::<BitcoinPsbt>(&psbt.serialize()).unwrap();

        psbt.finalize_mut(&secp).unwrap();

        let tx = psbt.extract_tx();

        let tx = Tx::consensus_deserialize(tx.serialize()).unwrap();

        let indexer = get_indexer();
        broadcast_tx(&indexer, &tx);

        let txid = tx.txid().to_string();
        println!("transfer txid: {txid:?}");

        (consignment, txid)
    }

    pub fn accept_transfer(&mut self, consignment: Transfer) {
        self.sync();
        let mut resolver = get_resolver();
        let validated_consignment = consignment.validate(&mut resolver, true).unwrap();
        let validation_status = validated_consignment.clone().into_validation_status();
        let validity = validation_status.validity();
        assert_eq!(validity, Validity::Valid);
        let mut attempts = 0;
        while let Err(e) = self
            .stored_wallet
            .stock_mut()
            .accept_transfer(validated_consignment.clone(), &mut resolver)
        {
            attempts += 1;
            if attempts > 3 {
                panic!("error accepting transfer: {e}");
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    pub fn contract_iface(
        &self,
        contract_id: ContractId,
        iface_type_name: &TypeName,
    ) -> ContractIface {
        self.stored_wallet
            .stock()
            .contract_iface(contract_id, iface_type_name.clone())
            .unwrap()
    }

    pub fn contract_fungible_allocations(
        &self,
        contract_iface: &ContractIface,
    ) -> Vec<FungibleAllocation> {
        contract_iface
            .fungible(fname!("assetOwner"), &self.stored_wallet.wallet().filter())
            .unwrap()
            .collect()
    }

    pub fn contract_data_allocations(&self, contract_iface: &ContractIface) -> Vec<DataAllocation> {
        contract_iface
            .data(fname!("assetOwner"), &self.stored_wallet.wallet().filter())
            .unwrap()
            .collect()
    }

    pub fn debug_logs(&self, contract_id: ContractId, iface_type_name: &TypeName) {
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
        for owned in &contract.iface.assignments {
            println!("  {}:", owned.name);
            if let Ok(allocations) =
                contract.fungible(owned.name.clone(), &self.stored_wallet.wallet().filter())
            {
                for allocation in allocations {
                    println!(
                        "    amount={}, utxo={}, witness={} # owned by the wallet",
                        allocation.state, allocation.seal, allocation.witness
                    );
                }
            }
            if let Ok(allocations) = contract.fungible(
                owned.name.clone(),
                &FilterExclude(&self.stored_wallet.wallet().filter()),
            ) {
                for allocation in allocations {
                    println!(
                        "    amount={}, utxo={}, witness={} # owner unknown",
                        allocation.state, allocation.seal, allocation.witness
                    );
                }
            }
        }

        let bp_runtime = self.stored_wallet.wallet();
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
}
