use crate::database::Account;
use crate::transactions::TransactionError;

pub trait Transaction {
    fn execute(&self, account: &mut Account) -> Result<(), TransactionError>;

    fn allowes_account_creation(&self) -> bool {
        false
    }

    fn allowed_on_frozen_account(&self) -> bool {
        false
    }
}
