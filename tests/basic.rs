use std::{path::PathBuf, str::FromStr};

use bpstd::{Wpkh, XpubDerivable};
use bpwallet::fs::FsTextStore;
use invoice::Network as InvoiceNetwork;
use rgb::SealType;
use rgbp::{
    descriptor::{Opret, Tapret},
    wallet::{OpretWallet, TapretWallet},
};

fn build_wallet_file_path(data_dir: &PathBuf, seal: &SealType, name: Option<&str>) -> PathBuf {
    let path = data_dir.join(seal.to_string());
    path.join(name.unwrap_or("default"))
}

fn create_wallet_provider(data_dir: &PathBuf, seal: &SealType, name: Option<&str>) -> FsTextStore {
    let wallet_file_path = build_wallet_file_path(data_dir, seal, name);
    FsTextStore::new(wallet_file_path).expect("Broken directory structure")
}

fn create_wallet(
    data_dir: &PathBuf,
    seal: &SealType,
    name: Option<&str>,
    descriptor: &str,
    network: InvoiceNetwork,
) {
    let provider = create_wallet_provider(data_dir, seal, name);
    let xpub = XpubDerivable::from_str(descriptor).expect("Invalid extended pubkey");
    let noise = xpub.xpub().chain_code().to_byte_array();
    match seal {
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

fn issue_nia() {
    println!("hello")
}
