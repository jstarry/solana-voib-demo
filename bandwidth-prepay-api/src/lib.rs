mod data;
mod instruction;

pub use data::*;
pub use instruction::*;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub enum AccountType {
    Unset,
    Contract,
    Invalid,
}

impl From<u8> for AccountType {
    fn from(value: u8) -> Self {
        match value {
            0 => AccountType::Unset,
            1 => AccountType::Contract,
            _ => AccountType::Invalid,
        }
    }
}
