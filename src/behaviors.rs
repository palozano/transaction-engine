//! This module defines common behavior through traits so there's more decoupling between the
//! components of the application.
//!
//! Right now, there are two important traits that decouple the application:
//! - [`TransactionSource`]: which defines that the transactions should be gathered (from file, io, etc).
//! - [`TransactionProcessor`]: which defines that the transactions should be applied.
//!
//! With these two traits, you can implement a CSV reader and a sequential processor for the
//! transactions (similar to the actual implementations that can be found below).
//! But you can also define an async reader and an async processor (with a little tweaking) so the
//! engine can apply the transactions in an async manner, or even a parallel processor if you fancy
//! it.
//!
//! An actual implementation could not have been finished in time, but I leave here some notes for
//! a future refactor :D
//!
//!
//! Here's some code for the async version of the [`TransactionSource`]:
//! ```rust
//! pub trait AsyncTransactionSource {
//!     type Stream<'a>: Stream<Item = Result<Transaction, Error>> + Send + 'a
//!         where Self: 'a;
//!
//!     fn stream_transactions<'a>(&'a mut self) -> Self::Stream<'a>;
//! }
//! ```
//!
//! Which would be then implement by a `CsvAsyncSource` or `StreamSource` or similar entity.
//!
//! And the async processor for, e.g., streams, would look like something similar to this:
//! ```rust
//! pub trait AsyncTransactionProcessor {
//!    fn process_transactions<'a, S>(
//!        &'a mut self,
//!        transactions: S,
//!        accounts: &'a mut Accounts,
//!    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
//!    where
//!        S: Stream<Item = Result<Transaction, Error>> + Send + 'a;
//! }
//!
//! use futures::StreamExt;
//!
//! impl AsyncTransactionProcessor for Engine {
//!     fn process_transactions<'a, S>(
//!        &'a mut self,
//!        mut transactions: S,
//!        accounts: &'a mut Accounts,
//!    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>
//!    where
//!        S: Stream<Item = Result<Transaction, Error>> + Send + 'a,
//!    {
//!       while let Some(result) = transactions.next().await {
//!           let transaction: Transaction = record?;
//!           let account = accounts.get_mut(transaction.client);
//!           match self.process(account, transaction) {
//!               Ok(_) => {}
//!               Err(e) => {
//!                   tracing::error!("{}", e);
//!               }
//!           }
//!       }
//!
//!       Ok(())
//!    }
//! }
//! ```
//!
//! Using the `futures` crate and tokio for the runtime.

use crate::{accounts::Accounts, engine::Engine, error::Error, transactions::Transaction};

/// Behavior expected from an entity providing [`Transaction`]s in a synchronous manner.
pub(crate) trait TransactionSource {
    type Iter<'a>: Iterator<Item = Result<Transaction, csv::Error>> + 'a
    where
        Self: 'a;

    /// Returns an iterator over [`Transaction`]s.
    fn get_transactions<'a>(&'a mut self) -> Self::Iter<'a>;
}

/// [`Transaction`] provider from a given CSV file.
pub struct CsvTransactionSource<R: std::io::Read> {
    reader: csv::Reader<R>,
}

impl<R: std::io::Read> CsvTransactionSource<R> {
    pub(crate) fn new(reader: csv::Reader<R>) -> Self {
        Self { reader }
    }
}

impl<R: std::io::Read> TransactionSource for CsvTransactionSource<R> {
    type Iter<'a>
        = csv::DeserializeRecordsIter<'a, R, Transaction>
    where
        Self: 'a;

    fn get_transactions<'a>(&'a mut self) -> Self::Iter<'a> {
        self.reader.deserialize::<Transaction>()
    }
}

/// Behavior expected from the entity in charge of processing [`Transaction`]s in a sequential
/// and synchronous manner.
pub(crate) trait TransactionProcessor {
    /// Process the collection of [`Transaction`]s given by an iterator.
    fn process_transactions<I>(
        &mut self,
        transactions: I,
        accounts: &mut Accounts,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = Result<Transaction, csv::Error>>;
}

impl TransactionProcessor for Engine {
    fn process_transactions<I>(
        &mut self,
        transactions: I,
        accounts: &mut Accounts,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = Result<Transaction, csv::Error>>,
    {
        for record in transactions {
            let transaction: Transaction = record?;
            let account = accounts.get_mut(transaction.client);
            match self.process(account, transaction) {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("{}", e);
                }
            }
        }

        Ok(())
    }
}
