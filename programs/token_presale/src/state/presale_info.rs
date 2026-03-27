use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct PresaleInfo {
    // Mint address of the presale token
    pub token_mint_address: Pubkey,
    // Mint address of the quote token
    pub sol_token_mint_address: Pubkey,
    pub usdt_token_mint_address: Pubkey,
    pub usdc_token_mint_address: Pubkey,
    // Softcap
    pub softcap_amount: u64,
    // Hardcap
    pub hardcap_amount: u64,
    // Total amount of presale tokens available in the presale
    pub deposit_token_amount: u64,
    // Total amount of presale tokens sold during the presale
    pub sold_token_amount: u64,
    // Total USD raised
    pub usd_total: u64,
    // Total amount of SOL
    pub sol_amount: u64,
    pub usdt_amount: u64,
    pub usdc_amount: u64,
    // Total amount of SOL
    pub sol_vault: u64,
    pub usdt_vault: u64,
    pub usdc_vault: u64,
    // Start time of presale
    pub start_time: u64,
    // End time of presale
    pub end_time: u64,
    // Maximum amount of presale tokens an address can purchase
    pub max_token_amount_per_address: u64,
    // Minimum amount of presale tokens an address can purchase
    pub min_token_amount_per_address: u64,
    // Round index
    pub round_index: u8,
    // Quote token per presale token
    pub price_per_token: u64,
    // Presale is buyable
    pub is_live: bool,
    // Identifier for finding the PDA
    pub identifier: u8,
    // Authority of the presale
    pub authority: Pubkey,
    pub authority: Pubkey,
    // Bump used when creating the PDA
    pub bump: u8,
    // vault
    pub vault: Pubkey,
}