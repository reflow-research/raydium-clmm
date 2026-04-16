use anchor_lang::prelude::*;

use crate::dam::DamModelWeights;

#[derive(Accounts)]
pub struct InitializeDamModel<'info> {
    #[account(mut, address = crate::admin::ID @ crate::error::ErrorCode::NotApproved)]
    pub owner: Signer<'info>,

    #[account(init, payer = owner, space = DamModelWeights::LEN)]
    pub dam_model: Account<'info, DamModelWeights>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_dam_model(
    ctx: Context<InitializeDamModel>,
    version: u8,
    n_features: u8,
    w_scale_q16: i32,
    bias_i32: i32,
    weights: Vec<i8>,
) -> Result<()> {
    ctx.accounts
        .dam_model
        .initialize(version, n_features, w_scale_q16, bias_i32, &weights)
}
