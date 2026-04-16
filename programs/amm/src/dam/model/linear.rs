use anchor_lang::prelude::*;

use crate::error::ErrorCode;

use super::super::{
    constants::{DAM_LOGIT_CLAMP_Q16, DAM_MODEL_MAX_W_SCALE_Q16},
    DamInference, DamInferenceModel, FeatureVector, LogitQ16, ProbabilityQ16,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearModel<const N: usize> {
    active_features: u8,
    bias_i32: i32,
    w_scale_q16: i32,
    weights_i8: [i8; N],
}

impl<const N: usize> LinearModel<N> {
    pub fn new(
        active_features: u8,
        w_scale_q16: i32,
        bias_i32: i32,
        weights_i8: [i8; N],
    ) -> Result<Self> {
        require!(
            usize::from(active_features) > 0 && usize::from(active_features) <= N,
            ErrorCode::DamInvalidModel
        );
        require!(
            w_scale_q16 > 0 && w_scale_q16 <= DAM_MODEL_MAX_W_SCALE_Q16,
            ErrorCode::DamInvalidModel
        );

        Ok(Self {
            active_features,
            bias_i32,
            w_scale_q16,
            weights_i8,
        })
    }

    fn compute_linear_logit_q16(&self, features: &FeatureVector<N>) -> (i64, i32) {
        let feature_count = usize::from(self.active_features);
        let mut dot: i64 = 0;

        for i in 0..feature_count {
            dot += i64::from(self.weights_i8[i]) * i64::from(features.as_array()[i]);
        }

        let logit_q16_i128 =
            i128::from(self.bias_i32) + i128::from(self.w_scale_q16) * i128::from(dot);
        let logit_q16 = clamp_logit_q16(logit_q16_i128);

        (dot, logit_q16)
    }
}

impl<const N: usize> DamInferenceModel<N> for LinearModel<N> {
    fn infer(&self, features: &FeatureVector<N>) -> Result<DamInference> {
        let (dot_i64, logit_q16) = self.compute_linear_logit_q16(features);
        let dot_i32 = dot_i64.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
        let probability_q16 = sigmoid_approx_q16(logit_q16);

        Ok(DamInference {
            dot_i32,
            logit_q16: LogitQ16::new(logit_q16),
            probability_q16: ProbabilityQ16::new(probability_q16),
        })
    }
}

fn clamp_logit_q16(logit_q16_i128: i128) -> i32 {
    logit_q16_i128.clamp(
        i128::from(-DAM_LOGIT_CLAMP_Q16),
        i128::from(DAM_LOGIT_CLAMP_Q16),
    ) as i32
}

fn sigmoid_approx_q16(logit_q16: i32) -> u16 {
    let x = i64::from(logit_q16.clamp(-DAM_LOGIT_CLAMP_Q16, DAM_LOGIT_CLAMP_Q16));
    let abs_x = x.unsigned_abs() as i64;
    let denominator = 65_536i64 + abs_x;
    let signed_component = (x * 32_768i64) / denominator;
    let q16 = (32_768i64 + signed_component).clamp(0, 65_535);
    q16 as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::dam::constants::DAM_FEATURE_COUNT;

    #[test]
    fn linear_model_matches_whirlpool_golden_vector() {
        let mut weights = [0i8; DAM_FEATURE_COUNT];
        weights[0] = 3;
        weights[1] = -2;
        weights[2] = 5;

        let model = LinearModel::new(DAM_FEATURE_COUNT as u8, 2_048, 10_000, weights).unwrap();

        let mut features = [0i8; DAM_FEATURE_COUNT];
        features[0] = 7;
        features[1] = -4;
        features[2] = 2;

        let inference = model
            .infer(&FeatureVector::from_array(features))
            .unwrap();

        assert_eq!(inference.dot_i32, 39);
        assert_eq!(inference.logit_q16.get(), 89_872);
        assert_eq!(inference.probability_q16.get(), 51_717);
    }
}
