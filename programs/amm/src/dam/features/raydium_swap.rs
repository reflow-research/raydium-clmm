use super::super::{
    schema::{DamFeatureSlot, DAM_FEATURE_COUNT},
    DamFeatureSource, FeatureVector,
};
use super::TxnSignals;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaydiumSwapObservation {
    pub amount_specified: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit_x64: u128,
    pub is_base_input: bool,
    pub zero_for_one: bool,
    pub vault_in_balance: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaydiumSwapFeatureFrame {
    features: FeatureVector<DAM_FEATURE_COUNT>,
}

impl RaydiumSwapFeatureFrame {
    pub fn from_observation(observation: &RaydiumSwapObservation, tx_signals: &TxnSignals) -> Self {
        let mut features = FeatureVector::<DAM_FEATURE_COUNT>::default();
        let output = features.as_mut_array();

        output[DamFeatureSlot::AmountLog2.index()] = u64_log2_bucket(observation.amount_specified);
        output[DamFeatureSlot::OtherAmountThresholdLog2.index()] =
            u64_log2_bucket(observation.other_amount_threshold);
        output[DamFeatureSlot::VaultInBalanceLog2.index()] =
            u64_log2_bucket(observation.vault_in_balance);
        output[DamFeatureSlot::AmountVaultRatioBpsBucket.index()] =
            ratio_bps_bucket(observation.amount_specified, observation.vault_in_balance);
        output[DamFeatureSlot::ThresholdVaultRatioBpsBucket.index()] = ratio_bps_bucket(
            observation.other_amount_threshold,
            observation.vault_in_balance,
        );
        output[DamFeatureSlot::AmountSpecifiedIsInput.index()] =
            bool_as_i8(observation.is_base_input);
        output[DamFeatureSlot::SwapDirection.index()] =
            if observation.zero_for_one { 1 } else { -1 };
        output[DamFeatureSlot::HasExplicitPriceLimit.index()] =
            bool_as_i8(observation.sqrt_price_limit_x64 != 0);
        output[DamFeatureSlot::ComputeUnitLimitLog2.index()] =
            u64_log2_bucket(u64::from(tx_signals.compute_unit_limit));
        output[DamFeatureSlot::ComputeUnitPriceMicroLamportsLog2.index()] =
            u64_log2_bucket(tx_signals.compute_unit_price_micro_lamports);
        output[DamFeatureSlot::ComputeBudgetSignalBucket.index()] =
            compute_budget_signal_bucket(tx_signals);
        output[DamFeatureSlot::PriorTopLevelIxCount.index()] =
            tx_signals.prior_top_level_ix_count.min(127) as i8;
        output[DamFeatureSlot::PriorNonComputeBudgetIxCount.index()] =
            tx_signals.prior_non_compute_budget_ix_count.min(127) as i8;
        output[DamFeatureSlot::PriorSameProgramIxCount.index()] =
            tx_signals.prior_same_program_ix_count.min(127) as i8;
        output[DamFeatureSlot::PriorInstructionDataBytesSumLog2.index()] =
            u64_log2_bucket(u64::from(tx_signals.prior_instruction_data_bytes_sum));
        output[DamFeatureSlot::PriorInstructionDataBytesMaxLog2.index()] =
            u64_log2_bucket(u64::from(tx_signals.prior_instruction_data_bytes_max));
        output[DamFeatureSlot::PriorNonemptyDataIxCount.index()] =
            tx_signals.prior_nonempty_data_ix_count.min(127) as i8;
        output[DamFeatureSlot::PriorUniqueFirstDataByteCount.index()] =
            tx_signals.prior_unique_first_data_byte_count.min(127) as i8;
        output[DamFeatureSlot::TotalTopLevelIxCount.index()] =
            tx_signals.total_top_level_ix_count.min(127) as i8;
        output[DamFeatureSlot::PostTopLevelIxCount.index()] =
            tx_signals.post_top_level_ix_count.min(127) as i8;
        output[DamFeatureSlot::PostNonComputeBudgetIxCount.index()] =
            tx_signals.post_non_compute_budget_ix_count.min(127) as i8;
        output[DamFeatureSlot::HasTopLevelMemoIxAnywhere.index()] =
            bool_as_i8(tx_signals.has_top_level_memo_ix_anywhere);
        output[DamFeatureSlot::PriorKnownRouterIxCount.index()] =
            tx_signals.prior_known_router_ix_count.min(127) as i8;
        output[DamFeatureSlot::RouterContextBucket.index()] =
            tx_signals.router_context_bucket.min(127) as i8;
        output[DamFeatureSlot::RouteOriginBucket.index()] =
            tx_signals.route_origin_bucket.min(3) as i8;
        output[DamFeatureSlot::RouteParseKnown.index()] = bool_as_i8(tx_signals.route_parse_known);
        output[DamFeatureSlot::MultipleRouterFamiliesTouched.index()] =
            bool_as_i8(tx_signals.multiple_router_families_touched);
        output[DamFeatureSlot::HasJitoTipTransferTopLevel.index()] =
            bool_as_i8(tx_signals.has_jito_tip_transfer_top_level);
        output[DamFeatureSlot::JitoTipLamportsMaxTopLevelLog2.index()] =
            u64_log2_bucket(tx_signals.jito_tip_lamports_max_top_level);
        output[DamFeatureSlot::JitoTipTransferCountTopLevel.index()] =
            tx_signals.jito_tip_transfer_count_top_level.min(127) as i8;
        output[DamFeatureSlot::JitoTipPositionBucketTopLevel.index()] =
            tx_signals.jito_tip_position_bucket_top_level.min(3) as i8;
        output[DamFeatureSlot::HasNonJitoTipTransferTopLevel.index()] =
            bool_as_i8(tx_signals.has_non_jito_tip_transfer_top_level);
        output[DamFeatureSlot::NonJitoTipLamportsMaxTopLevelLog2.index()] =
            u64_log2_bucket(tx_signals.non_jito_tip_lamports_max_top_level);
        output[DamFeatureSlot::NonJitoTipTransferCountTopLevel.index()] =
            tx_signals.non_jito_tip_transfer_count_top_level.min(127) as i8;
        output[DamFeatureSlot::TipServicePresenceContextBucketTopLevel.index()] = tx_signals
            .tip_service_presence_context_bucket_top_level
            .min(3)
            as i8;
        output[DamFeatureSlot::HasTopLevelAtaCreateIxAnywhere.index()] =
            bool_as_i8(tx_signals.has_top_level_ata_create_ix_anywhere);
        output[DamFeatureSlot::HasTopLevelCloseAccountAnywhere.index()] =
            bool_as_i8(tx_signals.has_top_level_close_account_anywhere);
        output[DamFeatureSlot::HasTopLevelTempTokenAccountCreateAndCloseSameTx.index()] =
            bool_as_i8(tx_signals.has_top_level_temp_token_account_create_and_close_same_tx);

        Self { features }
    }

    pub const fn from_raw(features: FeatureVector<DAM_FEATURE_COUNT>) -> Self {
        Self { features }
    }

    #[cfg(test)]
    pub const fn as_array(&self) -> &[i8; DAM_FEATURE_COUNT] {
        self.features.as_array()
    }
}

impl DamFeatureSource<DAM_FEATURE_COUNT> for RaydiumSwapFeatureFrame {
    fn write_features(&self, output: &mut FeatureVector<DAM_FEATURE_COUNT>) {
        *output = self.features;
    }
}

fn bool_as_i8(value: bool) -> i8 {
    if value {
        1
    } else {
        0
    }
}

fn u64_log2_bucket(value: u64) -> i8 {
    if value == 0 {
        0
    } else {
        (u64::BITS - 1 - value.leading_zeros()) as i8
    }
}

fn ratio_bps_bucket(numerator: u64, denominator: u64) -> i8 {
    if numerator == 0 || denominator == 0 {
        return 0;
    }

    let bps = (u128::from(numerator) * 10_000 / u128::from(denominator)).min(1_000_000) as u64;
    match bps {
        0..=1 => 0,
        2..=5 => 1,
        6..=10 => 2,
        11..=25 => 3,
        26..=50 => 4,
        51..=100 => 5,
        101..=250 => 6,
        251..=500 => 7,
        501..=1_000 => 8,
        1_001..=2_500 => 9,
        2_501..=5_000 => 10,
        5_001..=10_000 => 11,
        10_001..=25_000 => 12,
        25_001..=50_000 => 13,
        50_001..=100_000 => 14,
        100_001..=250_000 => 15,
        250_001..=500_000 => 16,
        _ => 17,
    }
}

fn compute_budget_signal_bucket(signals: &TxnSignals) -> i8 {
    match (
        signals.has_compute_budget_ix,
        signals.has_compute_unit_price_ix,
    ) {
        (false, false) => 0,
        (true, false) => 1,
        (true, true) => 2,
        (false, true) => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dam::DAM_INTENTIONALLY_ZERO_SLOTS;

    #[test]
    fn raydium_feature_frame_matches_frozen_abi_slots() {
        let observation = RaydiumSwapObservation {
            amount_specified: 1_024,
            other_amount_threshold: 64,
            sqrt_price_limit_x64: 1,
            is_base_input: true,
            zero_for_one: true,
            vault_in_balance: 65_536,
        };
        let tx_signals = TxnSignals {
            has_compute_budget_ix: true,
            has_compute_unit_price_ix: true,
            compute_unit_limit: 1_400_000,
            compute_unit_price_micro_lamports: 50_000,
            prior_top_level_ix_count: 2,
            prior_non_compute_budget_ix_count: 1,
            prior_same_program_ix_count: 1,
            prior_known_router_ix_count: 1,
            router_context_bucket: 3,
            route_origin_bucket: 1,
            route_parse_known: true,
            multiple_router_families_touched: true,
            prior_instruction_data_bytes_sum: 96,
            prior_instruction_data_bytes_max: 64,
            prior_nonempty_data_ix_count: 2,
            prior_unique_first_data_byte_count: 2,
            total_top_level_ix_count: 4,
            post_top_level_ix_count: 1,
            post_non_compute_budget_ix_count: 1,
            has_top_level_memo_ix_anywhere: true,
            has_top_level_ata_create_ix_anywhere: true,
            has_top_level_close_account_anywhere: true,
            has_top_level_temp_token_account_create_and_close_same_tx: true,
            has_jito_tip_transfer_top_level: true,
            jito_tip_lamports_max_top_level: 1_000,
            jito_tip_transfer_count_top_level: 2,
            jito_tip_position_bucket_top_level: 3,
            has_non_jito_tip_transfer_top_level: true,
            non_jito_tip_lamports_max_top_level: 2_000,
            non_jito_tip_transfer_count_top_level: 1,
            tip_service_presence_context_bucket_top_level: 3,
        };

        let features = RaydiumSwapFeatureFrame::from_observation(&observation, &tx_signals);
        let packed = features.as_array();

        assert_eq!(packed[DamFeatureSlot::AmountLog2.index()], 10);
        assert_eq!(packed[DamFeatureSlot::OtherAmountThresholdLog2.index()], 6);
        assert_eq!(packed[DamFeatureSlot::VaultInBalanceLog2.index()], 16);
        assert_eq!(packed[DamFeatureSlot::AmountVaultRatioBpsBucket.index()], 6);
        assert_eq!(
            packed[DamFeatureSlot::ThresholdVaultRatioBpsBucket.index()],
            2
        );
        assert_eq!(packed[DamFeatureSlot::AmountSpecifiedIsInput.index()], 1);
        assert_eq!(packed[DamFeatureSlot::SwapDirection.index()], 1);
        assert_eq!(packed[DamFeatureSlot::HasExplicitPriceLimit.index()], 1);
        assert_eq!(packed[DamFeatureSlot::RouteOriginBucket.index()], 1);
        assert_eq!(packed[DamFeatureSlot::RouteParseKnown.index()], 1);
        assert_eq!(
            packed[DamFeatureSlot::MultipleRouterFamiliesTouched.index()],
            1
        );
        assert_eq!(
            packed[DamFeatureSlot::HasTopLevelTempTokenAccountCreateAndCloseSameTx.index()],
            1
        );
    }

    #[test]
    fn raydium_feature_frame_keeps_intentionally_zero_slots_zero() {
        let observation = RaydiumSwapObservation {
            amount_specified: 4_096,
            other_amount_threshold: 512,
            sqrt_price_limit_x64: 42,
            is_base_input: false,
            zero_for_one: false,
            vault_in_balance: 131_072,
        };
        let tx_signals = TxnSignals {
            router_context_bucket: 7,
            route_origin_bucket: 2,
            route_parse_known: false,
            multiple_router_families_touched: true,
            has_top_level_ata_create_ix_anywhere: true,
            has_top_level_close_account_anywhere: true,
            has_top_level_temp_token_account_create_and_close_same_tx: true,
            ..TxnSignals::default()
        };

        let features = RaydiumSwapFeatureFrame::from_observation(&observation, &tx_signals);
        let packed = features.as_array();

        for slot in DAM_INTENTIONALLY_ZERO_SLOTS {
            assert_eq!(packed[slot.index()], 0);
        }
    }
}
