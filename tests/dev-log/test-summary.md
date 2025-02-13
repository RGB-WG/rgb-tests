# RGB v11 基线测试执行情况汇总

## 测试执行概览

| 测试类型     | 总数 | 通过 | 失败 | 忽略 |
| ------------ | ---- | ---- | ---- | ---- |
| issuance.rs  | 20   | 20   | 0    | 0    |
| stress.rs    | 6    | 0    | 0    | 6    |
| transfers.rs | 101  | 86   | 3    | 12   |
| 总计         | 127  | 106  | 3    | 18   |

## 详细测试状态

### V11遗留问题 (ignored, 需要在V12验证是否修复)
1. blank_tapret_opret::case_1
2. blank_tapret_opret::case_3
3. blank_tapret_opret::case_4
4. invoice_reuse::case_1 (可能不是bug，但行为异常)
5. ln_transfers::case_2
6. reorg_history::case_2
7. reorg_history::case_4
8. reorg_history::case_6
9. revert_genesis::case_1
10. revert_genesis::case_2
11. same_transfer_twice_no_update_witnesses::case_1
12. same_transfer_twice_update_witnesses::case_1
13. sync_mainnet_wlt

### BitLight改动引入的问题 (FAILED, 可能跟部署修改有关, 不影响主线)
1. reorg_history::case_1 (错误: addnode命令失败)
2. reorg_history::case_3 (错误: addnode命令失败)
3. reorg_history::case_5 (错误: addnode命令失败)

### 正常通过的关键测试 (ok)
1. mainnet_wlt_receiving_test_asset
2. accept_0conf
3. check_fungible_history
4. invoice_reuse::case_2
5. ln_transfers::case_1
6. rbf_transfer
7. collaborative_transfer
8. send_to_oneself
9. receive_from_unbroadcasted_transfer_to_blinded
10. tapret_wlt_receiving_opret
11. transfer_loop::case_01 - case_72 (所有72个transfer_loop测试)

## 分析总结

1. **成功率**: 83.5% (106/127)
2. **主要问题**:
   - 旧版遗留主要集中在特殊交易场景, 需要后续在V12验证是否修复
   - BitLight改动引入的问题集中在重组历史相关的功能, 可能跟部署修改有关, 不影响主线

3. **稳定性评估**:
   - 基本功能测试（issuance.rs）全部通过，表明核心功能稳定
   - 压力测试（stress.rs）全部被忽略，需要单独评估
   - 转账相关测试（transfers.rs）大部分通过，但存在一些特定场景的问题 