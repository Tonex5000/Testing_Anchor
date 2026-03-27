use {
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{
        token,
        associated_token,
    },
};

use pyth_solana_receiver_sdk::price_update::get_feed_id_from_hex;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use solana_program::clock::Clock;

use crate::state::PresaleInfo;
use crate::constants::{PRESALE_SEED, VAULT_SEED};
use crate::errors::PresaleError;

pub fn buy_token(
    ctx: Context<BuyToken>, 
    quote_amount: u64,
    _identifier: u8,
) -> Result<()> {
    require!(quote_amount.gt(&0u64), PresaleError::InvalidAmount);

    let price_update = &mut ctx.accounts.price_update;
    let presale_info = &mut ctx.accounts.presale_info;
    let vault = &mut ctx.accounts.vault;
    let bump = &[presale_info.bump];

    let cur_timestamp = u64::try_from(Clock::get()?.unix_timestamp).unwrap();

    // get time and compare with start and end time
    if presale_info.start_time > cur_timestamp {
        msg!("Presale not started yet.");
        return Err(PresaleError::PresaleNotStarted.into());
    }

    if presale_info.end_time < cur_timestamp {
        msg!("Presale already ended.");
        return Err(PresaleError::PresaleEnded.into())
    }

    // 1-Fetch latest price
    let feed_id = get_feed_id_from_hex(
        "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
    )?;
    let price = price_update.get_price_no_older_than(&Clock::get()?, 600, &feed_id)?;
    let sol_price = price.price as u64;
    let sol_confidence = price.conf;
    // 3-Log result
    msg!("SOL/USD price: ({} +- {})", sol_price, sol_confidence);

    let token_amount = quote_amount / 1000u64 * sol_price / presale_info.price_per_token;

    if token_amount > presale_info.deposit_token_amount - presale_info.sold_token_amount {
        msg!("Insufficient tokens in presale");
        return Err(PresaleError::InsufficientFund.into())
    }

    if presale_info.min_token_amount_per_address > (quote_amount) {
        msg!("Insufficient tokens in presale");
        return Err(PresaleError::InsufficientFund.into())
    }
    
    presale_info.sold_token_amount = presale_info.sold_token_amount + token_amount;
    presale_info.deposit_token_amount = presale_info.deposit_token_amount - token_amount;
    
    if presale_info.sold_token_amount > presale_info.hardcap_amount {
        msg!("Over hardcap amount!");
        return Err(PresaleError::Overhardcap.into())
    }
    
    presale_info.sol_amount = presale_info.sol_amount + quote_amount;
    presale_info.sol_vault = presale_info.sol_vault + quote_amount;

    presale_info.usd_total = presale_info.usd_total + (quote_amount * sol_price / 1000000000u64);

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: vault.to_account_info(),
            })
        , quote_amount
    )?;

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.from_associated_presale_token_account.to_account_info(),
                to: ctx.accounts.to_associated_presale_token_account.to_account_info(),
                authority: ctx.accounts.presale_info.to_account_info(),
            },
            &[&[PRESALE_SEED, ctx.accounts.presale_authority.key().as_ref(), [_identifier].as_ref(), bump][..]],
        ),
        token_amount,
    )?;

    msg!("Buy Token with SOL successfully.");

    Ok(())
}


#[derive(Accounts)]
#[instruction(    
    quote_amount: u64,
    identifier: u8,
)]
pub struct BuyToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub presale_token_mint_account: Account<'info, token::Mint>,

    pub price_update: Account<'info, PriceUpdateV2>,

    #[account(
        mut,
        seeds = [PRESALE_SEED, presale_authority.key().as_ref(), [identifier].as_ref()],
        bump = presale_info.bump
    )]
    pub presale_info: Box<Account<'info, PresaleInfo>>,
    
    pub presale_authority: SystemAccount<'info>,

    #[account( 
        mut,
        seeds = [VAULT_SEED, presale_info.key().as_ref()],
        bump
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault: AccountInfo<'info>,

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
    /// CHECK: This is not dangerous because this is provided from pyth network team.
    pub pyth_sol_account: AccountInfo<'info>
}