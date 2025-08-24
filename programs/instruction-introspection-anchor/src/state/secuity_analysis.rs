use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct SecurityAnalysis {
    pub suspicious_score: u8,      
    pub has_system_calls: bool,
    pub has_token_operations: bool,
    pub has_cross_program_invokes: bool,
    pub has_large_data: bool,
    pub repeated_programs: u8,
    pub total_accounts: u16,
    pub unique_programs: u8,
}
