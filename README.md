# Transaction engine
The following binary is a transaction engine that ingests CSV files with a certain structure and outputs the result of said transactions applied to a collection of accounts.

## Basics
The application builds, reads data and writes data in the requested manner. It also includes some documentation.

## Completeness
The application handles all the cases: deposits and Withdrawals, as well as disputes, resolutions and chargebacks.

## Correctness
As far as I could have tested it, the application works correctly. A combination of the type system and tests have been used. 

The type system mainly ensures that the input and output follows the correct format, as well as ensuring that operations are correctly applied to the correct type.

Some test files have been created through a script to generate sample data.

I wanted to introduce fuzzy testing, but I didn't have enough time.

## Safety
No unsafe code has been (directly) written in this application.

Errors are created when something "wrong" has been encountered. The following explains how errors are handled in the application:

### Errors
If the program encounters an error, it continues its execution, unless it involves reading the initial data. 

The possible errors considered to continue over, are: 
- wrongly formated transactions (as mentioned before), 
- debiting from an account with insufficient funds

A wrongly defined/formated transaction is one of the following:
- For deposits and withdrawals, if an amount is not present, the transaction is not applied.
- For disputes, resolutions and chargebacks, if an amount is present, the transaction is not applied.

Tracing has been added to the application, and it logs the errors to a file called `errors.log`.

## Efficiency
I defined the reader to not load the whole dataset in memory each time, but rather read each record and process it.

## Maintainability
The code is documented so it can be read by someone else and maintained in the future.

---

## Input
The program accepts one argument to be run, and the following will be discarded. 

## Output
The output from the program is piped to std out, following the requested format.

## Choices

### Precision
The rust crate `rust_decimal` is used to ensure that the same precision of four decimals is used for both input and output.

### Arguments
For the input, only one valid-UTF8 argument is accepted; otherwise, it will error.

#### Future improvements
If the program grows in complexity, the `clap` crate can be used, or even `inquiry` if more input from the user is needed.


## TODOs
- [X] Instead of loading everything in memory, iterate over the CSV while processing.
- [ ] Unit test all the things.
- [ ] Ports and adapters to decouple.
