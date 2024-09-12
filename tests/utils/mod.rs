pub mod chain;
pub mod helpers;

pub const TEST_DATA_DIR: &str = "test-data";
pub const INTEGRATION_DATA_DIR: &str = "integration";
pub const STRESS_DATA_DIR: &str = "stress";

pub const ELECTRUM_REGTEST_URL: &str = "127.0.0.1:50001";
pub const ELECTRUM_MAINNET_URL: &str = "ssl://electrum.iriswallet.com:50003";
pub const ESPLORA_REGTEST_URL: &str = "http://127.0.0.1:8094/regtest/api";
pub const ESPLORA_MAINNET_URL: &str = "https://blockstream.info/api";
pub const FAKE_TXID: &str = "e5a3e577309df31bd606f48049049d2e1e02b048206ba232944fcc053a176ccb:0";
pub const UDA_FIXED_INDEX: u32 = 0;

pub use std::{
    cell::OnceCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env::VarError,
    ffi::OsString,
    fmt::{self, Display},
    fs::OpenOptions,
    io::Write,
    path::{PathBuf, MAIN_SEPARATOR},
    process::{Command, Stdio},
    str::FromStr,
    sync::{Mutex, Once, OnceLock, RwLock},
    time::{Duration, Instant},
};

pub use amplify::{
    bmap,
    confinement::{Confined, U16},
    map, s, ByteArray, Wrapper,
};
use bitcoin_hashes::{sha256, Hash};
pub use bp::{
    seals::txout::{BlindSeal, CloseMethod, ExplicitSeal},
    ConsensusDecode, Outpoint, Sats, ScriptPubkey, Tx, Txid, Vout,
};
pub use bpstd::{
    h, signers::TestnetSigner, Address, DerivationPath, DerivationSeg, DerivedAddr, HardenedIndex,
    Keychain, Network, Terminal, XkeyOrigin, Xpriv, XprivAccount, Xpub, XpubAccount, XpubDerivable,
    XpubFp,
};
pub use bpwallet::{
    fs::FsTextStore, indexers::esplora::Client as EsploraClient, AnyIndexer, Indexer as BpIndexer,
    Wallet,
};
pub use descriptors::Wpkh;
pub use electrum::{Client as ElectrumClient, ElectrumApi, Param};
pub use file_format::FileFormat;
pub use ifaces::{
    rgb20, rgb21,
    rgb21::{EmbeddedMedia, TokenData},
    rgb25, IssuerWrapper, Rgb20, Rgb21, Rgb25,
};
pub use once_cell::sync::Lazy;
pub use psbt::{
    Beneficiary as PsbtBeneficiary, Payment, Prevout, Psbt, PsbtConstructor, PsbtMeta, PsbtVer,
};
pub use psrgbt::{RgbExt, RgbInExt, RgbPsbt, TxParams};
pub use rand::RngCore;
pub use rgb::{
    invoice::Pay2Vout,
    persistence::{ContractStateRead, MemContract, MemContractState, Stock},
    resolvers::AnyResolver,
    stl::ContractTerms,
    validation::{Failure, ResolveWitness, Scripts, Validity, WitnessResolverError},
    vm::{WitnessOrd, WitnessPos, XWitnessTx},
    BlindingFactor, DescriptorRgb, GenesisSeal, GraphSeal, Identity, RgbDescr, RgbKeychain,
    RgbWallet, TapretKey, TransferParams, Transition, WalletProvider, XOutpoint, XWitnessId,
};
pub use rgbstd::{
    containers::{BuilderSeal, ConsignmentExt, Fascia, FileContent, Kit, Transfer, ValidKit},
    interface::{
        ContractBuilder, ContractIface, DataAllocation, FilterExclude, FungibleAllocation, Iface,
        IfaceClass, IfaceId, IfaceImpl, NamedField,
    },
    invoice::{Beneficiary, RgbInvoice, RgbInvoiceBuilder, XChainNet},
    persistence::{fs::FsBinStore, PersistedState, SchemaIfaces, StashReadProvider},
    schema::SchemaId,
    stl::{
        AssetSpec, Attachment, Details, MediaType, Name, ProofOfReserves, RicardianContract, Ticker,
    },
    Allocation, Amount, ContractId, GlobalStateType, Layer1, Operation, OwnedFraction, Precision,
    Schema, TokenIndex, TxoSeal, XChain,
};
pub use rstest::rstest;
pub use schemata::{CollectibleFungibleAsset, NonInflatableAsset, UniqueDigitalAsset};
pub use strict_encoding::{fname, tn, FieldName, StrictSerialize, TypeName};
pub use strict_types::{StrictVal, TypeSystem};
pub use strum::IntoEnumIterator;
pub use strum_macros::EnumIter;
pub use time::OffsetDateTime;

pub use crate::utils::{chain::*, helpers::*};
