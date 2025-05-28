use super::SaleAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, Mint, transfer};
use anchor_spl::associated_token::AssociatedToken;
use super::error::ErrorCode;

#[derive(Accounts)]
pub struct InitSaleAccount<'info> {
    #[account(
        init,
        payer = owner,
        space = 8+core::mem::size_of::<SaleAccount>(),
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
    
    
    pub token_mint: Account<'info, Mint>,
    pub buy_token_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        constraint = owner_token_account.owner == owner.key(),
        constraint = owner_token_account.mint == token_mint.key()
    )]
    pub owner_token_account: Account<'info, TokenAccount>,
    
    
    #[account(
        init,
        payer = owner,
        associated_token::mint = token_mint,
        associated_token::authority = pda_account
    )]
    pub sale_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> InitSaleAccount<'info> {
    pub fn process(&mut self, sale_amount: u64, price_per_token: u64, end_time: i64) -> Result<()> {
        // 验证销售数量
        if sale_amount < self.token_mint.supply / 5 {
            msg!("Sale amount is too low.");
            return Err(ErrorCode::SaleAmountTooLow.into());
        }

        // 验证销售数量不能超过代币总量的80%
        if sale_amount > self.token_mint.supply {
            msg!("Sale amount is too high.");
            return Err(ErrorCode::SaleAmountTooHigh.into());
        }

        // 检查代币余额是否足够
        let owner_balance = self.owner_token_account.amount;
        if owner_balance < self.token_mint.supply {
            msg!("Insufficient token balance.");
            return Err(ErrorCode::InsufficientBalance.into());
        }

        // 验证价格
        if price_per_token == 0 {
            msg!("Price per token cannot be zero.");
            return Err(ErrorCode::InvalidPrice.into());
        }

        // 验证结束时间
        let current_time = Clock::get()?.unix_timestamp;
        if end_time <= current_time {
            msg!("End time must be in the future.");
            return Err(ErrorCode::InvalidEndTime.into());
        }

        msg!("self.token_mint.supply {}",self.token_mint.supply);
        // 划转token
        transfer(
            self.into_transfer_to_vault_context(),
            
            //代币总量转入
            self.token_mint.supply
        )?;

        let sale = &mut self.sale;
        sale.owner = self.owner.key();
        sale.token_mint = self.token_mint.key();
        sale.sale_amount = sale_amount;
        sale.remaining_amount = sale_amount;
        sale.price_per_token = price_per_token;
        sale.end_time = end_time;
        sale.is_active = true;
        sale.buy_token_mint = self.buy_token_mint.key();

        Ok(())
    }

    pub fn into_transfer_to_vault_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.owner_token_account.to_account_info(),
                to: self.sale_token_account.to_account_info(),
                authority: self.owner.to_account_info(),
            },
        )
    }
}
