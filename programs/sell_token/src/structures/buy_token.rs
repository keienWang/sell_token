use super::SaleAccount;
use super::UserPurchase;
use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Token,Mint,TokenAccount},
    token_interface::{ TokenInterface,Transfer,transfer},
};

use raydium_cp_swap::{
    cpi,
    program::RaydiumCpSwap,
    states::{AmmConfig, OBSERVATION_SEED, POOL_LP_MINT_SEED, POOL_SEED, POOL_VAULT_SEED},
};

use super::error::ErrorCode;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct BuyToken<'info> {
    #[account(
        mut,
        seeds = [crate::TOKEN_SEED, token_mint.key().as_ref()],
        bump
    )]
    pub sale: Account<'info, SaleAccount>,
/// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        mut,
        seeds = [crate::TOKEN_SEED], 
        bump,
    )]
    pub pda_account: AccountInfo<'info>, //合约pda账户
    
    #[account(
        mut,
        constraint = token_mint.key() == sale.token_mint,
    )]
    pub token_mint: Box<Account<'info, Mint>>,
    
    #[account(
        mut,
        constraint = buy_token_mint.key() == sale.buy_token_mint,
    )]
    pub buy_token_mint: Box<Account<'info, Mint>>,
    
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(
        mut,
        token::mint = buy_token_mint,
        token::authority = buyer,
    )]
    pub buyer_token_account: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        token::mint = buy_token_mint,
        token::authority = pda_account,
    )]
    pub sale_token_account: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        constraint = sale_sell_token_account.owner == pda_account.key(),
        constraint = sale_sell_token_account.mint == token_mint.key()
    )]
    pub sale_sell_token_account: Account<'info, TokenAccount>,

    // Raydium accounts
    pub cp_swap_program: Program<'info, RaydiumCpSwap>,
    
    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// CHECK: Authority is a PDA owned by Raydium
    #[account(
        seeds = [
            raydium_cp_swap::AUTH_SEED.as_bytes(),
        ],
        seeds::program = cp_swap_program.key(),
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Pool state is initialized by Raydium
    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_mint.key().as_ref(),
            buy_token_mint.key().as_ref(),
        ],
        seeds::program = cp_swap_program.key(),
        bump,
    )]
    pub pool_state: UncheckedAccount<'info>,

    /// CHECK: LP mint is initialized by Raydium
    #[account(
        mut,
        seeds = [
            POOL_LP_MINT_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        seeds::program = cp_swap_program.key(),
        bump,
    )]
    pub lp_mint: UncheckedAccount<'info>,

    /// CHECK: Creator LP token account is initialized by Raydium
    #[account(mut)]
    pub creator_lp_token: UncheckedAccount<'info>,

    /// CHECK: Token vaults are initialized by Raydium
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_mint.key().as_ref()
        ],
        seeds::program = cp_swap_program.key(),
        bump,
    )]
    pub token_0_vault: UncheckedAccount<'info>,

     /// CHECK: Token vaults are initialized by Raydium
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            buy_token_mint.key().as_ref()
        ],
        seeds::program = cp_swap_program.key(),
        bump,
    )]
    pub token_1_vault: UncheckedAccount<'info>,

    #[account(
        mut,
        address= raydium_cp_swap::create_pool_fee_reveiver::ID,
    )]
    pub create_pool_fee: Box<Account<'info, TokenAccount>>,

    /// CHECK: Observation state is initialized by Raydium
    #[account(
        mut,
        seeds = [
            OBSERVATION_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        seeds::program = cp_swap_program.key(),
        bump,
    )]
    pub observation_state: UncheckedAccount<'info>,

    #[account(
        init,
        payer = buyer,
        space = 8 + core::mem::size_of::<UserPurchase>(),
        seeds = [crate::TOKEN_PURCHASE, buyer.key().as_ref(), token_mint.key().as_ref()],
        bump
    )]
    pub user_purchase: Account<'info, UserPurchase>,
    pub token_program: Program<'info, Token>,
    /// Spl token program or token program 2022
    pub token_0_program: Interface<'info, TokenInterface>,
    /// Spl token program or token program 2022
    pub token_1_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> BuyToken<'info> {
    pub fn process(&mut self, amount: u64, bump_seed: u8, open_time: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        if current_time > self.sale.end_time {
            return Err(ErrorCode::SaleEnded.into());
        }

        if self.sale.remaining_amount == 0 {
            return Err(ErrorCode::NoTokensLeft.into());
        }
        
        let token_amount = amount.checked_div(self.sale.price_per_token)
            .ok_or(ErrorCode::Overflow)?;
            
        if token_amount == 0 {
            return Err(ErrorCode::AmountTooSmall.into());
        }

        let decimals = 10u128
            .checked_pow(self.token_mint.decimals.into())
            .ok_or(ErrorCode::Overflow)?;
    
        let token_amount_u128 = (token_amount as u128)
            .checked_mul(decimals)
            .ok_or(ErrorCode::Overflow)?;
    
        let actual_token_amount = std::cmp::min(token_amount_u128, self.sale.remaining_amount as u128);
    
        let amount = actual_token_amount
            .checked_mul(self.sale.price_per_token as u128)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(decimals)
            .ok_or(ErrorCode::Overflow)?;

        let actual_amount = u64::try_from(amount)
            .map_err(|_| ErrorCode::Overflow)?;

        transfer(
            self.into_transfer_to_buyer_context(),
            actual_amount
        )?;

        self.sale.remaining_amount = self.sale.remaining_amount
            .checked_sub(actual_token_amount as u64)
            .ok_or(ErrorCode::Overflow)?;

        if self.sale.remaining_amount == 0 {
            self.sale.is_active = false;

            // Add liquidity to Raydium
            //amount_0 为代币总量减去sale_amount
            
            let amount_0 = self.token_mint.supply
                .checked_sub(self.sale.sale_amount as u64)
                .ok_or(ErrorCode::Overflow)?;
                
            let amount_1 = self.sale_token_account.amount as u64;

            let cpi_accounts = cpi::accounts::Initialize {
                creator: self.pda_account.to_account_info(),
                amm_config: self.amm_config.to_account_info(),
                authority: self.authority.to_account_info(),
                pool_state: self.pool_state.to_account_info(),
                token_0_mint: self.token_mint.to_account_info(),
                token_1_mint: self.buy_token_mint.to_account_info(),
                lp_mint: self.lp_mint.to_account_info(),
                creator_token_0: self.sale_sell_token_account.to_account_info(),
                creator_token_1: self.sale_token_account.to_account_info(),
                creator_lp_token: self.creator_lp_token.to_account_info(),
                token_0_vault: self.token_0_vault.to_account_info(),
                token_1_vault: self.token_1_vault.to_account_info(),
                create_pool_fee: self.create_pool_fee.to_account_info(),
                observation_state: self.observation_state.to_account_info(),
                token_program: self.token_program.to_account_info(),
                token_0_program: self.token_0_program.to_account_info(),
                token_1_program: self.token_1_program.to_account_info(),
                associated_token_program: self.associated_token_program.to_account_info(),
                system_program: self.system_program.to_account_info(),
                rent: self.rent.to_account_info(),
            };

            let signer_seeds: &[&[&[u8]]] = &[&[
                crate::TOKEN_SEED,  
                &[bump_seed]
            ]];

            let cpi_ctx = CpiContext::new_with_signer(
                self.cp_swap_program.to_account_info(),
                cpi_accounts,
                signer_seeds
            );

            cpi::initialize(cpi_ctx, amount_0, amount_1, open_time)?;
        }

        msg!("Bought {} tokens for {} lamports", actual_token_amount, actual_amount);

        if self.user_purchase.user_address == self.buyer.key() {
            return Err(ErrorCode::UserAlreadyPurchased.into());
        }   

        self.user_purchase.user_address = self.buyer.key();
        self.user_purchase.token_amount = actual_token_amount as u64;
        self.user_purchase.token_price = self.sale.price_per_token;
        self.user_purchase.token_address = self.token_mint.key();
        self.user_purchase.purchase_amount = actual_amount;
        self.user_purchase.purchase_time = current_time;
        self.user_purchase.is_claim = false;

        Ok(())
    }

    pub fn into_transfer_to_buyer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.buyer_token_account.to_account_info(),
                to: self.sale_token_account.to_account_info(),
                authority: self.buyer.to_account_info(),
            },
        )
    }
} 