//! Main entrypoint of the application.

use behaviors::{CsvTransactionSource, TransactionProcessor, TransactionSource};
use io::csv_reader;

pub(crate) mod accounts;
pub(crate) mod behaviors;
pub(crate) mod engine;
pub(crate) mod error;
pub(crate) mod io;
pub(crate) mod primitives;
pub(crate) mod transactions;

fn main() -> Result<(), crate::error::Error> {
    // crate::errors::errors_to_file()?;

    // Create the source of the transactions.
    let file_path = crate::io::get_filepath()?;
    let reader = csv_reader(&file_path)?;
    let mut transaction_source = CsvTransactionSource::new(reader);
    // Create the account holder.
    let mut accounts = crate::accounts::Accounts::new();
    // Create the engine.
    let mut engine = crate::engine::Engine::new();

    // Process all the transactions with the engine.
    engine.process_transactions(&mut transaction_source.get_transactions(), &mut accounts)?;

    // Output the accounts.
    crate::io::write_csv(accounts)?;

    Ok(())
}
