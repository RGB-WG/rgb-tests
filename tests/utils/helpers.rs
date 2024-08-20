use std::fs;
use std::io::Write;
use rgb::PayError;
use super::*;

pub struct TestWallet {
    master_fp: XpubFp,
    wallet: RgbWallet<Wallet<XpubDerivable, RgbDescr>>,
    descriptor: RgbDescr,
    signer: TestnetSigner,
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

pub fn get_wallet(descriptor_type: DescriptorType) -> TestWallet {
    let mut seed = vec![0u8; 128];
    rand::thread_rng().fill_bytes(&mut seed);

    let xpriv = Xpriv::new_master(true, &seed);

    let xpub = xpriv.to_xpub();

    let master_fp = xpub.fingerprint();
    let derivation = DerivationPath::<HardenedIndex>::from_str("86'/1'/0'").unwrap();
    let origin = XkeyOrigin::new(master_fp, derivation);

    let signer_account = XprivAccount::new(xpriv, origin.clone());
    let signer = TestnetSigner::new(signer_account);

    let rgb_dir = PathBuf::from("tests")
        .join("tmp")
        .join(master_fp.to_string());
    fs::create_dir_all(&rgb_dir).unwrap();
    println!("wallet dir: {rgb_dir:?}");

    let mut keychains = vec![
        RgbKeychain::Internal,
        RgbKeychain::External,
        RgbKeychain::Rgb,
    ];
    if descriptor_type == DescriptorType::Tr {
        keychains.push(RgbKeychain::Tapret);
    }
    let xpub_derivable = XpubDerivable::try_custom(xpub, origin, keychains.into_iter().map(Keychain::from)).unwrap();
    let descriptor = match descriptor_type {
        DescriptorType::Wpkh => RgbDescr::Wpkh(Wpkh::from(xpub_derivable)),
        DescriptorType::Tr => RgbDescr::TapretKey(TapretKey::from(xpub_derivable)),
    };

    let mut bp_wallet: Wallet<XpubDerivable, RgbDescr> =
        Wallet::new_layer1(descriptor.clone(), Network::Regtest);
    let name = s!("wallet_name");
    let dir = rgb_dir.join(&name);
    bp_wallet.set_name(name);
    bp_wallet
        .set_fs_config(FsConfig {
            path: dir,
            autosave: true,
        })
        .unwrap();
    let stock = Stock::new(rgb_dir.to_owned());
    let mut wallet = RgbWallet::new(stock, bp_wallet);

    for asset_schema in AssetSchema::iter() {
        let valid_kit = asset_schema.get_valid_kit();
        wallet.stock_mut().import_kit(valid_kit).unwrap();
    }

    let mut wallet = TestWallet {
        master_fp,
        wallet,
        descriptor,
        signer,
    };

    wallet.sync();

    wallet
}

fn get_indexer() -> AnyIndexer {
    match INDEXER.get().unwrap() {
        Indexer::Electrum => {
            AnyIndexer::Electrum(Box::new(ElectrumClient::new(ELECTRUM_URL).unwrap()))
        }
        Indexer::Esplora => {
            AnyIndexer::Esplora(Box::new(EsploraClient::new_esplora(ESPLORA_URL).unwrap()))
        }
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
            inner.publish(tx).unwrap();
        }
        _ => unreachable!("unsupported indexer"),
    }
}

pub fn attachment_from_fpath(fpath: &str) -> Attachment {
    let file_bytes = std::fs::read(fpath).unwrap();
    let file_hash: sha256::Hash = Hash::hash(&file_bytes[..]);
    let digest = file_hash.to_byte_array().into();
    let mime = tree_magic_mini::from_filepath(fpath.as_ref())
        .unwrap()
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

impl TestWallet {
    pub fn get_utxo(&mut self) -> Outpoint {
        let address = self
            .wallet
            .wallet()
            .addresses(RgbKeychain::for_method(self.close_method()))
            .next()
            .expect("no addresses left")
            .addr
            .to_string();
        let txid = fund_wallet(address);
        self.sync();
        let mut vout = None;
        let coins = self.wallet.wallet().address_coins();
        assert!(!coins.is_empty());
        for (_derived_addr, utxos) in coins {
            for utxo in utxos {
                if utxo.outpoint.txid.to_string() == txid {
                    vout = Some(utxo.outpoint.vout_u32());
                }
            }
        }
        Outpoint {
            txid: Txid::from_str(&txid).unwrap(),
            vout: Vout::from_u32(vout.unwrap()),
        }
    }

    pub fn sync(&mut self) {
        let indexer = get_indexer();
        self.wallet
            .wallet_mut()
            .update(&indexer)
            .into_result()
            .unwrap();
    }

    pub fn close_method(&self) -> CloseMethod {
        self.wallet.wallet().seal_close_method()
    }

    pub fn issue_with_info(
        &mut self,
        asset_info: AssetInfo,
        close_method: CloseMethod,
        outpoint: Option<&Outpoint>,
    ) -> (ContractId, TypeName) {
        let outpoint = if let Some(outpoint) = outpoint {
            *outpoint
        } else {
            self.get_utxo()
        };

        let blind_seal = match close_method {
            CloseMethod::TapretFirst => BlindSeal::tapret_first_rand(outpoint.txid, outpoint.vout),
            CloseMethod::OpretFirst => BlindSeal::opret_first_rand(outpoint.txid, outpoint.vout),
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
        let resolver = get_resolver();
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
        outpoint: Option<&Outpoint>,
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
        outpoint: Option<&Outpoint>,
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
        invoice_type: InvoiceType,
    ) -> RgbInvoice {
        let network = self.wallet.wallet().network();
        let beneficiary = match invoice_type {
            InvoiceType::Blinded(outpoint) => {
                let outpoint = if let Some(outpoint) = outpoint {
                    outpoint
                } else {
                    self.get_utxo()
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
                let address = self
                    .wallet
                    .wallet()
                    .addresses(RgbKeychain::Rgb)
                    .next()
                    .expect("no addresses left")
                    .addr;
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

    pub fn transfer(
        &mut self,
        invoice: RgbInvoice,
        sats: Option<u64>,
        fee: Option<u64>,
    ) -> (Transfer, Tx) {
        self.sync();

        let fee = Sats::from_sats(fee.unwrap_or(400));
        let sats = Sats::from_sats(sats.unwrap_or(2000));
        let params = TransferParams::with(fee, sats);
        let (mut psbt, _psbt_meta, consignment) = self.wallet.pay(&invoice, params).map(|(psbt, meta, cs)| (psbt, Some(meta), Ok(cs))).or_else(|e| match e {
            PayError::Composition(_) => Err(e),
            PayError::Completion(err, psbt) => Ok((psbt, None, Err(err)))
        }).unwrap();

        let _sig_count = psbt.sign(&self.signer).unwrap();
        psbt.finalize(&self.descriptor);
        let tx = psbt.extract().unwrap();
        let txid = tx.txid().to_string();

        let mut path = PathBuf::from("tests")
            .join("tmp")
            .join(self.master_fp.to_string())
            .join("tx");
        let _ = fs::create_dir(&path); // no need to panic if the dir already exists
        path.push(txid.to_string());
        path.set_extension("yaml");
        let mut file = fs::File::create_new(path).unwrap();
        serde_yaml::to_writer(&mut file, &tx).unwrap();
        writeln!(file, "\n---").unwrap();
        serde_yaml::to_writer(&mut file, &psbt).unwrap();

        let indexer = get_indexer();
        broadcast_tx(&indexer, &tx);

        println!("transfer txid: {txid:?}");

        (consignment.unwrap(), tx)
    }

    pub fn accept_transfer(&mut self, consignment: Transfer) {
        self.sync();
        let resolver = get_resolver();
        let validated_consignment = consignment.validate(&resolver, true).unwrap();
        let validation_status = validated_consignment.clone().into_validation_status();
        let validity = validation_status.validity();
        assert_eq!(validity, Validity::Valid);
        let mut attempts = 0;
        while let Err(e) = self
            .wallet
            .stock_mut()
            .accept_transfer(validated_consignment.clone(), &resolver)
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

    pub fn contract_fungible_allocations<S: ContractStateRead>(
        &self,
        contract_iface: &ContractIface<S>,
    ) -> Vec<FungibleAllocation> {
        contract_iface
            .fungible(fname!("assetOwner"), &self.wallet.wallet().filter())
            .unwrap()
            .collect()
    }

    pub fn contract_data_allocations<S: ContractStateRead>(
        &self,
        contract_iface: &ContractIface<S>,
    ) -> Vec<DataAllocation> {
        contract_iface
            .data(fname!("assetOwner"), &self.wallet.wallet().filter())
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
                contract.fungible(owned.name.clone(), &self.wallet.wallet().filter())
            {
                for allocation in allocations {
                    println!(
                        "    amount={}, utxo={}, witness={:?} # owned by the wallet",
                        allocation.state, allocation.seal, allocation.witness
                    );
                }
            }
            if let Ok(allocations) = contract.fungible(
                owned.name.clone(),
                &FilterExclude(&self.wallet.wallet().filter()),
            ) {
                for allocation in allocations {
                    println!(
                        "    amount={}, utxo={}, witness={:?} # owner unknown",
                        allocation.state, allocation.seal, allocation.witness
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

    pub fn send(
        &mut self,
        recv_wlt: &mut TestWallet,
        transfer_type: TransferType,
        contract_id: ContractId,
        iface_type_name: &TypeName,
        amount: u64,
        sats: u64,
    ) -> (Transfer, Tx) {
        let invoice = match transfer_type {
            TransferType::Blinded => recv_wlt.invoice(
                contract_id,
                iface_type_name,
                amount,
                recv_wlt.close_method(),
                InvoiceType::Blinded(None),
            ),
            TransferType::Witness => recv_wlt.invoice(
                contract_id,
                iface_type_name,
                amount,
                recv_wlt.close_method(),
                InvoiceType::Witness,
            ),
        };
        let (consignment, tx) = self.transfer(invoice, Some(sats), None);
        mine(false);
        recv_wlt.accept_transfer(consignment.clone());
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
        let contract_iface = self.contract_iface(contract_id, iface_type_name);
        match asset_schema {
            AssetSchema::Nia | AssetSchema::Cfa => {
                let allocations = self.contract_fungible_allocations(&contract_iface);
                assert_eq!(allocations.len(), expected_fungible_allocations.len());
                assert!(allocations
                    .iter()
                    .all(|a| a.seal.method() == self.close_method()));
                for amount in expected_fungible_allocations {
                    assert_eq!(
                        allocations
                            .iter()
                            .filter(|a| a.state == Amount::from(amount))
                            .count(),
                        1
                    );
                }
            }
            AssetSchema::Uda => {
                let allocations = self.contract_data_allocations(&contract_iface);
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
}
