pub mod structures;
pub mod constants;

use constants::*;


use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use structures::{
    init_sale_account::*,
    withdraw_tokens::*,
    buy_token::*,
};

declare_id!("QWXKkZHHuVKooqKnRjdLoEPt8PGrjaFNB6bRURnDx4T");

#[program]
pub mod sell_token {
    use super::*;

    // pub fn sale_account(ctx: Context<InitSaleAccount>, sale_amount: u64, price_per_token: u64, end_time: i64) -> Result<()> {
    //     ctx.accounts.process(sale_amount, price_per_token, end_time)
    // }
    pub fn init_sale_account(ctx: Context<InitSaleAccount>, sale_amount: u64, price_per_token: u64, end_time: i64) -> Result<()> {
        ctx.accounts.process(sale_amount, price_per_token, end_time)
    }

    pub fn buy_token(ctx: Context<BuyToken>, amount: u64) -> Result<()> {
        ctx.accounts.process(amount)
    }

    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>) -> Result<()> {
        ctx.accounts.process()
    }
}


