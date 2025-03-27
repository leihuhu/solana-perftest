#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("95HvDrwhVst6NHgypA77UfSxZQ33YAaCRjv9gZoi725n");

#[program]
pub mod ballot {
    use super::*;

    pub fn init(
        _ctx: Context<Initialize>,
        _ballot_name: String,
        _sub_order: String,
        _proposals: Vec<String>,
    ) -> Result<()> {
        let ballot_box = &mut _ctx.accounts.ballot_box;
        ballot_box.proposals = _proposals.clone();
        ballot_box.votes.resize(_proposals.len(), 0);
        msg!(
            "new ballot: {:?}, and proposals is: {:?}",
            _ballot_name,
            _proposals
        );
        Ok(())
    }

    pub fn vote(ctx: Context<VoteAccounts>, order: u64) -> Result<()> {
        ctx.accounts.ballot_box.votes[order as usize] = ctx.accounts.ballot_box.votes
            [order as usize]
            .checked_add(1)
            .unwrap();
        msg!(
            "counter ballot_box votes : {:?}",
            ctx.accounts.ballot_box.votes
        );
        Ok(())
    }

    pub fn finalize(ctx: Context<FinalizeAccounts>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(_ballot_name: String, _sub_order: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        seeds = [_ballot_name.as_bytes(), _sub_order.as_bytes()],
        bump,
        space = 8 + BallotBox::INIT_SPACE,
        payer = payer
    )]
    pub ballot_box: Account<'info, BallotBox>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteAccounts<'info> {
    #[account(mut)]
    pub ballot_box: Account<'info, BallotBox>,
}

#[derive(Accounts)]
pub struct FinalizeAccounts<'info> {
    #[account(mut)]
    pub ballot_box: Account<'info, BallotBox>,
}

#[account]
#[derive(InitSpace)]
pub struct BallotBox {
    #[max_len(10, 50)]
    proposals: Vec<String>,
    #[max_len(10)]
    votes: Vec<u64>,
}
