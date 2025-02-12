pub mod chain;
pub mod helpers;

pub const TEST_DATA_DIR: &str = "test-data";
pub const INTEGRATION_DATA_DIR: &str = "integration";
pub const STRESS_DATA_DIR: &str = "stress";

pub const ELECTRUM_1_REGTEST_URL: &str = "127.0.0.1:50001";
pub const ELECTRUM_2_REGTEST_URL: &str = "127.0.0.1:50002";
pub const ELECTRUM_3_REGTEST_URL: &str = "127.0.0.1:50003";
pub const ELECTRUM_MAINNET_URL: &str = "ssl://electrum.iriswallet.com:50003";
pub const ESPLORA_1_REGTEST_URL: &str = "http://127.0.0.1:8094/regtest/api";
pub const ESPLORA_2_REGTEST_URL: &str = "http://127.0.0.1:8095/regtest/api";
pub const ESPLORA_3_REGTEST_URL: &str = "http://127.0.0.1:8096/regtest/api";
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
    fs::OpenOptions,
    io::Write,
    num::NonZeroU32,
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
    // seals::txout::{BlindSeal, CloseMethod, ExplicitSeal},
    ConsensusDecode,
    Outpoint,
    Sats,
    ScriptPubkey,
    SeqNo,
    Tx,
    Txid,
    Vout,
};
pub use bpstd::{
    h, signers::TestnetSigner, Address, DerivationPath, DerivationSeg, DerivedAddr, Descriptor,
    HardenedIndex, Keychain, Network, Terminal, XkeyOrigin, Xpriv, XprivAccount, Xpub, XpubAccount,
    XpubDerivable, XpubFp,
};
pub use bpwallet::{
    fs::FsTextStore, indexers::esplora::Client as EsploraClient, AnyIndexer, Indexer as BpIndexer,
    Wallet, WalletUtxo,
};
pub use descriptors::Wpkh;
pub use electrum::{Client as ElectrumClient, ElectrumApi, Param};
pub use file_format::FileFormat;
// pub use ifaces::{
//     rgb20, rgb21,
//     rgb21::{EmbeddedMedia, TokenData},
//     rgb25, IssuerWrapper, Rgb20, Rgb21, Rgb25,
// };
pub use once_cell::sync::Lazy;
pub use psbt::{
    Beneficiary as PsbtBeneficiary, Payment, Prevout, Psbt, PsbtConstructor, PsbtMeta, PsbtVer,
};
// pub use psrgbt::{RgbExt, RgbInExt, RgbPsbt, TxParams};
pub use rand::RngCore;
pub use rgbp::wallet::OpretWallet;
// pub use rgb::{
//     info::ContractInfo,
//     interface::{AllocatedState, AssignmentsFilter, ContractOp, OpDirection},
//     invoice::Pay2Vout,
//     persistence::{MemContract, MemContractState, Stock},
//     resolvers::AnyResolver,
//     stl::ContractTerms,
//     validation::{Failure, ResolveWitness, Scripts, Validity, WitnessResolverError},
//     vm::{WitnessOrd, WitnessPos, XWitnessTx},
//     BlindingFactor, DescriptorRgb, GenesisSeal, GraphSeal, Identity, OpId, RgbDescr, RgbKeychain,
//     RgbWallet, TapretKey, TransferParams, Transition, WalletProvider, XOutpoint, XWitnessId,
// };
use rgb::{ContractId, Stock};

// pub use rgbstd::{
//     containers::{
//         BuilderSeal, ConsignmentExt, Fascia, FileContent, IndexedConsignment, Kit, Transfer,
//         ValidKit,
//     },
//     interface::{
//         ContractBuilder, ContractIface, DataAllocation, FilterExclude, FungibleAllocation, Iface,
//         IfaceClass, IfaceId, IfaceImpl, NamedField,
//     },
//     invoice::{Beneficiary, RgbInvoice, RgbInvoiceBuilder, XChainNet},
//     persistence::{fs::FsBinStore, PersistedState, SchemaIfaces, StashReadProvider},
//     schema::SchemaId,
//     stl::{
//         AssetSpec, Attachment, Details, MediaType, Name, ProofOfReserves, RicardianContract, Ticker,
//     },
//     Allocation, Amount, ContractId, GlobalStateType, KnownState, Layer1, Operation,
//     OutputAssignment, OwnedFraction, Precision, Schema, TokenIndex, TxoSeal, XChain,
// };
pub use rstest::rstest;
// pub use schemata::{CollectibleFungibleAsset, NonInflatableAsset, UniqueDigitalAsset};
pub use serial_test::serial;
pub use strict_encoding::{fname, tn, FieldName, StrictSerialize, TypeName};
pub use strict_types::{StrictVal, TypeSystem};
pub use strum::IntoEnumIterator;
pub use strum_macros::EnumIter;
pub use time::OffsetDateTime;

pub use crate::utils::{chain::*, helpers::*};
