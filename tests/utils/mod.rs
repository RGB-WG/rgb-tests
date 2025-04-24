pub mod chain;
pub mod helper;

pub const TEST_DATA_DIR: &str = "test-data";
pub const SCHEMATA_DIR: &str = "tests/templates/schemata";
pub const ISSUANCE_DIR: &str = "tests/templates/issuance";
pub const NON_INFLATABLE_ASSET_TEMPLATE_PATH: &str = "tests/templates/issuance/NFA.yaml";
pub const FRACTIONAL_UNIQUE_ASSET_TEMPLATE_PATH: &str = "tests/templates/issuance/FUA.yaml";
pub const FRACTIONABLE_ASSET_COLLECTION_TEMPLATE_PATH: &str = "tests/templates/issuance/FAC.yaml";
pub const INTEGRATION_DATA_DIR: &str = "integration";
pub const STRESS_DATA_DIR: &str = "stress";

pub const ELECTRUM_1_REGTEST_URL: &str = "127.0.0.1:50001";
pub const ELECTRUM_2_REGTEST_URL: &str = "127.0.0.1:50002";
pub const ELECTRUM_3_REGTEST_URL: &str = "127.0.0.1:50003";
pub const ELECTRUM_MAINNET_URL: &str = "ssl://electrum.iriswallet.com:50003";
pub const ESPLORA_1_REGTEST_URL: &str = "http://127.0.0.1:3001";
pub const ESPLORA_2_REGTEST_URL: &str = "http://127.0.0.1:3002";
pub const ESPLORA_3_REGTEST_URL: &str = "http://127.0.0.1:3003";
pub const ESPLORA_MAINNET_URL: &str = "https://blockstream.info/api";
pub const FAKE_TXID: &str = "e5a3e577309df31bd606f48049049d2e1e02b048206ba232944fcc053a176ccb:0";
pub const UDA_FIXED_INDEX: u32 = 0;
pub const DEFAULT_FEE_ABS: u64 = 400;

pub const INSTANCE_1: u8 = 1;
pub const INSTANCE_2: u8 = 2;
pub const INSTANCE_3: u8 = 3;

pub use std::{
    cell::OnceCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env::VarError,
    ffi::OsString,
    fmt::{self, Display},
    fs::{File, OpenOptions},
    io::Write,
    num::NonZeroU32,
    ops::Deref,
    path::{Path, PathBuf, MAIN_SEPARATOR},
    process::{Command, Stdio},
    str::FromStr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex, Once, OnceLock, RwLock,
    },
    time::{Duration, Instant},
};

pub use amplify::{
    bmap,
    confinement::{Confined, U16},
    map, s, ByteArray, Wrapper,
};
pub use bitcoin_hashes::{sha256, Hash};
pub use bp::{
    seals::WTxoSeal, ConsensusDecode, Outpoint, Sats, ScriptPubkey, SeqNo, Tx, Txid, Vout,
};
pub use bpstd::{
    h, signers::TestnetSigner, Address, DerivationPath, DerivationSeg, DerivedAddr, Descriptor,
    HardenedIndex, Keychain, Network, Terminal, XkeyOrigin, Xpriv, XprivAccount, Xpub, XpubAccount,
    XpubDerivable, XpubFp,
};
pub use bpwallet::{
    fs::FsTextStore, indexers::esplora::Client as EsploraClient, seals::TxoSeal, AnyIndexer,
    Indexer as BpIndexer, Wallet,
};
pub use commit_verify::{Digest, DigestExt, Sha256};
pub use descriptors::Wpkh;
pub use electrum::{Client as ElectrumClient, ElectrumApi, Param};
pub use file_format::FileFormat;
pub use ifaces::{
    AssetName, Attachment, Details, EmbeddedMedia, MediaType, NftSpec, ProofOfReserves, Ticker,
    TokenIndex,
};
pub use indexmap::IndexMap;
pub use once_cell::sync::Lazy;
pub use psbt::{
    Beneficiary as PsbtBeneficiary, Payment, Prevout, Psbt, PsbtConstructor, PsbtMeta, PsbtVer,
    TxParams,
};
pub use rand::RngCore;
pub use rgb::{
    invoice::{RgbBeneficiary, RgbInvoice},
    popls::bp::{Coinselect, OpRequestSet, PaymentScript, PrefabBundle, RgbWallet, WalletProvider},
    AuthToken, CallScope, CellAddr, CodexId, Consensus, ContractId, ContractInfo, Contracts,
    CreateParams, EitherSeal, RgbSealDef, Schema, StateCalc, StockpileDir,
};
pub use rgbp::{
    descriptor::RgbDescr, CoinselectStrategy, Owner, PayError, RgbRuntime, RgbpRuntimeDir,
};
pub use rgpsbt::ScriptResolver;
pub use rstest::rstest;
pub use serial_test::serial;
pub use strict_encoding::{fname, tn, StrictSerialize};
pub use strict_types::{
    value::{Blob, StrictNum, StrictVal},
    FieldName, TypeName, TypeSystem, VariantName,
};
pub use strum::IntoEnumIterator;
pub use strum_macros::EnumIter;
pub use tabled::{
    settings::{object::Columns, Alignment, Modify, Style},
    Table, Tabled,
};
pub use time::OffsetDateTime;

pub const KEY_CHAIN_RGB: u8 = 9;
pub const KEY_CHAIN_TAPRET: u8 = 10;

pub use helper::asset_params::AssetParamsBuilder;
pub use helper::asset_types::{
    attachment_from_fpath, nft_spec, nft_spec_minimal, FACIssueParams, FUAIssueParams,
    NIAIssueParams,
};
pub use helper::coinselect::CustomCoinselectStrategy;
pub use helper::reporting::Report;
pub use helper::wallet::{DescriptorType, InvoiceType, TestWallet, TransferType};

pub use chain::{fund_wallet, indexer_url, is_tx_confirmed, mine_custom, Indexer, INDEXER};
pub use helper::asset_types::{ContractImmutableState, ContractOwnedState, ContractState};
pub use rgb::{Assignment, NamedState, StateAtom};
pub use rgb::{ConsumeError, ImmutableState, OwnedState, StateName};
pub use strict_types::value::EnumTag;
