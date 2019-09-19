use crate::result::{ProgramError, ProgramResult};
use crate::util::{
    expect_account_type, expect_key, expect_min_size, expect_n_accounts, expect_new_account,
    expect_owned_by, expect_signed,
};
use arrayref::array_ref;
use bandwidth_prepay_api::{BandwidthPrepayInstruction, ContractData, CONTRACT_SIZE, AccountType};
use solana_sdk::{account_info::AccountInfo, info, pubkey::Pubkey};
use std::convert::TryFrom;

fn initialize_account(program_id: &Pubkey, accounts: &mut [AccountInfo]) -> ProgramResult {
    info!("initialize account");
    expect_n_accounts(accounts, 4)?;

    let (initiator_account, accounts) = accounts.split_first_mut().unwrap();
    expect_signed(initiator_account)?;

    let (contract_account, accounts) = accounts.split_first_mut().unwrap();
    expect_owned_by(contract_account, program_id)?;
    expect_min_size(contract_account.data, CONTRACT_SIZE)?;
    expect_new_account(contract_account)?;

    let (gatekeeper_account, accounts) = accounts.split_first_mut().unwrap();
    let (provider_account, _) = accounts.split_first_mut().unwrap();

    ContractData::copy_to_bytes(
        contract_account.data,
        initiator_account.key,
        gatekeeper_account.key,
        provider_account.key,
    );
    Ok(())
}

fn spend(program_id: &Pubkey, accounts: &mut [AccountInfo], data: &[u8]) -> ProgramResult {
    info!("spend");
    expect_n_accounts(accounts, 3)?;

    let (gatekeeper_account, accounts) = accounts.split_first_mut().unwrap();
    expect_signed(gatekeeper_account)?;

    let (contract_account, accounts) = accounts.split_first_mut().unwrap();
    expect_owned_by(contract_account, program_id)?;
    expect_account_type(contract_account, AccountType::Contract)?;

    let (provider_account, _) = accounts.split_first_mut().unwrap();

    let contract_data =
        ContractData::from_bytes(contract_account.data).map_err(|_| ProgramError::InvalidInput)?;
    expect_key(gatekeeper_account, &contract_data.gatekeeper_id)?;
    expect_key(provider_account, &contract_data.provider_id)?;

    if data.len() != 8 {
        return Err(ProgramError::InvalidSpendData);
    }
    let amount = u64::from_le_bytes(*array_ref!(data, 0, 8));
    if *contract_account.lamports < amount {
        Err(ProgramError::BalanceTooLow)?
    }

    *contract_account.lamports -= amount;
    *provider_account.lamports += amount;

    Ok(())
}

fn refund(program_id: &Pubkey, accounts: &mut [AccountInfo]) -> ProgramResult {
    info!("refund");
    expect_n_accounts(accounts, 3)?;

    let (gatekeeper_account, accounts) = accounts.split_first_mut().unwrap();
    expect_signed(gatekeeper_account)?;

    let (contract_account, accounts) = accounts.split_first_mut().unwrap();
    expect_owned_by(contract_account, program_id)?;
    expect_account_type(contract_account, AccountType::Contract)?;

    let (initiator_account, _) = accounts.split_first_mut().unwrap();

    let contract_data =
        ContractData::from_bytes(contract_account.data).map_err(|_| ProgramError::InvalidInput)?;
    expect_key(gatekeeper_account, &contract_data.gatekeeper_id)?;
    expect_key(initiator_account, &contract_data.initiator_id)?;

    *initiator_account.lamports += *contract_account.lamports;
    *contract_account.lamports = 0;

    Ok(())
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &mut [AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (instruction, data) = data.split_at(1);
    let instruction = BandwidthPrepayInstruction::try_from(instruction[0].to_le())
        .map_err(|_| ProgramError::InvalidCommand)?;

    match instruction {
        BandwidthPrepayInstruction::InitializeAccount => initialize_account(program_id, accounts),
        BandwidthPrepayInstruction::Spend => spend(program_id, accounts, data),
        BandwidthPrepayInstruction::Refund => refund(program_id, accounts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bandwidth_prepay_instruction;
    use crate::id;
    use solana_runtime::bank::Bank;
    use solana_runtime::bank_client::BankClient;
    use solana_sdk::client::SyncClient;
    use solana_sdk::genesis_block::create_genesis_block;
    use solana_sdk::message::Message;
    use solana_sdk::signature::{Keypair, KeypairUtil};
    use solana_sdk::system_instruction;

    fn create_bank(lamports: u64) -> (Bank, Keypair) {
        let (genesis_block, mint_keypair) = create_genesis_block(lamports);
        let mut bank = Bank::new(&genesis_block);
        bank.add_instruction_processor(id(), process_instruction);
        (bank, mint_keypair)
    }

    #[test]
    fn test_bandwidth_prepay_initialize() {
        let (bank, alice_keypair) = create_bank(10_000);
        let bank_client = BankClient::new(bank);

        let alice_pubkey = alice_keypair.pubkey();
        let contract = Keypair::new().pubkey();
        let gatekeeper = Keypair::new().pubkey();
        let provider = Keypair::new().pubkey();

        let instructions = bandwidth_prepay_instruction::initialize(
            &alice_pubkey,
            &contract,
            &gatekeeper,
            &provider,
            500,
        );
        let message = Message::new(instructions);
        bank_client
            .send_message(&[&alice_keypair], message)
            .unwrap();
        assert_eq!(bank_client.get_balance(&contract).unwrap(), 500);
        assert_eq!(bank_client.get_balance(&alice_pubkey).unwrap(), 9_500);
        let account = bank_client.get_account_data(&contract).unwrap().unwrap();
        assert_eq!(account.len(), BandwidthPrepayState::max_size());
        let state = BandwidthPrepayState::deserialize(&account).unwrap();
        assert_eq!(state.gatekeeper_id, gatekeeper);
        assert_eq!(state.provider_id, provider);
        assert_eq!(state.initiator_id, alice_pubkey);
    }

    #[test]
    fn test_bandwidth_prepay_spend() {
        let (bank, alice_keypair) = create_bank(10_000);
        let bank_client = BankClient::new(bank);

        let alice_pubkey = alice_keypair.pubkey();
        let contract = Keypair::new().pubkey();
        let provider = Keypair::new().pubkey();
        let gatekeeper = Keypair::new();

        // Initialize contract
        let instructions = bandwidth_prepay_instruction::initialize(
            &alice_pubkey,
            &contract,
            &gatekeeper.pubkey(),
            &provider,
            500,
        );
        let message = Message::new(instructions);
        bank_client
            .send_message(&[&alice_keypair], message)
            .unwrap();

        // Make sure gatekeeper account exists
        let instruction = system_instruction::transfer(&alice_pubkey, &gatekeeper.pubkey(), 1);
        let message = Message::new(vec![instruction]);
        bank_client
            .send_message(&[&alice_keypair], message)
            .unwrap();
        assert_eq!(bank_client.get_balance(&gatekeeper.pubkey()).unwrap(), 1);

        let instruction =
            bandwidth_prepay_instruction::spend(&gatekeeper.pubkey(), &contract, &provider, 100);
        let message = Message::new(vec![instruction]);
        bank_client.send_message(&[&gatekeeper], message).unwrap();
        assert_eq!(bank_client.get_balance(&contract).unwrap(), 400);
        assert_eq!(bank_client.get_balance(&provider).unwrap(), 100);
    }

    #[test]
    fn test_bandwidth_prepay_refund() {
        let (bank, alice_keypair) = create_bank(10_000);
        let bank_client = BankClient::new(bank);

        let alice_pubkey = alice_keypair.pubkey();
        let contract = Keypair::new().pubkey();
        let provider = Keypair::new().pubkey();
        let gatekeeper = Keypair::new();

        // Initialize contract
        let instructions = bandwidth_prepay_instruction::initialize(
            &alice_pubkey,
            &contract,
            &gatekeeper.pubkey(),
            &provider,
            500,
        );
        let message = Message::new(instructions);
        bank_client
            .send_message(&[&alice_keypair], message)
            .unwrap();

        // Make sure gatekeeper account exists
        let instruction = system_instruction::transfer(&alice_pubkey, &gatekeeper.pubkey(), 1);
        let message = Message::new(vec![instruction]);
        bank_client
            .send_message(&[&alice_keypair], message)
            .unwrap();
        assert_eq!(bank_client.get_balance(&gatekeeper.pubkey()).unwrap(), 1);

        let instruction =
            bandwidth_prepay_instruction::refund(&gatekeeper.pubkey(), &contract, &alice_pubkey);
        let message = Message::new(vec![instruction]);
        bank_client.send_message(&[&gatekeeper], message).unwrap();
        assert_eq!(bank_client.get_balance(&contract).unwrap(), 0);
        assert_eq!(bank_client.get_balance(&provider).unwrap(), 0);
        assert_eq!(bank_client.get_balance(&alice_pubkey).unwrap(), 9_999);
    }
}
