use crate::database::MemDatabase;
use crate::transactions::{
    Chargeback, Deposit, Dispute, Resolve, Transaction, TransactionError, Withdrawal,
};
use crate::transport::record::{ClientId, Record};

#[derive(derive_new::new)]
pub struct Dispatcher<'a> {
    db: &'a mut MemDatabase,
}

impl Dispatcher<'_> {
    pub fn dispatch(&mut self, row: &Result<Record, csv::Error>) {
        if let Err(err) = self.try_dispatch(row) {
            match err {
                TransactionError::Denied(cause) => log::warn!("Tranaction denied: {}", cause),
                TransactionError::Rejected(cause) => log::error!("Tranaction rejected: {}", cause),
            }
        } else {
            log::info!("Transation successfully completed");
        }
    }

    fn try_dispatch(&mut self, row: &Result<Record, csv::Error>) -> Result<(), TransactionError> {
        let rec = {
            match row {
                Ok(record) => Ok(record),
                Err(err) => Err(TransactionError::reject(format!("{}", err))),
            }
        }?;

        match rec.r#type.as_str() {
            "deposit" => self.process(rec.client, Deposit::new(rec.tx, rec.amount()?)),
            "withdrawal" => self.process(rec.client, Withdrawal::new(rec.tx, rec.amount()?)),
            "dispute" => self.process(rec.client, Dispute::new(rec.tx)),
            "resolve" => self.process(rec.client, Resolve::new(rec.tx)),
            "chargeback" => self.process(rec.client, Chargeback::new(rec.tx)),
            _ => {
                let msg = format!("Invalid transaction type: {:?}", rec.r#type);
                Err(TransactionError::reject(msg))
            }
        }
    }

    fn process(
        &mut self,
        client_id: ClientId,
        transaction: impl Transaction + std::fmt::Debug,
    ) -> Result<(), TransactionError> {
        log::debug!("== Processing {:?} on account: {}", transaction, client_id);

        let account = if transaction.allowes_account_creation() {
            self.db.get_account_or_create(client_id)
        } else {
            self.db.get_account(client_id)?
        };

        if account.locked && !transaction.allowed_on_frozen_account() {
            Err(TransactionError::deny("Not allowed on a frozen account"))?;
        }

        transaction.execute(account)
    }
}
