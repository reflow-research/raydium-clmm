#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DamFeatureSlot {
    AmountLog2 = 0,
    OtherAmountThresholdLog2 = 1,
    VaultInBalanceLog2 = 2,
    AmountVaultRatioBpsBucket = 3,
    ThresholdVaultRatioBpsBucket = 4,
    AmountSpecifiedIsInput = 5,
    SwapDirection = 6,
    HasExplicitPriceLimit = 7,
    ComputeUnitLimitLog2 = 8,
    ComputeUnitPriceMicroLamportsLog2 = 9,
    ComputeBudgetSignalBucket = 10,
    PriorTopLevelIxCount = 11,
    PriorNonComputeBudgetIxCount = 12,
    PriorSameProgramIxCount = 13,
    PriorInstructionDataBytesSumLog2 = 14,
    PriorInstructionDataBytesMaxLog2 = 15,
    PriorNonemptyDataIxCount = 16,
    PriorUniqueFirstDataByteCount = 17,
    TotalTopLevelIxCount = 18,
    PostTopLevelIxCount = 19,
    PostNonComputeBudgetIxCount = 20,
    HasTopLevelMemoIxAnywhere = 21,
    PriorKnownRouterIxCount = 22,
    RouterContextBucket = 23,
    RouteOriginBucket = 24,
    RouteParseKnown = 25,
    MultipleRouterFamiliesTouched = 26,
    HasJitoTipTransferTopLevel = 27,
    JitoTipLamportsMaxTopLevelLog2 = 28,
    JitoTipTransferCountTopLevel = 29,
    JitoTipPositionBucketTopLevel = 30,
    HasNonJitoTipTransferTopLevel = 31,
    NonJitoTipLamportsMaxTopLevelLog2 = 32,
    NonJitoTipTransferCountTopLevel = 33,
    TipServicePresenceContextBucketTopLevel = 34,
    HasTopLevelAtaCreateIxAnywhere = 35,
    HasTopLevelCloseAccountAnywhere = 36,
    HasTopLevelTempTokenAccountCreateAndCloseSameTx = 37,
    RouteRoundTripSameMint = 38,
    RouteRoundTripParseKnown = 39,
    ProvisionalReserved = 40,
    ReservedZero41 = 41,
    ReservedZero42 = 42,
}

impl DamFeatureSlot {
    pub const fn index(self) -> usize {
        self as usize
    }
}

pub const DAM_FEATURE_SCHEMA_ID: &str = "dam_cpi_structural_atomic_v1_abi_v1";
pub const DAM_FEATURE_COUNT: usize = 43;

pub const DAM_FEATURE_SLOT_NAMES: [&str; DAM_FEATURE_COUNT] = [
    "amount_log2",
    "other_amount_threshold_log2",
    "vault_in_balance_log2",
    "amount_vault_ratio_bps_bucket",
    "threshold_vault_ratio_bps_bucket",
    "amount_specified_is_input",
    "swap_direction",
    "has_explicit_price_limit",
    "compute_unit_limit_log2",
    "compute_unit_price_micro_lamports_log2",
    "compute_budget_signal_bucket",
    "prior_top_level_ix_count",
    "prior_non_compute_budget_ix_count",
    "prior_same_program_ix_count",
    "prior_instruction_data_bytes_sum_log2",
    "prior_instruction_data_bytes_max_log2",
    "prior_nonempty_data_ix_count",
    "prior_unique_first_data_byte_count",
    "total_top_level_ix_count",
    "post_top_level_ix_count",
    "post_non_compute_budget_ix_count",
    "has_top_level_memo_ix_anywhere",
    "prior_known_router_ix_count",
    "router_context_bucket",
    "route_origin_bucket",
    "route_parse_known",
    "multiple_router_families_touched",
    "has_jito_tip_transfer_top_level",
    "jito_tip_lamports_max_top_level_log2",
    "jito_tip_transfer_count_top_level",
    "jito_tip_position_bucket_top_level",
    "has_non_jito_tip_transfer_top_level",
    "non_jito_tip_lamports_max_top_level_log2",
    "non_jito_tip_transfer_count_top_level",
    "tip_service_presence_context_bucket_top_level",
    "has_top_level_ata_create_ix_anywhere",
    "has_top_level_close_account_anywhere",
    "has_top_level_temp_token_account_create_and_close_same_tx",
    "route_round_trip_same_mint",
    "route_round_trip_parse_known",
    "provisional_reserved",
    "reserved_zero",
    "reserved_zero",
];

pub const DAM_INTENTIONALLY_ZERO_SLOTS: [DamFeatureSlot; 5] = [
    DamFeatureSlot::RouteRoundTripSameMint,
    DamFeatureSlot::RouteRoundTripParseKnown,
    DamFeatureSlot::ProvisionalReserved,
    DamFeatureSlot::ReservedZero41,
    DamFeatureSlot::ReservedZero42,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dam_feature_schema_matches_frozen_abi() {
        assert_eq!(DAM_FEATURE_SCHEMA_ID, "dam_cpi_structural_atomic_v1_abi_v1");
        assert_eq!(DAM_FEATURE_COUNT, 43);
        assert_eq!(
            DAM_FEATURE_SLOT_NAMES,
            [
                "amount_log2",
                "other_amount_threshold_log2",
                "vault_in_balance_log2",
                "amount_vault_ratio_bps_bucket",
                "threshold_vault_ratio_bps_bucket",
                "amount_specified_is_input",
                "swap_direction",
                "has_explicit_price_limit",
                "compute_unit_limit_log2",
                "compute_unit_price_micro_lamports_log2",
                "compute_budget_signal_bucket",
                "prior_top_level_ix_count",
                "prior_non_compute_budget_ix_count",
                "prior_same_program_ix_count",
                "prior_instruction_data_bytes_sum_log2",
                "prior_instruction_data_bytes_max_log2",
                "prior_nonempty_data_ix_count",
                "prior_unique_first_data_byte_count",
                "total_top_level_ix_count",
                "post_top_level_ix_count",
                "post_non_compute_budget_ix_count",
                "has_top_level_memo_ix_anywhere",
                "prior_known_router_ix_count",
                "router_context_bucket",
                "route_origin_bucket",
                "route_parse_known",
                "multiple_router_families_touched",
                "has_jito_tip_transfer_top_level",
                "jito_tip_lamports_max_top_level_log2",
                "jito_tip_transfer_count_top_level",
                "jito_tip_position_bucket_top_level",
                "has_non_jito_tip_transfer_top_level",
                "non_jito_tip_lamports_max_top_level_log2",
                "non_jito_tip_transfer_count_top_level",
                "tip_service_presence_context_bucket_top_level",
                "has_top_level_ata_create_ix_anywhere",
                "has_top_level_close_account_anywhere",
                "has_top_level_temp_token_account_create_and_close_same_tx",
                "route_round_trip_same_mint",
                "route_round_trip_parse_known",
                "provisional_reserved",
                "reserved_zero",
                "reserved_zero",
            ]
        );
    }

    #[test]
    fn intentionally_zero_slots_match_frozen_abi() {
        let slot_indexes: Vec<usize> = DAM_INTENTIONALLY_ZERO_SLOTS
            .iter()
            .map(|slot| slot.index())
            .collect();
        assert_eq!(slot_indexes, vec![38, 39, 40, 41, 42]);
    }
}
