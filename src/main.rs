use crate::database::MemDatabase;
use crate::dispatcher::Dispatcher;
use crate::transport::{CsvExporter, CvsFileImporter};
use clap::Parser;
use std::error::Error;

mod cli;
mod database;
mod dispatcher;
mod logging;
mod tests;
mod transactions;
mod transport;

fn main() -> Result<(), Box<dyn Error>> {
    let cli_args = cli::Cli::parse();
    if cli_args.log {
        logging::setup()?;
    }
    log::info!("Transactions file: {}", cli_args.transactions.display());

    let mut importer = CvsFileImporter::new(cli_args.transactions)?;
    let mut db = MemDatabase::new();
    let mut dispatcher = Dispatcher::new(&mut db);

    for row in importer.read_rows() {
        dispatcher.dispatch(&row);
    }

    let writer = csv::Writer::from_writer(std::io::stdout());
    let mut exporter = CsvExporter::new(writer);
    exporter.dump_accounts(db.accounts())?;

    if cli_args.printdb {
        eprintln!("{:#?}", db);
    }

    Ok(())
}
