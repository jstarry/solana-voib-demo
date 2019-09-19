use solana_sdk::pubkey::Pubkey;
use crate::AccountType;

/// Contract data size
/// Breakdown: account type (1) + gatekeeper key (32) + provider key (32) + initiator key (32)
pub const CONTRACT_SIZE: usize = 1 + 3 * 32;

#[derive(PartialEq, Debug)]
pub struct ContractData {
    pub initiator_id: Pubkey,
    pub gatekeeper_id: Pubkey,
    pub provider_id: Pubkey,
}

impl ContractData {
    pub fn copy_to_bytes(
        dst: &mut [u8],
        initiator_id: &Pubkey,
        gatekeeper_id: &Pubkey,
        provider_id: &Pubkey,
    ) {
        let (account_type, dst) = dst.split_at_mut(1);
        account_type[0] = AccountType::Contract as u8;

        let (dst_initiator_id, dst) = dst.split_at_mut(32);
        dst_initiator_id.copy_from_slice(initiator_id.as_ref());

        let (dst_gatekeeper_id, dst) = dst.split_at_mut(32);
        dst_gatekeeper_id.copy_from_slice(gatekeeper_id.as_ref());

        let (dst_provider_id, _) = dst.split_at_mut(32);
        dst_provider_id.copy_from_slice(provider_id.as_ref());
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() != CONTRACT_SIZE {
            Err("Invalid data size for contract account".to_string())?;
        }

        // Ignore account type
        let (_, data) = data.split_at(1);

        let (initiator_id, data) = data.split_at(32);
        let initiator_id = Pubkey::new(initiator_id);

        let (gatekeeper_id, data) = data.split_at(32);
        let gatekeeper_id = Pubkey::new(gatekeeper_id);

        let (provider_id, _) = data.split_at(32);
        let provider_id = Pubkey::new(provider_id);

        Ok(Self {
            initiator_id,
            gatekeeper_id,
            provider_id,
        })
    }
}
