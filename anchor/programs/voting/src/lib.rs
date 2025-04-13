#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("DbvNMXG4yXJTkYh96taB9f8Mp2jCUoBnkACssFjv9Vgp");

pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

#[program]
pub mod voting {
    use super::*;

    pub fn initialize_poll(
        ctx: Context<InitializePoll>,
        _poll_id: u64,
        description: String,
        poll_start: u64,
        poll_end: u64,
    ) -> Result<()> {
        if description.len() > 200 {
            msg!("The description exceeds the word limit");
            return err!(ErrorCode::ExceedsWordLimit);
        }

        let poll = &mut ctx.accounts.poll;
        poll.poll_id = _poll_id;
        poll.description = description;
        poll.poll_start = poll_start;
        poll.poll_end = poll_end;
        poll.candidate_amount = 0;
        poll.candidate_list = Vec::new();

        Ok(())
    }

    pub fn initialize_candidate(
        ctx: Context<InitializeCandidate>,
        candidate_name: String,
        image_url: String,
        _poll_id: u64,
    ) -> Result<()> {
        if candidate_name.len() > 32 || image_url.len() > 128 {
            msg!("Word Limit Exceeded");
            return err!(ErrorCode::ExceedsWordLimit);
        }

        let poll: &mut Account<Poll> = &mut ctx.accounts.poll;

        require!(
            !poll
                .candidate_list
                .iter()
                .any(|c| c.candidate_name == candidate_name),
            ErrorCode::CandidateAlreadyExists
        );

        let new_candidate = CandidateDetail {
            candidate_name: candidate_name.clone(),
            candidate_votes: 0,
        };

        poll.candidate_list.push(new_candidate);
        poll.candidate_amount += 1;

        let candidate = &mut ctx.accounts.candidate;
        candidate.candidate_name = candidate_name;
        candidate.candidate_votes = 0;
        candidate.image_url = image_url;

        Ok(())
    }

    pub fn vote(ctx: Context<Vote>, candidate_name: String, _poll_id: u64) -> Result<()> {
        let poll: &Account<Poll> = &ctx.accounts.poll;

        if !poll
            .candidate_list
            .iter()
            .any(|c| c.candidate_name == candidate_name)
        {
            msg!("Candidate not found.");
            return err!(ErrorCode::UnauthorisedCandidate);
        }

        let is_cast = &mut ctx.accounts.is_cast_vote;

        if is_cast.vote {
            msg!("Restricted: one-vote-per-voter rule.");
            return err!(ErrorCode::AlreadyVoted);
        }

        is_cast.vote = true;

        let candidate: &mut Account<Candidate> = &mut ctx.accounts.candidate;
        candidate.candidate_votes += 1;

        msg!("Voted for candidate: {}", candidate.candidate_name);
        msg!("Votes: {}", candidate.candidate_votes);

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, Poll>,

    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_ref()],
        bump
    )]
    pub candidate: Account<'info, Candidate>,

    #[account(
        init_if_needed,
        payer = signer,
        space = ANCHOR_DISCRIMINATOR_SIZE + IsCast::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub is_cast_vote: Account<'info, IsCast>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct InitializeCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, Poll>,

    #[account(
        init_if_needed,
        payer = signer,
        space = ANCHOR_DISCRIMINATOR_SIZE + Candidate::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()],
        bump
    )]
    pub candidate: Account<'info, Candidate>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        space = ANCHOR_DISCRIMINATOR_SIZE + Poll::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll: Account<'info, Poll>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Candidate {
    #[max_len(32)]
    pub candidate_name: String,
    pub candidate_votes: u64,
    #[max_len(128)]
    pub image_url: String,
}

#[account]
#[derive(InitSpace)]
pub struct Poll {
    pub poll_id: u64,
    #[max_len(200)]
    pub description: String,
    pub poll_start: u64,
    pub poll_end: u64,
    pub candidate_amount: u64,
    #[max_len(32)]
    pub candidate_list: Vec<CandidateDetail>,
}

#[account]
#[derive(InitSpace)]
pub struct IsCast {
    pub vote: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct CandidateDetail {
    #[max_len(128)]
    pub candidate_name: String,
    pub candidate_votes: u64,
}
//Custom error codes
#[error_code]
pub enum ErrorCode {
    #[msg("You have already voted in this poll.")]
    AlreadyVoted,

    #[msg("You exceeded the word limit")]
    ExceedsWordLimit,

    #[msg("This candidate is not registered")]
    UnauthorisedCandidate,

    #[msg("Candidate already exists")]
    CandidateAlreadyExists,
}
