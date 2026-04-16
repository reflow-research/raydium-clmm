use anchor_lang::prelude::*;

use crate::{
    dam::{combine_trade_fee_rate, DamPoolConfig, DAM_POOL_CONFIG_SEED},
    states::{AmmConfig, PoolState},
};

#[derive(Accounts)]
pub struct InitializeDamPoolConfig<'info> {
    #[account(mut, address = crate::admin::ID @ crate::error::ErrorCode::NotApproved)]
    pub owner: Signer<'info>,

    #[account(address = pool_state.load()?.amm_config)]
    pub amm_config: Box<Account<'info, AmmConfig>>,

    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    #[account(
        init,
        payer = owner,
        seeds = [DAM_POOL_CONFIG_SEED, pool_state.key().as_ref()],
        bump,
        space = DamPoolConfig::LEN
    )]
    pub dam_pool_config: Account<'info, DamPoolConfig>,

    pub system_program: Program<'info, System>,
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_dam_pool_config(
    ctx: Context<InitializeDamPoolConfig>,
    enabled: bool,
    flags: u8,
    fee_add_cap: u32,
    risk_threshold_q16: u16,
    model_pubkey: Pubkey,
) -> Result<()> {
    let _ = combine_trade_fee_rate(ctx.accounts.amm_config.trade_fee_rate, fee_add_cap)?;

    ctx.accounts.dam_pool_config.initialize(
        ctx.accounts.pool_state.key(),
        ctx.bumps.dam_pool_config,
        enabled,
        flags,
        fee_add_cap,
        risk_threshold_q16,
        model_pubkey,
    )?;

    ctx.accounts.pool_state.load_mut()?.set_dam_required(enabled);
    Ok(())
}
