use {
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{associated_token, token},
};

use crate::constants::PRESALE_SEED;
use crate::constants::VAULT_SEED;
use crate::errors::PresaleError;
use crate::state::PresaleInfo;

pub fn withdraw_sol(ctx: Context<WithdrawSol>, _identifier: u8) -> Result<()> {
    let presale_info = &mut ctx.accounts.presale_info;
    let bump = ctx.bumps.vault;
    let presale_key = presale_info.key();
    let vault_seed: &[&[&[u8]]] = &[&[VAULT_SEED, presale_key.as_ref(), &[bump]]];
    let cur_timestamp = u64::try_from(Clock::get()?.unix_timestamp).unwrap();

        require!(
            presale_info.authority == ctx.accounts.buyer.key(),
            PresaleError::Unauthorized
        );
        require!(
            cur_timestamp > presale_info.end_time,
            PresaleError::NotEndedYet
        );

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.buyer.to_account_info(),
                },
                vault_seed,
            ),
            presale_info.sol_amount,
        )?;
        presale_info.sol_amount = 0;
    msg!("Withdraw Sol: {}", presale_info.sol_amount);
    msg!("Withdraw sol successfully.");

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    identifier: u8
)]
pub struct WithdrawSol<'info> {
    #[account(
        mut,
        seeds = [PRESALE_SEED, presale_authority.key().as_ref(), [identifier].as_ref()],
        bump = presale_info.bump
    )]
    pub presale_info: Box<Account<'info, PresaleInfo>>,
    #[account(
        mut,
        seeds = [VAULT_SEED, presale_info.key().as_ref()],
        bump
    )]
    /// CHECK:
    pub vault: AccountInfo<'info>,
    pub presale_authority: SystemAccount<'info>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}
