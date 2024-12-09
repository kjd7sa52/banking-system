# Simple Banking System

Coding exercise without real purpose.

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

## Design

### Initial Assumptions

Certain places in the requirements document aren't complete. Project has been developed with the assumptions below. At the same time, architecture of the application provides enough flexibility to apply changes in scope of uncertain requirements.

- Rows in the input file that don't conform to the specified input format are ignored. Application continues to process the following rows.

- All the transactions that are described below as invalid or not allowed are ignored. Application continues to process the following transactions.

- "Frozen account" and "locked account" are synonyms. Transactions to the locked accounts are not allowed, except for *Deposit*. There is no transaction that could unlock an account.
  > Transaction for unlocking the account can be added by creating a file in `transactions` module. File must define a structure that implements `Transaction` trait. Finally structure must be added to `try_dispatch()` in `dispatcher.rs`.

- *Deposit* and *Withdrawal* are valid only if corresponding field `amount` has value that is positive (greater than 0).
> This requirement can be easily relaxed in `amount()` getter defined in `transport/record.rs`. Such feature could be useful for instance for creating accounts with no initial funds.

- Only *Deposit* can result in creation of a new account. Remaining transactions are invalid when they refer to a hypothetical new account (with no funds and no previous transactions).
> This can be easily altered by defining `allowes_account_creation()` that returns `true` for corresponding transaction.

- Only *Deposit* transaction can be disputed.
  > In case of an ATM, this would make even more sense to dispute *Withdrawal*. This would require additional considerations and is currently not implemented. Required changes would be applied in the implementation of `execute()` in `dispute.rs`, `resolve.rs`, `chargeback.rs`.
  > Transactions *Dispute*/*Resolve*/*Chargeback* cannot be disputed as they don't even have own transaction ID (their `tx` corresponds to the transaction being disputed).

- *Dispute* transaction is rejected if corresponding amount is no longer available in the account.
> Ignoring this limitation could result in negative total amounts in the accounts.

- *Resolve*/*Chargeback* can apply only to the transactions that are under dispute.

- *Dispute* can apply only to the transactions that are not under dispute. Transaction can be disputed and resolved unlimited number of times.

- Since transactions have globally unique identifiers, `client_id` of *Dispute*/*Resolve*/*Chargeback* seems to carry redundant information. Despite of this, `client_id` is expected to be valid and correspond to the transaction indicated by `tx`. Otherwise, transaction *Dispute*/*Resolve*/*Chargeback* in question is considered invalid.

- Transactions that re-use value of `tx` used before can be ignored. This is however not required from the application.

- Applications terminates with exit code other than 0 in case of errors not related to the content of the input file. This applies for instance to non-existing input file, inaccessible input file, invalid command line arguments, etc. In remaining cases, application terminates with exit code 0.

### Design Decisions

