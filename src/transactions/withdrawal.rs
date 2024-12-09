use crate::database::Account;
use crate::transactions::{Transaction, TransactionError};
use crate::transport::record::{Amount, TransactionId};

#[derive(Debug, derive_new::new)]
pub struct Withdrawal {
    // used only for logging and for future purposes
    _transaction_id: TransactionId,
    amount: Amount,
}

impl Transaction for Withdrawal {
    fn execute(&self, account: &mut Account) -> Result<(), TransactionError> {
        if account.amount_available() < self.amount {
            Err(TransactionError::deny("Available funds are not sufficient"))?;
        }
        account.amount_total -= self.amount;
        Ok(())
    }
}
