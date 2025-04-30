use super::SaleAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, Mint, transfer};
use anchor_spl::associated_token::AssociatedToken;

use super::error::ErrorCode;

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    #[account(
        mut,
        constraint = sale.is_active @ ErrorCode::SaleNotActive,
        constraint = sale.owner == owner.key() @ ErrorCode::Unauthorized
    )]
    pub sale: Account<'info, SaleAccount>,
    
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        constraint = owner_token_account.owner == owner.key(),
        constraint = owner_token_account.mint == token_mint.key()
    )]
    pub owner_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = sale_token_account.owner == sale.key(),
        constraint = sale_token_account.mint == token_mint.key()
    )]
    pub sale_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> WithdrawTokens<'info> {
    pub fn process(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // 检查销售是否已结束
        if current_time < self.sale.end_time {
            msg!("Sale has not ended yet.");
            return Err(ErrorCode::SaleNotEnded.into());
        }

        // 获取销售账户中的代币余额
        let sale_balance = self.sale_token_account.amount;
        
        // 如果还有剩余代币（未卖完）
        if sale_balance > 0 {
            // 验证剩余代币数量是否与销售记录一致
            if sale_balance != self.sale.remaining_amount {
                msg!("Token balance mismatch.");
                return Err(ErrorCode::BalanceMismatch.into());
            }

            // 转移剩余代币回所有者账户
            transfer(
                self.into_transfer_to_owner_context(),
                sale_balance
            )?;

            // 更新销售账户状态
            self.sale.remaining_amount = 0;
            self.sale.is_active = false;

            msg!("Withdrew {} unsold tokens back to owner", sale_balance);
        } else {
            // 如果代币已全部售出，关闭销售
            self.sale.is_active = false;
            msg!("All tokens have been sold, sale is now closed");
        }

        Ok(())
    }

    pub fn into_transfer_to_owner_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.sale_token_account.to_account_info(),
                to: self.owner_token_account.to_account_info(),
                authority: self.sale.to_account_info(),
            },
        )
    }
}
