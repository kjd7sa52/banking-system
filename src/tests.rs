#[cfg(test)]
mod tests {
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal::Decimal;

    use crate::database::{Account, MemDatabase};
    use crate::dispatcher::Dispatcher;
    use crate::transport::record::{ClientId, Record, TransactionId};

    // Test Framework

    struct TestApp {
        pub db: MemDatabase,
    }

    impl TestApp {
        fn new() -> Self {
            let db = MemDatabase::new();
            Self { db }
        }
        fn dispatch<T: Into<String>, U: Into<Option<i32>>>(
            &mut self,
            transaction_type: T,
            client_id: ClientId,
            transaction_id: TransactionId,
            amount: U,
        ) {
            let amount = match amount.into() {
                Some(x) => Decimal::from_i32(x),
                None => None,
            };

            let mut dispatcher = Dispatcher::new(&mut self.db);
            dispatcher.dispatch(&Ok(Record::new(
                transaction_type.into(),
                client_id,
                transaction_id,
                amount.into(),
            )));
        }

        fn first_account(&mut self) -> &Account {
            self.db.accounts().values().next().unwrap()
        }

        fn assert_first_account_total(&mut self, amount: i32) {
            let first_total = self.first_account().amount_total;
            assert_eq!(
                first_total,
                amount.into(),
                "total amount should be {} but is {}",
                first_total,
                amount
            );
        }
        fn assert_first_account_held(&mut self, amount: i32) {
            let first_held = self.first_account().amount_held;
            assert_eq!(
                first_held,
                amount.into(),
                "total amount should be {} but is {}",
                first_held,
                amount
            );
        }
    }

    // Invalid Record

    #[test]
    fn test_invalid_transaction_type_is_ignored() {
        let mut ta = TestApp::new();
        ta.dispatch("invalid_transaction_type", 10, 100, 10);
        assert_eq!(ta.db.accounts().len(), 0);
    }

    #[test]
    fn test_invalid_amount_is_ignored() {
        let mut ta = TestApp::new();
        let mut dp = Dispatcher::new(&mut ta.db);
        dp.dispatch(&Ok(Record::new(
            "deposit".to_string(),
            10,
            100,
            Decimal::from_i32(0),
        )));
        assert_eq!(ta.db.accounts().len(), 0);
    }

    // Deposit

    #[test]
    fn test_account_is_created_when_deposit_to_nonexisting_account_occurs() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 10);
        assert_eq!(ta.db.accounts().len(), 1);
        assert_eq!(ta.db.accounts().keys().next().unwrap(), &10);
    }

    #[test]
    fn test_deposits_add_funds_to_the_account() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 10);
        ta.dispatch("deposit", 10, 101, 20);
        ta.dispatch("deposit", 10, 102, 30);
        ta.assert_first_account_total(60);
        ta.assert_first_account_held(00);
    }

    #[test]
    fn test_duplicated_deposits_are_rejected() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 10);
        ta.dispatch("deposit", 10, 100, 20);
        ta.assert_first_account_total(10);
        ta.assert_first_account_held(00);
    }

    // Withdrawal

    #[test]
    fn test_account_is_not_created_when_withdrawal_from_nonexisting_account_occurs() {
        let mut ta = TestApp::new();
        ta.dispatch("withdrawal", 10, 200, 10);
        assert_eq!(ta.db.accounts().len(), 0);
    }

    #[test]
    fn test_withdrawals_deduct_funds_from_the_account() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("withdrawal", 10, 200, 10);
        ta.dispatch("withdrawal", 10, 201, 20);
        ta.dispatch("withdrawal", 10, 202, 30);
        ta.assert_first_account_total(140);
        ta.assert_first_account_held(00);
    }

    #[test]
    fn test_withdrawals_are_rejected_when_total_funds_are_insufficient() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("withdrawal", 10, 200, 300);
        ta.assert_first_account_total(200);
        ta.assert_first_account_held(00);
    }

    #[test]
    fn test_withdrawals_are_rejected_when_total_funds_are_sufficient_but_available_funds_are_not() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 400);
        ta.dispatch("deposit", 10, 101, 200);
        ta.dispatch("dispute", 10, 100, None);
        ta.dispatch("withdrawal", 10, 200, 300);
        ta.assert_first_account_total(600);
        ta.assert_first_account_held(400);
    }

    // Dispute

    #[test]
    fn test_dispute_holds_corresponding_funds() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("dispute", 10, 101, None);
        ta.dispatch("dispute", 10, 100, None);
        ta.assert_first_account_total(300);
        ta.assert_first_account_held(300);
    }

    #[test]
    fn test_dispute_is_declined_when_disputed_amount_is_greater_than_total() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("withdrawal", 10, 200, 150);
        ta.dispatch("dispute", 10, 100, None);
        ta.assert_first_account_total(50);
        ta.assert_first_account_held(00);
    }

    #[test]
    fn test_dispute_is_declined_when_disputed_amount_is_lower_than_total_but_greater_than_available(
    ) {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 400);
        ta.dispatch("deposit", 10, 101, 200);
        ta.dispatch("withdrawal", 10, 200, 100);
        ta.dispatch("dispute", 10, 100, None);
        ta.dispatch("dispute", 10, 101, None);
        ta.assert_first_account_total(500);
        ta.assert_first_account_held(400);
    }

    #[test]
    fn test_dispute_of_nonexisting_transaction_is_denied() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("dispute", 10, 102, None);
        ta.assert_first_account_total(300);
        ta.assert_first_account_held(00);
    }

    #[test]
    fn test_duplicated_dispute_is_rejected() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("dispute", 10, 101, None);
        ta.dispatch("dispute", 10, 101, None);
        ta.assert_first_account_total(300);
        ta.assert_first_account_held(100);
    }

    // Resolve

    #[test]
    fn test_resolve_subtracts_disputed_funds_from_held() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("dispute", 10, 100, None);
        ta.dispatch("resolve", 10, 100, None);
        ta.assert_first_account_total(300);
        ta.assert_first_account_held(00);
    }

    #[test]
    fn test_transaction_that_is_not_disputed_cannot_be_resolved() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("resolve", 10, 100, None);
        ta.assert_first_account_total(300);
        ta.assert_first_account_held(00);
    }

    #[test]
    fn test_resolved_transfer_can_be_disputed_again() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("dispute", 10, 100, None);
        ta.dispatch("resolve", 10, 100, None);
        ta.dispatch("dispute", 10, 100, None);
        ta.assert_first_account_total(300);
        ta.assert_first_account_held(200);
    }

    // Cashback

    #[test]
    fn test_chargeback_subtracts_disputed_funds_from_held_and_total() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("dispute", 10, 100, None);
        ta.dispatch("chargeback", 10, 100, None);
        ta.assert_first_account_total(100);
        ta.assert_first_account_held(00);
        assert_eq!(ta.first_account().locked, true);
    }

    #[test]
    fn test_transaction_that_is_not_disputed_cannot_be_chargedback() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("chargeback", 10, 100, None);
        ta.assert_first_account_total(300);
        ta.assert_first_account_held(00);
        assert_eq!(ta.first_account().locked, false);
    }

    #[test]
    fn test_chargeback_twice_not_possible() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("dispute", 10, 100, None);
        ta.dispatch("chargeback", 10, 100, None);
        ta.dispatch("chargeback", 10, 100, None);
        ta.assert_first_account_total(100);
        ta.assert_first_account_held(00);
        assert_eq!(ta.first_account().locked, true);
    }

    // Mixed

    #[test]
    fn test_deposit_on_frozen_account_possible() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("dispute", 10, 100, None);
        ta.dispatch("chargeback", 10, 100, None);
        ta.dispatch("deposit", 10, 102, 300);
        ta.assert_first_account_total(400);
        ta.assert_first_account_held(00);
        assert_eq!(ta.first_account().locked, true);
    }

    #[test]
    fn test_withdrawal_from_frozen_account_not_possible() {
        let mut ta = TestApp::new();
        ta.dispatch("deposit", 10, 100, 200);
        ta.dispatch("deposit", 10, 101, 100);
        ta.dispatch("dispute", 10, 100, None);
        ta.dispatch("chargeback", 10, 100, None);
        ta.dispatch("withdrawal", 10, 200, 50);
        ta.assert_first_account_total(100);
        ta.assert_first_account_held(00);
        assert_eq!(ta.first_account().locked, true);
    }
}
