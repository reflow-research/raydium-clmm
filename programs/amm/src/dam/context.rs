use anchor_lang::{
    prelude::*,
    solana_program::{program::set_return_data, sysvar::instructions},
};

use crate::{
    error::ErrorCode,
    states::config::FEE_RATE_DENOMINATOR_VALUE,
};

use super::{
    accounts::{DamModelWeights, DamPoolConfig},
    constants::{DAM_CONFIG_FLAG_EMIT_RETURN_DATA, DAM_POOL_CONFIG_SEED},
    engine::decide_fee_add,
    features::{parse_txn_signals, RaydiumSwapObservation, TxnSignals},
    DamDecision, DamSwapReturnData, FeeRateU32, RaydiumSwapFeatureFrame,
};

#[derive(Clone, Debug)]
pub struct DamRuntime<'info> {
    pub config: DamPoolConfig,
    pub model: DamModelWeights,
    pub instructions_sysvar: Option<AccountInfo<'info>>,
}

impl<'info> DamRuntime<'info> {
    pub fn parse_optional(
        remaining_accounts: &'info [AccountInfo<'info>],
        pool_key: Pubkey,
    ) -> Result<Option<Self>> {
        if remaining_accounts.is_empty() {
            return Ok(None);
        }

        require!(
            remaining_accounts.len() == 2 || remaining_accounts.len() == 3,
            ErrorCode::DamInvalidRemainingAccounts
        );

        let config_info = &remaining_accounts[0];
        let model_info = &remaining_accounts[1];
        let instructions_sysvar = remaining_accounts.get(2).cloned();

        let (expected_config_key, expected_bump) =
            Pubkey::find_program_address(&[DAM_POOL_CONFIG_SEED, pool_key.as_ref()], &crate::id());
        require_keys_eq!(
            config_info.key(),
            expected_config_key,
            ErrorCode::DamInvalidRemainingAccounts
        );

        let config = Account::<DamPoolConfig>::try_from(config_info)?.into_inner();
        let model = Account::<DamModelWeights>::try_from(model_info)?.into_inner();

        require_keys_eq!(config.pool, pool_key, ErrorCode::DamInvalidRemainingAccounts);
        require!(
            config.bump == [expected_bump],
            ErrorCode::DamInvalidRemainingAccounts
        );
        require_keys_eq!(
            config.model_pubkey,
            model_info.key(),
            ErrorCode::DamInvalidRemainingAccounts
        );
        if let Some(ix_sysvar) = instructions_sysvar.as_ref() {
            require_keys_eq!(
                ix_sysvar.key(),
                instructions::ID,
                ErrorCode::DamInvalidRemainingAccounts
            );
        }

        model.validate()?;
        let _ = FeeRateU32::new(config.fee_add_cap)?;

        Ok(Some(Self {
            config,
            model,
            instructions_sysvar,
        }))
    }

    pub fn txn_signals(&self, current_program_id: &Pubkey) -> Result<TxnSignals> {
        parse_txn_signals(self.instructions_sysvar.as_ref(), current_program_id)
    }

    pub fn decide(
        &self,
        observation: &RaydiumSwapObservation,
        current_program_id: &Pubkey,
    ) -> Result<DamDecision> {
        if !self.config.enabled {
            return Ok(DamDecision::default());
        }

        require!(
            self.instructions_sysvar.is_some(),
            ErrorCode::DamInvalidRemainingAccounts
        );

        let tx_signals = self.txn_signals(current_program_id)?;
        let feature_frame = RaydiumSwapFeatureFrame::from_observation(observation, &tx_signals);
        let model = self.model.linear_model()?;
        let policy = self.config.fee_policy()?;
        decide_fee_add(&feature_frame, &model, &policy)
    }

    pub fn decide_fee_add(
        &self,
        observation: &RaydiumSwapObservation,
        current_program_id: &Pubkey,
    ) -> Result<u32> {
        let decision = self.decide(observation, current_program_id)?;
        Ok(decision.fee_add.get())
    }

    pub fn emit_return_data_if_enabled(&self, decision: &DamDecision) -> Result<()> {
        if !self.should_emit_return_data() {
            return Ok(());
        }

        let payload = DamSwapReturnData::from_decision(decision).try_to_vec()?;
        set_return_data(&payload);
        Ok(())
    }

    pub fn should_emit_return_data(&self) -> bool {
        self.config.flags & DAM_CONFIG_FLAG_EMIT_RETURN_DATA != 0
    }
}

pub fn combine_trade_fee_rate(base_fee_rate: u32, dam_fee_add: u32) -> Result<u32> {
    let effective_fee_rate = base_fee_rate
        .checked_add(dam_fee_add)
        .ok_or(ErrorCode::DamFeeRateExceeded)?;

    require!(
        effective_fee_rate < FEE_RATE_DENOMINATOR_VALUE,
        ErrorCode::DamFeeRateExceeded
    );

    Ok(effective_fee_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_trade_fee_rate_rejects_fee_over_denominator() {
        let err = combine_trade_fee_rate(FEE_RATE_DENOMINATOR_VALUE - 10, 10).unwrap_err();
        assert!(anchor_lang::error!(ErrorCode::DamFeeRateExceeded).eq(&err));
    }

    #[test]
    fn combine_trade_fee_rate_accepts_valid_sum() {
        let combined = combine_trade_fee_rate(3_000, 13_765).unwrap();
        assert_eq!(combined, 16_765);
    }
}
