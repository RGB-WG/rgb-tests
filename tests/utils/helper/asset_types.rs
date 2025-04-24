use super::*;

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
    pub allocations: Vec<(TxoSeal, u64)>,
}

/// Parameters for FUA (Fractional unique asset) issuance
#[derive(Clone)]
pub struct FUAIssueParams {
    /// Asset name
    pub name: String,
    /// Asset details
    pub details: String,
    /// Decimal precision for the asset
    pub precision: String,
    /// Total circulating supply
    pub circulating_supply: u64,
    /// Initial token allocations (outpoint, amount)
    pub initial_allocations: Vec<(Outpoint, u64)>,
}

impl Default for FUAIssueParams {
    fn default() -> Self {
        Self {
            name: "DemoFUA".to_string(),
            details: "Demo FUA details".to_string(),
            precision: "centiMilli".to_string(),
            circulating_supply: 10_000,
            initial_allocations: vec![],
        }
    }
}

impl FUAIssueParams {
    /// Create new CFA issuance parameters
    pub fn new(
        name: impl Into<String>,
        details: impl Into<String>,
        precision: impl Into<String>,
        circulating_supply: u64,
    ) -> Self {
        Self {
            name: name.into(),
            details: details.into(),
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

/// Parameters for FAC (Fractionable Asset Collection) issuance
#[derive(Clone)]
pub struct FACIssueParams {
    /// Collection name
    pub name: String,
    /// Collection details
    pub details: String,
    /// Total number of fractions
    pub total_fractions: u64,
    /// Token index
    pub index: u32,
    /// Initial token allocation (outpoint, amount)
    pub initial_allocation: Option<(Outpoint, u64)>,
    /// NFT specification
    pub nft_spec: Option<NftSpec>,
}

impl Default for FACIssueParams {
    fn default() -> Self {
        Self {
            name: "FAC".to_string(),
            details: "Demo FAC details".to_string(),
            total_fractions: 10_000,
            index: UDA_FIXED_INDEX,
            initial_allocation: None,
            nft_spec: None,
        }
    }
}

impl FACIssueParams {
    /// Create new FAC issuance parameters
    pub fn new(name: impl Into<String>, details: impl Into<String>, total_fractions: u64) -> Self {
        Self {
            name: name.into(),
            details: details.into(),
            index: UDA_FIXED_INDEX,
            total_fractions,
            initial_allocation: None,
            nft_spec: None,
        }
    }

    /// Set token allocation
    pub fn with_allocation(&mut self, outpoint: Outpoint, amount: u64) -> &mut Self {
        self.initial_allocation = Some((outpoint, amount));
        self
    }

    /// Set NFT specification
    pub fn with_nft_spec(&mut self, nft_spec: NftSpec) -> &mut Self {
        self.nft_spec = Some(nft_spec);
        self
    }
}

/// Create a minimal NFT spec with just index
pub fn nft_spec_minimal() -> NftSpec {
    NftSpec {
        index: TokenIndex::from(UDA_FIXED_INDEX),
        ..Default::default()
    }
}

/// Create a complete NFT spec with all details
pub fn nft_spec(
    ticker: &str,
    name: &str,
    details: &str,
    preview: EmbeddedMedia,
    media: Attachment,
    attachments: BTreeMap<u8, Attachment>,
    reserves: ProofOfReserves,
) -> NftSpec {
    let mut nft_spec = nft_spec_minimal();
    nft_spec.preview = Some(preview);
    nft_spec.media = Some(media);
    nft_spec.attachments = Confined::try_from(attachments.clone()).unwrap();
    nft_spec.reserves = Some(reserves);
    nft_spec.ticker = Some(Ticker::try_from(ticker.to_string()).unwrap());
    nft_spec.name = Some(AssetName::try_from(name.to_string()).unwrap());
    nft_spec.details = Some(Details::from_str(details).unwrap());
    nft_spec
}

/// Function to create a file attachment from a file path
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
    pub fractions: Vec<(Outpoint, u64)>, // (outpoint, amount)
}

/// Complete RGB21 contract state
#[derive(Debug, Clone)]
pub struct RGB21ContractState {
    pub immutable: RGB21ContractImmutableState,
    pub owned: RGB21ContractOwnedState,
}
