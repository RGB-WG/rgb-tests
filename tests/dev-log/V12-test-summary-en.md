# RGB v0.12 First Phase Testing Summary

This is the first test run of RGB v0.12. While the full test suite is still under development, we have completed the design of most transfer test cases.

## Test Execution Overview

| Test Type                          | Total | Passed | Failed | Ignored |
| ---------------------------------- | ----- | ------ | ------ | ------- |
| [issuance.rs](http://issuance.rs/) | 16    | 16     | 0      | 0       |
| [transfer.rs](http://transfer.rs/) | 36    | 9      | 12     | 15      |
| Total                              | 52    | 25     | 12     | 15      |

## Detailed Test Status

### Ignored Tests

1. blank_tapret_opret::case_1..4 - Fix needed (calling to method absent in Codex API)
2. mainnet_wlt_receiving_test_asset - Fix needed
    - Error: The `Mound` structure in RGB v0.12 does not support setting the mainnet
    - Context: From issue_mainnet_wlt_receiving_test_asset.md: "The default `Mound.testnet` is eq true, which cannot correctly initialize the mainnet wallet"
3. reorg_history::case_1 - Fix needed
    - Error: "Unable to accept a consignment: unknown seal definition for cell address qMWtQjXCWjJAXdrg7npyI2KZz3vXNVyZhoomqF7v8z4:0."
    - Context: Test simulates blockchain reorganization with Linear history and ChangeOrder type
4. reorg_history::case_2 - Fix needed
    - Error: "assertion `left == right` failed, left: [10, 20], right: [600]"
    - Context: Test simulates blockchain reorganization with Linear history and Revert type
5. reorg_history::case_3 - Fix needed
    - Error: "Unable to accept a consignment: unknown seal definition for cell address c6z0I0hYqaO6dV9qOjrP1lK4PJprjVAaAOdGCoqAdOY:0."
    - Context: Test simulates blockchain reorganization with Branching history and ChangeOrder type
6. reorg_history::case_4 - Fix needed
    - Error: "assertion `left == right` failed, left: [200, 399], right: [600]"
    - Context: Test simulates blockchain reorganization with Branching history and Revert type
7. reorg_history::case_5 - Fix needed
    - Error: "Unable to accept a consignment: unknown seal definition for cell address FrGmm~6ro7YOlE9bEuyCLcLt9AlX2uZOZRmjHEq6yyA:0."
    - Context: Test simulates blockchain reorganization with Merging history and ChangeOrder type
8. reorg_history::case_6 - Fix needed
    - Error: "assertion `left == right` failed, left: [599], right: [400]"
    - Context: Test simulates blockchain reorganization with Merging history and Revert type
9. same_transfer_twice_update_witnesses::case_1 - Awaiting new rollback procedure API in RGB v0.12
    - Context: Test requires the ability to update witnesses, which was removed in v0.12 and will be replaced with a new rollback procedure
10. same_transfer_twice_update_witnesses::case_2 - Awaiting new rollback procedure API in RGB v0.12
- Context: Same issue as case_1
1. collaborative_transfer - Pending multi-signature workflow documentation
- Context: Test requires implementation of multi-signature workflow. Current documentation is minimal and the underlying API is complex to understand. Initial implementation was unsuccessful. Plan to focus on this implementation in later phases after core functionality is stable.
1. ln_transfers - Pending Lightning Network integration documentation
- Context: Test requires implementation of multi-signature workflow. Current documentation is minimal and the underlying API is complex to understand. Initial implementation was unsuccessful. Plan to focus on this implementation in later phases after core functionality is stable.

### Failed Tests

1. rbf_transfer - Fulfill(StateInsufficient)
    - Error: `called Result::unwrap() on an Err value: Fulfill(StateInsufficient)`
    - Context: Test attempts to replace a transaction with a higher fee (RBF) but fails because the state is insufficient to fulfill the second transfer
2. same_transfer_twice_no_update_witnesses::case_1 - Fulfill(StateInsufficient)
    - Error: `called Result::unwrap() on an Err value: Fulfill(StateInsufficient)`
    - Context: Test attempts to use the same invoice twice without updating witnesses, which is not supported in v0.12
3. same_transfer_twice_no_update_witnesses::case_2 - Fulfill(StateInsufficient)
    - Error: `called Result::unwrap() on an Err value: Fulfill(StateInsufficient)`
    - Context: Same issue as case_1, related to invoice reuse
4. tapret_wlt_receiving_opret - Transaction already in block chain
    - Error: "Transaction already in block chain"
    - Context: Test attempts to create multiple transfers between Taproot and WPKH wallets. The test passes when run individually but fails with transaction conflicts when run as part of the full test suite, suggesting a need to restructure test isolation
5. transfer_loop::case_01 - calling to method absent in Codex API
    - Error: Method missing in the new Codex API
    - Context: Test attempts to transfer assets between different wallet types and asset schemas
6. transfer_loop::case_02 - calling to method absent in Codex API
    - Same issue as case_01
7. transfer_loop::case_03 - calling to method absent in Codex API
    - Same issue as case_01
8. transfer_loop::case_04 - calling to method absent in Codex API
    - Same issue as case_01
9. transfer_loop::case_05 - calling to method absent in Codex API
    - Same issue as case_01
10. transfer_loop::case_06 - calling to method absent in Codex API
    - Same issue as case_01
11. transfer_loop::case_07 - calling to method absent in Codex API
    - Same issue as case_01
12. transfer_loop::case_08 - calling to method absent in Codex API
    - Same issue as case_01

### Passed Tests

1. issue_cfa::case_1 - case_4 (all 4 tests)
    - Context: Successfully issues Collectible Fungible Assets with various parameters
2. issue_nia::case_1 - case_4 (all 4 tests)
    - Context: Successfully issues Non-Inflatable Assets with various parameters
3. issue_cfa_multiple_utxos::case_1 - case_4 (all 4 tests)
    - Context: Successfully issues Collectible Fungible Assets with allocations across multiple UTXOs
4. issue_nia_multiple_utxos::case_1 - case_4 (all 4 tests)
    - Context: Successfully issues Non-Inflatable Assets with allocations across multiple UTXOs
5. accept_0conf
    - Context: Successfully tests accepting unconfirmed (0-conf) transactions
6. send_to_oneself
    - Context: Successfully tests sending assets to the same wallet
7. check_fungible_history
    - Context: Successfully tests checking the history of fungible assets
8. transfer_loop::case_09 - case_12 (4 tests)
    - Context: Successfully tests a subset of transfer loop cases with specific wallet and asset combinations

## Analysis Summary

1. **Success Rate**: 48% (25/52)
2. **Main Issues**:
    - Asset issuance tests ([issuance.rs](http://issuance.rs/)) all pass, except for UDA assets missing `.issuer`, indicating core issuance functionality has been successfully migrated to v0.12
    - Transfer-related tests ([transfer.rs](http://transfer.rs/)) have numerous issues, primarily in these areas:
        - **API Instability**: Multiple transfer_loop tests and blank_tapret_opret tests fail with "calling to method absent in Codex API" errors, indicating breaking changes and instability in the API transition from v0.11 to v0.12. Some previously available methods have been removed or modified, requiring updates to test implementations
        - **State Insufficient errors**: rbf_transfer and same_transfer_twice_no_update_witnesses tests fail with "Fulfill(StateInsufficient)" errors, indicating that the state management for these scenarios needs to be updated
        - **Invoice reuse not supported**: Tests attempting to reuse the same invoice fail with "Fulfill(StateInsufficient)" errors, as noted in same_transfer_twice_no_update_witnesses: "In RGB V0.12, since it's not possible to use RBF with the same invoice, we cannot test for this inflation attack"
        - **Blockchain reorganization handling**: The API does not yet have a stable mechanism for handling blockchain reorganization scenarios, as evidenced by the reorg_history test failures showing incorrect asset allocations after reorganization
3. **Migration Progress Assessment**:
    - Asset issuance functionality migration is highly complete, including multi-UTXO allocation scenarios
    - Basic transfer functionality is partially available, such as accept_0conf, send_to_oneself, and some transfer_loop tests
    - Advanced transfer functionality (such as RBF, reorganization history, Lightning Network integration, etc.) has not yet been fully migrated
4. **Next Steps and Support Needed**:
    - Request guidance to resolve "calling to method absent in Codex API" issues
    - Fix StateInsufficient errors
    - Implement contract architecture support for blockchain reorganization scenarios, with related rollback procedure API
    - Add support for invoice reuse and RBF scenarios
    - Provide documentation on payment scripts and internal state transition APIs to implement more complex transfer scenario tests, such as multi-signature and Lightning Network integration