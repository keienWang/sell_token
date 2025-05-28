pub mod structures;
pub mod constants;

use constants::*;


use anchor_lang::prelude::*;


use structures::{
    init_sale_account::*,
    withdraw_tokens::*,
    buy_token::*,
    withdraw_sale_tokens::*,
};

declare_id!("8u2V6SHBURgDV23rvWFKBvPvhthYKP3eHfYgGJzQHLps");

#[program]
pub mod sell_token {
    use super::*;

    // pub fn sale_account(ctx: Context<InitSaleAccount>, sale_amount: u64, price_per_token: u64, end_time: i64) -> Result<()> {
    //     ctx.accounts.process(sale_amount, price_per_token, end_time)
    // }
    pub fn init_sale_account(ctx: Context<InitSaleAccount>, sale_amount: u64, price_per_token: u64, end_time: i64) -> Result<()> {
        ctx.accounts.process(sale_amount, price_per_token, end_time)
    }

    pub fn buy_token(ctx: Context<BuyToken>, amount: u64,open_time: u64) -> Result<()> {
        let bump = ctx.bumps.pda_account;
        ctx.accounts.process(amount, bump,open_time)
    }

    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>) -> Result<()> {
        let bump = ctx.bumps.pda_account;
        ctx.accounts.process(bump)
    }

    pub fn withdraw_sale_tokens(ctx: Context<WithdrawSaleTokens>) -> Result<()> {
        let bump = ctx.bumps.pda_account;
        ctx.accounts.process(bump)
    }
}


