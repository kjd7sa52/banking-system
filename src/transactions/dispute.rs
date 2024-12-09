use crate::database::Account;
use crate::transactions::{Transaction, TransactionError};
use crate::transport::record::TransactionId;

#[derive(Debug, derive_new::new)]
pub struct Dispute {
    transaction_id: TransactionId,
}

impl Transaction for Dispute {
    fn execute(&self, account: &mut Account) -> Result<(), TransactionError> {
        let amount = {
            let amount_available = account.amount_available();
            let transfer = account.try_get_transfer_mut(&self.transaction_id)?;

            if transfer.disputed {
                let msg = "Corresponding transfer already disputed";
                Err(TransactionError::deny(msg))?;
            }

            if amount_available < transfer.amount {
                let msg = "Available funds are not sufficient";
                Err(TransactionError::deny(msg))?;
            }

            transfer.disputed = true;
            transfer.amount
        };

        account.amount_held += amount;
        Ok(())
    }
}
