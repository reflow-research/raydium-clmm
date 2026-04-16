use anchor_lang::prelude::*;

use super::types::{DamInference, FeatureVector, FeeRateU32, ProbabilityQ16, RiskScoreQ16};

pub trait DamFeatureSource<const N: usize> {
    fn write_features(&self, output: &mut FeatureVector<N>);
}

pub trait DamInferenceModel<const N: usize> {
    fn infer(&self, features: &FeatureVector<N>) -> Result<DamInference>;
}

pub trait DamFeePolicy {
    fn compute_risk(&self, probability_q16: ProbabilityQ16) -> RiskScoreQ16;
    fn compute_fee_add(&self, risk_q16: RiskScoreQ16) -> Result<FeeRateU32>;
}
