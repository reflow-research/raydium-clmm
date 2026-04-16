use anchor_lang::{
    prelude::*,
    solana_program::{program::set_return_data, sysvar::instructions},
};

use crate::{error::ErrorCode, states::config::FEE_RATE_DENOMINATOR_VALUE};

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

        require_keys_eq!(
            config.pool,
            pool_key,
            ErrorCode::DamInvalidRemainingAccounts
        );
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

pub fn resolve_dam_fee_add<'info>(
    dam_runtime: Option<&DamRuntime<'info>>,
    dam_required: bool,
    observation: &RaydiumSwapObservation,
    current_program_id: &Pubkey,
) -> Result<u32> {
    let Some(dam_runtime) = dam_runtime else {
        if dam_required {
            return err!(ErrorCode::DamInvalidRemainingAccounts);
        }
        return Ok(0);
    };

    if dam_required && !dam_runtime.config.enabled {
        return err!(ErrorCode::DamInvalidConfig);
    }

    let decision = dam_runtime.decide(observation, current_program_id)?;
    dam_runtime.emit_return_data_if_enabled(&decision)?;
    Ok(decision.fee_add.get())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::{AccountSerialize, Owner};

    use crate::dam::{
        DamModelWeights, DamPoolConfig, DAM_FEATURE_COUNT, DAM_MODEL_EXPECTED_VERSION,
        DAM_MODEL_STORAGE_FEATURE_CAPACITY, DAM_POOL_CONFIG_SEED,
    };

    fn build_test_observation() -> RaydiumSwapObservation {
        RaydiumSwapObservation {
            amount_specified: 1_024,
            other_amount_threshold: 64,
            sqrt_price_limit_x64: 1,
            is_base_input: true,
            zero_for_one: true,
            vault_in_balance: 65_536,
        }
    }

    fn build_test_model() -> DamModelWeights {
        let mut model = DamModelWeights {
            version: 0,
            n_features: 0,
            padding: [0; 2],
            bias_i32: 0,
            w_scale_q16: 0,
            weights_i8: [0; DAM_MODEL_STORAGE_FEATURE_CAPACITY],
            reserved: [0; 64],
        };
        let weights = vec![0i8; DAM_FEATURE_COUNT];
        model
            .initialize(
                DAM_MODEL_EXPECTED_VERSION,
                DAM_FEATURE_COUNT as u8,
                2_048,
                10_000,
                &weights,
            )
            .unwrap();
        model
    }

    fn build_test_config(
        pool_key: Pubkey,
        bump: u8,
        enabled: bool,
        model_pubkey: Pubkey,
    ) -> DamPoolConfig {
        let mut config = DamPoolConfig {
            pool: Pubkey::default(),
            bump: [0; 1],
            enabled: false,
            flags: 0,
            padding: [0; 5],
            fee_add_cap: 0,
            risk_threshold_q16: 0,
            reserved0: [0; 2],
            model_pubkey: Pubkey::default(),
            reserved: [0; 64],
        };
        config
            .initialize(pool_key, bump, enabled, 0, 30_000, 40_000, model_pubkey)
            .unwrap();
        config
    }

    fn build_anchor_account_info<T>(key: Pubkey, account: &T) -> AccountInfo<'static>
    where
        T: AccountSerialize + Owner,
    {
        let mut data = Vec::with_capacity(256);
        account.try_serialize(&mut data).unwrap();

        let key = Box::leak(Box::new(key));
        let lamports = Box::leak(Box::new(0u64));
        let owner = Box::leak(Box::new(T::owner()));
        let data = Box::leak(data.into_boxed_slice());

        AccountInfo::new(key, false, false, lamports, data, owner, false, 0)
    }

    fn build_raw_account_info(key: Pubkey) -> AccountInfo<'static> {
        let key = Box::leak(Box::new(key));
        let lamports = Box::leak(Box::new(0u64));
        let owner = Box::leak(Box::new(anchor_lang::system_program::ID));
        let data = Box::leak(Vec::<u8>::new().into_boxed_slice());

        AccountInfo::new(key, false, false, lamports, data, owner, false, 0)
    }

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

    #[test]
    fn resolve_dam_fee_add_requires_runtime_when_pool_requires_dam() {
        let err =
            resolve_dam_fee_add(None, true, &build_test_observation(), &crate::id()).unwrap_err();
        assert!(anchor_lang::error!(ErrorCode::DamInvalidRemainingAccounts).eq(&err));
    }

    #[test]
    fn resolve_dam_fee_add_rejects_disabled_runtime_on_required_pool() {
        let pool_key = Pubkey::new_unique();
        let (_config_key, bump) =
            Pubkey::find_program_address(&[DAM_POOL_CONFIG_SEED, pool_key.as_ref()], &crate::id());
        let model_key = Pubkey::new_unique();
        let runtime = DamRuntime {
            config: build_test_config(pool_key, bump, false, model_key),
            model: build_test_model(),
            instructions_sysvar: None,
        };

        let err = resolve_dam_fee_add(
            Some(&runtime),
            true,
            &build_test_observation(),
            &crate::id(),
        )
        .unwrap_err();
        assert!(anchor_lang::error!(ErrorCode::DamInvalidConfig).eq(&err));
    }

    #[test]
    fn parse_optional_rejects_wrong_model_account() {
        let pool_key = Pubkey::new_unique();
        let (config_key, bump) =
            Pubkey::find_program_address(&[DAM_POOL_CONFIG_SEED, pool_key.as_ref()], &crate::id());
        let expected_model_key = Pubkey::new_unique();
        let wrong_model_key = Pubkey::new_unique();

        let config = build_test_config(pool_key, bump, true, expected_model_key);
        let model = build_test_model();

        let accounts = Box::leak(
            vec![
                build_anchor_account_info(config_key, &config),
                build_anchor_account_info(wrong_model_key, &model),
            ]
            .into_boxed_slice(),
        );

        let err = DamRuntime::parse_optional(accounts, pool_key).unwrap_err();
        assert!(anchor_lang::error!(ErrorCode::DamInvalidRemainingAccounts).eq(&err));
    }

    #[test]
    fn parse_optional_rejects_wrong_instruction_sysvar() {
        let pool_key = Pubkey::new_unique();
        let (config_key, bump) =
            Pubkey::find_program_address(&[DAM_POOL_CONFIG_SEED, pool_key.as_ref()], &crate::id());
        let model_key = Pubkey::new_unique();

        let config = build_test_config(pool_key, bump, true, model_key);
        let model = build_test_model();

        let accounts = Box::leak(
            vec![
                build_anchor_account_info(config_key, &config),
                build_anchor_account_info(model_key, &model),
                build_raw_account_info(Pubkey::new_unique()),
            ]
            .into_boxed_slice(),
        );

        let err = DamRuntime::parse_optional(accounts, pool_key).unwrap_err();
        assert!(anchor_lang::error!(ErrorCode::DamInvalidRemainingAccounts).eq(&err));
    }
}
