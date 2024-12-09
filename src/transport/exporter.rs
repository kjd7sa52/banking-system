use crate::database::Account;
use crate::transport::record::ClientId;
use std::collections::HashMap;

#[derive(derive_new::new)]
pub struct CsvExporter<W: std::io::Write> {
    writer: csv::Writer<W>,
}

impl<W: std::io::Write> CsvExporter<W> {
    pub fn dump_accounts(
        &mut self,
        accounts: &HashMap<ClientId, Account>,
    ) -> Result<(), csv::Error> {
        let header = &["client", "available", "held", "total", "locked"];
        self.writer.write_record(header)?;
        for (client_id, account) in accounts.iter() {
            self.writer.serialize((
                client_id,
                account.amount_available(),
                account.amount_held,
                account.amount_total,
                account.locked,
            ))?;
        }
        self.writer.flush()?;
        Ok(())
    }
}
