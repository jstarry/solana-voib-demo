use crate::connection_params::NewConnParams;
use bandwidth_prepay_api::ContractData;
use bs58;
use jsonrpc_core::types::error::Error;
use log::*;
use solana_sdk::client::Client;
use solana_sdk::message::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, KeypairUtil};
use solana_sdk::transaction::Transaction;
use solana_sdk::transport::{Result as TransportResult, TransportError};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{io, mem};

pub fn check_contract<T: Client>(
    parsed_params: &NewConnParams,
    client: &Arc<T>,
    gatekeeper_id: &Pubkey,
) -> TransportResult<(u64, ContractData)> {
    let data = client.get_account_data(&parsed_params.contract_pubkey)?;
    if data.is_none() {
        return Err(TransportError::IoError(io::Error::new(
            io::ErrorKind::Other,
            "Contract account contains no data".to_string(),
        )));
    }
    let lamports = client.get_balance(&parsed_params.contract_pubkey)?;
    let contract_data = ContractData::from_bytes(&data.unwrap()).map_err(|err| {
        error!(
            "unable to deserialize contract account: {:?}, {}",
            parsed_params.contract_pubkey, err
        );
        TransportError::IoError(io::Error::new(
            io::ErrorKind::Other,
            format!("Unable to deserialize contract account: {:?}", err),
        ))
    })?;
    if gatekeeper_id != &contract_data.gatekeeper_id {
        error!(
            "incorrect contract_data gatekeeper_id: {:?}",
            contract_data.gatekeeper_id
        );
        return Err(TransportError::IoError(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Incorrect contract_data gatekeeper_id: {:?}",
                contract_data.gatekeeper_id
            ),
        )));
    }
    Ok((lamports, contract_data))
}

pub fn verify_pubkey(input: String) -> Result<Pubkey, Error> {
    let pubkey_vec = bs58::decode(input).into_vec().map_err(|err| {
        info!("verify_pubkey: invalid input: {:?}", err);
        Error::invalid_request()
    })?;
    if pubkey_vec.len() != mem::size_of::<Pubkey>() {
        info!(
            "verify_pubkey: invalid pubkey_vec length: {}",
            pubkey_vec.len()
        );
        Err(Error::invalid_request())
    } else {
        Ok(Pubkey::new(&pubkey_vec))
    }
}

pub fn charge_contract<T: Client>(
    parsed_params: &NewConnParams,
    client: &Arc<T>,
    program_id: &Pubkey,
    contract_data: &ContractData,
    gatekeeper: &Keypair,
    amount: u64,
) -> TransportResult<()> {
    let message = build_spend_message(
        program_id,
        gatekeeper,
        &parsed_params.contract_pubkey,
        &contract_data.provider_id,
        amount,
    );
    let _ = client.send_message(&[gatekeeper], message)?;
    Ok(())
}

fn build_spend_message(
    program_id: &Pubkey,
    gatekeeper: &Keypair,
    contract_pubkey: &Pubkey,
    provider_id: &Pubkey,
    amount: u64,
) -> Message {
    let instruction = bandwidth_prepay_api::spend(
        *program_id,
        &gatekeeper.pubkey(),
        contract_pubkey,
        provider_id,
        amount,
    );
    Message::new(vec![instruction])
}

pub fn build_and_sign_spend_transaction<T: Client>(
    client: &Arc<T>,
    program_id: &Pubkey,
    gatekeeper: &Keypair,
    contract_pubkey: &Pubkey,
    provider_id: &Pubkey,
    amount: u64,
) -> Transaction {
    let (blockhash, _) = client.get_recent_blockhash().unwrap();
    let message = build_spend_message(program_id, gatekeeper, contract_pubkey, provider_id, amount);
    Transaction::new(&[gatekeeper], message, blockhash)
}

pub fn submit_transaction_loop<T: Client>(solana_receiver: &Receiver<(Arc<T>, Transaction)>) {
    loop {
        if let Ok((client, transaction)) = solana_receiver.try_recv() {
            if let Err(e) = client.async_send_transaction(transaction) {
                error!(
                    "Error sending charge transaction to solana fullnode: {:?}",
                    e
                );
            };
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn refund<T: Client>(
    parsed_params: &NewConnParams,
    client: &Arc<T>,
    program_id: &Pubkey,
    contract_data: &ContractData,
    gatekeeper: &Keypair,
) -> TransportResult<()> {
    let instruction = bandwidth_prepay_api::refund(
        *program_id,
        &gatekeeper.pubkey(),
        &parsed_params.contract_pubkey,
        &contract_data.initiator_id,
    );
    let message = Message::new(vec![instruction]);
    let _ = client.send_message(&[gatekeeper], message)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_runtime::bank::Bank;
    use solana_runtime::bank_client::BankClient;
    use solana_runtime::genesis_utils::{create_genesis_block, GenesisBlockInfo};
    use solana_runtime::loader_utils::load_program;
    use solana_sdk::client::SyncClient;
    use solana_sdk::bpf_loader;
    use solana_sdk::system_instruction;
    use std::sync::mpsc::channel;
    use std::thread::Builder;
    use std::fs::File;
    use std::io::Read;

    fn create_bank(lamports: u64) -> (Arc<BankClient>, Pubkey, Keypair) {
        let GenesisBlockInfo { genesis_block, mint_keypair, .. } = create_genesis_block(lamports);
        let bank_client = BankClient::new(Bank::new(&genesis_block));
        let mut program_file = File::open("../dist/programs/bandwidth_prepay.so").expect("program should exist");
        let mut program_bytes = Vec::new();
        program_file.read_to_end(&mut program_bytes).unwrap();
        let program_id = load_program(&bank_client, &mint_keypair, &bpf_loader::id(), program_bytes);
        (Arc::new(bank_client), program_id, mint_keypair)
    }

    #[test]
    fn test_verify_pubkey() {
        let pubkey = Pubkey::new_rand();
        let pubkey_str = pubkey.to_string();
        let other_str = "randomString".to_string();
        let strange_str = "d_e_a_d.beef".to_string();
        let empty_str = "".to_string();
        assert_eq!(verify_pubkey(pubkey_str.to_string()).unwrap(), pubkey);
        assert_eq!(
            verify_pubkey(other_str.to_string()),
            Err(Error::invalid_request())
        );
        assert_eq!(
            verify_pubkey(strange_str.to_string()),
            Err(Error::invalid_request())
        );
        assert_eq!(
            verify_pubkey(empty_str.to_string()),
            Err(Error::invalid_request())
        );
    }

    #[test]
    fn test_check_contract() {
        let (bank_client, program_id, alice_keypair) = create_bank(10_000);

        let alice_pubkey = alice_keypair.pubkey();
        let contract = Keypair::new().pubkey();
        let gatekeeper = Keypair::new().pubkey();
        let provider = Keypair::new().pubkey();

        let params = NewConnParams {
            contract_pubkey: contract,
            destination: "127.0.0.1:1234".to_string(),
            fee_interval: 1000,
        };

        let expected_data = ContractData {
            gatekeeper_id: gatekeeper.clone(),
            provider_id: provider.clone(),
            initiator_id: alice_pubkey.clone(),
        };

        let instructions = bandwidth_prepay_api::initialize(
            &program_id,
            &alice_pubkey,
            &contract,
            &gatekeeper,
            &provider,
            500,
        );
        let message = Message::new(instructions);
        bank_client.send_message(&[&alice_keypair], message).unwrap();

        assert_eq!(
            check_contract(&params, &bank_client, &gatekeeper).unwrap(),
            (500, expected_data)
        );

        assert!(check_contract(&params, &bank_client, &Pubkey::new(&vec![4; 32])).is_err());
        let params = NewConnParams {
            contract_pubkey: Pubkey::new(&vec![5; 32]),
            destination: "127.0.0.1:1234".to_string(),
            fee_interval: 1000,
        };
        assert!(check_contract(&params, &bank_client, &gatekeeper).is_err());
    }

    #[test]
    fn test_charge_contract() {
        let (bank_client, program_id, alice_keypair) = create_bank(10_000);

        let alice_pubkey = alice_keypair.pubkey();
        let contract = Keypair::new().pubkey();
        let gatekeeper = Keypair::new();
        let provider = Keypair::new().pubkey();

        // Initialize Contract
        let instructions = bandwidth_prepay_api::initialize(
            &program_id,
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

        let params = NewConnParams {
            contract_pubkey: contract.clone(),
            destination: "127.0.0.1:1234".to_string(),
            fee_interval: 1000,
        };
        let data = ContractData {
            gatekeeper_id: gatekeeper.pubkey(),
            provider_id: provider.clone(),
            initiator_id: alice_pubkey.clone(),
        };

        charge_contract(&params, &bank_client, &program_id, &data, &gatekeeper, 100).unwrap();

        let balance = bank_client.get_balance(&contract).unwrap();
        assert_eq!(balance, 400);
        let account_data = bank_client.get_account_data(&contract).unwrap().unwrap();
        let data = ContractData::from_bytes(&account_data).unwrap();
        assert_eq!(data.gatekeeper_id, gatekeeper.pubkey());
        assert_eq!(data.provider_id, provider);
        assert_eq!(data.initiator_id, alice_pubkey);
        let balance = bank_client.get_balance(&provider).unwrap();
        assert_eq!(balance, 100);
    }

    #[test]
    fn test_submit_transaction_loop() {
        let (bank_client, _program_id, alice_keypair) = create_bank(10_000);
        let client_clone0 = bank_client.clone();
        let client_clone1 = bank_client.clone();

        let alice_pubkey = alice_keypair.pubkey();
        let recipient = Keypair::new().pubkey();

        let (sender, receiver) = channel();
        Builder::new()
            .name("test_submit_transaction_loop".to_string())
            .spawn(move || {
                submit_transaction_loop(&receiver);
            })
            .unwrap();

        let instruction = system_instruction::transfer(&alice_pubkey, &recipient, 100);
        let message = Message::new(vec![instruction]);
        let (blockhash, _) = bank_client.get_recent_blockhash().unwrap();
        let transaction = Transaction::new(&[&alice_keypair], message, blockhash);

        sender.send((client_clone0, transaction)).unwrap();
        let mut balance = 0;
        while balance == 0 {
            balance = bank_client.get_balance(&recipient).unwrap();
        }
        assert_eq!(balance, 100);
        assert_eq!(bank_client.get_balance(&alice_pubkey).unwrap(), 9_899);

        let instruction = system_instruction::transfer(&alice_pubkey, &recipient, 90);
        let message = Message::new(vec![instruction]);
        let (blockhash, _) = bank_client.get_recent_blockhash().unwrap();
        let transaction = Transaction::new(&[&alice_keypair], message, blockhash);

        sender.send((client_clone1, transaction)).unwrap();
        while balance == 100 {
            balance = bank_client.get_balance(&recipient).unwrap();
        }
        assert_eq!(balance, 190);
        assert_eq!(bank_client.get_balance(&alice_pubkey).unwrap(), 9_809);
    }

    #[test]
    fn test_refund() {
        let (bank_client, program_id, alice_keypair) = create_bank(10_000);

        let alice_pubkey = alice_keypair.pubkey();
        let contract = Keypair::new().pubkey();
        let gatekeeper = Keypair::new();
        let provider = Keypair::new().pubkey();

        // Initialize Contract
        let instructions = bandwidth_prepay_api::initialize(
            &program_id,
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

        let params = NewConnParams {
            contract_pubkey: contract.clone(),
            destination: "127.0.0.1:1234".to_string(),
            fee_interval: 1000,
        };
        let data = ContractData {
            gatekeeper_id: gatekeeper.pubkey(),
            provider_id: provider.clone(),
            initiator_id: alice_pubkey.clone(),
        };

        charge_contract(&params, &bank_client, &program_id, &data, &gatekeeper, 100).unwrap();
        refund(&params, &bank_client, &program_id, &data, &gatekeeper).unwrap();

        let balance = bank_client.get_balance(&contract).unwrap();
        assert_eq!(balance, 0);
        let balance = bank_client.get_balance(&provider).unwrap();
        assert_eq!(balance, 100);
        let balance = bank_client.get_balance(&alice_pubkey).unwrap();
        assert_eq!(balance, 9_898);
    }
}
