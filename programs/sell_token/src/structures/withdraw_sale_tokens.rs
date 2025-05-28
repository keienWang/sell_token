use super::SaleAccount;
use super::UserPurchase;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, Mint, transfer};
use anchor_spl::associated_token::AssociatedToken;

use super::error::ErrorCode;

/// 销售账户所有者提取代币的结构体
/// 用于处理销售结束后，所有者提取剩余代币或销售所得的购买代币
#[derive(Accounts)]
pub struct WithdrawSaleTokens<'info> {
    /// 销售账户
    /// 验证：
    /// 1. 销售必须处于活跃状态
    /// 2. 调用者必须是销售账户的所有者
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
    
    
    /// 销售代币的Mint账户
    /// 验证：必须与销售账户中记录的代币Mint一致
    #[account(
        mut,
        constraint = token_mint.key() == sale.token_mint,
    )]
    pub token_mint: Account<'info, Mint>,
    
    /// 购买代币的Mint账户
    /// 验证：必须与销售账户中记录的购买代币Mint一致
    #[account(
        mut,
        constraint = buy_token_mint.key() == sale.buy_token_mint,
    )]
    pub buy_token_mint: Account<'info, Mint>,

    /// 销售账户所有者
    /// 必须是交易的签名者
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// 所有者的代币账户
    /// 验证：
    /// 1. 账户所有者必须是销售账户所有者
    /// 2. 代币类型必须与销售代币一致
    #[account(
        mut,
        constraint = owner_token_account.owner == owner.key(),
        constraint = owner_token_account.mint == token_mint.key()
    )]
    pub owner_token_account: Account<'info, TokenAccount>,
    
    /// 销售账户的代币账户
    /// 验证：
    /// 1. 账户所有者必须是销售账户
    /// 2. 代币类型必须与销售代币一致
    #[account(
        mut,
        constraint = sale_token_account.owner == pda_account.key(),
        constraint = sale_token_account.mint == token_mint.key()
    )]
    pub sale_token_account: Account<'info, TokenAccount>,

    /// 所有者的购买代币账户
    /// 验证：
    /// 1. 账户所有者必须是销售账户所有者
    /// 2. 代币类型必须与购买代币一致
    #[account(
        mut,
        constraint = owner_buy_token_account.owner == owner.key(),
        constraint = owner_buy_token_account.mint == buy_token_mint.key()
    )]
    pub owner_buy_token_account: Account<'info, TokenAccount>,

    /// 合约的购买代币账户
    /// 验证：
    /// 1. 账户所有者必须是销售账户
    /// 2. 代币类型必须与购买代币一致
    #[account(
        mut,
        constraint = contract_token_account.owner == pda_account.key(),
        constraint = contract_token_account.mint == buy_token_mint.key()
    )]
    pub contract_token_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    /// 代币程序
    pub token_program: Program<'info, Token>,
    /// 关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> WithdrawSaleTokens<'info> {
    /// 处理代币提取的主要逻辑
    pub fn process(&mut self,bump_seed:u8) -> Result<()> {
        // 获取当前时间
        let current_time = Clock::get()?.unix_timestamp;
        
        // 检查销售是否已结束
        if current_time < self.sale.end_time {
            msg!("Sale has not ended yet.");
            return Err(ErrorCode::SaleNotEnded.into());
        }

        // 获取销售账户中的代币余额
        let sale_balance = self.sale_token_account.amount;
        let contract_balance = self.contract_token_account.amount;
        
        // 处理代币提取逻辑
        if sale_balance > 0 {
            // 如果还有剩余代币（未卖完）
            // 验证剩余代币数量是否与销售记录一致
            if sale_balance != self.sale.remaining_amount {
                msg!("Token balance mismatch.");
                return Err(ErrorCode::BalanceMismatch.into());
            }

            // 构建签名者种子
            let signer_seeds: &[&[&[u8]]] = &[&[crate::TOKEN_SEED, &[bump_seed]]];

            // 转移剩余代币回所有者账户
            let transfer_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.sale_token_account.to_account_info(),
                    to: self.owner_token_account.to_account_info(),
                    authority: self.sale.to_account_info(),
                },
                signer_seeds,
            );
            transfer(transfer_ctx, sale_balance)?;


            msg!("Withdrew {} unsold tokens back to owner", sale_balance);
        } else if contract_balance > 0 {
            // 如果代币已全部售出，转移购买代币到所有者账户
            // 构建签名者种子
            let signer_seeds: &[&[&[u8]]] = &[&[crate::TOKEN_SEED, &[bump_seed]]];

            // 转移购买代币到所有者账户
            let transfer_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.contract_token_account.to_account_info(),
                    to: self.owner_buy_token_account.to_account_info(),
                    authority: self.sale.to_account_info(),
                },
                signer_seeds
            );
            transfer(transfer_ctx, contract_balance)?;


            msg!("Withdrew {} buy tokens to owner", contract_balance);
        } else {
            // 如果两种代币都没有，返回错误
            msg!("No tokens to withdraw");
            return Err(ErrorCode::NoTokensToWithdraw.into());
        }

        // 更新销售账户状态
        self.sale.remaining_amount = 0;
        self.sale.is_active = false;

        Ok(())
    }
}