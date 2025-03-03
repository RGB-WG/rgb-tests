# RGB v0.12 第一阶段测试执行情况汇总

这是RGB v0.12的首次测试运行。虽然完整的测试套件仍在开发中，但我们已经完成了大部分转账测试用例的设计。

## 测试执行概览

| 测试类型    | 总数 | 通过 | 失败 | 忽略 |
| ----------- | ---- | ---- | ---- | ---- |
| issuance.rs | 16   | 16   | 0    | 0    |
| transfer.rs | 36   | 9    | 12   | 15   |
| 总计        | 52   | 25   | 12   | 15   |

## 详细测试状态

### 已忽略的测试 (ignored)
1. blank_tapret_opret::case_1..4 - 需要修复 (调用Codex API中不存在的方法)
2. mainnet_wlt_receiving_test_asset - 需要修复
   - 错误：RGB v0.12中的`Mound`结构不支持设置主网
   - 上下文：来自issue_mainnet_wlt_receiving_test_asset.md："默认的`Mound.testnet`等于true，无法正确初始化主网钱包"
3. reorg_history::case_1 - 需要修复
   - 错误："Unable to accept a consignment: unknown seal definition for cell address qMWtQjXCWjJAXdrg7npyI2KZz3vXNVyZhoomqF7v8z4:0."
   - 上下文：测试模拟具有线性历史和ChangeOrder类型的区块链重组
4. reorg_history::case_2 - 需要修复
   - 错误："assertion `left == right` failed, left: [10, 20], right: [600]"
   - 上下文：测试模拟具有线性历史和Revert类型的区块链重组
5. reorg_history::case_3 - 需要修复
   - 错误："Unable to accept a consignment: unknown seal definition for cell address c6z0I0hYqaO6dV9qOjrP1lK4PJprjVAaAOdGCoqAdOY:0."
   - 上下文：测试模拟具有分支历史和ChangeOrder类型的区块链重组
6. reorg_history::case_4 - 需要修复
   - 错误："assertion `left == right` failed, left: [200, 399], right: [600]"
   - 上下文：测试模拟具有分支历史和Revert类型的区块链重组
7. reorg_history::case_5 - 需要修复
   - 错误："Unable to accept a consignment: unknown seal definition for cell address FrGmm~6ro7YOlE9bEuyCLcLt9AlX2uZOZRmjHEq6yyA:0."
   - 上下文：测试模拟具有合并历史和ChangeOrder类型的区块链重组
8. reorg_history::case_6 - 需要修复
   - 错误："assertion `left == right` failed, left: [599], right: [400]"
   - 上下文：测试模拟具有合并历史和Revert类型的区块链重组
9. same_transfer_twice_update_witnesses::case_1 - 等待RGB v0.12中的新回滚程序API
   - 上下文：测试需要更新见证的能力，该功能在v0.12中已被移除，将由新的回滚程序API替代
10. same_transfer_twice_update_witnesses::case_2 - 等待RGB v0.12中的新回滚程序API
    - 上下文：与case_1相同的问题
11. collaborative_transfer - 等待多重签名工作流文档
    - 上下文：测试需要实现多重签名工作流。当前文档很少，底层API复杂难懂。初步实现未成功。计划在核心功能稳定后的后续阶段专注于此实现。
12. ln_transfers - 等待闪电网络集成文档
    - 上下文：测试需要实现多重签名工作流。当前文档很少，底层API复杂难懂。初步实现未成功。计划在核心功能稳定后的后续阶段专注于此实现。

### 失败的测试 (failed)
1. rbf_transfer - Fulfill(StateInsufficient)
   - 错误：`called Result::unwrap() on an Err value: Fulfill(StateInsufficient)`
   - 上下文：测试尝试用更高的手续费替换交易(RBF)，但由于状态不足以完成第二次转账而失败
2. same_transfer_twice_no_update_witnesses::case_1 - Fulfill(StateInsufficient)
   - 错误：`called Result::unwrap() on an Err value: Fulfill(StateInsufficient)`
   - 上下文：测试尝试在不更新见证的情况下两次使用相同的发票，这在v0.12中不受支持
3. same_transfer_twice_no_update_witnesses::case_2 - Fulfill(StateInsufficient)
   - 错误：`called Result::unwrap() on an Err value: Fulfill(StateInsufficient)`
   - 上下文：与case_1相同的问题，与发票重用相关
4. tapret_wlt_receiving_opret - Transaction already in block chain
   - 错误："Transaction already in block chain"
   - 上下文：测试尝试在Taproot和WPKH钱包之间创建多次转账。该测试单独运行时通过，但作为完整测试套件的一部分运行时因交易冲突而失败，表明需要重构测试隔离
5. transfer_loop::case_01 - calling to method absent in Codex API
   - 错误：新的Codex API中缺少方法
   - 上下文：测试尝试在不同钱包类型和资产模式之间转移资产
6. transfer_loop::case_02 - calling to method absent in Codex API
   - 与case_01相同的问题
7. transfer_loop::case_03 - calling to method absent in Codex API
   - 与case_01相同的问题
8. transfer_loop::case_04 - calling to method absent in Codex API
   - 与case_01相同的问题
9. transfer_loop::case_05 - calling to method absent in Codex API
   - 与case_01相同的问题
10. transfer_loop::case_06 - calling to method absent in Codex API
    - 与case_01相同的问题
11. transfer_loop::case_07 - calling to method absent in Codex API
    - 与case_01相同的问题
12. transfer_loop::case_08 - calling to method absent in Codex API
    - 与case_01相同的问题

### 通过的测试 (passed)
1. issue_cfa::case_1 - case_4 (所有4个测试)
   - 上下文：成功发行具有各种参数的可收集的可替代资产(CFA)
2. issue_nia::case_1 - case_4 (所有4个测试)
   - 上下文：成功发行具有各种参数的不可膨胀资产(NIA)
3. issue_cfa_multiple_utxos::case_1 - case_4 (所有4个测试)
   - 上下文：成功发行在多个UTXO上分配的可收集的可替代资产(CFA)
4. issue_nia_multiple_utxos::case_1 - case_4 (所有4个测试)
   - 上下文：成功发行在多个UTXO上分配的不可膨胀资产(NIA)
5. accept_0conf
   - 上下文：成功测试接受未确认(0-conf)交易
6. send_to_oneself
   - 上下文：成功测试向同一钱包发送资产
7. check_fungible_history
   - 上下文：成功测试检查可替代资产的历史
8. transfer_loop::case_09 - case_12 (4个测试)
   - 上下文：成功测试特定钱包和资产组合的部分转账循环情况

## 分析总结

1. **成功率**: 48% (25/52)
2. **主要问题**:
   - 资产发行相关测试（issuance.rs）全部通过，除了UDA资产缺少`.issuer`，表明核心发行功能已成功迁移到v0.12
   - 转账相关测试（transfer.rs）存在较多问题，主要集中在以下几个方面：
     - **API不稳定性**：多个transfer_loop测试和blank_tapret_opret测试失败，出现"calling to method absent in Codex API"错误，表明从v0.11到v0.12的API转换过程中存在破坏性变更和不稳定性。一些以前可用的方法已被移除或修改，需要更新测试实现
     - **状态不足错误**：rbf_transfer和same_transfer_twice_no_update_witnesses测试失败，出现"Fulfill(StateInsufficient)"错误，表明这些场景的状态管理需要更新
     - **发票不支持重用**：尝试重用相同发票的测试失败，出现"Fulfill(StateInsufficient)"错误，正如same_transfer_twice_no_update_witnesses中所述："在RGB V0.12中，由于不可能使用RBF和相同的发票，我们无法测试这种通胀攻击"
     - **区块链重组处理**：API尚未有稳定的机制来处理区块链重组场景，这从reorg_history测试失败中可以看出，重组后资产分配不正确
3. **迁移进展评估**:
   - 资产发行功能迁移完成度高，包括多UTXO分配的情况
   - 基本转账功能部分可用，如accept_0conf、send_to_oneself和部分transfer_loop测试
   - 高级转账功能（如RBF、重组历史、闪电网络集成等）尚未完成迁移
4. **下一步期望优先获取的支持**:
   - 请求指导解决"calling to method absent in Codex API"问题
   - 修复StateInsufficient错误
   - 实现合约架构支持区块链重组场景，有相关的rollback procedure API
   - 添加对发票重用和RBF场景的支持
   - 提供关于支付脚本和内部状态转换API的文档，以实现更复杂的转账场景测试，如多签名和闪电网络集成
