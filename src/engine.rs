//! This module defines the engine of the program, which is in charge of processing a collection of
//! transaction onto a collection of accounts.

use crate::{
    accounts::Account,
    error::{Error, TransactionError},
    primitives::Tx,
    transactions::{Transaction, TxType},
};
use std::collections::HashMap;

/// Engine in charge of applying transactions.
pub(crate) struct Engine {
    ledger: HashMap<Tx, Transaction>,
}

impl Engine {
    pub(crate) fn new() -> Self {
        Self {
            ledger: HashMap::new(),
        }
    }

    /// Get a transaction from the ledger/historical records.
    fn get_transaction(&self, tx: Tx) -> Result<&Transaction, Error> {
        self.ledger
            .get(&tx)
            .ok_or(TransactionError::MissingDispute(tx).into())
    }

    /// Process the [`Transaction`] onto the corresponding [`Account`] b
    pub(crate) fn process(
        &mut self,
        account: &mut Account,
        transaction: Transaction,
    ) -> Result<(), Error> {
        transaction.is_valid()?;

        // TODO: check all possible transactions? or only a subset?
        if self.ledger.contains_key(&transaction.tx) {
            return Err(TransactionError::DuplicateFound(transaction.tx).into());
        }

        match transaction.variant {
            TxType::Deposit => self.process_deposit(account, transaction)?,
            TxType::Withdrawal => self.process_withdrawal(account, transaction)?,
            TxType::Dispute => self.process_dispute(account, transaction)?,
            TxType::Resolve => self.process_resolution(account, transaction)?,
            TxType::Chargeback => self.process_chargeback(account, transaction)?,
        }

        Ok(())
    }

    /// All the actions necessary for a [`TxType::Deposit`].
    fn process_deposit(
        &mut self,
        account: &mut Account,
        transaction: Transaction,
    ) -> Result<(), Error> {
        // Safe to unwrap since there's a check for valid transactions earlier.
        account.credit(transaction.amount.unwrap())?;

        // Record the deposit in the history.
        self.ledger.insert(transaction.tx, transaction);
        Ok(())
    }

    /// All the actions involved in a [`TxType::Withdrawal`].
    fn process_withdrawal(
        &mut self,
        account: &mut Account,
        transaction: Transaction,
    ) -> Result<(), Error> {
        // Safe to unwrap since there's a check for valid transactions earlier.
        account.debit(transaction.amount.unwrap())?;

        // Record the withdrawal in the history.
        self.ledger.insert(transaction.tx, transaction);
        Ok(())
    }

    // TODO: the three functions below share some common functionality that can be refactored into
    // a new function, so there's less boilerplate.

    /// All the actions involved in a [`TxType::Dispute`].
    fn process_dispute(
        &mut self,
        account: &mut Account,
        transaction: Transaction,
    ) -> Result<(), Error> {
        // If there exists a previous transaction.
        let past_transaction = self.get_transaction(transaction.tx)?;
        // And it was a deposit.
        if past_transaction.variant == TxType::Deposit {
            return Err(TransactionError::OnlyDepositsCanBeDisputed(transaction.tx).into());
        }

        // If the dispute matches the previous transaction client.
        if past_transaction.client != transaction.client {
            return Err(TransactionError::WrongClient(
                transaction.tx,
                past_transaction.client,
                transaction.client,
            )
            .into());
        }

        // Safe to unwrap since there's a check for valid transactions earlier.
        account.dispute(transaction.amount.unwrap(), transaction.tx)?;

        Ok(())
    }

    /// All the actions involed in a resolution ([`TxType::Resolve`]).
    fn process_resolution(
        &mut self,
        account: &mut Account,
        transaction: Transaction,
    ) -> Result<(), Error> {
        // If there exists a previous transaction.
        let past_transaction = self.get_transaction(transaction.tx)?;

        // And has the same client.
        if past_transaction.client != transaction.client {
            return Err(TransactionError::WrongClient(
                transaction.tx,
                past_transaction.client,
                transaction.client,
            )
            .into());
        }

        account.resolve(transaction.tx)?;

        Ok(())
    }

    /// All the actions involved in a [`TxType::Chargeback`].
    fn process_chargeback(
        &mut self,
        account: &mut Account,
        transaction: Transaction,
    ) -> Result<(), Error> {
        // If there exists a previous transaction.
        let past_transaction = self.get_transaction(transaction.tx)?;

        // And has the same client.
        if past_transaction.client != transaction.client {
            return Err(TransactionError::WrongClient(
                transaction.tx,
                past_transaction.client,
                transaction.client,
            )
            .into());
        }

        account.chargeback(transaction.tx)?;

        Ok(())
    }
}
