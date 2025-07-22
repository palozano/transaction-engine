//! Main entrypoint of the application.

pub(crate) mod accounts;
pub(crate) mod engine;
pub(crate) mod error;
pub(crate) mod io;
pub(crate) mod primitives;
pub(crate) mod transactions;

fn main() -> Result<(), crate::error::Error> {
    // errors_to_file()?;

    // Get the file path from the user's input.
    let file = crate::io::get_filepath()?;
    // Create a reader for the transactions in the file.
    let mut transaction_reader = crate::io::csv_reader(&file)?;
    // Create an empty collection of accounts to be used by the engine.
    let mut accounts = crate::accounts::Accounts::new();

    // Create an engine and feed the accounts and transacions to be processed.
    let mut engine = crate::engine::Engine::new();
    engine.process_transactions(&mut transaction_reader, &mut accounts)?;

    // Write the output.
    crate::io::write_csv(accounts)?;

    Ok(())
}

// NOTE: this produces some output for errors, but since it was not specified if other files could
// be produced, it is not going to be used.
#[allow(dead_code)]
/// Set up tracing and the file to write to.
fn errors_to_file() -> Result<(), crate::error::Error> {
    // The file to log to
    let log_file = std::sync::Mutex::new(std::fs::File::create("error.log")?);

    // Set the subscriber for the tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .with_writer(log_file)
        .compact()
        .init();

    Ok(())
}
