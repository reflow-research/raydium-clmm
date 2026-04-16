use anchor_lang::prelude::*;

use crate::error::ErrorCode;

use super::super::{
    constants::DAM_MAX_RISK_THRESHOLD_Q16,
    DamFeePolicy, FeeRateU32, ProbabilityQ16, RiskScoreQ16,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThresholdFeePolicy {
    fee_add_cap: FeeRateU32,
    risk_threshold_q16: u16,
}

impl ThresholdFeePolicy {
    pub fn new(fee_add_cap: FeeRateU32, risk_threshold_q16: u16) -> Result<Self> {
        require!(
            risk_threshold_q16 < u16::MAX && risk_threshold_q16 <= DAM_MAX_RISK_THRESHOLD_Q16,
            ErrorCode::DamInvalidConfig
        );

        Ok(Self {
            fee_add_cap,
            risk_threshold_q16,
        })
    }

    pub const fn fee_add_cap(self) -> FeeRateU32 {
        self.fee_add_cap
    }

    pub const fn risk_threshold_q16(self) -> u16 {
        self.risk_threshold_q16
    }
}

impl DamFeePolicy for ThresholdFeePolicy {
    fn compute_risk(&self, probability_q16: ProbabilityQ16) -> RiskScoreQ16 {
        let probability_q16 = probability_q16.get();
        if probability_q16 <= self.risk_threshold_q16 {
            return RiskScoreQ16::new(0);
        }

        let denominator = (u16::MAX - self.risk_threshold_q16).max(1);
        let scaled = u32::from(probability_q16 - self.risk_threshold_q16) * u32::from(u16::MAX)
            / u32::from(denominator);

        RiskScoreQ16::new(scaled.min(u32::from(u16::MAX)) as u16)
    }

    fn compute_fee_add(&self, risk_q16: RiskScoreQ16) -> Result<FeeRateU32> {
        let fee_add = u128::from(self.fee_add_cap.get()) * u128::from(risk_q16.get())
            / u128::from(u16::MAX);

        FeeRateU32::new(fee_add as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_policy_matches_whirlpool_golden_values() {
        let policy = ThresholdFeePolicy::new(FeeRateU32::new(30_000).unwrap(), 40_000).unwrap();

        let risk = policy.compute_risk(ProbabilityQ16::new(51_717));
        let fee_add = policy.compute_fee_add(risk).unwrap();

        assert_eq!(risk.get(), 30_071);
        assert_eq!(fee_add.get(), 13_765);
    }
}
