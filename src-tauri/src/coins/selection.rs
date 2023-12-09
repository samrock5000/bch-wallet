//! Coin Selection algos.
//! copied from https://github.com/bitcoindevkit/bdk/tree/master/crates/bdk
use core::fmt;
use rand::prelude::SliceRandom;

use bitcoinsuite_core::{ser::BitcoinSer, tx::Output};

use super::utxo::{
     UnspentUtxos, Utxo,
};

///Total satoshi value for a given address.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TotalSatoshiAmount(pub u64);
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
///Change satoshi value for a given address.
pub struct ChangeSatoshiAmount(pub u64);
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UtxoCandidates {
    pub selected: Vec<Utxo>,
    pub change: Option<ChangeSatoshiAmount>,
}

#[derive(Debug, Clone)]
/// Remaining amount after performing coin selection
pub enum Excess {
    /// It's not possible to create spendable output from excess using the current drain output
    NoChange {
        /// Threshold to consider amount as dust for this particular change script_pubkey
        dust_threshold: u64,
        /// Exceeding amount of current selection over outgoing value and fee costs
        remaining_amount: u64,
        /// The calculated fee for the drain TxOut with the selected script_pubkey
        change_fee: u64,
    },
    /// It's possible to create spendable output from excess using the current drain output
    Change {
        /// Effective amount available to create change after deducting the change output fee
        amount: u64,
        /// The deducted change output fee
        fee: u64,
    },
}
/// A [`Utxo`] with its `satisfaction_weight`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedUtxo {
    /// The weight of the witness data and `scriptSig` expressed in [weight units]. This is used to
    /// properly maintain the feerate when adding this input to a transaction during coin selection.
    ///
    /// [weight units]: https://en.bitcoin.it/wiki/Weight_units
    pub satisfaction_weight: usize,
    /// The UTXO
    pub utxo: Utxo,
}
// Result of a successful coin selection
#[derive(Debug, Clone)]
pub struct CoinSelectionResult {
    /// List of outputs selected for use as inputs
    pub selected: Vec<Utxo>,
    /// Total fee amount for the selected utxos in satoshis
    pub fee_amount: u64,
    /// Remaining amount after deducing fees and outgoing outputs
    pub excess: Excess,
}

impl CoinSelectionResult {
    /// The total value of the inputs selected.
    pub fn selected_amount(&self) -> u64 {
        self.selected.iter().map(|u| u.output.value).sum()
    }
}
/// Fee rate
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
// Internally stored as satoshi/vbyte
pub struct FeeRate(pub f32);

impl FeeRate {
    /// Create a new instance checking the value provided
    ///
    /// ## Panics
    ///
    /// Panics if the value is not [normal](https://doc.rust-lang.org/std/primitive.f32.html#method.is_normal) (except if it's a positive zero) or negative.
    fn new_checked(value: f32) -> Self {
        assert!(value.is_normal() || value == 0.0);
        assert!(value.is_sign_positive());

        FeeRate(value)
    }

    /// Create a new instance of [`FeeRate`] given a float fee rate in sats/kwu
    pub fn from_sat_per_kwu(sat_per_kwu: f32) -> Self {
        FeeRate::new_checked(sat_per_kwu / 250.0_f32)
    }

    /// Create a new instance of [`FeeRate`] given a float fee rate in sats/kvb
    pub fn from_sat_per_kvb(sat_per_kvb: f32) -> Self {
        FeeRate::new_checked(sat_per_kvb / 1000.0_f32)
    }

    /// Create a new instance of [`FeeRate`] given a float fee rate in btc/kvbytes
    ///
    /// ## Panics
    ///
    /// Panics if the value is not [normal](https://doc.rust-lang.org/std/primitive.f32.html#method.is_normal) (except if it's a positive zero) or negative.
    pub fn from_btc_per_kvb(btc_per_kvb: f32) -> Self {
        FeeRate::new_checked(btc_per_kvb * 1e5)
    }

    /// Create a new instance of [`FeeRate`] given a float fee rate in satoshi/vbyte
    ///
    /// ## Panics
    ///
    /// Panics if the value is not [normal](https://doc.rust-lang.org/std/primitive.f32.html#method.is_normal) (except if it's a positive zero) or negative.
    pub fn from_sat_per_vb(sat_per_vb: f32) -> Self {
        FeeRate::new_checked(sat_per_vb)
    }

    /// Create a new [`FeeRate`] with the default min relay fee value
    pub const fn default_min_relay_fee() -> Self {
        FeeRate(1.0)
    }

    /// Calculate fee rate from `fee` and weight units (`wu`).
    pub fn from_wu(fee: u64, wu: Weight) -> FeeRate {
        Self::from_vb(fee, wu.to_vbytes_ceil() as usize)
    }

    /// Calculate fee rate from `fee` and `vbytes`.
    pub fn from_vb(fee: u64, vbytes: usize) -> FeeRate {
        let rate = fee as f32 / vbytes as f32;
        Self::from_sat_per_vb(rate)
    }

    /// Return the value as satoshi/vbyte
    pub fn as_sat_per_vb(&self) -> f32 {
        self.0
    }

    /// Return the value as satoshi/kwu
    pub fn sat_per_kwu(&self) -> f32 {
        self.0 * 250.0_f32
    }

    /// Calculate absolute fee in Satoshis using size in weight units.
    pub fn fee_wu(&self, wu: Weight) -> u64 {
        self.fee_vb(wu.to_vbytes_ceil() as usize)
    }

    /// Calculate absolute fee in Satoshis using size in virtual bytes.
    pub fn fee_vb(&self, vbytes: usize) -> u64 {
        (self.as_sat_per_vb() * vbytes as f32).ceil() as u64
    }
}

impl Default for FeeRate {
    fn default() -> Self {
        FeeRate::default_min_relay_fee()
    }
}

impl core::ops::Sub for FeeRate {
    type Output = Self;

    fn sub(self, other: FeeRate) -> Self::Output {
        FeeRate(self.0 - other.0)
    }
}

/// Trait implemented by types that can be used to measure weight units.
pub trait Vbytes {
    /// Convert weight units to virtual bytes.
    fn vbytes(self) -> usize;
}

impl Vbytes for usize {
    fn vbytes(self) -> usize {
        // ref: https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki#transaction-size-calculations
        (self as f32 / 4.0).ceil() as usize
    }
}

/// Trait for generalized coin selection algorithms

pub trait CoinSelectionAlgorithm: core::fmt::Debug {
    /// Perform the coin selection
    ///
    /// - `database`: a reference to the wallet's database that can be used to lookup additional
    ///               details for a specific UTXO
    /// - `required_utxos`: the utxos that must be spent regardless of `target_amount` with their
    ///                     weight cost
    /// - `optional_utxos`: the remaining available utxos to satisfy `target_amount` with their
    ///                     weight cost
    /// - `fee_rate`: fee rate to use
    /// - `target_amount`: the outgoing amount in satoshis and the fees already
    ///                    accumulated from added outputs and transaction’s header.
    /// - `drain_script`: the script to use in case of change
    #[allow(clippy::too_many_arguments)]
    fn coin_select(
        &self,
        required_utxos: Vec<WeightedUtxo>,
        optional_utxos: Vec<WeightedUtxo>,
        fee_rate: FeeRate,
        target_amount: u64,
        drain_script: &Output,
    ) -> Result<CoinSelectionResult, Error>;
}

/// Simple and dumb coin selection
///
/// This coin selection algorithm sorts the available UTXOs by value and then picks them starting
/// from the largest ones until the required amount is reached.
#[derive(Debug, Default, Clone, Copy)]
pub struct LargestFirstCoinSelection;

impl CoinSelectionAlgorithm for LargestFirstCoinSelection {
    fn coin_select(
        &self,
        required_utxos: Vec<WeightedUtxo>,
        mut optional_utxos: Vec<WeightedUtxo>,
        fee_rate: FeeRate,
        target_amount: u64,
        drain_script: &Output,
    ) -> Result<CoinSelectionResult, Error> {
        // We put the "required UTXOs" first and make sure the optional UTXOs are sorted,
        // initially smallest to largest, before being reversed with `.rev()`.
        let utxos = {
            optional_utxos.sort_unstable_by_key(|wu| wu.utxo.output.value /* txout().value */);
            required_utxos
                .into_iter()
                .map(|utxo| (true, utxo))
                .chain(optional_utxos.into_iter().rev().map(|utxo| (false, utxo)))
        };

        select_sorted_utxos(utxos, fee_rate, target_amount, &drain_script)
    }
}

/// Branch and bound coin selection
///
/// Code adapted from Bitcoin Core's implementation and from Mark Erhardt Master's Thesis: <http://murch.one/wp-content/uploads/2016/11/erhardt2016coinselection.pdf>
#[derive(Debug, Clone)]
pub struct BranchAndBoundCoinSelection {
    size_of_change: u64,
}

impl Default for BranchAndBoundCoinSelection {
    fn default() -> Self {
        Self {
            // P2PKH cost of change -> value (8 bytes) + script len (1 bytes) + script (25 bytes)
            size_of_change: 8 + 1 + 25,
        }
    }
}

impl BranchAndBoundCoinSelection {
    /// Create new instance with target size for change output
    pub fn new(size_of_change: u64) -> Self {
        Self { size_of_change }
    }
}

#[derive(Debug, Clone)]
// Adds fee information to an UTXO.
struct OutputGroup {
    weighted_utxo: WeightedUtxo,
    // Amount of fees for spending a certain utxo, calculated using a certain FeeRate
    fee: u64,
    // The effective value of the UTXO, i.e., the utxo value minus the fee for spending it
    effective_value: i64,
}

impl OutputGroup {
    fn new(weighted_utxo: WeightedUtxo, fee_rate: FeeRate) -> Self {
        let fee = fee_rate.fee_wu(Weight::from_wu(
            (TXIN_BASE_WEIGHT + weighted_utxo.satisfaction_weight) as u64,
        ));
        let effective_value = weighted_utxo.utxo.output.value as i64 - fee as i64;
        OutputGroup {
            weighted_utxo,
            fee,
            effective_value,
        }
    }
}

const BNB_TOTAL_TRIES: usize = 100_000;

impl CoinSelectionAlgorithm for BranchAndBoundCoinSelection {
    fn coin_select(
        &self,
        required_utxos: Vec<WeightedUtxo>,
        optional_utxos: Vec<WeightedUtxo>,
        fee_rate: FeeRate,
        target_amount: u64,
        drain_script: &Output,
    ) -> Result<CoinSelectionResult, Error> {
        // Mapping every (UTXO, usize) to an output group
        let required_utxos: Vec<OutputGroup> = required_utxos
            .into_iter()
            .map(|u| OutputGroup::new(u, fee_rate))
            .collect();

        // Mapping every (UTXO, usize) to an output group, filtering UTXOs with a negative
        // effective value
        let optional_utxos: Vec<OutputGroup> = optional_utxos
            .into_iter()
            .map(|u| OutputGroup::new(u, fee_rate))
            .filter(|u| u.effective_value.is_positive())
            .collect();

        let curr_value = required_utxos
            .iter()
            .fold(0, |acc, x| acc + x.effective_value);

        let curr_available_value = optional_utxos
            .iter()
            .fold(0, |acc, x| acc + x.effective_value);

        let cost_of_change = self.size_of_change as f32 * fee_rate.as_sat_per_vb();

        // `curr_value` and `curr_available_value` are both the sum of *effective_values* of
        // the UTXOs. For the optional UTXOs (curr_available_value) we filter out UTXOs with
        // negative effective value, so it will always be positive.
        //
        // Since we are required to spend the required UTXOs (curr_value) we have to consider
        // all their effective values, even when negative, which means that curr_value could
        // be negative as well.
        //
        // If the sum of curr_value and curr_available_value is negative or lower than our target,
        // we can immediately exit with an error, as it's guaranteed we will never find a solution
        // if we actually run the BnB.
        let total_value: Result<u64, _> = (curr_available_value + curr_value).try_into();
        match total_value {
            Ok(v) if v >= target_amount => {}
            _ => {
                // Assume we spend all the UTXOs we can (all the required + all the optional with
                // positive effective value), sum their value and their fee cost.
                let (utxo_fees, utxo_value) = required_utxos
                    .iter()
                    .chain(optional_utxos.iter())
                    .fold((0, 0), |(mut fees, mut value), utxo| {
                        fees += utxo.fee;
                        value += utxo.weighted_utxo.utxo.output.value;

                        (fees, value)
                    });
                // println!("target_amount + utxo_fees {:?} | target {:?}", v, target_amount);
                // Add to the target the fee cost of the UTXOs
                return Err(Error::InsufficientFunds {
                    needed: target_amount + utxo_fees,
                    available: utxo_value,
                });
            }
        }

        let target_amount = target_amount
            .try_into()
            .expect("Bitcoin amount to fit into i64");

        if curr_value > target_amount {
            // remaining_amount can't be negative as that would mean the
            // selection wasn't successful
            // target_amount = amount_needed + (fee_amount - vin_fees)
            let remaining_amount = (curr_value - target_amount) as u64;

            let excess = decide_change(remaining_amount, fee_rate, &drain_script);

            return Ok(BranchAndBoundCoinSelection::calculate_cs_result(
                vec![],
                required_utxos,
                excess,
            ));
        }

        Ok(self
            .bnb(
                required_utxos.clone(),
                optional_utxos.clone(),
                curr_value,
                curr_available_value,
                target_amount,
                cost_of_change,
                &drain_script,
                fee_rate,
            )
            .unwrap_or_else(|_| {
                self.single_random_draw(
                    required_utxos,
                    optional_utxos,
                    curr_value,
                    target_amount,
                    &drain_script,
                    fee_rate,
                )
            }))
    }
}

impl BranchAndBoundCoinSelection {
    // TODO: make this more Rust-onic :)
    // (And perhaps refactor with less arguments?)
    #[allow(clippy::too_many_arguments)]
    fn bnb(
        &self,
        required_utxos: Vec<OutputGroup>,
        mut optional_utxos: Vec<OutputGroup>,
        mut curr_value: i64,
        mut curr_available_value: i64,
        target_amount: i64,
        cost_of_change: f32,
        drain_script: &Output,
        fee_rate: FeeRate,
    ) -> Result<CoinSelectionResult, Error> {
        // current_selection[i] will contain true if we are using optional_utxos[i],
        // false otherwise. Note that current_selection.len() could be less than
        // optional_utxos.len(), it just means that we still haven't decided if we should keep
        // certain optional_utxos or not.
        let mut current_selection: Vec<bool> = Vec::with_capacity(optional_utxos.len());

        // Sort the utxo_pool
        optional_utxos.sort_unstable_by_key(|a| a.effective_value);
        optional_utxos.reverse();

        // Contains the best selection we found
        let mut best_selection = Vec::new();
        let mut best_selection_value = None;

        // Depth First search loop for choosing the UTXOs
        for _ in 0..BNB_TOTAL_TRIES {
            // Conditions for starting a backtrack
            let mut backtrack = false;
            // Cannot possibly reach target with the amount remaining in the curr_available_value,
            // or the selected value is out of range.
            // Go back and try other branch
            if curr_value + curr_available_value < target_amount
                || curr_value > target_amount + cost_of_change as i64
            {
                backtrack = true;
            } else if curr_value >= target_amount {
                // Selected value is within range, there's no point in going forward. Start
                // backtracking
                backtrack = true;

                // If we found a solution better than the previous one, or if there wasn't previous
                // solution, update the best solution
                if best_selection_value.is_none() || curr_value < best_selection_value.unwrap() {
                    best_selection = current_selection.clone();
                    best_selection_value = Some(curr_value);
                }

                // If we found a perfect match, break here
                if curr_value == target_amount {
                    break;
                }
            }

            // Backtracking, moving backwards
            if backtrack {
                // Walk backwards to find the last included UTXO that still needs to have its omission branch traversed.
                while let Some(false) = current_selection.last() {
                    current_selection.pop();
                    curr_available_value += optional_utxos[current_selection.len()].effective_value;
                }

                if current_selection.last_mut().is_none() {
                    // We have walked back to the first utxo and no branch is untraversed. All solutions searched
                    // If best selection is empty, then there's no exact match
                    if best_selection.is_empty() {
                        return Err(Error::BnBNoExactMatch);
                    }
                    break;
                }

                if let Some(c) = current_selection.last_mut() {
                    // Output was included on previous iterations, try excluding now.
                    *c = false;
                }

                let utxo = &optional_utxos[current_selection.len() - 1];
                curr_value -= utxo.effective_value;
            } else {
                // Moving forwards, continuing down this branch
                let utxo = &optional_utxos[current_selection.len()];

                // Remove this utxo from the curr_available_value utxo amount
                curr_available_value -= utxo.effective_value;

                // Inclusion branch first (Largest First Exploration)
                current_selection.push(true);
                curr_value += utxo.effective_value;
            }
        }
        // Check for solution
        if best_selection.is_empty() {
            return Err(Error::BnBTotalTriesExceeded);
        }

        // Set output set
        let selected_utxos = optional_utxos
            .into_iter()
            .zip(best_selection)
            .filter_map(|(optional, is_in_best)| if is_in_best { Some(optional) } else { None })
            .collect::<Vec<OutputGroup>>();

        let selected_amount = best_selection_value.unwrap();

        // remaining_amount can't be negative as that would mean the
        // selection wasn't successful
        // target_amount = amount_needed + (fee_amount - vin_fees)
        let remaining_amount = (selected_amount - target_amount) as u64;

        let excess = decide_change(remaining_amount, fee_rate, drain_script);

        Ok(BranchAndBoundCoinSelection::calculate_cs_result(
            selected_utxos,
            required_utxos,
            excess,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn single_random_draw(
        &self,
        required_utxos: Vec<OutputGroup>,
        mut optional_utxos: Vec<OutputGroup>,
        curr_value: i64,
        target_amount: i64,
        drain_script: &Output,
        fee_rate: FeeRate,
    ) -> CoinSelectionResult {
        optional_utxos.shuffle(&mut rand::thread_rng());
        let selected_utxos = optional_utxos.into_iter().fold(
            (curr_value, vec![]),
            |(mut amount, mut utxos), utxo| {
                if amount >= target_amount {
                    (amount, utxos)
                } else {
                    amount += utxo.effective_value;
                    utxos.push(utxo);
                    (amount, utxos)
                }
            },
        );

        // remaining_amount can't be negative as that would mean the
        // selection wasn't successful
        // target_amount = amount_needed + (fee_amount - vin_fees)
        let remaining_amount = (selected_utxos.0 - target_amount) as u64;

        let excess = decide_change(remaining_amount, fee_rate, drain_script);

        BranchAndBoundCoinSelection::calculate_cs_result(selected_utxos.1, required_utxos, excess)
    }

    fn calculate_cs_result(
        mut selected_utxos: Vec<OutputGroup>,
        mut required_utxos: Vec<OutputGroup>,
        excess: Excess,
    ) -> CoinSelectionResult {
        selected_utxos.append(&mut required_utxos);
        let fee_amount = selected_utxos.iter().map(|u| u.fee).sum::<u64>();
        let selected = selected_utxos
            .into_iter()
            .map(|u| u.weighted_utxo.utxo)
            .collect::<Vec<_>>();
        CoinSelectionResult {
            selected,
            fee_amount,
            excess,
        }
    }
}

pub(crate) const TXIN_BASE_WEIGHT: usize = (32 + 4 + 4) * 4;

fn select_sorted_utxos(
    utxos: impl Iterator<Item = (bool, WeightedUtxo)>,
    fee_rate: FeeRate,
    target_amount: u64,
    drain_script: &Output,
) -> Result<CoinSelectionResult, Error> {
    let mut selected_amount = 0;
    let mut fee_amount = 0;
    let selected = utxos
        .scan(
            (&mut selected_amount, &mut fee_amount),
            |(selected_amount, fee_amount), (must_use, weighted_utxo)| {
                if must_use || **selected_amount < target_amount + **fee_amount {
                    **fee_amount += fee_rate.fee_wu(Weight::from_wu(
                        (TXIN_BASE_WEIGHT + weighted_utxo.satisfaction_weight) as u64,
                    ));

                    **selected_amount += weighted_utxo.utxo.output.value;

                    Some(weighted_utxo.utxo)
                } else {
                    None
                }
            },
        )
        .collect::<Vec<_>>();

    let amount_needed_with_fees = target_amount + fee_amount;
    if selected_amount < amount_needed_with_fees {
        return Err(Error::InsufficientFunds {
            needed: amount_needed_with_fees,
            available: selected_amount,
        });
    }

    let remaining_amount = selected_amount - amount_needed_with_fees;


    let excess = decide_change(remaining_amount, fee_rate, drain_script);

    Ok(CoinSelectionResult {
        selected,
        fee_amount,
        excess,
    })
}

/// Decide if change can be created
///
/// - `remaining_amount`: the amount in which the selected coins exceed the target amount
/// - `fee_rate`: required fee rate for the current selection
/// - `drain_script`: script to consider change creation
pub fn decide_change(remaining_amount: u64, fee_rate: FeeRate, drain_script: &Output) -> Excess {
    // drain_output_len = size(len(script_pubkey)) + len(script_pubkey) + size(output_value)
    // let drain_output_len = serialize(drain_script).len() + 8usize;
    let drain_output_len = drain_script.ser_len(); /* + 8usize; */
    let change_fee = fee_rate.fee_vb(drain_output_len);
    let drain_val = remaining_amount.saturating_sub(change_fee);
    // is dust
    if drain_val < calculate_dust(&drain_script) {
        let dust_threshold = calculate_dust(&drain_script);
        Excess::NoChange {
            dust_threshold,
            change_fee,
            remaining_amount,
        }
    } else {
        Excess::Change {
            amount: drain_val,
            fee: change_fee,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    /// Generic error
    Generic(String),
    /// Cannot build a tx without recipients
    NoRecipients,
    /// `manually_selected_only` option is selected but no utxo has been passed
    NoUtxosSelected,
    /// Output created is under the dust limit, 546 satoshis
    OutputBelowDustLimit(usize),
    /// Wallet's UTXO set is not enough to cover recipient's requested plus fee
    InsufficientFunds {
        /// Sats needed for some transaction
        needed: u64,
        /// Sats available for spending
        available: u64,
    },
    /// Branch and bound coin selection tries to avoid needing a change by finding the right inputs for
    /// the desired outputs plus fee, if there is not such combination this error is thrown
    BnBNoExactMatch,
    /// Branch and bound coin selection possible attempts with sufficiently big UTXO set could grow
    /// exponentially, thus a limit is set, and when hit, this error is thrown
    BnBTotalTriesExceeded,
}

//from bitcoin-rust crate
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Weight(u64);

impl Weight {
    /// 0 wu.
    ///
    /// Equivalent to [`MIN`](Self::MIN), may better express intent in some contexts.
    pub const ZERO: Weight = Weight(0);

    /// Minimum possible value (0 wu).
    ///
    /// Equivalent to [`ZERO`](Self::ZERO), may better express intent in some contexts.
    pub const MIN: Weight = Weight(u64::min_value());

    /// Maximum possible value.
    pub const MAX: Weight = Weight(u64::max_value());

    /// Directly constructs `Weight` from weight units.
    pub const fn from_wu(wu: u64) -> Self {
        Weight(wu)
    }

    /// Constructs `Weight` from virtual bytes.
    ///
    /// # Errors
    ///
    /// Returns `None` on overflow.
    pub fn from_vb(vb: u64) -> Option<Self> {
        vb.checked_mul(4).map(Weight::from_wu)
    }

    /// Constructs `Weight` from virtual bytes without overflow check.
    pub const fn from_vb_unchecked(vb: u64) -> Self {
        Weight::from_wu(vb * 4)
    }

    /// Constructs `Weight` from witness size.
    pub const fn from_witness_data_size(witness_size: u64) -> Self {
        Weight(witness_size)
    }

    /// Constructs `Weight` from non-witness size.
    pub const fn from_non_witness_data_size(non_witness_size: u64) -> Self {
        Weight(non_witness_size * 4)
    }

    /// Returns raw weight units.
    ///
    /// Can be used instead of `into()` to avoid inference issues.
    pub const fn to_wu(self) -> u64 {
        self.0
    }

    /// Converts to vB rounding down.
    pub const fn to_vbytes_floor(self) -> u64 {
        self.0 / 4
    }

    /// Converts to vB rounding up.
    pub const fn to_vbytes_ceil(self) -> u64 {
        (self.0 + 3) / 4
    }

    /// Checked addition.
    ///
    /// Computes `self + rhs` returning `None` if overflow occurred.
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    /// Checked subtraction.
    ///
    /// Computes `self - rhs` returning `None` if overflow occurred.
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    /// Checked multiplication.
    ///
    /// Computes `self * rhs` returning `None` if overflow occurred.
    pub fn checked_mul(self, rhs: u64) -> Option<Self> {
        self.0.checked_mul(rhs).map(Self)
    }

    /// Checked division.
    ///
    /// Computes `self / rhs` returning `None` if `rhs == 0`.
    pub fn checked_div(self, rhs: u64) -> Option<Self> {
        self.0.checked_div(rhs).map(Self)
    }
}

/// Alternative will display the unit.
impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{} wu", self.0)
        } else {
            fmt::Display::fmt(&self.0, f)
        }
    }
}

pub fn selection_final_candidates(
    selected: &CoinSelectionResult,
) -> Result<UtxoCandidates, String> {
    let change = match selected.excess {
        Excess::Change { amount, fee: _ } => Some(amount),
        Excess::NoChange {
            dust_threshold: _,
            remaining_amount: _,
            change_fee: _,
        } => None,
    };
    if let Some(change) = change {
        Ok(UtxoCandidates {
            selected: selected.selected.clone(),
            change: Some(ChangeSatoshiAmount(change)),
        })
    } else {
        Ok(UtxoCandidates {
            selected: selected.selected.clone(),
            change: None,
        })
    }
}
pub fn non_token_amount_from_utxo(utxos: &UnspentUtxos) -> u64 {
    let mut sum = 0;
    utxos
        .non_token
        .iter()
        .for_each(|utxo| sum += utxo.0.output.value);
    sum
}

pub fn calculate_dust(output: &Output) -> u64 {
    output.ser_len() as u64 * 3 + 444 as u64
}
/*
mod test {
    use bitcoincash_addr::Address;
    use bitcoinsuite_core::hash::ShaRmd160;

    use super::*;

    /*    #[test]
    fn coin_selection() {
        let fee = FeeRate::from_sat_per_vb(0.0);
        let test_addr = "bchtest:qzxu4ynqdgyjr2hvt5xcx7x35ncdz8zffsf2hgn9mp";
        let pkh: &[u8] = &Address::decode(test_addr).unwrap().body;

        let script = Script::p2pkh(&ShaRmd160::from_be_slice(pkh).unwrap());
        let db_utxos = get_db_utxo_unspent(test_addr);
        let db_utxos = serde_json_to_utxo(db_utxos.unwrap(), test_addr).unwrap();
        let mut utxos: Vec<WeightedUtxo> = Vec::new();
        db_utxos.non_token.iter().for_each(|utxo| {
            let x = WeightedUtxo {
                satisfaction_weight: fee.0 as usize,
                utxo: utxo.clone(),
            };
            utxos.push(x);
        });

        //total 10008378
        let selection = BranchAndBoundCoinSelection::default().coin_select(
            utxos,
            vec![],
            FeeRate::from_sat_per_vb(1.0),
            54,
            &script,
        );

        println!("{:#?}", selection_final(selection.unwrap()));
    } */
}
*/
