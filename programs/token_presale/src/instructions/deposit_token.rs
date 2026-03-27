use {
    anchor_lang::prelude::*,
    anchor_spl::{
        token,
        associated_token,
    },
};

use crate::state::PresaleInfo;
use crate::constants::PRESALE_SEED;
use crate::errors::PresaleError;

pub fn deposit_token(
    ctx: Context<DepositToken>, 
    amount: u64,
    identifier: u8,
) -> Result<()> {

    msg!("Depositing presale tokens to presale {}...", identifier);
    msg!("Mint: {}", &ctx.accounts.mint_account.to_account_info().key());   
    msg!("From Token Address: {}", &ctx.accounts.from_associated_token_account.key());     
    msg!("To Token Address: {}", &ctx.accounts.to_associated_token_account.key()); 
    
    let presale_info = &mut ctx.accounts.presale_info;
    let deposit_token_address = ctx.accounts.mint_account.key();

    require!(
        deposit_token_address == presale_info.token_mint_address,
        PresaleError::NotAllowedToken
    );

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.from_associated_token_account.to_account_info(),
                to: ctx.accounts.to_associated_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount,
    )?;
    presale_info.deposit_token_amount = presale_info.deposit_token_amount + amount;
    msg!("Tokens deposited successfully.");

    return Ok(());
}

#[derive(Accounts)]
#[instruction(
    amount: u64,
    identifier: u8,
)]
pub struct DepositToken<'info> {
    #[account(mut)]
    pub mint_account: Account<'info, token::Mint>,
    #[account(
        mut,
        associated_token::mint = mint_account,
        associated_token::authority = from_authority,
    )]
    pub from_associated_token_account: Account<'info, token::TokenAccount>,
    #[account(constraint = payer.key() == from_authority.key())]
    pub from_authority: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_account,
        associated_token::authority = presale_info,
    )]
    pub to_associated_token_account: Account<'info, token::TokenAccount>,
    #[account(
        mut,
        seeds = [PRESALE_SEED, from_authority.key().as_ref(), [identifier].as_ref()],
        bump = presale_info.bump
    )]
    pub presale_info: Box<Account<'info, PresaleInfo>>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}