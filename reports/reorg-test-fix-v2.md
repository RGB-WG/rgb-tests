# RGB Reorg Test Analysis Report

## Introduction

After updating to the latest dependencies (including Sonic develop and rgb-std fix/forward branch), the `Revert` type reorg scenarios in the `reorg_history` test case are still failing. 

## Dependency Versions and Additional Failures

### Current Dependency Versions

| Dependency  | Version/Commit                                                    | Note |
| ----------- | ----------------------------------------------------------------- | ---- |
| **rgb-std** | `fix/forward` branch (`38af4585372df1326d0daac9b3b14c6d02b919a1`) | ⚠️    |
| **sonic**   | `develop` branch (`323ac8078ff313c02d51c6d0d65218570aaeaafb`)     | ⚠️    |

### Additional Test Failures

**Note to the doctor**: With the current versions of rgb-std and sonic, the `rbf_transfer` test case is also failing. This suggests that the issues with transaction dependency tracking affect not only reorg scenarios but also Replace-By-Fee (RBF) functionality, which relies on similar state management mechanisms.

## Test Process and Failure Point

The reorg test proceeds in three distinct stages:

1. **Stage 1**: Execute RGB asset transactions on Bitcoin node INSTANCE_2.
2. **Stage 2**: On Bitcoin node INSTANCE_3, simulate a blockchain reorganization:
   - For `ChangeOrder` scenarios: broadcast the same transactions but in a different order
   - For `Revert` scenarios: broadcast only subsequent transactions (tx_1, tx_2) without the initial transaction (tx_0)
   - **Check asset state validity after sync** ← Current Failure Point
3. **Stage 3**: Reconnect both Bitcoin nodes, allowing them to synchronize:
   - For `Revert` scenarios: previously reverted transaction tx_0 is resubmitted to the mempool
   - Check final asset state validity after complete synchronization

**Note**: Unlike the previous implementation where failures occurred in Stage 3, the current implementation fails in Stage 2. 

## Test Scenario Overview

The `reorg_history` test case verifies correct handling of RGB asset states across different history types and reorganization types.

### Test Matrix

| History Type | Reorg Type  | Status |
| ------------ | ----------- | ------ |
| Linear       | ChangeOrder | ✅ Pass |
| Linear       | Revert      | ❌ Fail |
| Branching    | ChangeOrder | ✅ Pass |
| Branching    | Revert      | ❌ Fail |
| Merging      | ChangeOrder | ✅ Pass |
| Merging      | Revert      | ❌ Fail |

## Detailed Analysis: Linear-Revert Scenario

Using the `HistoryType::Linear` and `ReorgType::Revert` case as an example:

### 1. Initial Setup

- Two Bitcoin node instances (INSTANCE_2 and INSTANCE_3) are created
- An RGB20 asset with total supply of 600 is issued on INSTANCE_2
- Initial allocation: wlt_1 owns all 600 units

### 2. Transaction Sequence (INSTANCE_2)

| TX ID | Operation     | Transfer Amount | Resulting State                    |
| ----- | ------------- | --------------- | ---------------------------------- |
| tx_0  | wlt_1 → wlt_2 | 590             | wlt_1: 10, wlt_2: 590              |
| tx_1  | wlt_2 → wlt_1 | 100             | wlt_1: 10+100=110, wlt_2: 490      |
| tx_2  | wlt_1 → wlt_2 | 80              | wlt_1: 10+20=30, wlt_2: 490+80=570 |


### 3. Reorg Scenario (INSTANCE_3) - Stage 2 Failure Point

On INSTANCE_3, only tx_1 and tx_2 are broadcast, **tx_0 is not broadcast**, simulating a scenario where tx_0 has been reverted:

```rust
broadcast_tx_and_mine(&txs[1], INSTANCE_3);
broadcast_tx_and_mine(&txs[2], INSTANCE_3);
```

After switching the wallet to INSTANCE_3 and syncing:

```rust
wlt_1.switch_to_instance(INSTANCE_3);
wlt_2.switch_to_instance(INSTANCE_3);
```

### 4. Actual Results vs. Expected Results (Stage 2)

**Expected Result**:
- When tx_0 is reverted, subsequent tx_1 and tx_2 that depend on the asset transfers in tx_0 should be invalid
- wlt_1 should maintain its original 600 assets, while wlt_2 should have no assets

**Actual Result**:
- tx_0 related state is correctly marked as `Archived`
- However, tx_1 and tx_2 related states incorrectly remain valid
- wlt_1 assets: 10 (archived) + 20 = 20, not the expected 600
- wlt_2 assets: 490 + 80 = 570, not the expected 0

### 5. State Details (After switching to INSTANCE_3)

The debug output reveals the problematic state:

```
[tests/transfer.rs:1023:13] "after revert tx_0" = "after revert tx_0"
[tests/transfer.rs:1023:13] wlt_1.runtime().state_own(contract_id).owned = {
    VariantName("amount"): {
        CellAddr {opid: Opid(Array<32>(32130de9519410fb7887643d37aa4dbe63d7cd073c3edb537887c8a8049351e6))}: OwnedState {
            assignment: Assignment {
                seal: Outpoint {
                    txid: Array<32>(9b2654ce652961f071047fb2e4f68fa9966acfbd1d287f56f823afa9e540a9bd),
                    vout: Vout(1),
                },
                data: Number(Uint(10)),
            },
            status: Archived,  // Correctly marked as archived
        },
        CellAddr {opid: Opid(Array<32>(d30adf7c234714d23c78cd02b5ae6b1b6d19cc5eb23ddb7a9553922ed268640b))}: OwnedState {
            assignment: Assignment {
                seal: Outpoint {
                    txid: Array<32>(a1e19720651a890e5a217248442a7b5c6cc32db16da70348234b3eeac8b9cb69),
                    vout: Vout(1),
                },
                data: Number(Uint(20)),
            },
            status: Mined(2085),  // Should be invalid but still considered valid
        },
    },
}

[tests/transfer.rs:1027:13] "after revert tx_0" = "after revert tx_0"
[tests/transfer.rs:1027:13] wlt_2.runtime().state_own(contract_id).owned = {
    VariantName("amount"): {
        CellAddr {opid: Opid(Array<32>(d30adf7c234714d23c78cd02b5ae6b1b6d19cc5eb23ddb7a9553922ed268640b))}: OwnedState {
            assignment: Assignment {
                seal: Outpoint {
                    txid: Array<32>(c43bbaada4324315d6efe7e62c906698fdbc81dc59f13265727a506ee85ba6df),
                    vout: Vout(0),
                },
                data: Number(Uint(80)),
            },
            status: Mined(2085),
        },
        CellAddr {opid: Opid(Array<32>(ef3258c075f938b49ab62dd8c7b13ab1d2211a87022f49e270d1fe73cb8d741e))}: OwnedState {
            assignment: Assignment {
                seal: Outpoint {
                    txid: Array<32>(bb501ff63075458275be50866ea568cad1db40f51b829bf17322f2f78f5188cb),
                    vout: Vout(1),
                },
                data: Number(Uint(490)),
            },
            status: Mined(2084),
        },
    },
}
```

The test fails at this point with the assertion:

```
thread 'reorg_history::case_2' panicked at tests/utils/helper/wallet.rs:593:9:
assertion `left == right` failed
  left: [20]
 right: [600]
```