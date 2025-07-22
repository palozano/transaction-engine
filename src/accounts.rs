use crate::{
    error::{AccountError, Error, TransactionError},
    primitives::{Client, Funds, Tx},
};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub(crate) struct Account {
    client: Client,
    #[serde(serialize_with = "round")]
    available: Funds,
    #[serde(serialize_with = "round")]
    held: Funds,
    #[serde(serialize_with = "round")]
    total: Funds,
    locked: bool,
    #[serde(skip)]
    disputed_transactions: HashMap<Tx, Funds>,
}

/// Helper for serialize the [`Funds`] values to four decimal places, as requested.
fn round<S>(funds: &Funds, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&funds.round_dp(4).to_string())
}

impl Account {
    /// Creates a new account, given a client ID, with all the funds set to zero.
    pub(crate) fn new(client: Client) -> Self {
        Self {
            client,
            available: Funds::ZERO,
            held: Funds::ZERO,
            total: Funds::ZERO,
            locked: false,
            disputed_transactions: HashMap::new(),
        }
    }

    /// Checks if the account is locked, and errors if so.
    fn locked(&self) -> Result<(), Error> {
        if self.locked {
            return Err(AccountError::AccountLocked(self.client).into());
        } else {
            Ok(())
        }
    }

    fn get_disputed(&self, tx: Tx) -> Result<Funds, Error> {
        self.disputed_transactions
            .get(&tx)
            .cloned()
            .ok_or(TransactionError::MissingDispute(tx).into())
    }

    /// Update the `total` field from an account.
    fn update_total(&mut self) -> Result<(), AccountError> {
        if let Some(value) = self.available.checked_add(self.held) {
            self.total = value;
            Ok(())
        } else {
            Err(AccountError::Overflow(self.client))
        }
    }

    /// Removes funds from an account.
    ///
    /// It checks:
    /// - if the account is locked,
    /// - if there's an overflow when computing the corresponding values.
    pub(crate) fn credit(&mut self, funds: Funds) -> Result<(), Error> {
        self.locked()?;

        if let Some(value) = self.available.checked_add(funds) {
            self.available = value;
            self.update_total()?;
            Ok(())
        } else {
            Err(AccountError::Overflow(self.client).into())
        }
    }

    /// Removes funds from an account.
    ///
    /// It checks:
    /// - if the account is locked,
    /// - if the account has enought funds,
    /// - if there's an underflow when computing the corresponding values.
    pub(crate) fn debit(&mut self, funds: Funds) -> Result<(), Error> {
        self.locked()?;

        if self.available < funds {
            return Err(AccountError::InsufficientFunds(self.client).into());
        }

        // NOTE: this should never error, since the check is done above.
        if let Some(value) = self.available.checked_sub(funds) {
            self.available = value;
            self.update_total()?;
            Ok(())
        } else {
            Err(AccountError::Underflow(self.client).into())
        }
    }

    /// Opens a dispute for a [`Transaction`].
    ///
    /// The operations that are performed are:
    /// - Reduce `available` by the disputed value.
    /// - Increase `held` by the same amount.
    pub(crate) fn dispute(&mut self, funds: Funds, tx: Tx) -> Result<(), Error> {
        self.locked()?;

        if self.disputed_transactions.contains_key(&tx) {
            return Err(TransactionError::ExistingDispute(tx).into());
        }

        if self.available < funds {
            return Err(AccountError::InsufficientFunds(self.client).into());
        }

        // NOTE: this operation should never error, since the check is done above.
        if let Some(value) = self.available.checked_sub(funds) {
            self.available = value;
        } else {
            return Err(AccountError::Underflow(self.client).into());
        }

        if let Some(value) = self.held.checked_add(funds) {
            self.held = value;
        } else {
            return Err(AccountError::Overflow(self.client).into());
        }

        // Keep track of the disputed ammount for each "open" dispute.
        self.disputed_transactions.insert(tx, funds);

        self.update_total()?;
        Ok(())
    }

    /// Resolves a dispute that was opened for a [`Transaction`].
    ///
    /// The operations that are performed are:
    /// - Increase `available` by the disputed value.
    /// - Reduce `held` by the same amount.
    pub(crate) fn resolve(&mut self, tx: Tx) -> Result<(), Error> {
        self.locked()?;

        let amount = self.get_disputed(tx)?;

        if let Some(value) = self.held.checked_sub(amount) {
            self.held = value;
        } else {
            return Err(AccountError::Underflow(self.client).into());
        }

        if let Some(value) = self.available.checked_add(amount) {
            self.available = value;
        } else {
            return Err(AccountError::Overflow(self.client).into());
        }

        self.update_total()?;

        // Untrack the dispute if everything succeeded
        self.disputed_transactions.remove(&tx);

        Ok(())
    }

    /// Performs a chargeback for a transaction.
    pub(crate) fn chargeback(&mut self, tx: Tx) -> Result<(), Error> {
        let amount = self.get_disputed(tx)?;

        if let Some(value) = self.held.checked_sub(amount) {
            self.held = value;
        } else {
            return Err(AccountError::Underflow(self.client).into());
        }

        self.locked = true;

        // Untrack the dispute if everything succeeded
        self.disputed_transactions.remove(&tx);

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Accounts(HashMap<Client, Account>);

impl Accounts {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn inner(self) -> HashMap<Client, Account> {
        self.0
    }

    /// Checks if an account exists, otherwise creates it.
    fn exists(&mut self, client: Client) {
        if let Some(_account) = self.0.get(&client) {
            return;
        } else {
            self.0.insert(client, Account::new(client));
        }
    }

    /// Get a mutable reference to an account. If the account does not exist, it creates one.
    pub(crate) fn get_mut(&mut self, client: Client) -> &mut Account {
        self.exists(client);
        // SAFETY: safe to unwrap, since account exists by previous step.
        self.0.get_mut(&client).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_deposit() {}

    fn funds(amount: f32) -> Decimal {
        Decimal::from_f32_retain(amount).unwrap()
    }

    #[test]
    fn test_account_creation() {
        let client = 1;
        let acc = Account::new(client);
        assert_eq!(acc.available, Funds::ZERO);
        assert_eq!(acc.held, Funds::ZERO);
        assert_eq!(acc.total, Funds::ZERO);
        assert!(!acc.locked);
    }

    #[test]
    fn test_accounts_create_and_retrieve_account() {
        let client = 1;
        let mut accounts = Accounts::new();
        let account = accounts.get_mut(client);
        account.credit(funds(5.0)).unwrap();

        let retrieved = accounts.get_mut(client);
        assert_eq!(retrieved.available, funds(5.0));
    }

    #[test]
    fn test_credit_increases_available_and_total() {
        let client = 1;
        let mut acc = Account::new(client);
        acc.credit(funds(10.0)).unwrap();
        assert_eq!(acc.available, funds(10.0));
        assert_eq!(acc.total, funds(10.0));
    }

    #[test]
    fn test_debit_decreases_available_and_total() {
        let client = 1;
        let mut acc = Account::new(client);
        acc.credit(funds(20.0)).unwrap();
        acc.debit(funds(5.0)).unwrap();
        assert_eq!(acc.available, funds(15.0));
        assert_eq!(acc.total, funds(15.0));
    }

    #[test]
    fn test_debit_fails_with_insufficient_funds() {
        let client = 1;
        let mut acc = Account::new(client);
        let result = acc.debit(funds(1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_dispute_moves_funds_from_available_to_held() {
        let client = 1;
        let tx_id = 1;
        let mut acc = Account::new(client);
        acc.credit(funds(10.0)).unwrap();
        acc.dispute(funds(5.0), tx_id).unwrap();
        assert_eq!(acc.available, funds(5.0));
        assert_eq!(acc.held, funds(5.0));
        assert_eq!(acc.total, funds(10.0));
    }

    #[test]
    fn test_resolve_moves_funds_back_to_available() {
        let client = 1;
        let tx_id = 1;
        let mut acc = Account::new(client);
        acc.credit(funds(10.0)).unwrap();
        acc.dispute(funds(5.0), tx_id).unwrap();
        acc.resolve(tx_id).unwrap();
        assert_eq!(acc.available, funds(10.0));
        assert_eq!(acc.held, Funds::ZERO);
    }

    #[test]
    fn test_chargeback_removes_funds_and_locks_account() {
        let client = 1;
        let tx_id = 1;
        let mut acc = Account::new(client);
        acc.credit(funds(10.0)).unwrap();
        acc.dispute(funds(5.0), tx_id).unwrap();
        acc.chargeback(tx_id).unwrap();
        assert_eq!(acc.available, funds(5.0));
        assert_eq!(acc.held, Funds::ZERO);
        assert!(acc.locked);
    }

    #[test]
    fn test_locked_account_cannot_credit_or_debit() {
        let client = 1;
        let tx_id = 1;
        let mut acc = Account::new(client);
        acc.credit(funds(10.0)).unwrap();
        acc.dispute(funds(5.0), tx_id).unwrap();
        acc.chargeback(tx_id).unwrap();

        assert!(acc.credit(funds(5.0)).is_err());
        assert!(acc.debit(funds(5.0)).is_err());
    }
}
