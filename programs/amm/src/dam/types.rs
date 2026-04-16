use anchor_lang::prelude::*;

use crate::error::ErrorCode;

use super::constants::{DAM_FEE_RATE_HARD_LIMIT, DAM_MODEL_EXPECTED_VERSION};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct FeeRateU32(u32);

impl FeeRateU32 {
    pub fn new(value: u32) -> Result<Self> {
        require!(value <= DAM_FEE_RATE_HARD_LIMIT, ErrorCode::DamInvalidConfig);
        Ok(Self(value))
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct ProbabilityQ16(u16);

impl ProbabilityQ16 {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct RiskScoreQ16(u16);

impl RiskScoreQ16 {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct LogitQ16(i32);

impl LogitQ16 {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct FeatureVector<const N: usize>([i8; N]);

impl<const N: usize> Default for FeatureVector<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N: usize> FeatureVector<N> {
    pub const fn from_array(values: [i8; N]) -> Self {
        Self(values)
    }

    pub const fn as_array(&self) -> &[i8; N] {
        &self.0
    }

    pub fn as_mut_array(&mut self) -> &mut [i8; N] {
        &mut self.0
    }

    pub fn set(&mut self, index: usize, value: i8) {
        if let Some(slot) = self.0.get_mut(index) {
            *slot = value;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamInference {
    pub dot_i32: i32,
    pub logit_q16: LogitQ16,
    pub probability_q16: ProbabilityQ16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamDecision {
    pub inference: DamInference,
    pub risk_q16: RiskScoreQ16,
    pub fee_add: FeeRateU32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamSwapReturnData {
    pub version: u8,
    pub dot_i32: i32,
    pub logit_q16: i32,
    pub probability_q16: u16,
    pub risk_q16: u16,
    pub fee_add: u32,
}

impl DamSwapReturnData {
    pub fn from_decision(decision: &DamDecision) -> Self {
        Self {
            version: DAM_MODEL_EXPECTED_VERSION,
            dot_i32: decision.inference.dot_i32,
            logit_q16: decision.inference.logit_q16.get(),
            probability_q16: decision.inference.probability_q16.get(),
            risk_q16: decision.risk_q16.get(),
            fee_add: decision.fee_add.get(),
        }
    }
}
