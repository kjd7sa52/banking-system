use crate::database::transfer::Transfer;
use crate::transactions::TransactionError;
use crate::transport::record::{Amount, TransactionId};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Account {
    pub amount_held: Amount,
    pub amount_total: Amount,
    pub locked: bool,
    transfers: HashMap<TransactionId, Transfer>,
}

impl Account {
    pub fn insert_transfer(&mut self, transaction_id: TransactionId, transfer: Transfer) {
        self.transfers.insert(transaction_id, transfer);
    }

    pub fn remove_transfer(&mut self, transaction_id: &TransactionId) {
        self.transfers.remove(transaction_id);
    }

    pub fn contains_transfer(&self, transaction_id: &TransactionId) -> bool {
        self.transfers.contains_key(transaction_id)
    }

    pub fn try_get_transfer_mut(
        &mut self,
        transaction_id: &TransactionId,
    ) -> Result<&mut Transfer, TransactionError> {
        if let Some(transfer) = self.transfers.get_mut(transaction_id) {
            return Ok(transfer);
        }
        Err(TransactionError::reject("Corresponding transfer not found"))
    }

    pub fn amount_available(&self) -> Amount {
        self.amount_total - self.amount_held
    }
}
