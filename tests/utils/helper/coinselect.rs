use super::*;

/// Custom RGB coinselection strategy for more precise control over UTXO selection
///
/// # Usage Example
///
/// ```
/// // Create wallet
/// let mut wallet = get_wallet(&DescriptorType::Wpkh);
///
/// // Set to true small size strategy (selects UTXOs with maximum values)
/// wallet.set_coinselect_strategy(CustomCoinselectStrategy::TrueSmallSize);
///
/// // Or use standard strategies
/// wallet.set_coinselect_strategy(CustomCoinselectStrategy::Standard(CoinselectStrategy::Aggregate));
/// wallet.set_coinselect_strategy(CustomCoinselectStrategy::Standard(CoinselectStrategy::SmallSize));
///
/// // For transfers requiring specific UTXOs (like testing reorganization history), use:
/// let (consignment, tx) = wallet.transfer_with_specific_utxo(invoice, specific_utxo, sats, fee, broadcast, report);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CustomCoinselectStrategy {
    /// Use standard RGB coinselection strategies
    /// - Aggregate: Collects many small outputs until target amount is reached
    /// - SmallSize: Collects minimum number of outputs but without proper value sorting
    Standard(CoinselectStrategy),

    /// Enhanced coinselection strategy that truly minimizes transaction size
    /// by selecting the minimum number of UTXOs with largest asset values.
    /// This strategy:
    /// 1. First sorts all available colored UTXOs by their asset amount
    /// 2. Selects the minimum number of largest-value UTXOs needed to satisfy the transfer
    /// 3. Results in smallest possible transaction size by using fewer inputs
    TrueSmallSize,
}

impl Default for CustomCoinselectStrategy {
    fn default() -> Self {
        Self::Standard(CoinselectStrategy::default())
    }
}

/// Implementation of the Coinselect trait for our custom strategy
impl Coinselect for CustomCoinselectStrategy {
    fn coinselect<'a>(
        &mut self,
        invoiced_state: &StrictVal,
        calc: &mut StateCalc,
        // Sorted vector by values
        owned_state: impl IntoIterator<
            Item = &'a OwnedState<Outpoint>,
            IntoIter: DoubleEndedIterator<Item = &'a OwnedState<Outpoint>>,
        >,
    ) -> Option<Vec<(CellAddr, Outpoint)>> {
        match self {
            // For standard strategies, delegate to the original implementation
            CustomCoinselectStrategy::Standard(strategy) => {
                strategy.coinselect(invoiced_state, calc, owned_state)
            }

            // True small size implementation - sort by value before selection
            CustomCoinselectStrategy::TrueSmallSize => {
                // Clone the state to allow sorting (we need to own the data)
                let mut value_sorted_state: Vec<(CellAddr, &StrictVal, u64, Outpoint)> =
                    owned_state
                        .into_iter()
                        .filter_map(|state| {
                            // Extract numeric value (assuming we're dealing with u64 values)
                            let val = &state.assignment.data;
                            let amount: u64 = val.unwrap_num().unwrap_uint();
                            Some((state.addr, val, amount, state.assignment.seal))
                        })
                        .collect();

                // Sort by value in descending order (largest first)
                value_sorted_state.sort_by(|a, b| b.2.cmp(&a.2));

                // Now use the sorted state for iteration
                let res = value_sorted_state
                    .into_iter()
                    .take_while(|(_, val, _, _)| {
                        if calc.is_satisfied(invoiced_state) {
                            return false;
                        }
                        calc.accumulate(val).is_ok()
                    })
                    .map(|(addr, _, _, outpoint)| (addr, outpoint))
                    .collect();

                if !calc.is_satisfied(invoiced_state) {
                    return None;
                };

                Some(res)
            }
        }
    }
}
