use anchor_lang::prelude::*;

use crate::error::ErrorCode;

use super::{
    constants::{
        DAM_BASE_FEATURE_COUNT, DAM_CONFIG_FLAG_EMIT_RETURN_DATA, DAM_FEATURE_COUNT,
        DAM_MAX_RISK_THRESHOLD_Q16, DAM_MODEL_EXPECTED_VERSION, DAM_MODEL_MAX_FEATURES,
        DAM_MODEL_MAX_W_SCALE_Q16,
    },
    model::LinearModel,
    policy::ThresholdFeePolicy,
    FeeRateU32,
};

#[account]
#[derive(Debug)]
pub struct DamPoolConfig {
    pub pool: Pubkey,
    pub bump: [u8; 1],

    pub enabled: bool,
    pub flags: u8,

    pub padding: [u8; 5],

    pub fee_add_cap: u32,
    pub risk_threshold_q16: u16,
    pub reserved0: [u8; 2],

    pub model_pubkey: Pubkey,

    pub reserved: [u8; 64],
}

impl DamPoolConfig {
    pub const LEN: usize = 8 + 32 + 1 + 1 + 1 + 5 + 4 + 2 + 2 + 32 + 64;

    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        pool: Pubkey,
        bump: u8,
        enabled: bool,
        flags: u8,
        fee_add_cap: u32,
        risk_threshold_q16: u16,
        model_pubkey: Pubkey,
    ) -> Result<()> {
        self.pool = pool;
        self.bump = [bump];
        self.padding = [0; 5];
        self.reserved0 = [0; 2];
        self.reserved = [0; 64];
        self.update(enabled, flags, fee_add_cap, risk_threshold_q16, model_pubkey)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        enabled: bool,
        flags: u8,
        fee_add_cap: u32,
        risk_threshold_q16: u16,
        model_pubkey: Pubkey,
    ) -> Result<()> {
        let _ = FeeRateU32::new(fee_add_cap)?;
        require!(
            risk_threshold_q16 < u16::MAX && risk_threshold_q16 <= DAM_MAX_RISK_THRESHOLD_Q16,
            ErrorCode::DamInvalidConfig
        );
        require!(model_pubkey != Pubkey::default(), ErrorCode::DamInvalidConfig);

        self.enabled = enabled;
        self.flags = flags;
        self.fee_add_cap = fee_add_cap;
        self.risk_threshold_q16 = risk_threshold_q16;
        self.reserved0 = [0; 2];
        self.model_pubkey = model_pubkey;
        Ok(())
    }

    pub fn should_emit_return_data(&self) -> bool {
        self.flags & DAM_CONFIG_FLAG_EMIT_RETURN_DATA != 0
    }

    pub fn fee_policy(&self) -> Result<ThresholdFeePolicy> {
        ThresholdFeePolicy::new(FeeRateU32::new(self.fee_add_cap)?, self.risk_threshold_q16)
    }
}

#[account]
#[derive(Debug)]
pub struct DamModelWeights {
    pub version: u8,
    pub n_features: u8,
    pub padding: [u8; 2],

    pub bias_i32: i32,
    pub w_scale_q16: i32,
    pub weights_i8: [i8; DAM_MODEL_MAX_FEATURES],

    pub reserved: [u8; 64],
}

impl DamModelWeights {
    pub const LEN: usize = 8 + 1 + 1 + 2 + 4 + 4 + DAM_MODEL_MAX_FEATURES + 64;

    pub fn initialize(
        &mut self,
        version: u8,
        n_features: u8,
        w_scale_q16: i32,
        bias_i32: i32,
        weights: &[i8],
    ) -> Result<()> {
        require!(
            version == DAM_MODEL_EXPECTED_VERSION,
            ErrorCode::DamInvalidModel
        );
        self.version = version;
        self.padding = [0; 2];
        self.reserved = [0; 64];
        self.update(n_features, w_scale_q16, bias_i32, weights)
    }

    pub fn update(
        &mut self,
        n_features: u8,
        w_scale_q16: i32,
        bias_i32: i32,
        weights: &[i8],
    ) -> Result<()> {
        require!(
            self.version == DAM_MODEL_EXPECTED_VERSION,
            ErrorCode::DamInvalidModel
        );

        let feature_count = usize::from(n_features);
        require!(
            feature_count >= DAM_BASE_FEATURE_COUNT && feature_count <= DAM_FEATURE_COUNT,
            ErrorCode::DamInvalidModel
        );
        require!(
            weights.len() == feature_count && feature_count <= DAM_MODEL_MAX_FEATURES,
            ErrorCode::DamInvalidModel
        );
        require!(
            w_scale_q16 > 0 && w_scale_q16 <= DAM_MODEL_MAX_W_SCALE_Q16,
            ErrorCode::DamInvalidModel
        );

        self.n_features = n_features;
        self.padding = [0; 2];
        self.bias_i32 = bias_i32;
        self.w_scale_q16 = w_scale_q16;
        self.weights_i8 = [0; DAM_MODEL_MAX_FEATURES];
        self.weights_i8[..feature_count].copy_from_slice(weights);
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        require!(
            self.version == DAM_MODEL_EXPECTED_VERSION,
            ErrorCode::DamInvalidModel
        );

        let feature_count = usize::from(self.n_features);
        require!(
            feature_count >= DAM_BASE_FEATURE_COUNT && feature_count <= DAM_FEATURE_COUNT,
            ErrorCode::DamInvalidModel
        );
        require!(
            self.w_scale_q16 > 0 && self.w_scale_q16 <= DAM_MODEL_MAX_W_SCALE_Q16,
            ErrorCode::DamInvalidModel
        );
        Ok(())
    }

    pub fn linear_model(&self) -> Result<LinearModel<DAM_FEATURE_COUNT>> {
        self.validate()?;

        let mut weights = [0i8; DAM_FEATURE_COUNT];
        weights.copy_from_slice(&self.weights_i8[..DAM_FEATURE_COUNT]);

        LinearModel::new(self.n_features, self.w_scale_q16, self.bias_i32, weights)
    }
}
