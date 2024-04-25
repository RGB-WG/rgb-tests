use super::*;

static INIT: Once = Once::new();

pub static INDEXER: OnceLock<Indexer> = OnceLock::new();

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub enum Indexer {
    Electrum,
    #[default]
    Esplora,
}

impl fmt::Display for Indexer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

pub fn initialize() {
    INIT.call_once(|| {
        INDEXER.get_or_init(|| match std::env::var("INDEXER") {
            Ok(val) if val.to_lowercase() == Indexer::Esplora.to_string() => Indexer::Esplora,
            Ok(val) if val.to_lowercase() == Indexer::Electrum.to_string() => Indexer::Electrum,
            Err(VarError::NotPresent) => Indexer::Esplora,
            _ => {
                panic!("invalid indexer. possible values: `esplora` (default), `electrum`")
            }
        });
        if std::env::var("SKIP_INIT").is_ok() {
            println!("skipping services initialization");
            return;
        }
        let start_services_file = PathBuf::from("tests").join("start_services.sh");
        println!("starting test services...");
        let output = Command::new(start_services_file)
            .env("PROFILE", INDEXER.get().unwrap().to_string())
            .output()
            .expect("failed to start test services");
        if !output.status.success() {
            println!("{output:?}");
            panic!("failed to start test services");
        }
        _wait_indexer_sync();
    });
}

static MINER: Lazy<RwLock<Miner>> = Lazy::new(|| RwLock::new(Miner { no_mine_count: 0 }));

#[derive(Clone, Debug)]
pub struct Miner {
    no_mine_count: u32,
}

fn _bitcoin_cli() -> Vec<String> {
    let compose_file = PathBuf::from("tests").join("docker-compose.yml");
    let mut cmd = vec![
        s!("-f"),
        compose_file.to_string_lossy().to_string(),
        s!("exec"),
        s!("-T"),
    ];
    match INDEXER.get().unwrap() {
        Indexer::Electrum => cmd.extend(vec![
            "-u".to_string(),
            "blits".to_string(),
            "bitcoind".to_string(),
            "bitcoin-cli".to_string(),
            "-regtest".to_string(),
        ]),
        Indexer::Esplora => cmd.extend(vec!["esplora".to_string(), "cli".to_string()]),
    };
    cmd
}

impl Miner {
    fn mine(&self) -> bool {
        if self.no_mine_count > 0 {
            return false;
        }
        self.force_mine()
    }

    fn force_mine(&self) -> bool {
        let output = Command::new("docker")
            .stdin(Stdio::null())
            .arg("compose")
            .args(_bitcoin_cli())
            .arg("-rpcwallet=miner")
            .arg("-generate")
            .arg("1")
            .output()
            .expect("failed to mine");
        if !output.status.success() {
            println!("{output:?}");
            panic!("failed to mine");
        }
        _wait_indexer_sync();
        true
    }

    fn stop_mining(&mut self) {
        self.no_mine_count += 1;
    }

    fn resume_mining(&mut self) {
        if self.no_mine_count > 0 {
            self.no_mine_count -= 1;
        }
    }
}

pub fn mine(resume: bool) {
    let t_0 = OffsetDateTime::now_utc();
    if resume {
        resume_mining();
    }
    loop {
        if (OffsetDateTime::now_utc() - t_0).as_seconds_f32() > 120.0 {
            println!("forcibly breaking mining wait");
            resume_mining();
        }
        let mined = MINER.read().as_ref().unwrap().mine();
        if mined {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

pub fn mine_but_no_resume() {
    let t_0 = OffsetDateTime::now_utc();
    loop {
        if (OffsetDateTime::now_utc() - t_0).as_seconds_f32() > 120.0 {
            println!("forcibly breaking mining wait");
            resume_mining();
        }
        let miner = MINER.write().unwrap();
        if miner.no_mine_count <= 1 {
            miner.force_mine();
            break;
        }
        drop(miner);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

pub fn stop_mining() {
    MINER.write().unwrap().stop_mining()
}

pub fn stop_mining_when_alone() {
    let t_0 = OffsetDateTime::now_utc();
    loop {
        if (OffsetDateTime::now_utc() - t_0).as_seconds_f32() > 120.0 {
            println!("forcibly breaking stop wait");
            stop_mining();
        }
        let mut miner = MINER.write().unwrap();
        if miner.no_mine_count == 0 {
            miner.stop_mining();
            break;
        }
        drop(miner);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

pub fn resume_mining() {
    MINER.write().unwrap().resume_mining()
}

pub fn get_height() -> u32 {
    let output = Command::new("docker")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .arg("compose")
        .args(_bitcoin_cli())
        .arg("getblockcount")
        .output()
        .expect("failed to call getblockcount");
    if !output.status.success() {
        println!("{output:?}");
        panic!("failed to get block count");
    }
    let blockcount_str =
        std::str::from_utf8(&output.stdout).expect("could not parse blockcount output");
    blockcount_str
        .trim()
        .parse::<u32>()
        .expect("could not parse blockcount")
}

fn _wait_indexer_sync() {
    let t_0 = OffsetDateTime::now_utc();
    let blockcount = get_height();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        match INDEXER.get().unwrap() {
            Indexer::Electrum => {
                let electrum_client =
                    ElectrumClient::new(ELECTRUM_REGTEST_URL).expect("cannot get electrum client");
                if electrum_client.block_header(blockcount as usize).is_ok() {
                    break;
                }
            }
            Indexer::Esplora => {
                let esplora_client = EsploraClient::new_esplora(ESPLORA_REGTEST_URL).unwrap();
                if esplora_client.block_hash(blockcount).is_ok() {
                    break;
                }
            }
        }
        if (OffsetDateTime::now_utc() - t_0).as_seconds_f32() > 25.0 {
            panic!("indexer not syncing with bitcoind");
        }
    }
}

pub fn send_to_address(address: String, sats: Option<u64>) -> String {
    let sats = Sats::from_sats(sats.unwrap_or(100_000_000));
    let btc = format!("{}.{:0>8}", sats.btc_floor(), sats.sats_rem());
    let output = Command::new("docker")
        .stdin(Stdio::null())
        .arg("compose")
        .args(_bitcoin_cli())
        .arg("-rpcwallet=miner")
        .arg("sendtoaddress")
        .arg(address)
        .arg(btc)
        .output()
        .expect("failed to fund wallet");
    if !output.status.success() {
        println!("{output:?}");
        panic!("failed to send to address");
    }
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

pub fn fund_wallet(address: String, sats: Option<u64>) -> String {
    let txid = send_to_address(address, sats);
    mine(false);
    txid
}
