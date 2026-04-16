use anchor_lang::prelude::*;

use crate::{
    dam::{combine_trade_fee_rate, DamPoolConfig, DAM_POOL_CONFIG_SEED},
    error::ErrorCode,
    states::{AmmConfig, PoolState},
};

#[derive(Accounts)]
pub struct SetDamPoolConfig<'info> {
    #[account(address = crate::admin::ID @ ErrorCode::NotApproved)]
    pub owner: Signer<'info>,

    #[account(address = pool_state.load()?.amm_config)]
    pub amm_config: Box<Account<'info, AmmConfig>>,

    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    #[account(
        mut,
        seeds = [DAM_POOL_CONFIG_SEED, pool_state.key().as_ref()],
        bump = dam_pool_config.bump[0],
        has_one = pool
    )]
    pub dam_pool_config: Account<'info, DamPoolConfig>,

    /// CHECK: validated by `has_one = pool`
    #[account(address = pool_state.key())]
    pub pool: UncheckedAccount<'info>,
}

#[allow(clippy::too_many_arguments)]
pub fn set_dam_pool_config(
    ctx: Context<SetDamPoolConfig>,
    enabled: bool,
    flags: u8,
    fee_add_cap: u32,
    risk_threshold_q16: u16,
    model_pubkey: Pubkey,
) -> Result<()> {
    let _ = combine_trade_fee_rate(ctx.accounts.amm_config.trade_fee_rate, fee_add_cap)?;

    ctx.accounts.dam_pool_config.update(
        enabled,
        flags,
        fee_add_cap,
        risk_threshold_q16,
        model_pubkey,
    )?;

    ctx.accounts.pool_state.load_mut()?.set_dam_required(enabled);
    Ok(())
}
