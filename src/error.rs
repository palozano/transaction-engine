//! This module defines several errors to be used throughout the application.
//!
//! I'm aware that most of this stuff can be replaced with the `anyhow` crate or similar. However,
//! I wanted to implement the errors myself because it helps me find errors in the application and
//! think about the process a bit more.

use crate::primitives::{Client, Tx};

// NOTE: this could be used for a broader, friendlier interface for errors. However, I find more concrete errors easier and faster to iterate and prototype with, since I see where and how I fail.
//
// pub(crate) type Result<T = ()> = core::result::Result<T, Box<dyn std::error::Error>>;

/// The general error used to report failures in the code.
///
/// Takes into account all the possible errors that can arise (IO, CSV parsing, transaction
/// application and account management).
#[derive(Debug)]
pub(crate) enum Error {
    /// Error while dealing with accounts.
    Account(AccountError),
    /// Error while dealing with transactions.
    Transaction(TransactionError),
    /// Error while dealing with input-output.
    Io(std::io::Error),
    /// Error while dealing with CSV files.
    Csv(csv::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Account(error) => write!(f, "Error while managing account: {}", error),
            Error::Transaction(error) => write!(f, "Error while processing transaction: {}", error),
            Error::Io(error) => write!(f, "IO related error: {}", error),
            Error::Csv(error) => write!(f, "CSV related error: {}", error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Error::Csv(err)
    }
}

/// Errors while dealing with [`Account`]s.
#[derive(Debug)]
pub(crate) enum AccountError {
    /// There are not enough funds in the client's account.
    InsufficientFunds(Client),
    /// The client's account is locked and cannot perfom operations.
    AccountLocked(Client),
    /// The client's account overflowed.
    Overflow(Client),
    /// The client's account underflowed.
    Underflow(Client),
}

impl From<AccountError> for Error {
    fn from(err: AccountError) -> Self {
        Error::Account(err)
    }
}

impl std::fmt::Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountError::InsufficientFunds(c) => {
                write!(f, "Account {} does not have enough funds", c)
            }
            AccountError::AccountLocked(c) => {
                write!(f, "Account {} is locked and cannot perform operations", c)
            }
            AccountError::Overflow(c) => write!(f, "Account {} overflowed", c),
            AccountError::Underflow(c) => write!(f, "Account {} underflowed", c),
        }
    }
}

/// Errors while applying [`Transaction`]s.
#[derive(Debug, PartialEq)]
pub(crate) enum TransactionError {
    /// The transaction is missing the amount field.
    MissingAmount(Tx),
    /// The transaction should not have an amount field.
    AmountPresent(Tx),
    /// The amount present is non positive.
    NonPositiveAmount(Tx),
    /// The transaction is a duplicate of a previous one.
    DuplicateFound(Tx),
    /// A dispute already exists for the transaction.
    ExistingDispute(Tx),
    /// There is no dispute for the transaction.
    MissingDispute(Tx),
    /// Only a deposit transaction can be disputed.
    OnlyDepositsCanBeDisputed(Tx),
    /// The client in the dispute is not the same as the one in the original transaction.
    WrongClient(Tx, Client, Client),
}

impl From<TransactionError> for Error {
    fn from(err: TransactionError) -> Self {
        Error::Transaction(err)
    }
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::MissingAmount(t) => {
                write!(f, "Transaction {} is missing 'amount' and is required.", t)
            }
            TransactionError::AmountPresent(t) => {
                write!(
                    f,
                    "Transaction {} has 'amount' present and is not required.",
                    t
                )
            }
            TransactionError::NonPositiveAmount(t) => {
                write!(f, "Transaction {} has a negative amount.", t)
            }
            TransactionError::DuplicateFound(t) => {
                write!(f, "Transaction {} is duplicated.", t)
            }
            TransactionError::ExistingDispute(t) => {
                write!(f, "Another dispute exists for transaction {}", t)
            }
            TransactionError::MissingDispute(t) => {
                write!(f, "There is no dispute for transaction {}", t)
            }
            TransactionError::WrongClient(t, old_client, new_client) => write!(
                f,
                "Client mismatch for transaction {} while opening a dispute: original is {} and found {}",
                t, old_client, new_client
            ),
            TransactionError::OnlyDepositsCanBeDisputed(t) => write!(
                f,
                "Transaction {} is a dispute that refers to a past transaction that is not a deposit.",
                t,
            ),
        }
    }
}

// NOTE: this produces some output in a log file for errors that arise while executing,
// but since it was not specified if other files could be produced, it is commented out.
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
