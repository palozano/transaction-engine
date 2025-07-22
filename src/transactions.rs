//! This module defines the shape of a transaction, its types and checks based on them.

use crate::{
    error::TransactionError,
    primitives::{Client, Funds, Tx},
};
use serde::Deserialize;

/// The representation of a transaction.
#[derive(Debug, Deserialize)]
pub(crate) struct Transaction {
    /// The type of transaction.
    #[serde(rename = "type")]
    pub(crate) variant: TxType,
    /// The client's ID associated with this transaction.
    pub(crate) client: Client,
    /// The transaction ID.
    pub(crate) tx: Tx,
    /// The (optional) amount for this transaction.
    pub(crate) amount: Option<Funds>,
}

/// Transaction types available.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TxType {
    /// A credit to the client's asset account, i.e., increase the available and total funds.
    Deposit,
    /// A debit to the client's asset account, i.e., decrease the available and total funds.
    Withdrawal,
    /// A client's claim that a transaction was an error and should be reversed (not now, but in
    /// the future). The available funds decrease by the amount disputed, their held funds increase
    /// by the same amount.
    Dispute,
    /// A resolution to a dispute, releasing the associated funds: the held funds are transfered
    /// back to the available funds.
    Resolve,
    /// The final state of a dispute, when a client reverses a transaction: held funds are
    /// withdrawn (i.e, the total funds decrease). Freezes the client's account.
    Chargeback,
}

impl Transaction {
    /// Check if the transaction has the necessary fields based on its type.
    ///
    /// The checks are:
    /// - for [`TxType::Deposit`] and [`TxType::Withdrawal`], an amount must be present.
    /// - for [`TxType::Dispute`], [`TxType::Resolve`] and [`TxType::Chargeback`], an amount must
    /// not be present.
    pub(crate) fn is_valid(&self) -> Result<(), TransactionError> {
        if matches!(self.variant, TxType::Deposit | TxType::Withdrawal) && self.amount.is_none() {
            return Err(TransactionError::MissingAmount(self.tx));
        }

        if matches!(
            self.variant,
            TxType::Dispute | TxType::Resolve | TxType::Chargeback
        ) && self.amount.is_some()
        {
            return Err(TransactionError::AmountPresent(self.tx));
        }

        if let Some(value) = self.amount
            && value <= Funds::ZERO
        {
            return Err(TransactionError::NonPositiveAmount(self.tx));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn funds(amount: f32) -> Decimal {
        Decimal::from_f32_retain(amount).unwrap()
    }

    #[test]
    fn test_valid_deposit() {
        let t = Transaction {
            variant: TxType::Deposit,
            client: 1,
            tx: 100,
            amount: Some(funds(10.0)),
        };

        assert!(t.is_valid().is_ok());
    }

    #[test]
    fn test_valid_withdrawal() {
        let t = Transaction {
            variant: TxType::Withdrawal,
            client: 2,
            tx: 101,
            amount: Some(funds(5.0)),
        };

        assert!(t.is_valid().is_ok());
    }

    #[test]
    fn test_invalid_deposit_missing_amount() {
        let t = Transaction {
            variant: TxType::Deposit,
            client: 3,
            tx: 102,
            amount: None,
        };

        assert_eq!(
            t.is_valid().unwrap_err(),
            TransactionError::MissingAmount(102)
        );
    }

    #[test]
    fn test_invalid_withdrawal_missing_amount() {
        let t = Transaction {
            variant: TxType::Withdrawal,
            client: 4,
            tx: 103,
            amount: None,
        };

        assert_eq!(
            t.is_valid().unwrap_err(),
            TransactionError::MissingAmount(103)
        );
    }

    #[test]
    fn test_invalid_dispute_with_amount() {
        let t = Transaction {
            variant: TxType::Dispute,
            client: 5,
            tx: 104,
            amount: Some(funds(10.0)),
        };

        assert_eq!(
            t.is_valid().unwrap_err(),
            TransactionError::AmountPresent(104)
        );
    }

    #[test]
    fn test_invalid_resolve_with_amount() {
        let t = Transaction {
            variant: TxType::Resolve,
            client: 6,
            tx: 105,
            amount: Some(funds(1.0)),
        };

        assert_eq!(
            t.is_valid().unwrap_err(),
            TransactionError::AmountPresent(105)
        );
    }

    #[test]
    fn test_invalid_chargeback_with_amount() {
        let t = Transaction {
            variant: TxType::Chargeback,
            client: 7,
            tx: 106,
            amount: Some(funds(1.0)),
        };

        assert_eq!(
            t.is_valid().unwrap_err(),
            TransactionError::AmountPresent(106)
        );
    }

    #[test]
    fn test_valid_dispute_without_amount() {
        let t = Transaction {
            variant: TxType::Dispute,
            client: 8,
            tx: 107,
            amount: None,
        };

        assert!(t.is_valid().is_ok());
    }

    #[test]
    fn test_invalid_negative_amount() {
        let t = Transaction {
            variant: TxType::Deposit,
            client: 9,
            tx: 108,
            amount: Some(funds(-5.0)),
        };

        assert_eq!(
            t.is_valid().unwrap_err(),
            TransactionError::NonPositiveAmount(108)
        );
    }

    #[test]
    fn test_invalid_zero_amount() {
        let t = Transaction {
            variant: TxType::Withdrawal,
            client: 10,
            tx: 109,
            amount: Some(funds(0.0)),
        };

        assert_eq!(
            t.is_valid().unwrap_err(),
            TransactionError::NonPositiveAmount(109)
        );
    }
}
