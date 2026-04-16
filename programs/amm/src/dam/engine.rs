use anchor_lang::prelude::*;

use super::{
    DamDecision, DamFeatureSource, DamFeePolicy, DamInferenceModel, FeatureVector,
};

pub fn decide_fee_add<S, M, P, const N: usize>(
    source: &S,
    model: &M,
    policy: &P,
) -> Result<DamDecision>
where
    S: DamFeatureSource<N>,
    M: DamInferenceModel<N>,
    P: DamFeePolicy,
{
    let mut features = FeatureVector::<N>::default();
    source.write_features(&mut features);

    let inference = model.infer(&features)?;
    let risk_q16 = policy.compute_risk(inference.probability_q16);
    let fee_add = policy.compute_fee_add(risk_q16)?;

    Ok(DamDecision {
        inference,
        risk_q16,
        fee_add,
    })
}
