use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use amplify::ByteArray;
use bpstd::XpubDerivable;
use invoice::Network as InvoiceNetwork;
// use bpwallet::cli::ResolverOpt;
use bpwallet::fs::FsTextStore;
use bpwallet::Wpkh;
use rgb::popls::bp::file::{DirBarrow, DirMound};
use rgb::SealType;
use rgbp::descriptor::{Opret, Tapret};
use rgbp::wallet::file::DirRuntime;
use rgbp::wallet::{OpretWallet, TapretWallet};

#[derive(Debug, Clone)]
struct Context {
    data_dir: PathBuf,
    seal: SealType,
    name: Option<String>,
}

fn build_wallet_file_path(context: Context) -> PathBuf {
    let path = context.data_dir.join(context.seal.to_string());
    path.join(context.name.unwrap_or("default".to_string()))
}

fn create_wallet_provider(context: Context) -> FsTextStore {
    let wallet_file_path = build_wallet_file_path(context);
    FsTextStore::new(wallet_file_path).expect("Broken directory structure")
}

fn create_wallet(context: Context, descriptor: &str, network: InvoiceNetwork) {
    let provider = create_wallet_provider(context.clone());
    let xpub = XpubDerivable::from_str(descriptor).expect("Invalid extended pubkey");
    let noise = xpub.xpub().chain_code().to_byte_array();
    match context.seal {
        SealType::BitcoinOpret => {
            OpretWallet::create(
                provider,
                Opret::new_unfunded(Wpkh::from(xpub), noise),
                network,
                true,
            )
            .expect("Unable to create wallet");
        }
        SealType::BitcoinTapret => {
            TapretWallet::create(
                provider,
                Tapret::key_only_unfunded(xpub, noise),
                network,
                true,
            )
            .expect("Unable to create wallet");
        }
    }
}

fn mound(context: Context, init: bool) -> DirMound {
    if init {
        let _ = fs::create_dir_all(context.data_dir.join(SealType::BitcoinOpret.to_string()));
        let _ = fs::create_dir_all(context.data_dir.join(SealType::BitcoinTapret.to_string()));
    }
    DirMound::load(&context.data_dir)
}

fn runtime(context: Context, name: Option<String>) -> DirRuntime {
    let provider = create_wallet_provider(context.clone());
    let wallet = match context.seal {
        SealType::BitcoinOpret => {
            let wallet = OpretWallet::load(provider, true).unwrap_or_else(|_| {
                panic!(
                    "Error: unable to load opret wallet from path `{}`",
                    build_wallet_file_path(context.clone()).display()
                )
            });
            DirBarrow::with_opret(context.seal, mound(context.clone(), true), wallet)
        }
        SealType::BitcoinTapret => {
            let wallet = TapretWallet::load(provider, true).unwrap_or_else(|_| {
                panic!(
                    "Error: unable to load tapret wallet from path `{}`",
                    build_wallet_file_path(context.clone()).display()
                )
            });
            DirBarrow::with_tapret(context.seal, mound(context.clone(), true), wallet)
        }
    };
    // TODO: Sync wallet if needed
    wallet.into()
}

// fn indexer(context: Context, resolver: ResolverOpt) -> AnyIndexer {
//     let network = context.network.to_string();
//     match (&resolver.esplora, &resolver.electrum, &resolver.mempool) {
//         (None, Some(url), None) => AnyIndexer::Electrum(Box::new(
//             electrum::Client::new(url).expect("Unable to initialize indexer"),
//         )),
//         (Some(url), None, None) => AnyIndexer::Esplora(Box::new(
//             esplora::Client::new_esplora(&url.replace("{network}", &network))
//                 .expect("Unable to initialize indexer"),
//         )),
//         (None, None, Some(url)) => AnyIndexer::Mempool(Box::new(
//             esplora::Client::new_mempool(&url.replace("{network}", &network))
//                 .expect("Unable to initialize indexer"),
//         )),
//         _ => {
//             eprintln!(
//                 "Error: no blockchain indexer specified; use either --esplora --mempool or \
//                  --electrum argument"
//             );
//             exit(1);
//         }
//     }
// }

fn issue_nia() {
    println!("hello")
}
