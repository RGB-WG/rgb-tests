# RGB Reorg Test Report

## 1. Test Design Overview

### 1.1 Test Purpose
Verify the state management capability of RGB v0.12 during blockchain reorganization (reorg), ensuring that asset states can be correctly rolled back and restored.

### 1.2 Test Scenarios
The test was designed with two reorg modes (ReorgType) and three history types (HistoryType), resulting in 6 test case combinations:

#### Reorg Modes (ReorgType)
- **ChangeOrder**: Change the transaction broadcasting order, simulating scenarios where transactions are confirmed in different orders in the blockchain
- **Revert**: Do not broadcast the initial transaction, simulating scenarios where certain transactions are completely excluded from the blockchain

#### History Types (HistoryType)
- **Linear**: Linear transfer history, a simple scenario where assets are transferred from A to B, and then partially back to A
- **Branching**: Branching transfer history, where assets are first fully transferred to B, then B transfers part back to A, and B transfers the remaining part (minus 1) back to A
- **Merging**: Merging transfer history, where A holds two assets, transfers them to B in two transactions, and then B transfers almost all (minus 1) back to A

### 1.3 Test Implementation Principles
The tests simulate blockchain reorganization through the following methods:

1. **Dual-node Environment**:
   - Using two independent Bitcoin node instances (INSTANCE_2 and INSTANCE_3)
   - Initially connecting the two nodes to synchronize them

2. **Reorg Simulation**:
   - Disconnecting the nodes, executing transactions in normal order on INSTANCE_2
   - Based on the test scenario, broadcasting transactions in different orders or ignoring certain transactions on INSTANCE_3
   - Switching the wallet to INSTANCE_3 instance and checking state changes

3. **Asset Reception Method**:
   - Using the Auth Token method (pre-created UTXO) to receive assets
   - This makes transactions relatively independent and allows flexible adjustment of broadcast order

4. **State Verification**:
   - Verifying asset allocation states after reorg using the `check_allocations` method
   - Testing final state consistency after nodes reconnect

## 2. Test Case Details

### 2.1 Passed Test Cases

| Case   | History Type | Reorg Type  | Status   |
| ------ | ------------ | ----------- | -------- |
| case_1 | Linear       | ChangeOrder | Passed ✅ |
| case_3 | Branching    | ChangeOrder | Passed ✅ |
| case_5 | Merging      | ChangeOrder | Passed ✅ |

**ChangeOrder Test Implementation Details**:
- Execute transactions in the order [tx_0, tx_1, tx_2] on INSTANCE_2
- Broadcast in different orders on INSTANCE_3 (such as [tx_2, tx_1, tx_0] in the Linear scenario)
- Results match expectations, indicating that RGB correctly handles changes in transaction order

### 2.2 Failed Test Cases

| Case   | History Type | Reorg Type | Status   | Failure Details |
| ------ | ------------ | ---------- | -------- | --------------- |
| case_2 | Linear       | Revert     | Failed ❌ |                 |
| case_4 | Branching    | Revert     | Failed ❌ |                 |
| case_6 | Merging      | Revert     | Failed ❌ |                 |

**Revert Test Implementation Details**:
- Disconnect INSTANCE_3 from INSTANCE_2
- Execute transactions in the complete order [tx_0, tx_1, tx_2] on INSTANCE_2
- Only broadcast subsequent transactions [tx_1, tx_2] on INSTANCE_3, without broadcasting the initial transaction tx_0
- Expect that without the initial transaction, the asset state should roll back to the initial allocation (passed)
- Reconnect INSTANCE_3 and INSTANCE_2, check state consistency (failed)
    - tx_0 is mined in INSTANCE_2, reverted in INSTANCE_3
    - Additionally, INSTANCE_3's chain is 1 block longer than INSTANCE_2
    - But when the two nodes reconnect, tx_0 will be resubmitted to the mempool
    - Expected asset state to be restored to its original state

## 3. Issues Discovered and Fix Progress

### 3.1 State Rollback Issues in Revert Scenarios
All Revert test cases failed. After multiple PR fixes, we are currently in the final stage, addressing the state forwarding after block reorg.

#### 3.1.1 Issue One: Incorrect Transaction Status in Indexer (Resolved ✅)
- **Symptom**: Transaction status (TxStatus) in the transaction indexer was not correctly Responded
- **Root Cause**: The transaction indexer did not correctly reflect the confirmation status of transactions in the blockchain, causing the rollback logic based on these transactions to fail
- **Solution**: Fixed the transaction status tracking logic in the indexer
- **PR Link**: [BP-WG/bp-wallet#86](https://github.com/BP-WG/bp-wallet/pull/86)

#### 3.1.2 Issue Two: Infinite Loop in Rollback Function (Resolved ✅)
- **Symptom**: During the rollback operation process, the same operation was repeatedly added to the rollback chain, causing performance issues and potential infinite loops
- **Root Cause**: The `rollback` function did not check for duplicate operations when building the dependency chain, and incorrectly increased the loop index
- **Solution**:
  - Added duplicate check logic: `if chain.contains(&spent) { continue; }`
  - Removed the incorrect index increment: `index += 1` (the loop itself already increases the index)
- **PR Link**: [AluVM/sonic#18](https://github.com/AluVM/sonic/pull/18)

#### 3.1.3 Issue Three: Dependency Recovery Problem in Forward Function (Pending ⏳)
- **Symptom**: When attempting to restore operations after rollback, subsequent operations dependent on the restored operation are not automatically restored
- **Case Analysis**:
  - In the first phase of Revert, after tx_0 is reverted in INSTANCE_3, although tx_1 and tx_2 are normally broadcast, the asset transfer chain depends on tx_0, so in the absence of tx_0, the asset correctly reverts to its issuance state
  - In the second phase of Revert, after the two Bitcoin nodes reconnect and a reorg occurs (tx_0 is mined in INSTANCE_2, reverted in INSTANCE_3, and INSTANCE_3's chain is 1 block longer than INSTANCE_2; when the two nodes reconnect, tx_0 is resubmitted to the mempool), the expected behavior is that tx_0 (mempool), tx_1 (mined), tx_2 (mined) would sync and restore to the state after witness transaction tx_2, but currently it only restores to the state after witness transaction tx_0
- **Analysis**:
  - The `rollback` function can find the complete dependency chain through the `spent_by` method and roll back
  - The `forward` function lacks a similar mechanism to restore the complete dependency chain

## 4. Recommended Next Steps

1. **Resolve Forward Function Issues**:
   - Hope to receive solutions or suggestions from the doctor 