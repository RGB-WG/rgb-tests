# RGB Asset Issuance Test Report

## Overview

This test report covers the issuance functionality of different asset types in the RGB protocol, including RGB20 (NIA), RGB25 (FUA), and RGB21 (FAC) assets.

## Test Environment
- Testing Framework: Rust Test
- Test File: `tests/issuance.rs`
- Test Cases: 10 test cases covering different asset types and wallet types

## Test Results Summary

### Successful Cases (8/10)

1. **RGB20 - NIA Assets**
   - Single UTXO issuance tests (`issue_nia::case_1`, `issue_nia::case_2`)
   - Multiple UTXO issuance tests (`issue_nia_multiple_utxos::case_1`, `issue_nia_multiple_utxos::case_2`)
   - Support for both Wpkh and Tr wallet types

2. **RGB25 - FUA Assets**
   - Single UTXO issuance tests (`issue_fua::case_1`, `issue_fua::case_2`)
   - Multiple UTXO issuance tests (`issue_fua_multiple_utxos::case_1`, `issue_fua_multiple_utxos::case_2`)
   - Support for both Wpkh and Tr wallet types

### Failed Cases (2/10)

**RGB21 - FAC Assets**
- Test cases: `issue_fac::case_1`, `issue_fac::case_2`
- Context:
  Despite completing the adaptation for the latest RGB21 asset structure, we still encountered issuance failures during testing
- Error message:
  ```
  failed to issue contract: Inner(Genesis(Named(TypeName("DigitalCollection")), ScriptUnspecified))
  ```
- Failure analysis:
  We suspect that the `FN_ASSET_SPEC` validation script implementation is incompatible with the RGB21 asset structure, causing validation failures. We respectfully request the Doctor's judgment on whether different validation logic needs to be created for RGB21 assets.

## Next Steps

1. Current focus is on stability and performance testing of RGB20 assets
2. RGB21-related issues are merely being collected for future solutions from the Doctor
3. Collect and synchronize feedback on RGB21 asset issuance problems

*Note: RGB21 is not our main focus at this stage, as we are currently concentrating on improving the functionality of RGB20.* 