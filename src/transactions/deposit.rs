use crate::database::{Account, Transfer};
use crate::transactions::{Transaction, TransactionError};
use crate::transport::record::{Amount, TransactionId};

#[derive(Debug, derive_new::new)]
pub struct Deposit {
    transaction_id: TransactionId,
    amount: Amount,
}

impl Transaction for Deposit {
    fn execute(&self, account: &mut Account) -> Result<(), TransactionError> {
        if account.contains_transfer(&self.transaction_id) {
            Err(TransactionError::reject("Duplicated transaction ID"))?;
        }
        account.amount_total += self.amount;

        let transfer = Transfer::new(self.amount, false);
        let msg = format!("Transfer recorded: {:?}", transfer);

        account.insert_transfer(self.transaction_id, transfer);
        log::debug!("{}", msg);
        Ok(())
    }

    fn allowes_account_creation(&self) -> bool {
        true
    }

    fn allowed_on_frozen_account(&self) -> bool {
        true
    }
}
