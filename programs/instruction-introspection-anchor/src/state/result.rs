use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct IntrospectionResult {
    pub total_instructions: u8,
    pub compute_budget_instructions: u8,
    pub token_instructions: u8,
    pub system_instructions: u8,
    pub custom_program_instructions: u8,
    pub total_accounts_touched: u16,
    pub total_data_bytes: u32,
    pub transaction_complexity: u8,
}