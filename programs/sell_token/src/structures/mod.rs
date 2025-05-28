use anchor_lang::prelude::*;

pub mod error;
pub mod init_sale_account;
pub mod buy_token;
pub mod withdraw_tokens;
pub mod withdraw_sale_tokens;

// pub  mod  change_admin;


#[account]
pub struct SaleAccount {
    pub owner: Pubkey,  // 所有者
    pub token_mint: Pubkey, // 代币Mint
    pub sale_amount: u64, // 销售数量
    pub remaining_amount: u64, // 剩余数量
    pub price_per_token: u64, // 每代币价格
    pub buy_token_mint: Pubkey, // 购买代币Mint
    pub end_time: i64, // 结束时间
    pub is_active: bool, // 是否活跃
}

//用户购买结构 
#[account]
pub struct UserPurchase {
    pub user_address: Pubkey, // 用户地址
    pub token_amount: u64, // 代币数量
    pub token_price: u64, // 代币价格
    pub token_address: Pubkey, // 代币地址
    pub purchase_amount: u64, // 购买数量
    pub purchase_time: i64, // 购买时间
    pub is_claim: bool, // 是否已领取
}




