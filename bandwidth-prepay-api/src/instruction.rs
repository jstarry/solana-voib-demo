use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_instruction;
use std::convert::TryFrom;
use crate::CONTRACT_SIZE;

#[repr(u8)]
pub enum BandwidthPrepayInstruction {
    InitializeAccount = 1,
    Spend,
    Refund,
}

impl TryFrom<u8> for BandwidthPrepayInstruction {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(BandwidthPrepayInstruction::InitializeAccount),
            2 => Ok(BandwidthPrepayInstruction::Spend),
            3 => Ok(BandwidthPrepayInstruction::Refund),
            _ => Err(()),
        }
    }
}

pub fn initialize(
    program_id: &Pubkey,
    initiator_id: &Pubkey,
    contract_pubkey: &Pubkey,
    gatekeeper_id: &Pubkey,
    provider_id: &Pubkey,
    lamports: u64,
) -> Vec<Instruction> {
    let space = CONTRACT_SIZE as u64;
    vec![
        system_instruction::create_account(
            &initiator_id,
            contract_pubkey,
            lamports,
            space,
            &program_id,
        ),
        initialize_account(
            *program_id,
            initiator_id,
            contract_pubkey,
            gatekeeper_id,
            provider_id,
        ),
    ]
}

fn initialize_account(
    program_id: Pubkey,
    initiator_id: &Pubkey,
    contract_pubkey: &Pubkey,
    gatekeeper_id: &Pubkey,
    provider_id: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*initiator_id, true),
        AccountMeta::new(*contract_pubkey, false),
        AccountMeta::new(*gatekeeper_id, false),
        AccountMeta::new(*provider_id, false),
    ];
    let data: Vec<u8> = vec![BandwidthPrepayInstruction::InitializeAccount as u8];
    Instruction {
        program_id,
        data,
        accounts,
    }
}

pub fn spend(
    program_id: Pubkey,
    gatekeeper_id: &Pubkey,
    contract_pubkey: &Pubkey,
    provider_id: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*gatekeeper_id, true),
        AccountMeta::new(*contract_pubkey, false),
        AccountMeta::new(*provider_id, false),
    ];
    let mut data = vec![BandwidthPrepayInstruction::Spend as u8];
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction {
        program_id,
        data,
        accounts,
    }
}

pub fn refund(
    program_id: Pubkey,
    gatekeeper_id: &Pubkey,
    contract_pubkey: &Pubkey,
    initiator_id: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*gatekeeper_id, true),
        AccountMeta::new(*contract_pubkey, false),
        AccountMeta::new(*initiator_id, false),
    ];
    let data: Vec<u8> = vec![BandwidthPrepayInstruction::Refund as u8];
    Instruction {
        program_id,
        data,
        accounts,
    }
}
