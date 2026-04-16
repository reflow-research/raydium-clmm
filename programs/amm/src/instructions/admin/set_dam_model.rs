use anchor_lang::prelude::*;

use crate::{dam::DamModelWeights, error::ErrorCode};

#[derive(Accounts)]
pub struct SetDamModel<'info> {
    #[account(address = crate::admin::ID @ ErrorCode::NotApproved)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub dam_model: Account<'info, DamModelWeights>,
}

pub fn set_dam_model(
    ctx: Context<SetDamModel>,
    n_features: u8,
    w_scale_q16: i32,
    bias_i32: i32,
    weights: Vec<i8>,
) -> Result<()> {
    ctx.accounts
        .dam_model
        .update(n_features, w_scale_q16, bias_i32, &weights)
}
