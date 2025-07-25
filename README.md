# Transaction engine
The following binary is a transaction engine that ingests CSV files with a certain structure and outputs the result of said transactions applied to a collection of accounts.

## Basics
The application builds, reads data and writes data in the requested manner. It also includes some documentation.

## Completeness
The application handles all the cases: deposits and withdrawals, as well as disputes, resolutions and chargebacks.

## Correctness
As far as I could have tested it, the application works correctly. A combination of the type system and tests have been used. 

The type system mainly ensures that the input and output follows the correct format, as well as ensuring that operations are correctly applied to the correct type.

I wanted to introduce fuzzy testing, but I didn't have enough time. I added (manually) transactions to a `transaction.csv` file to test if it works correctly.

## Safety
No unsafe code has been (directly) written in this application.

Errors are created when something "wrong" has been encountered. The following explains how errors are handled in the application:

## Errors
If the program encounters an error, it continues its execution, unless it involves reading the initial data. 

The possible errors considered to continue over, are: 
- wrongly formated transactions (as mentioned before), 
- debiting from an account with insufficient funds

A wrongly defined/formated transaction is one of the following:
- For deposits and withdrawals, if an amount is not present, the transaction is not applied.
- For disputes, resolutions and chargebacks, if an amount is present, the transaction is not applied.

Tracing has been added to the application, and it logs the errors to a file called `errors.log`, however it's not enabled by default (the function to enable is commented out).

## Efficiency
I defined the reader to not load the whole dataset in memory each time, but rather read each record and process it.

## Maintainability
The code is documented so it can be read by someone else and maintained in the future.

---

## Disclaimer: Use of AI
1. I have used AI while writing unit tests: I asked ChatGPT to give me unit tests for the functions I wrote, and then I fix them (because they are usually wrong). 

1. I also asked for a bit of help to ChatGPT while writing the traits to refactor the behavior. I almost got them correct at the first try without AI, but I always mess up the generics in some way, so I used ChatGPT to go faster.

Other than those two cases, I didn't use more AI while developing this.

---

## Future improvements
- [ ] If the program grows in complexity, the `clap` crate can be used, or even `inquiry` if more input from the user is needed.
- [ ] Add integration tests, if possible.
- [ ] Refactor the code to remove duplication.
- [ ] Refactor the code to decouple it more.


