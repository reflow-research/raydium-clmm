use super::super::{
    constants::DAM_FEATURE_COUNT, DamFeatureSource, FeatureVector,
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
    pub fn from_observation(
        observation: &RaydiumSwapObservation,
        tx_signals: &TxnSignals,
    ) -> Self {
        let feature_12 = 0i8;
        let feature_19 = tx_signals.router_context_bucket.min(127) as i8;
        let feature_40_f12_x_f19 = mul_i8_clamped(feature_12, feature_19);

        Self {
            features: FeatureVector::from_array([
                u64_log2_bucket(observation.amount_specified),
                u64_log2_bucket(observation.other_amount_threshold),
                u64_log2_bucket(observation.vault_in_balance),
                u64_log2_bucket(u64::from(tx_signals.compute_unit_limit)),
                u64_log2_bucket(tx_signals.compute_unit_price_micro_lamports),
                ratio_bps_bucket(observation.amount_specified, observation.vault_in_balance),
                ratio_bps_bucket(
                    observation.other_amount_threshold,
                    observation.vault_in_balance,
                ),
                bool_as_i8(observation.is_base_input),
                if observation.zero_for_one { 1 } else { -1 },
                bool_as_i8(observation.sqrt_price_limit_x64 != 0),
                tx_signals.prior_top_level_ix_count.min(127) as i8,
                tx_signals.prior_non_compute_budget_ix_count.min(127) as i8,
                feature_12,
                0,
                0,
                0,
                tx_signals.prior_same_program_ix_count.min(127) as i8,
                compute_budget_signal_bucket(tx_signals),
                tx_signals.prior_known_router_ix_count.min(127) as i8,
                feature_19,
                u64_log2_bucket(u64::from(tx_signals.prior_instruction_data_bytes_sum)),
                u64_log2_bucket(u64::from(tx_signals.prior_instruction_data_bytes_max)),
                tx_signals.prior_nonempty_data_ix_count.min(127) as i8,
                tx_signals.prior_unique_first_data_byte_count.min(127) as i8,
                tx_signals.total_top_level_ix_count.min(127) as i8,
                tx_signals.post_top_level_ix_count.min(127) as i8,
                tx_signals.post_non_compute_budget_ix_count.min(127) as i8,
                bool_as_i8(tx_signals.has_memo_ix_anywhere),
                bool_as_i8(tx_signals.has_ata_create_ix_anywhere),
                bool_as_i8(tx_signals.has_jito_tip_transfer_top_level),
                u64_log2_bucket(tx_signals.jito_tip_lamports_max_top_level),
                tx_signals.jito_tip_transfer_count_top_level.min(127) as i8,
                tx_signals.jito_tip_position_bucket_top_level.min(3) as i8,
                tx_signals.cpi_stack_height.min(127) as i8,
                tx_signals.processed_sibling_ix_count.min(127) as i8,
                tx_signals.processed_sibling_unique_program_count.min(127) as i8,
                u64_log2_bucket(u64::from(tx_signals.processed_sibling_ix_data_sum)),
                bool_as_i8(tx_signals.has_jito_tip_transfer_sibling),
                u64_log2_bucket(tx_signals.jito_tip_lamports_max_sibling),
                tx_signals.jito_tip_presence_context_bucket.min(3) as i8,
                feature_40_f12_x_f19,
                bool_as_i8(tx_signals.has_non_jito_tip_transfer_top_level),
                u64_log2_bucket(tx_signals.non_jito_tip_lamports_max_top_level),
                tx_signals.non_jito_tip_transfer_count_top_level.min(127) as i8,
                tx_signals.tip_service_presence_context_bucket_top_level.min(3) as i8,
            ]),
        }
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

fn mul_i8_clamped(a: i8, b: i8) -> i8 {
    let product = i16::from(a) * i16::from(b);
    product.clamp(i16::from(i8::MIN), i16::from(i8::MAX)) as i8
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

    #[test]
    fn raydium_feature_frame_uses_whirlpool_legacy_layout() {
        let observation = RaydiumSwapObservation {
            amount_specified: 1_024,
            other_amount_threshold: 64,
            sqrt_price_limit_x64: 1,
            is_base_input: true,
            zero_for_one: true,
            vault_in_balance: 65_536,
        };

        let features =
            RaydiumSwapFeatureFrame::from_observation(&observation, &TxnSignals::default());

        assert_eq!(features.as_array()[0], 10);
        assert_eq!(features.as_array()[1], 6);
        assert_eq!(features.as_array()[2], 16);
        assert_eq!(features.as_array()[5], 6);
        assert_eq!(features.as_array()[6], 2);
        assert_eq!(features.as_array()[7], 1);
        assert_eq!(features.as_array()[8], 1);
        assert_eq!(features.as_array()[9], 1);
    }
}
