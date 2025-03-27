use anchor_lang::prelude::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("59XDJqwb67HEYq779ZGwDdv5ZUCncqqGXAicnAxkb7U9");

#[account]
#[derive(Default)]
pub struct UserBalance {
    /// CHECK: 该字段通过程序逻辑中的签名验证保证安全性
    pub owner: Pubkey,
    pub balance: u64,
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 8, // discriminator(8) + owner(32) + balance(8)
        seeds = [b"balance", payer.key().as_ref()], // PDA
        bump
    )]
    pub user_account: Account<'info, UserBalance>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut, has_one = owner)]
    pub from: Account<'info, UserBalance>,
    #[account(mut)]
    pub to: Account<'info, UserBalance>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

#[error_code]
pub enum TokenError {
    #[msg("Owner Mismatch")]
    OwnerMismatch,
    #[msg("Insufficient Balance")]
    InsufficientBalance,
    #[msg("Overflow")]
    Overflow,
}

#[program]
pub mod simple_transfer {
    use super::*;

    pub fn init_user(ctx: Context<InitializeUser>, initial_balance: u64) -> Result<()> {
        let user = &mut ctx.accounts.user_account;
        user.owner = ctx.accounts.payer.key();
        user.balance = initial_balance;
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        let from = &mut ctx.accounts.from;
        let to = &mut ctx.accounts.to;

        require!(
            from.owner == ctx.accounts.owner.key(),
            TokenError::OwnerMismatch
        );
        require!(from.balance >= amount, TokenError::InsufficientBalance);


        from.balance = from
            .balance
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;
        to.balance = to.balance.checked_add(amount).ok_or(TokenError::Overflow)?;

        Ok(())
    }
}

