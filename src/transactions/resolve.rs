use crate::database::Account;
use crate::transactions::{Transaction, TransactionError};
use crate::transport::record::TransactionId;

#[derive(Debug, derive_new::new)]
pub struct Resolve {
    transaction_id: TransactionId,
}

impl Transaction for Resolve {
    fn execute(&self, account: &mut Account) -> Result<(), TransactionError> {
        let amount = {
            let transfer = account.try_get_transfer_mut(&self.transaction_id)?;

            if !transfer.disputed {
                let msg = "Corresponding transfer not disputed";
                Err(TransactionError::deny(msg))?;
            }

            transfer.disputed = false;
            transfer.amount
        };
        account.amount_held -= amount;
        Ok(())
    }
}
