pub mod utils;
use utils::{chain::initialize, runtime::TestRuntime};

#[test]
fn issue_nia() {
    initialize();

    let mut wlt1 = TestRuntime::new(&utils::DescriptorType::Tr);
    let issued_supply = 1000;
    let wout = true;
    let mut sats = 9000;

    let outpoint = wlt1.get_utxo(None);
    let contract_id = wlt1.issue_nia("nia1", issued_supply, outpoint);

    wlt1.check_allocations(
        contract_id,
        utils::AssetSchema::Nia,
        vec![issued_supply],
        true,
    );
}
