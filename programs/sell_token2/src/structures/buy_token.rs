use super::SaleAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, Mint, transfer};
use anchor_spl::associated_token::AssociatedToken;

use super::error::ErrorCode;

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(
        mut,
        constraint = sale.is_active @ ErrorCode::SaleNotActive,
        constraint = sale.end_time > Clock::get()?.unix_timestamp @ ErrorCode::SaleEnded
    )]
    pub sale: Account<'info, SaleAccount>,
    
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(
        mut,
        constraint = buyer_token_account.owner == buyer.key(),
        constraint = buyer_token_account.mint == token_mint.key()
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = sale_token_account.owner == sale.key(),
        constraint = sale_token_account.mint == token_mint.key()
    )]
    pub sale_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> BuyToken<'info> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        // 验证销售是否还有足够的代币
        if self.sale.remaining_amount == 0 {
            msg!("No tokens left for sale.");
            return Err(ErrorCode::NoTokensLeft.into());
        }

        // 计算可购买的代币数量
        let token_amount = amount.checked_div(self.sale.price_per_token)
            .ok_or(ErrorCode::Overflow)?;
            
        if token_amount == 0 {
            msg!("Amount too small to buy any tokens.");
            return Err(ErrorCode::AmountTooSmall.into());
        }

        // 确保不会购买超过剩余数量
        let actual_token_amount = std::cmp::min(token_amount, self.sale.remaining_amount);
        let actual_amount = actual_token_amount.checked_mul(self.sale.price_per_token)
            .ok_or(ErrorCode::Overflow)?;

        // 执行代币转移
        transfer(
            self.into_transfer_to_buyer_context(),
            actual_token_amount
        )?;

        // 更新销售账户状态
        self.sale.remaining_amount = self.sale.remaining_amount.checked_sub(actual_token_amount)
            .ok_or(ErrorCode::Overflow)?;

        // 如果所有代币都已售出，关闭销售
        if self.sale.remaining_amount == 0 {
            self.sale.is_active = false;
        }

        // 记录购买事件
        msg!("Bought {} tokens for {} lamports", actual_token_amount, actual_amount);

        Ok(())
    }

    pub fn into_transfer_to_buyer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.sale_token_account.to_account_info(),
                to: self.buyer_token_account.to_account_info(),
                authority: self.sale.to_account_info(),
            },
        )
    }
} 