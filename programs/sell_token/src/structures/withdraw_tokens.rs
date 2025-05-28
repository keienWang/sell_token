use super::SaleAccount;
use super::UserPurchase;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::associated_token::AssociatedToken;

use super::error::ErrorCode;

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    #[account(
        mut,
        seeds = [crate::TOKEN_SEED, token_mint.key().as_ref()],
        bump)]
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
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = buy_token_mint.key() == sale.buy_token_mint,
    )]
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
        mut,
        constraint = sale_token_account.owner == pda_account.key(),
        constraint = sale_token_account.mint == token_mint.key()
    )]
    pub sale_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = refund_token_account.owner == owner.key(),
        constraint = refund_token_account.mint == buy_token_mint.key()
    )]
    pub refund_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = contract_token_account.owner == pda_account.key(),
        constraint = contract_token_account.mint == buy_token_mint.key()
    )]
    pub contract_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_purchase.user_address == owner.key(),
        constraint = user_purchase.token_address == token_mint.key(),
        seeds = [crate::TOKEN_PURCHASE, owner.key().as_ref(),token_mint.key().as_ref()],
        bump
    )]
    pub user_purchase: Account<'info, UserPurchase>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> WithdrawTokens<'info> {
    pub fn process(&mut self,bump_seed:u8) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // 检查销售是否已结束
        if current_time < self.sale.end_time {
            msg!("Sale has not ended yet.");
            return Err(ErrorCode::SaleNotEnded.into());
        }

        // 检查用户是否已购买
        if self.user_purchase.user_address != self.owner.key() {
            msg!("User has not purchased.");
            return Err(ErrorCode::UserNotPurchased.into());
        }   

    
        // 如果还有剩余代币（未卖完）
        if self.sale.remaining_amount > 0 {
            // 验证剩余代币数量是否与销售记录一致
            // if sale_balance != self.sale.remaining_amount {
            //     msg!("Token balance mismatch.");
            //     return Err(ErrorCode::BalanceMismatch.into());
            // }

            // 计算用户应得的退款金额
            let refund_amount = self.user_purchase.purchase_amount;

            let signer_seeds: &[&[&[u8]]] = &[&[crate::TOKEN_SEED,  &[bump_seed]]];

            // 生成从 GDTC 托管账户到用户 LP Token 账户的转账指令
            let transfer_instruction = spl_token::instruction::transfer(
                &self.token_program.key(),
                &self.contract_token_account.key(),
                &self.refund_token_account.key(),
                &self.sale.key(),
                &[],
                refund_amount as u64, 
            )?;
       
            // 执行带签名的 CPI 调用
            invoke_signed(
                &transfer_instruction,
                &[
                   self.token_program.to_account_info(),
                   self.contract_token_account.to_account_info(),
                   self.refund_token_account.to_account_info(),
                   self.sale.to_account_info(),
                ],
                signer_seeds,
            )?;

            // 更新用户购买记录
            self.user_purchase.purchase_amount = 0;
            self.user_purchase.token_amount = 0;

            msg!("Refunded {} buy tokens to user", refund_amount);
        } else {
            // 如果代币已全部售出，发放用户购买的代币
            let token_amount = self.user_purchase.token_amount;
            
            // 转移代币到用户账户
            // transfer(
            //     self.into_transfer_tokens_context(),
            //     token_amount
            // )?;

            let signer_seeds: &[&[&[u8]]] = &[&[crate::TOKEN_SEED,   &[bump_seed]]];

            // 生成从 GDTC 托管账户到用户 LP Token 账户的转账指令
            let transfer_instruction = spl_token::instruction::transfer(
                &self.token_program.key(),
                &self.sale_token_account.key(),
                &self.owner_token_account.key(),
                &self.sale.key(),
                &[],
                token_amount as u64, 
            )?;
       
            // 执行带签名的 CPI 调用
            invoke_signed(
                &transfer_instruction,
                &[
                   self.token_program.to_account_info(),
                   self.sale_token_account.to_account_info(),
                   self.owner_token_account.to_account_info(),
                   self.sale.to_account_info(),
                ],
                signer_seeds,
            )?;

            // 更新用户购买记录
            self.user_purchase.purchase_amount = 0;
            self.user_purchase.token_amount = 0;

            msg!("Distributed {} tokens to user", token_amount);
        }

        // 更新销售账户状态
        self.sale.remaining_amount = 0;
        self.sale.is_active = false;

        Ok(())
    }

 
}
