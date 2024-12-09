use crate::database::Account;
use crate::transactions::{Transaction, TransactionError};
use crate::transport::record::TransactionId;

#[derive(Debug, derive_new::new)]
pub struct Chargeback {
    transaction_id: TransactionId,
}

impl Transaction for Chargeback {
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
        account.amount_total -= amount;
        account.remove_transfer(&self.transaction_id);
        account.locked = true;
        Ok(())
    }
}
