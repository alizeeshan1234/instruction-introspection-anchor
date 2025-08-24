use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
use anchor_spl::associated_token::{get_associated_token_address, AssociatedToken};

use crate::state::{IntrospectionResult, InstructionSummary, SecurityAnalysis};

use anchor_lang::solana_program::sysvar::instructions::{
    load_current_index_checked,
    load_instruction_at_checked,
};

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    pub recipient: UncheckedAccount<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = sender,
        associated_token::mint = mint,
        associated_token::authority = sender,
    )]
    pub from: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = sender,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub to: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = sender,
        space = 8 + InstructionSummary::INIT_SPACE,
        seeds = [b"instruction_summary", sender.key().as_ref()],
        bump,
    )]
    pub instructions_summary: Account<'info, InstructionSummary>,

    #[account(
        init_if_needed,
        payer = sender,
        space = 8 + SecurityAnalysis::INIT_SPACE,
        seeds = [b"security_analysis", sender.key().as_ref()],
        bump,
    )]
    pub security_analysis: Account<'info, SecurityAnalysis>,

    #[account(
        init_if_needed,
        payer = sender,
        space = 8 + IntrospectionResult::INIT_SPACE,
        seeds = [b"introspection_result", sender.key().as_ref()],
        bump,
    )]
    pub instrospection_result: Account<'info, IntrospectionResult>,

    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn token_transfer_handler(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.from.to_account_info(),
        to: ctx.accounts.to.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.sender.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

    transfer_checked(cpi_context, amount, ctx.accounts.mint.decimals)?;

    let ix_sysvar = ctx.accounts.instructions.to_account_info();
    let current_index = load_current_index_checked(&ix_sysvar)?;

    let ix = load_instruction_at_checked(current_index as usize, &ix_sysvar)?;

    require_keys_eq!(ix.program_id, crate::id(), ErrorCode::InvalidProgramId);

    let summary = &mut ctx.accounts.instructions_summary;
    summary.program_id = ix.program_id;
    let account_keys: Vec<Pubkey> = ix.accounts.iter().take(10).map(|meta| meta.pubkey).collect();
    summary.accounts = account_keys;
    summary.accounts_count = ix.accounts.len() as u8;
    summary.data_length = ix.data.len() as u64;
    summary.is_signer_required = ix.accounts.iter().any(|meta| meta.is_signer);
    summary.is_writable_required = ix.accounts.iter().any(|meta| meta.is_writable);

    let analysis = &mut ctx.accounts.security_analysis;
    let mut suspicious_score = 0;
    let mut total_accounts = 0;
    let mut unique_programs: std::collections::HashSet<Pubkey> = std::collections::HashSet::new();

    let mut has_system_calls = false;
    let mut has_token_operations = false;
    let mut has_cross_program_invokes = false;
    let mut has_large_data = false;
    let mut repeated_programs: u8 = 0;

    let mut compute_budget_instructions: u8 = 0;
    let mut token_instructions: u8 = 0;
    let mut system_instructions: u8 = 0;
    let mut custom_program_instructions: u8 = 0;
    let mut total_data_bytes: u32 = 0;


    let total_ix = current_index + 1;

    for i in 0..total_ix {
        let ix = load_instruction_at_checked(i as usize, &ix_sysvar)?;

        if !unique_programs.insert(ix.program_id) {
            repeated_programs += 1;
        };

        if ix.program_id == anchor_lang::system_program::ID {
            has_system_calls = true;
        };

        if ix.program_id == anchor_spl::token::ID {
            has_token_operations = true;
        };

        if ix.accounts.iter().any(|meta| meta.is_signer && meta.is_writable) {
            has_cross_program_invokes = true;
        };

        if ix.data.len() > 200 { 
            has_large_data = true;
        }

        total_accounts = ix.accounts.len() as u16;

        if ix.program_id == anchor_lang::system_program::ID {
            system_instructions += 1;
        } else if ix.program_id == anchor_spl::token::ID {
            token_instructions += 1;
        } else if ix.program_id.to_string() == "ComputeBudget111111111111111111111111111111" {
            compute_budget_instructions += 1;
        } else {
            custom_program_instructions += 1;
        }

        total_data_bytes += ix.data.len() as u32;
    }

    if has_system_calls { suspicious_score += 1 };
    if has_token_operations { suspicious_score += 1 };
    if has_cross_program_invokes { suspicious_score += 1 };
    if has_large_data { suspicious_score += 2; }
    if repeated_programs > 0 { suspicious_score += repeated_programs; }

    analysis.suspicious_score = suspicious_score;
    analysis.has_system_calls = has_system_calls;
    analysis.has_token_operations = has_token_operations;
    analysis.has_cross_program_invokes = has_cross_program_invokes;
    analysis.has_large_data = has_large_data;
    analysis.repeated_programs = repeated_programs;
    analysis.total_accounts = total_accounts;
    analysis.unique_programs = unique_programs.len() as u8;

    let resut = &mut ctx.accounts.instrospection_result;
    resut.total_instructions = total_ix as u8;
    resut.compute_budget_instructions = compute_budget_instructions;
    resut.token_instructions = token_instructions;
    resut.system_instructions = system_instructions;
    resut.custom_program_instructions = custom_program_instructions;
    resut.total_accounts_touched = total_accounts;
    resut.total_data_bytes = total_data_bytes;
    resut.transaction_complexity = (total_ix + custom_program_instructions as u16 + (total_data_bytes / 100) as u16) as u8;

    msg!("Instruction Summary: {:?}", summary);
    msg!("Security Analysis: {:?}", analysis);
    msg!("Result: {:?}", resut);

    Ok(())
}
