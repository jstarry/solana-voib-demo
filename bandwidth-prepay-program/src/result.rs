use solana_sdk::info;

pub type ProgramResult = core::result::Result<(), ProgramError>;

#[derive(Debug)]
pub enum ProgramError {
    AccountDataTooSmall = 1,
    AccountNotNew,
    BalanceTooLow,
    InvalidAccount,
    InvalidAccountType,
    InvalidCommand,
    InvalidInput,
    InvalidKey,
    InvalidSpendData,
    MissingSigner,
}

impl ProgramError {
    pub fn print(&self) {
        match self {
            ProgramError::AccountDataTooSmall => info!("Error: AccountDataTooSmall"),
            ProgramError::AccountNotNew => info!("Error: AccountNotNew"),
            ProgramError::BalanceTooLow => info!("Error: BalanceTooLow"),
            ProgramError::InvalidAccount => info!("Error: InvalidAccount"),
            ProgramError::InvalidAccountType => info!("Error: InvalidAccountType"),
            ProgramError::InvalidCommand => info!("Error: InvalidCommand"),
            ProgramError::InvalidInput => info!("Error: InvalidInput"),
            ProgramError::InvalidKey => info!("Error: InvalidKey"),
            ProgramError::InvalidSpendData => info!("Error: InvalidSpendData"),
            ProgramError::MissingSigner => info!("Error: MissingSigner"),
        }
    }
}
