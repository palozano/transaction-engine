//! This module defines functions to interact with the input for the application and the output
//! that is expected from it.

use crate::accounts::Accounts;
use std::{fs, io};

/// Gets the path of the file containing the transactions, which is given as an argument when
/// calling the binary.
pub(crate) fn get_filepath() -> Result<String, io::Error> {
    std::env::args().nth(1).ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "missing transactions file",
    ))
}

/// Create a transaction CSV reader for the given file path.
pub(crate) fn csv_reader(file_path: &str) -> csv::Result<csv::Reader<fs::File>> {
    // Create a CSV reader.
    let rdr = csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_path(file_path)?;

    Ok(rdr)
}

/// Writes the given collection of [`Accounts`] to std out.
pub(crate) fn write_csv(accounts: Accounts) -> csv::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for (_, acc) in accounts.inner() {
        wtr.serialize(acc)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::Account;
    use rust_decimal::Decimal;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn funds(amount: f32) -> Decimal {
        Decimal::from_f32_retain(amount).unwrap()
    }

    #[test]
    fn test_get_filepath_returns_error_when_missing_argument() {
        let result = get_filepath();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_csv_reader_reads_valid_csv() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(
            temp,
            "client,available,held,total,locked\n1,1.0,0.0,1.0,false"
        )
        .unwrap();

        let reader_result = csv_reader(temp.path().to_str().unwrap());
        assert!(reader_result.is_ok());

        let mut rdr = reader_result.unwrap();
        let mut count = 0;
        for result in rdr.records() {
            assert!(result.is_ok());
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn test_csv_reader_invalid_path() {
        let result = csv_reader("nonexistent_file.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_csv_outputs_valid_csv() {
        let mut accounts = Accounts::new();
        let client = 1;
        let mut acc = Account::new(client);
        acc.credit(funds(5.0)).unwrap();
        accounts.get_mut(client).credit(funds(5.0)).unwrap();

        let mut output = Vec::new();
        {
            let mut writer = csv::Writer::from_writer(&mut output);
            for (_, acc) in accounts.inner() {
                writer.serialize(acc).unwrap();
            }
        }

        let output_str = String::from_utf8(output).unwrap();
        println!("{}", output_str);
        assert!(output_str.contains("1"));
        assert!(output_str.contains("5"));
    }
}
