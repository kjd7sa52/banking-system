# Simple Banking System

![](https://github.com/kjd7sa52/banking-system/actions/workflows/tests.yml/badge.svg)

Coding exercise.

## Usage

```sh
cargo run -- examples/simple_transactions.csv
```

- Add `--log` flag to see processing logs.
- Add `--printdb` to see full preview of the database.

## Development

```sh
# formatting:
cargo fmt
# linting:
cargo clippy
# testing:
cargo test
# building:
cargo build
```

## Initial Assumptions

Certain sections of the requirements document are incomplete. The project has been developed based on the assumptions listed below. At the same time, architecture of the application provides enough flexibility to apply changes in scope of uncertain requirements.

- Rows in the input file that don't conform to the specified input format are ignored. Application continues to process the following rows.

- All the transactions that are described below as invalid or not allowed are ignored. Application continues to process the following transactions.

- "Frozen account" and "locked account" are synonyms. Transactions to the locked accounts are not allowed, except for *Deposit*. There is no transaction that could unlock an account.
  > Transaction for unlocking the account can be added by creating a file in `transactions` module. File must define a structure that implements `Transaction` trait. Finally structure must be added to `try_dispatch()` in `dispatcher.rs`.
  > Transactions that are allowed on frozen a account must return `true` from their `allowed_on_frozen_account()`.

- *Deposit* and *Withdrawal* are valid only if corresponding field `amount` has value that is positive (greater than 0).
  > This requirement can be easily relaxed in `amount()` getter defined in `record.rs`. Such feature could be useful for instance for creating accounts with no initial funds.

- Only *Deposit* can result in creation of a new account. Remaining transactions are invalid when they refer to a hypothetical new account (with no funds and no previous transactions).
  > This can be easily altered by defining `allowes_account_creation()` that returns `true` for corresponding transaction.

- Only *Deposit* transaction can be disputed.
  > In case of an ATM, this would make even more sense to dispute *Withdrawal*. This would require additional considerations and is currently not implemented. Required changes would be applied in the implementation of `execute()` in `dispute.rs`, `resolve.rs`, `chargeback.rs`. Additionally `withdrawal.rs` must be changed to store `Transfer` objects with negative `amount` under given account. No changes to `Transfer` are needed.
  > Transactions *Dispute*/*Resolve*/*Chargeback* cannot be disputed as they don't even have own transaction ID (their `tx` corresponds to the transaction being disputed).

- *Dispute* transaction is rejected if corresponding amount is no longer available in the account.
  > Ignoring this limitation could result in negative total amounts in the accounts. If this is intended, block under `amount_available < transfer.amount` in `dispute.rs` should be removed.

- *Resolve*/*Chargeback* can apply only to the transactions that are under dispute.

- *Dispute* can apply only to the transactions that are not under dispute. Transaction can be disputed and resolved unlimited number of times.

- Since transactions have globally unique identifiers, `client_id` of *Dispute*/*Resolve*/*Chargeback* seems to carry redundant information. Despite of this, `client_id` is expected to be valid and correspond to the transaction indicated by `tx`. Otherwise, transaction *Dispute*/*Resolve*/*Chargeback* in question is considered invalid.

- Transactions that re-use value of `tx` used before can be ignored. This is however not required from the application.
  > Application doesn't keep track of the transactions other than *Deposit*. It will ignore *Deposit* transaction with a `tx` re-used within the same account. Other cases of `tx` duplication are not detected. Application does normal processing of such transactions.

- Applications terminates with exit code other than 0 in case of errors not related to the content of the input file. This applies for instance to non-existing input file, inaccessible input file, invalid command line arguments, etc. In remaining cases, application terminates with exit code 0.

- Amounts are truncated to four digits past the decimal point.

## Architecture Overview

Application has been developed with maintainability, scalability and performance in mind. Last but not least, efforts have been made to provide best possible robustness. This **minimizes changes for an unconscious developer to introduce bugs** in the future development. Examples of this will be provided later in this chapter.

### Dispatcher

Central part of the application is the `Dispatcher` that implements modified [Command Pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/command.html). Transaction types correspond to commands. Commands are implemented in `transactions` directory.

- For better performance, `Dispatcher` does dispatching statically.

- `Dispatcher::process` performs operations that are common for the commands. Commands provide flags that enable/disable those operations. See `allowes_account_creation`, `allowed_on_frozen_account`.
  
  > For robustness, default values of the flags define *safe* default behavior.
  
- `Transaction::execute()` has access only to a single account.
  > This provides robustness and perhaps simplifies concurrent processing potentially introduced in the future.

- Only *Deposit* transaction can be reverted . This is the only command that is stored in history.

- For optimizing memory usage, we don't store entire commands in history. Instead `Transfer` object is stored.
  > Field `amount` can take positive and negative values, which makes it suitable for storing *Withdrawal* if such feature is requested in the future.
  > It turns out that only boolean flag `disputed` is required to encode possible states of `Transfer`. Note that Transfers that are charged back are removed from the history, which virtually encodes the third state.

- Each account has its dedicated `transfers` for storing history.
  > Transactions are identified by globally unique identifiers. This allows for storing them in a container that would be shared between accounts. This would potentially result in more optimal memory usage (less fragmentation). On the other hand, this appears to complicate data flow in the application. That's why distributed approach has been applied.

### Importer & Exporter

`Exporter` currently implements [Strategy Pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/strategy.html). This allows for storing the output data not only in stdout, but also other pipes/files.

In future development, `Importer` is assumed to implement similar pattern. This will allow for replacing source of data with relational database or perhaps other storage.

### Amounts

Data type used for amounts is `Decimal` from [rust_decimal](https://docs.rs/rust_decimal/latest/rust_decimal/) crate that is dedicated for financial operations. This opens many options for integrating with an SQL database or adjusting behavior of the application. For instance we may easily change how the application handles input data with more than 4 digits past the decimal point.

`held_amount` and `available_amount` always complement each other to `total_amount`. Based on this fact, we only sotre and operate on two variables. The third is calculated based on the other variables on demand. This ensures coherency thus increases robustness of the code.

*Deposit* and *Withdrawal* have been identified as potentially the most frequent operations. They modify both `available_amount` and `total_amount`. This is why decision has been made to store only one of them and calculate the remaining. Analyzing further, it turns out that across the entire logic there is less to implement if we operate on `total_amount` and not on `available_amount`. This is why `available_amount` is decided to be calculated and remaining two fields stored under `Account`.

### Error Handling

Application defines `TransactionError` that represents errors introduced by incorrect input data. Typically caused by an external system/user. They are recoverable from the application point of view. Remainig errors are not recoverable and cause the application to exit with non-zero exit code. Application is never expected to panic.

- `TransactionError::Rejected` is used against records of the input data that are unconditionally invalid. For example they don't comply to the specifed format or are semantically incorrect. This error is reported on `error` logging level.

- `TransactionError::Denied` is used against transactions that cannot be executed due to the circumstances. For instance insufficient funds. This error is reported on `warning` logging level.

According to the [YAGNI](https://es.wikipedia.org/wiki/YAGNI) principle, variants of `TransactionError` don't implement `Error` trait. This may be added in the future if necessary.

### Testing

Applicacation contains one layer of tests. I've found it convininent to collect them in a single file: `tests.rs`.  The same file includes a slim test framework `TestApp` that makes tests more readable and ensures [DRY](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself)ness.

Features of external crates are assumed to be tested in their repos. For instance handling of parsing errors by [csv](https://docs.rs/csv/1.3.1/csv/) crate.

Project has initially configured CI where formatting/linting/testing/building is triggered on every push.

## Author

- [Grzegorz Krason](<mailto:grzegorz.krason@gmail.com>)
