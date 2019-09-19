#![allow(unreachable_code)]

mod processor;
mod result;
mod util;

use solana_sdk::{account_info::AccountInfo, entrypoint, entrypoint::SUCCESS, pubkey::Pubkey};

entrypoint!(_entrypoint);
fn _entrypoint(program_id: &Pubkey, accounts: &mut [AccountInfo], data: &[u8]) -> u32 {
    match processor::process_instruction(program_id, accounts, data) {
        Err(err) => {
            err.print();
            err as u32
        }
        _ => SUCCESS,
    }
}
