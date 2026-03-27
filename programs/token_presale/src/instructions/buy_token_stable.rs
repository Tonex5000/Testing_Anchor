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

pub fn buy_token_stable(
    ctx: Context<BuyTokenStable>, 
    quote_amount: u64,
    identifier: u8,
) -> Result<()> {

    msg!("Depositing presale tokens to presale {}...", identifier);
    msg!("Mint: {}", &ctx.accounts.deposit_token_mint_account.to_account_info().key());   
    msg!("From Token Address: {}", &ctx.accounts.from_associated_token_account.key());     
    msg!("To Token Address: {}", &ctx.accounts.to_associated_token_account.key()); 
    
    let presale_info = &mut ctx.accounts.presale_info;
    let deposit_token_address = ctx.accounts.deposit_token_mint_account.key();
    let cur_timestamp = u64::try_from(Clock::get()?.unix_timestamp).unwrap();
    let bump = &[presale_info.bump];

    if presale_info.start_time > cur_timestamp {
        msg!("Presale not started yet.");
        return Err(PresaleError::PresaleNotStarted.into());
    }

    if presale_info.end_time < cur_timestamp {
        msg!("Presale already ended.");
        return Err(PresaleError::PresaleEnded.into())
    }

    if deposit_token_address != presale_info.usdt_token_mint_address &&
        deposit_token_address != presale_info.usdc_token_mint_address {
            msg!("Not allowed token.");
            return Err(PresaleError::NotAllowedToken.into())
    }

    let token_amount = quote_amount * 100000000u64 / presale_info.price_per_token;

    if token_amount > presale_info.deposit_token_amount - presale_info.sold_token_amount {
        msg!("Insufficient tokens in presale");
        return Err(PresaleError::InsufficientFund.into())
    }

    presale_info.sold_token_amount = presale_info.sold_token_amount + token_amount;
    presale_info.deposit_token_amount = presale_info.deposit_token_amount - token_amount;

    if presale_info.sold_token_amount > presale_info.hardcap_amount {
        msg!("Over hardcap amount!");
        return Err(PresaleError::Overhardcap.into())
    }    

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.from_associated_token_account.to_account_info(),
                to: ctx.accounts.to_associated_token_account.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        quote_amount,
    )?;

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.from_associated_presale_token_account.to_account_info(),
                to: ctx.accounts.to_associated_presale_token_account.to_account_info(),
                authority: presale_info.to_account_info(),
            },
            &[&[PRESALE_SEED, ctx.accounts.presale_authority.key().as_ref(), [identifier].as_ref(), bump][..]],
        ),
        token_amount,
    )?;

    if deposit_token_address == presale_info.usdt_token_mint_address {
        presale_info.usdt_amount = presale_info.usdt_amount + quote_amount;
        presale_info.usdt_vault = presale_info.usdt_vault + quote_amount;
    }

    if deposit_token_address == presale_info.usdc_token_mint_address {
        presale_info.usdc_amount = presale_info.usdc_amount + quote_amount;
        presale_info.usdc_vault = presale_info.usdc_vault + quote_amount;
    }

    presale_info.usd_total = presale_info.usd_total + (quote_amount * 100u64);

    msg!("Buy Token with USDT or USDC successfully.");

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    quote_amount: u64,
    identifier: u8,
)]
pub struct BuyTokenStable<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    pub presale_authority: SystemAccount<'info>,

    #[account(mut)]
    pub deposit_token_mint_account: Account<'info, token::Mint>,

    #[account(mut)]
    pub presale_token_mint_account: Account<'info, token::Mint>,

    #[account(
        mut,
        associated_token::mint = deposit_token_mint_account,
        associated_token::authority = buyer,
    )]
    pub from_associated_token_account: Account<'info, token::TokenAccount>,

    #[account(
        mut,
        associated_token::mint = deposit_token_mint_account,
        associated_token::authority = presale_info,
    )]
    pub to_associated_token_account: Account<'info, token::TokenAccount>,

    #[account(
        mut,
        seeds = [PRESALE_SEED, presale_authority.key().as_ref(), [identifier].as_ref()],
        bump = presale_info.bump
    )]
    pub presale_info: Box<Account<'info, PresaleInfo>>,

    #[account(
        mut,
        associated_token::mint = presale_token_mint_account,
        associated_token::authority = presale_info,
    )]
    pub from_associated_presale_token_account: Account<'info, token::TokenAccount>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = presale_token_mint_account,
        associated_token::authority = buyer,
    )]
    pub to_associated_presale_token_account: Account<'info, token::TokenAccount>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}