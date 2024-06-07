pub mod api;
pub mod chain;

pub const ELECTRUM_URL: &str = "127.0.0.1:50001";
pub const ESPLORA_URL: &str = "http://127.0.0.1:8094/regtest/api";
pub const FAKE_TXID: &str = "e5a3e577309df31bd606f48049049d2e1e02b048206ba232944fcc053a176ccb:0";
pub const UDA_FIXED_INDEX: u32 = 0;

pub use std::{
    cell::OnceCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    env::VarError,
    ffi::OsString,
    fmt::{self, Display},
    path::{PathBuf, MAIN_SEPARATOR},
    process::{Command, Stdio},
    str::FromStr,
    sync::{Mutex, Once, OnceLock, RwLock},
};

pub use amplify::{
    confinement::{Confined, U16},
    map, s, Wrapper,
};
pub use bitcoin::{
    consensus,
    hashes::{sha256, Hash},
    util::{
        bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey},
        key::Secp256k1,
        psbt::{serialize::Serialize, Psbt as BitcoinPsbt},
    },
};
pub use bp::{
    seals::txout::{BlindSeal, CloseMethod},
    ConsensusDecode, Outpoint, Sats, Tx, Txid,
};
pub use bpstd::{
    DerivationPath as BpDerivationPath, DerivationSeg, HardenedIndex, Keychain, Network, Xpub,
    XpubDerivable, XpubFp, XpubOrigin,
};
pub use bpwallet::{AnyIndexer, Runtime as BpRuntime, Wallet as BpWallet};
pub use descriptors::Wpkh;
pub use dw_psbt::{
    serialize::Deserialize,
    sign::{MemoryKeyProvider, MemorySigningAccount, SignAll},
    Psbt as DwPsbt,
};
pub use electrum::{Client as ElectrumClient, ElectrumApi};
pub use esplora::Builder as EsploraClient;
pub use ifaces::{
    rgb20, rgb21,
    rgb21::{EmbeddedMedia, TokenData},
    rgb25, IssuerWrapper, Rgb20, Rgb21, Rgb25,
};
pub use miniscript::psbt::PsbtExt;
pub use once_cell::sync::Lazy;
pub use psbt::PsbtVer;
pub use rand::RngCore;
pub use rgb::{
    invoice::Pay2Vout, stl::ContractTerms, AnyResolver, DescriptorRgb, GenesisSeal, GraphSeal,
    Identity, RgbDescr, RgbKeychain, StoredStock, StoredWallet, TapretKey, TransferParams,
    WalletProvider,
};
pub use rgbstd::{
    containers::{BuilderSeal, FileContent, Kit, Transfer, ValidKit},
    interface::{
        ContractBuilder, ContractIface, DataAllocation, FilterExclude, FungibleAllocation, Iface,
        IfaceClass, IfaceId, IfaceImpl, NamedField,
    },
    invoice::{Beneficiary, RgbInvoice, RgbInvoiceBuilder, XChainNet},
    persistence::{
        fs::{LoadFs, StoreFs},
        PersistedState, SchemaIfaces, StashReadProvider, Stock,
    },
    schema::SchemaId,
    stl::{
        AssetSpec, Attachment, Details, MediaType, Name, ProofOfReserves, RicardianContract, Ticker,
    },
    validation::{Scripts, Validity},
    Allocation, Amount, ContractId, GlobalStateType, Layer1, OwnedFraction, Precision, Schema,
    TokenIndex, TxoSeal, XChain,
};
pub use rstest::rstest;
pub use schemata::{CollectibleFungibleAsset, NonInflatableAsset, UniqueDigitalAsset};
pub use strict_encoding::{fname, tn, FieldName, StrictSerialize, TypeName};
pub use strict_types::{StrictVal, TypeSystem};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
pub use time::OffsetDateTime;

pub use crate::utils::{api::*, chain::*};
