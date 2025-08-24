pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("BKo2i9aY4ohURE4GhJa9aQxvACzvywLc95TGa5Dmxxph");

#[program]
pub mod instruction_introspection_anchor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn process_transfer_introspection(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        instructions::token_transfer_handler(ctx, amount)
    }
}
