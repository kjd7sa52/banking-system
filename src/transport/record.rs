use crate::transactions::TransactionError;
use rust_decimal::Decimal;

pub type ClientId = u16;
pub type TransactionId = u32;
pub type Amount = Decimal;
const AMOUNT_SCALE: u32 = 4;

#[derive(Debug, serde::Deserialize, derive_new::new)]
pub struct Record {
    pub r#type: String,
    pub client: ClientId,
    pub tx: TransactionId,
    amount: Option<Decimal>,
}

impl Record {
    pub fn amount(&self) -> Result<Amount, TransactionError> {
        match self.amount {
            Some(amount) => {
                if amount <= Decimal::new(0, 0) {
                    let msg = format!("Amount must be positive in: {:?}", self);
                    Err(TransactionError::reject(msg))
                } else {
                    Ok(amount.trunc_with_scale(AMOUNT_SCALE))
                }
            }
            None => {
                let msg = format!("Amount missing in: {:?}", self);
                Err(TransactionError::reject(msg))
            }
        }
    }
}
