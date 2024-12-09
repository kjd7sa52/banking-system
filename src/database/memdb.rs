use crate::database::account::Account;
use crate::transactions::TransactionError;
use crate::transport::record::ClientId;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct MemDatabase {
    accounts: HashMap<ClientId, Account>,
}

impl MemDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accounts(&self) -> &HashMap<ClientId, Account> {
        &self.accounts
    }

    pub fn get_account(&mut self, client_id: ClientId) -> Result<&mut Account, TransactionError> {
        self.accounts
            .get_mut(&client_id)
            .ok_or(TransactionError::reject("Account not found"))
    }

    pub fn get_account_or_create(&mut self, client_id: ClientId) -> &mut Account {
        let account_entry = self.accounts.entry(client_id);
        account_entry.or_insert_with(|| {
            let account = Account::default();
            log::debug!("Account created, client ID: {}", client_id);
            account
        })
    }
}
