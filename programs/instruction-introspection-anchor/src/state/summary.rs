use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct InstructionSummary {
    pub program_id: Pubkey,
    #[max_len(10)]
    pub accounts: Vec<Pubkey>, 
    pub accounts_count: u8,
    pub data_length: u64,
    pub is_signer_required: bool,
    pub is_writable_required: bool,
}