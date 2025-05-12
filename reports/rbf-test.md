# RGB RBF Transfer Test Report

## 1. Test Design Overview

### 1.1 Test Purpose
To verify the asset transfer process under the RGB20 protocol with RBF (Replace-By-Fee), ensuring that asset states, witness transactions, and consignment serialization behave as expected.

### 1.2 Test Scenario
This test focuses on the following process:
- Initiate a normal transfer, with the witness transaction entering the mempool.
- Use RBF to broadcast a replacement transaction with a higher fee.
- (Debug step) Check whether the consignment serialization reflects the latest witness transaction.
- Mine and confirm the new transaction, then check asset allocation and witness status.
- Perform a subsequent transfer to verify the handling of historical witness information.

### 1.3 Test Implementation
- Two wallet instances are used to simulate asset issuance, transfer, RBF replacement, and asset return.
- APIs such as `tx_status` and `check_allocations` are used to verify transaction status and asset allocation.
- (Debug step) Special attention is paid to whether the consignment serialization updates witness information in a timely manner.

## 2. Test Case Details

### 2.1 Test Case Flow

1. **Asset Issuance and Initial Transfer**
   - Wallet 1 issues an NIA asset and allocates it to itself.
   - Wallet 1 initiates a transfer to Wallet 2, generating a witness transaction (mempool status).
   - Wallet 2 receives and accepts the consignment.

2. **RBF Replacement Process**
   - Wallet 1 uses RBF to broadcast a replacement transaction with a higher fee.
   - A new consignment is generated;
   - The new transaction is mined and confirmed; both old and new transaction statuses are checked.

3. **Asset Allocation and Return**
   - Wallet 2 accepts the final consignment.
   - Asset allocations in both wallets are verified.
   - Wallet 2 returns assets to Wallet 1, verifying the handling of historical witness information.

### 2.2 Key Debug Information and Observations

- **Consignment Serialization Issue**  
  During debugging, it was found that after RBF and subsequent consignment generation, the serialized witness information still refers to the invalid (old) witness.  
  The likely cause is that during the initial transfer, the relationship between terminal (auth-token) and cell-addr (opid-pos) is established in `RgbRuntime::exec` when generating the Prefab, but RBF does not update this relationship, causing the terminal to always reference the old operation.

- **Internal State Complexity**  
  The current internal state management is complex, so we are cautious about making changes to the RBF logic and expect a fix from the Doctor.

### 2.3 Debug Log Excerpts

- The initial transfer's witness transaction is in the mempool, and consignment serialization is correct.
- After RBF, the new consignment still serializes the old witness.
- After mining, the old witness transaction status becomes Unknown, and the new witness transaction is Mined.
- In subsequent transfers, the consignment still contains the opid corresponding to the invalid witness transaction.

## 3. Identified Issues and Progress

### 3.1 Consignment Serialization Not Updated After RBF
- **Symptom**: After RBF, the consignment still serializes the old (invalid) witness.
- **Root Cause**: The relationship between terminal (auth-token) and cell-addr (opid-pos) is not updated in time.
- **Solution**: Further analysis and a fix from the Doctor are needed.


**Debug Log for RBF Transfer Test Case**

``` test-case debug info
// First, perform a normal transfer; the witness transaction is in the mempool
// At the same time, the consignment serializes the witness transaction correctly
transfer txid: 353d9cfa6606c38985b8ecd2b79b74471515c98077e16d35f93bd11c07a6fe2a, consignment: 1
[rgb-std/src/contract.rs:521:21] opid = Opid(
    Array<32>(5a7a6ca82d1b59a06cf42fb673c1f2257787e7d1408ef03c46e3550851e556d2),
)
[rgb-std/src/contract.rs:521:21] &wid = Array<32>(353d9cfa6606c38985b8ecd2b79b74471515c98077e16d35f93bd11c07a6fe2a)
[rgb-std/src/contract.rs:521:21] status = Tentative

[tests/transfer.rs:150:5] first_txid = Array<32>(353d9cfa6606c38985b8ecd2b79b74471515c98077e16d35f93bd11c07a6fe2a)
[tests/transfer.rs:150:5] tx_status(first_txid, wlt_1.instance) = Mempool

// Based on the previous transfer, perform RBF, generating consignment2 and a new witness transaction, but the serialized witness is still the old one
transfer txid: e2a057f50d037d48cd68bcdee4eadd29345db51caa16c1b763a6d6b07753be29, consignment: 2
[rgb-std/src/contract.rs:521:21] opid = Opid(
    Array<32>(5a7a6ca82d1b59a06cf42fb673c1f2257787e7d1408ef03c46e3550851e556d2),
)
[rgb-std/src/contract.rs:521:21] &wid = Array<32>(353d9cfa6606c38985b8ecd2b79b74471515c98077e16d35f93bd11c07a6fe2a)
[rgb-std/src/contract.rs:521:21] status = Tentative

// Mining and accept are performed here

// Then check the status of the old and new transactions
// The old witness transaction is found to be invalid
[tests/transfer.rs:168:5] first_txid = Array<32>(353d9cfa6606c38985b8ecd2b79b74471515c98077e16d35f93bd11c07a6fe2a)
[tests/transfer.rs:168:5] tx_status(first_txid, wlt_1.instance) = Unknown
// The new one is confirmed
[tests/transfer.rs:169:5] second_txid = Array<32>(e2a057f50d037d48cd68bcdee4eadd29345db51caa16c1b763a6d6b07753be29)
[tests/transfer.rs:169:5] tx_status(second_txid, wlt_1.instance) = Mined(
    MiningInfo {
        height: 1129,
        time: 1747048278,
        block_hash: Array<32>(5b8516b3a3554f67eb4d0103634ee409df04971fa803fb5c10643888ebed3fde),
    },
)

// In the subsequent transfer, it can be seen that the opid corresponding to the invalid witness transaction is still included
transfer txid: 23cf85a01dbf024eeb8caf58170297097bb2ff01e5eaaeb7dd29c25116b819c4, consignment: 3
[rgb-std/src/contract.rs:521:21] opid = Opid(
    Array<32>(5a7a6ca82d1b59a06cf42fb673c1f2257787e7d1408ef03c46e3550851e556d2),
)
[rgb-std/src/contract.rs:521:21] &wid = Array<32>(353d9cfa6606c38985b8ecd2b79b74471515c98077e16d35f93bd11c07a6fe2a)
[rgb-std/src/contract.rs:521:21] status = Archived
[rgb-std/src/contract.rs:521:21] opid = Opid(
    Array<32>(bfbe0249576dac4dd64b6f3abff32efce0d899a24228172b905e0b73110c33cc),
)
[rgb-std/src/contract.rs:521:21] &wid = Array<32>(23cf85a01dbf024eeb8caf58170297097bb2ff01e5eaaeb7dd29c25116b819c4)
[rgb-std/src/contract.rs:521:21] status = Tentative
```