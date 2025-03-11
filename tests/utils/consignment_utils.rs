use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use amplify::confinement::SmallOrdMap;
use amplify::hex::ToHex;
use amplify::{Bytes16, Display, From};
use bp::seals::WTxoSeal;
use commit_verify::ReservedBytes;
use hypersonic::{Articles, ContractId, Operation};
use rgb::sigs::ContentSigs;
use rgb::{
    Contract, PublishedWitness, RgbSealDef, Schema, SealWitness, SingleUseSeal,
    MAGIC_BYTES_CONSIGNMENT,
};
use serde::{Deserialize, Serialize};
use strict_encoding::{
    DecodeError, StreamReader, StreamWriter, StrictDecode, StrictEncode, StrictReader, StrictWriter,
};

#[derive(Debug, Display, From)]
#[display(doc_comments)]
pub enum ConsignmentParseError {
    /// I/O error: {0}
    #[from]
    Io(io::Error),

    /// Strict encoding error: {0}
    #[from]
    Decoding(DecodeError),

    /// Unrecognized magic bytes in consignment stream ({0})
    UnrecognizedMagic(String),

    /// Unknown contract {0} can't be consumed; please import contract articles first.
    UnknownContract(ContractId),

    // Because MoundConsumeError does not implement Debug,
    // It affects the Debug and Error implementation of the entire structure,
    // So here we store the string representation of the MoundConsumeError error
    MoundConsume(String),

    /// Serialization error: {0}
    SerializationError(String),

    /// Invalid witness count: {0}
    InvalidWitnessCount(u64),

    /// Serde YAML error: {0}
    #[from]
    SerdeYaml(serde_yaml::Error),

    /// Invalid data: {0}
    InvalidData(String),
}

impl std::error::Error for ConsignmentParseError {}

/// Parse consignment data into a directory
/// This function follows the logic of dump_consignment in rgb-std/cli/src/dump.rs
pub fn parse_consignment<SealDef>(src: &Path, dst: &Path) -> Result<(), ConsignmentParseError>
where
    SealDef: RgbSealDef + Serialize,
    SealDef::Src: Serialize,
    <SealDef::Src as SingleUseSeal>::CliWitness:
        Serialize + for<'de> Deserialize<'de> + StrictEncode + StrictDecode,
    <SealDef::Src as SingleUseSeal>::PubWitness:
        Serialize + for<'de> Deserialize<'de> + StrictEncode + StrictDecode,
    <<SealDef::Src as SingleUseSeal>::PubWitness as PublishedWitness<SealDef::Src>>::PubId:
        Ord + From<[u8; 32]> + Into<[u8; 32]> + Serialize,
{
    fs::create_dir_all(dst)?;

    let file = File::open(src)?;
    let mut stream = StrictReader::with(StreamReader::new::<{ usize::MAX }>(file));

    let magic_bytes = Bytes16::strict_decode(&mut stream)?;
    if magic_bytes.to_byte_array() != MAGIC_BYTES_CONSIGNMENT {
        return Err(ConsignmentParseError::UnrecognizedMagic(
            magic_bytes.to_hex(),
        ));
    }

    // Version
    ReservedBytes::<2>::strict_decode(&mut stream)?;

    let contract_id = ContractId::strict_decode(&mut stream)?;
    println!(
        "Parsing consignment for {} into '{}'",
        contract_id,
        dst.display()
    );

    let mut op_count = 1;
    let mut seal_count = 0;
    let mut witness_count = 0;

    print!("Processing contract articles ... ");
    let articles = Articles::strict_decode(&mut stream)?;

    // Save genesis
    let out = File::create(dst.join(format!(
        "0000-genesis.{}.yaml",
        articles.contract.genesis_opid()
    )))?;
    serde_yaml::to_writer(&out, &articles.contract.genesis)?;

    // Save codex
    let out = File::create(dst.join(format!("codex.{}.yaml", articles.schema.codex.codex_id())))?;
    serde_yaml::to_writer(&out, &articles.schema.codex)?;

    // Save schema
    let out = File::create(dst.join("schema.yaml"))?;
    serde_yaml::to_writer(&out, &articles.schema)?;

    // Save contract
    let out = File::create(dst.join("contract.yaml"))?;
    serde_yaml::to_writer(&out, &articles.contract)?;

    // Save contract signatures
    let out = File::create(dst.join("contract_sigs.yaml"))?;
    serde_yaml::to_writer(&out, &articles.contract_sigs)?;

    // Save seals
    let out = File::create(dst.join("0000-seals.yml"))?;
    let defined_seals = SmallOrdMap::<u16, SealDef>::strict_decode(&mut stream)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    serde_yaml::to_writer(&out, &defined_seals)?;
    seal_count += defined_seals.len();

    let count = u64::strict_decode(&mut stream)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    if count != 0 {
        println!("error");
        return Err(ConsignmentParseError::InvalidWitnessCount(count));
    }
    println!("success");

    // Create operations directory
    let operations_dir = dst.join("operations");
    fs::create_dir_all(&operations_dir)?;

    println!();
    loop {
        match Operation::strict_decode(&mut stream) {
            Ok(operation) => {
                let opid = operation.opid();

                // save operation
                let out =
                    File::create(operations_dir.join(format!("{op_count:04}-op.{opid}.yaml")))?;
                serde_yaml::to_writer(&out, &operation)?;

                // save seals
                let out = File::create(operations_dir.join(format!("{op_count:04}-seals.yml")))?;
                let defined_seals = SmallOrdMap::<u16, SealDef>::strict_decode(&mut stream)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                serde_yaml::to_writer(&out, &defined_seals)?;
                seal_count += defined_seals.len();

                // save witnesses
                let len = u64::strict_decode(&mut stream)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                for no in 0..len {
                    let out = File::create(
                        operations_dir.join(format!("{op_count:04}-witness-{:02}.yml", no + 1)),
                    )?;
                    let witness = SealWitness::<SealDef::Src>::strict_decode(&mut stream)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                    serde_yaml::to_writer(&out, &witness)?;
                }

                witness_count += len as usize;
                op_count += 1;
            }
            Err(DecodeError::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(ConsignmentParseError::Decoding(e)),
        }
        print!(
            "\rParsing stream ... {op_count} operations, {seal_count} seals, {witness_count} \
             witnesses processed",
        );
    }
    println!();
    Ok(())
}

#[test]
fn test_parse_and_rebuild_consignment() {
    let src = Path::new("test-data/integration/00d99ed6/consignment-551.rgb");
    let dst = Path::new("test-data/integration/00d99ed6/output").to_owned();
    parse_consignment::<WTxoSeal>(src, dst.as_path()).unwrap();

    let rebuild = dst.join("rebuild.rgb").as_path().to_owned();
    rebuild_consignment::<WTxoSeal>(dst.as_path(), rebuild.as_path()).unwrap();

    let dst_rebuild = Path::new("test-data/integration/00d99ed6/rebuild_output");
    parse_consignment::<WTxoSeal>(rebuild.as_path(), dst_rebuild).unwrap();
}

/// Modify file content
pub fn modify_file(path: &Path, pattern: &str, replacement: &str) -> io::Result<()> {
    let content = fs::read_to_string(path)?;
    let modified = content.replace(pattern, replacement);
    fs::write(path, modified)?;
    Ok(())
}

/// Rebuild consignment from parsed files
/// This function follows the logic of mound.rs(fn consign), stockpile.rs(fn consign) and stock.rs(fn export_aux)
pub fn rebuild_consignment<SealDef>(
    src_dir: &Path,
    dst_path: &Path,
) -> Result<(), ConsignmentParseError>
where
    SealDef: RgbSealDef + Serialize + for<'de> Deserialize<'de>,
    SealDef::Src: Serialize + for<'de> Deserialize<'de>,
    <SealDef::Src as SingleUseSeal>::CliWitness:
        Serialize + for<'de> Deserialize<'de> + StrictEncode + StrictDecode,
    <SealDef::Src as SingleUseSeal>::PubWitness:
        Serialize + for<'de> Deserialize<'de> + StrictEncode + StrictDecode,
    <<SealDef::Src as SingleUseSeal>::PubWitness as PublishedWitness<SealDef::Src>>::PubId:
        Ord + From<[u8; 32]> + Into<[u8; 32]> + Serialize,
{
    // Create output file
    let file = File::create(dst_path)?;
    let mut writer = StrictWriter::with(StreamWriter::new::<{ usize::MAX }>(file));

    // Write magic bytes
    writer = MAGIC_BYTES_CONSIGNMENT.strict_encode(writer)?;
    writer = 0x00u16.strict_encode(writer)?;

    // Load contract ID
    let contract_id_file = src_dir.join("contract_id.yaml");
    let contract_id: ContractId = if contract_id_file.exists() {
        serde_yaml::from_reader(File::open(contract_id_file)?)?
    } else {
        // Try to extract from other files
        let contract_file = src_dir.join("contract.yaml");
        if contract_file.exists() {
            let contract: Contract = serde_yaml::from_reader(File::open(contract_file)?)?;
            contract.contract_id()
        } else {
            return Err(ConsignmentParseError::InvalidData(
                "Contract ID not found".to_string(),
            ));
        }
    };

    println!("rebuild_consignment: contract_id: {}", contract_id);

    // Write contract ID
    writer = contract_id.strict_encode(writer)?;

    // Load and write articles
    let schema_file = src_dir.join("schema.yaml");
    let contract_file = src_dir.join("contract.yaml");
    let contract_sigs_file = src_dir.join("contract_sigs.yaml");

    if !schema_file.exists() || !contract_file.exists() || !contract_sigs_file.exists() {
        return Err(ConsignmentParseError::InvalidData(
            "Missing required article files".to_string(),
        ));
    }

    let schema: Schema = serde_yaml::from_reader(File::open(schema_file)?)?;
    let contract: Contract = serde_yaml::from_reader(File::open(contract_file)?)?;
    let contract_sigs: ContentSigs = serde_yaml::from_reader(File::open(contract_sigs_file)?)?;

    let articles = Articles {
        schema,
        contract,
        contract_sigs,
    };

    // Write articles
    writer = articles.strict_encode(writer)?;

    println!("Writing genesis seals");
    // Load and write genesis seals
    let genesis_seals_file = src_dir.join("0000-seals.yml");
    if !genesis_seals_file.exists() {
        return Err(ConsignmentParseError::InvalidData(
            "Missing genesis seals file".to_string(),
        ));
    }

    let genesis_seals_file = File::open(genesis_seals_file)?;

    let genesis_seals: SmallOrdMap<u16, SealDef> =
        serde_yaml::from_reader(genesis_seals_file).map_err(|e| dbg!(e))?;
    writer = genesis_seals.strict_encode(writer)?;

    // Write 0 witnesses for genesis
    writer = (0u64).strict_encode(writer)?;

    println!("Writing operations");
    // Load and write operations
    let operations_dir = src_dir.join("operations");
    if !operations_dir.exists() || !operations_dir.is_dir() {
        return Err(ConsignmentParseError::InvalidData(
            "Operations directory not found".to_string(),
        ));
    }

    // Get all operation files
    let mut operation_files = Vec::new();
    for entry in fs::read_dir(&operations_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
            operation_files.push(path);
        }
    }

    // Sort operation files by name to ensure correct order
    operation_files.sort_by(|a, b| {
        let a_name = a.file_name().unwrap().to_string_lossy();
        let b_name = b.file_name().unwrap().to_string_lossy();
        a_name.cmp(&b_name)
    });

    // Process each operation
    for (op_index, op_file) in operation_files.iter().enumerate() {
        println!("Processing operation: {}", op_file.display());
        let op_count = op_index + 1; // Operation count starts from 1

        // Load and write operation
        let operation: Operation = serde_yaml::from_reader(File::open(op_file)?)?;
        writer = operation.strict_encode(writer)?;

        // Load and write operation seals
        let seals_file = operations_dir.join(format!("{op_count:04}-seals.yml"));
        if !seals_file.exists() {
            return Err(ConsignmentParseError::InvalidData(format!(
                "Missing seals file for operation {}",
                op_count
            )));
        }

        let seals_file = File::open(seals_file)?;
        let seals: SmallOrdMap<u16, SealDef> =
            serde_yaml::from_reader(seals_file).map_err(|e| dbg!(e))?;
        writer = seals.strict_encode(writer)?;

        // Count witness files for this operation
        let mut witness_count = 0;
        let mut witness_files = Vec::new();
        for i in 1..100 {
            // Assuming maximum 99 witnesses per operation
            let witness_file = operations_dir.join(format!("{op_count:04}-witness-{:02}.yml", i));
            if witness_file.exists() {
                witness_count += 1;
                witness_files.push(witness_file);
            } else {
                break;
            }
        }

        // Write witness count
        writer = (witness_count as u64).strict_encode(writer)?;

        // Load and write witnesses
        for witness_file in witness_files {
            let witness =
                serde_yaml::from_reader::<_, SealWitness<SealDef::Src>>(File::open(witness_file)?)?;
            writer = witness.strict_encode(writer)?;
        }
    }

    println!(
        "Rebuilt consignment from {} to {}",
        src_dir.display(),
        dst_path.display()
    );
    Ok(())
}

/// Create an attacked consignment
pub fn create_attack_consignment(
    src: &Path,
    attack_type: &str,
) -> Result<PathBuf, ConsignmentParseError> {
    // Create a temporary directory for parsing
    let temp_dir_path = PathBuf::from("tests/fixtures/v0.12/temp").join(attack_type);

    // If the directory exists, clear it; otherwise, create a new directory
    if temp_dir_path.exists() {
        // Check if the directory is empty
        if temp_dir_path.read_dir()?.next().is_some() {
            // The directory is not empty, clear it
            for entry in fs::read_dir(&temp_dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
            }
        }
    } else {
        // The directory does not exist, create it
        fs::create_dir_all(&temp_dir_path)?;
    }

    // Parse the consignment
    parse_consignment::<WTxoSeal>(src, &temp_dir_path)?;

    // Create the attack path
    let attack_path = PathBuf::from(format!("tests/fixtures/attack_{}.rgb", attack_type));

    // Modify files based on attack type
    match attack_type {
        "chain" => {
            // Attack type: chain
            // Modify consensus from bitcoin to liquid in contract.yaml
            let contract_path = temp_dir_path.join("contract.yaml");
            if contract_path.exists() {
                modify_file(&contract_path, "consensus: bitcoin", "consensus: liquid")?;
                println!("Modified contract.yaml: changed consensus from bitcoin to liquid");
            } else {
                return Err(ConsignmentParseError::InvalidData(
                    "contract.yaml not found".to_string(),
                ));
            }
        }
        "genesis_schema_id" => {
            // Attack type: genesis_schema_id
            // Modify codexId in contract.yaml and schema.yaml
            let contract_path = temp_dir_path.join("contract.yaml");
            let schema_path = temp_dir_path.join("schema.yaml");

            if contract_path.exists() && schema_path.exists() {
                // Read files to find the original codexId pattern
                let contract_content = fs::read_to_string(&contract_path)?;
                if let Some(line) = contract_content
                    .lines()
                    .find(|line| line.contains("codexId:"))
                {
                    // let original_codex_id = line.trim();
                    let original_codex_id = line.split("codexId:").nth(1).unwrap().trim();
                    let mut modified_codex_id = original_codex_id.to_owned();
                    modified_codex_id.push('1');

                    modify_file(&contract_path, original_codex_id, &modified_codex_id)?;
                    modify_file(&schema_path, original_codex_id, &modified_codex_id)?;
                    println!("Modified codexId in contract.yaml and schema.yaml");
                } else {
                    return Err(ConsignmentParseError::InvalidData(
                        "codexId not found in contract.yaml".to_string(),
                    ));
                }
            } else {
                return Err(ConsignmentParseError::InvalidData(
                    "contract.yaml or schema.yaml not found".to_string(),
                ));
            }
        }
        "genesis_testnet" => {
            // Attack type: genesis_testnet
            // Modify testnet flag from true to false in contract.yaml
            let contract_path = temp_dir_path.join("contract.yaml");
            if contract_path.exists() {
                modify_file(&contract_path, "testnet: true", "testnet: false")?;
                println!("Modified contract.yaml: changed testnet from true to false");
            } else {
                return Err(ConsignmentParseError::InvalidData(
                    "contract.yaml not found".to_string(),
                ));
            }
        }
        "bundles_pubwitness_data_input_sequence" => {
            // Attack type: bundles_pubwitness_data_input_sequence
            // Modify sequence from 0 to 1 in operations/0001-witness-01.yml
            let witness_path = temp_dir_path.join("operations").join("0001-witness-01.yml");
            if witness_path.exists() {
                modify_file(&witness_path, "sequence: 0", "sequence: 1")?;
                println!("Modified operations/0001-witness-01.yml: changed sequence from 0 to 1");
            } else {
                return Err(ConsignmentParseError::InvalidData(
                    "operations/0001-witness-01.yml not found".to_string(),
                ));
            }
        }
        "resolver_error" => {
            // Attack type: resolver_error
            // This attack is handled by switching the esplora instance in the wallet
            // No modification to the consignment is needed
            println!("Resolver error attack: No modification to consignment needed.");
            println!("This attack is handled by switching the esplora instance in the wallet.");

            // Just copy the original file to the attack path
            fs::copy(src, &attack_path)?;
            return Ok(attack_path);
        }
        _ => {
            return Err(ConsignmentParseError::InvalidData(format!(
                "Unknown attack type: {}",
                attack_type
            )));
        }
    }

    // Rebuild consignment from modified files
    rebuild_consignment::<WTxoSeal>(&temp_dir_path, &attack_path)?;

    // Clean up temporary directory
    // fs::remove_dir_all(&temp_dir_path)?;

    println!("Created attack consignment at {}", attack_path.display());
    Ok(attack_path)
}

#[test]
fn test_create_attack_consignment() {
    let src = Path::new("test-data/integration/00d99ed6/consignment-551.rgb");
    let chain_attack_path = create_attack_consignment(src, "chain").unwrap();
    assert!(chain_attack_path.exists());

    let genesis_schema_id_attack_path =
        create_attack_consignment(src, "genesis_schema_id").unwrap();
    assert!(genesis_schema_id_attack_path.exists());

    let genesis_testnet_attack_path = create_attack_consignment(src, "genesis_testnet").unwrap();
    assert!(genesis_testnet_attack_path.exists());

    let bundles_pubwitness_data_input_sequence_attack_path =
        create_attack_consignment(src, "bundles_pubwitness_data_input_sequence").unwrap();
    assert!(bundles_pubwitness_data_input_sequence_attack_path.exists());

    let resolver_error_attack_path = create_attack_consignment(src, "resolver_error").unwrap();
    assert!(resolver_error_attack_path.exists());
}
